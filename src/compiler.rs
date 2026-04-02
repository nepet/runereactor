use crate::error::CompileError;
use crate::parser::{Directive, Expr, Policy};
use crate::types::{Condition, Op, Restriction, RunePolicy};

/// Convert an `Expr` tree into CNF: a Vec of clauses, where each clause is a
/// Vec of `Condition` (alternatives within a disjunction).
fn expr_to_cnf(expr: &Expr) -> Vec<Vec<Condition>> {
    match expr {
        Expr::Cond { field, op, value } => {
            vec![vec![Condition {
                field: field.clone(),
                op: *op,
                value: value.clone(),
            }]]
        }
        Expr::And(children) => {
            let mut clauses = Vec::new();
            for child in children {
                clauses.extend(expr_to_cnf(child));
            }
            clauses
        }
        Expr::Or(children) => {
            // Start with a single empty clause, then cross-product with each child's CNF.
            let mut result: Vec<Vec<Condition>> = vec![vec![]];
            for child in children {
                let child_cnf = expr_to_cnf(child);
                result = distribute_or(&result, &child_cnf);
            }
            result
        }
    }
}

/// Cross-product two CNF clause-sets to produce the distributed OR.
/// For each clause L in `left` and each clause R in `right`, produce L ∪ R.
fn distribute_or(
    left: &[Vec<Condition>],
    right: &[Vec<Condition>],
) -> Vec<Vec<Condition>> {
    let mut result = Vec::new();
    for l in left {
        for r in right {
            let mut combined = l.clone();
            combined.extend(r.iter().cloned());
            result.push(combined);
        }
    }
    result
}

/// Compile a parsed policy AST into a flat `RunePolicy`.
///
/// This performs:
/// - Tag/Id → simple restrictions
/// - AllowMethods → single restriction with method=X alternatives
/// - When blocks → CNF normalization + negation bypass (method/M prepended)
/// - Global blocks → CNF normalization, restrictions applied directly
pub fn compile(policy: &Policy) -> Result<RunePolicy, CompileError> {
    let mut restrictions = Vec::new();

    for directive in &policy.directives {
        match directive {
            Directive::Tag { field, value } => {
                restrictions.push(Restriction {
                    alternatives: vec![Condition {
                        field: field.clone(),
                        op: Op::Comment,
                        value: value.clone(),
                    }],
                });
            }
            Directive::Id(hex) => {
                restrictions.push(Restriction {
                    alternatives: vec![Condition {
                        field: "id".to_string(),
                        op: Op::Eq,
                        value: hex.clone(),
                    }],
                });
            }
            Directive::AllowMethods(methods) => {
                let alternatives = methods
                    .iter()
                    .map(|m| Condition {
                        field: "method".to_string(),
                        op: Op::Eq,
                        value: m.clone(),
                    })
                    .collect();
                restrictions.push(Restriction { alternatives });
            }
            Directive::When { method, body } => {
                let clauses = expr_to_cnf(body);
                let bypass = Condition {
                    field: "method".to_string(),
                    op: Op::Ne,
                    value: method.clone(),
                };
                for clause in clauses {
                    let mut alternatives = vec![bypass.clone()];
                    alternatives.extend(clause);
                    restrictions.push(Restriction { alternatives });
                }
            }
            Directive::Global(body) => {
                let clauses = expr_to_cnf(body);
                for clause in clauses {
                    restrictions.push(Restriction {
                        alternatives: clause,
                    });
                }
            }
        }
    }

    Ok(RunePolicy { restrictions })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Directive, Expr, Policy};
    use crate::types::Op;

    #[test]
    fn test_compile_tag() {
        let policy = Policy {
            directives: vec![Directive::Tag {
                field: "operator_id".to_string(),
                value: "default-operator".to_string(),
            }],
        };
        let rune = compile(&policy).unwrap();
        assert_eq!(rune.restrictions.len(), 1);
        assert_eq!(rune.restrictions[0].alternatives.len(), 1);
        let c = &rune.restrictions[0].alternatives[0];
        assert_eq!(c.field, "operator_id");
        assert_eq!(c.op, Op::Comment);
        assert_eq!(c.value, "default-operator");
    }

    #[test]
    fn test_compile_id() {
        let policy = Policy {
            directives: vec![Directive::Id("deadbeef".to_string())],
        };
        let rune = compile(&policy).unwrap();
        assert_eq!(rune.restrictions.len(), 1);
        let c = &rune.restrictions[0].alternatives[0];
        assert_eq!(c.field, "id");
        assert_eq!(c.op, Op::Eq);
        assert_eq!(c.value, "deadbeef");
    }

    #[test]
    fn test_compile_allow_methods() {
        let policy = Policy {
            directives: vec![Directive::AllowMethods(vec![
                "listfunds".to_string(),
                "xpay".to_string(),
            ])],
        };
        let rune = compile(&policy).unwrap();
        assert_eq!(rune.restrictions.len(), 1);
        let alts = &rune.restrictions[0].alternatives;
        assert_eq!(alts.len(), 2);
        assert_eq!(alts[0].field, "method");
        assert_eq!(alts[0].op, Op::Eq);
        assert_eq!(alts[0].value, "listfunds");
        assert_eq!(alts[1].value, "xpay");
    }

    #[test]
    fn test_compile_when_simple() {
        let policy = Policy {
            directives: vec![Directive::When {
                method: "xpay".to_string(),
                body: Expr::Cond {
                    field: "pnameamount_msat".to_string(),
                    op: Op::Lt,
                    value: "1000".to_string(),
                },
            }],
        };
        let rune = compile(&policy).unwrap();
        assert_eq!(rune.restrictions.len(), 1);
        let alts = &rune.restrictions[0].alternatives;
        assert_eq!(alts.len(), 2);
        assert_eq!(alts[0].field, "method");
        assert_eq!(alts[0].op, Op::Ne);
        assert_eq!(alts[0].value, "xpay");
        assert_eq!(alts[1].field, "pnameamount_msat");
        assert_eq!(alts[1].op, Op::Lt);
        assert_eq!(alts[1].value, "1000");
    }

    #[test]
    fn test_compile_when_or() {
        let policy = Policy {
            directives: vec![Directive::When {
                method: "xpay".to_string(),
                body: Expr::Or(vec![
                    Expr::Cond {
                        field: "a".to_string(),
                        op: Op::Eq,
                        value: "1".to_string(),
                    },
                    Expr::Cond {
                        field: "b".to_string(),
                        op: Op::Eq,
                        value: "2".to_string(),
                    },
                ]),
            }],
        };
        let rune = compile(&policy).unwrap();
        assert_eq!(rune.restrictions.len(), 1);
        let alts = &rune.restrictions[0].alternatives;
        assert_eq!(alts.len(), 3);
        assert_eq!(alts[0].op, Op::Ne);
        assert_eq!(alts[0].value, "xpay");
        assert_eq!(alts[1].field, "a");
        assert_eq!(alts[2].field, "b");
    }

    #[test]
    fn test_compile_when_and() {
        // When with (A or B) and C → 2 restrictions, each with method/M bypass
        let policy = Policy {
            directives: vec![Directive::When {
                method: "M".to_string(),
                body: Expr::And(vec![
                    Expr::Or(vec![
                        Expr::Cond {
                            field: "A".to_string(),
                            op: Op::Eq,
                            value: "1".to_string(),
                        },
                        Expr::Cond {
                            field: "B".to_string(),
                            op: Op::Eq,
                            value: "2".to_string(),
                        },
                    ]),
                    Expr::Cond {
                        field: "C".to_string(),
                        op: Op::Eq,
                        value: "3".to_string(),
                    },
                ]),
            }],
        };
        let rune = compile(&policy).unwrap();
        assert_eq!(rune.restrictions.len(), 2);
        // First restriction: method/M | A | B
        let r0 = &rune.restrictions[0].alternatives;
        assert_eq!(r0.len(), 3);
        assert_eq!(r0[0].op, Op::Ne);
        assert_eq!(r0[0].value, "M");
        assert_eq!(r0[1].field, "A");
        assert_eq!(r0[2].field, "B");
        // Second restriction: method/M | C
        let r1 = &rune.restrictions[1].alternatives;
        assert_eq!(r1.len(), 2);
        assert_eq!(r1[0].op, Op::Ne);
        assert_eq!(r1[0].value, "M");
        assert_eq!(r1[1].field, "C");
    }

    #[test]
    fn test_compile_dnf_distribution() {
        // Global with (A and B) or C → CNF: [[A,C], [B,C]]
        let policy = Policy {
            directives: vec![Directive::Global(Expr::Or(vec![
                Expr::And(vec![
                    Expr::Cond {
                        field: "A".to_string(),
                        op: Op::Eq,
                        value: "1".to_string(),
                    },
                    Expr::Cond {
                        field: "B".to_string(),
                        op: Op::Eq,
                        value: "2".to_string(),
                    },
                ]),
                Expr::Cond {
                    field: "C".to_string(),
                    op: Op::Eq,
                    value: "3".to_string(),
                },
            ]))],
        };
        let rune = compile(&policy).unwrap();
        assert_eq!(rune.restrictions.len(), 2);
        // First: [A, C]
        let r0 = &rune.restrictions[0].alternatives;
        assert_eq!(r0.len(), 2);
        assert_eq!(r0[0].field, "A");
        assert_eq!(r0[1].field, "C");
        // Second: [B, C]
        let r1 = &rune.restrictions[1].alternatives;
        assert_eq!(r1.len(), 2);
        assert_eq!(r1[0].field, "B");
        assert_eq!(r1[1].field, "C");
    }

    #[test]
    fn test_compile_global() {
        let policy = Policy {
            directives: vec![Directive::Global(Expr::Cond {
                field: "rate".to_string(),
                op: Op::Lt,
                value: "100".to_string(),
            })],
        };
        let rune = compile(&policy).unwrap();
        assert_eq!(rune.restrictions.len(), 1);
        let c = &rune.restrictions[0].alternatives[0];
        assert_eq!(c.field, "rate");
        assert_eq!(c.op, Op::Lt);
        assert_eq!(c.value, "100");
    }

    #[test]
    fn test_compile_end_to_end_operator_rf() {
        let input = "\
tag: operator_id default-operator
allow methods: listfunds, xpay
when xpay:
  pnameamount_msat < 1000000001 or pnameamount_msat !
";
        let policy = crate::parser::parse_policy(input).unwrap();
        let rune = compile(&policy).unwrap();

        assert_eq!(rune.restrictions.len(), 3);

        // [operator_id#default-operator]
        let r0 = &rune.restrictions[0];
        assert_eq!(r0.alternatives.len(), 1);
        assert_eq!(r0.alternatives[0].field, "operator_id");
        assert_eq!(r0.alternatives[0].op, Op::Comment);
        assert_eq!(r0.alternatives[0].value, "default-operator");

        // [method=listfunds | method=xpay]
        let r1 = &rune.restrictions[1];
        assert_eq!(r1.alternatives.len(), 2);
        assert_eq!(r1.alternatives[0].field, "method");
        assert_eq!(r1.alternatives[0].op, Op::Eq);
        assert_eq!(r1.alternatives[0].value, "listfunds");
        assert_eq!(r1.alternatives[1].field, "method");
        assert_eq!(r1.alternatives[1].op, Op::Eq);
        assert_eq!(r1.alternatives[1].value, "xpay");

        // [method/xpay | pnameamount_msat<1000000001 | pnameamount_msat!]
        let r2 = &rune.restrictions[2];
        assert_eq!(r2.alternatives.len(), 3);
        assert_eq!(r2.alternatives[0].field, "method");
        assert_eq!(r2.alternatives[0].op, Op::Ne);
        assert_eq!(r2.alternatives[0].value, "xpay");
        assert_eq!(r2.alternatives[1].field, "pnameamount_msat");
        assert_eq!(r2.alternatives[1].op, Op::Lt);
        assert_eq!(r2.alternatives[1].value, "1000000001");
        assert_eq!(r2.alternatives[2].field, "pnameamount_msat");
        assert_eq!(r2.alternatives[2].op, Op::Missing);
        assert_eq!(r2.alternatives[2].value, "");
    }
}
