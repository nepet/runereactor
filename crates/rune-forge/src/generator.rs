use serde::Deserialize;

use crate::error::CompileError;

#[derive(Debug, Deserialize)]
pub struct ConditionSpec {
    pub field: String,
    pub op: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct WhenSpec {
    pub method: String,
    pub conditions: Vec<ConditionSpec>,
}

#[derive(Debug, Deserialize)]
pub struct TagSpec {
    pub field: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct PolicySpec {
    pub tag: Option<TagSpec>,
    pub id: Option<String>,
    pub methods: Vec<String>,
    #[serde(default)]
    pub when: Vec<WhenSpec>,
    #[serde(default)]
    pub global: Vec<ConditionSpec>,
}

pub fn generate_policy(spec: &PolicySpec) -> Result<String, CompileError> {
    let mut output = String::new();

    if let Some(tag) = &spec.tag {
        output.push_str(&format!("tag: {} {}\n", tag.field, tag.value));
        output.push('\n');
    }

    if let Some(id) = &spec.id {
        output.push_str(&format!("id: {}\n", id));
        output.push('\n');
    }

    if !spec.methods.is_empty() {
        output.push_str(&format!("allow methods: {}\n", spec.methods.join(", ")));
    }

    for when in &spec.when {
        output.push('\n');
        output.push_str(&format!("when {}:\n", when.method));
        for cond in &when.conditions {
            if cond.op == "!" {
                output.push_str(&format!("  {} !\n", cond.field));
            } else {
                output.push_str(&format!("  {} {} {}\n", cond.field, cond.op, cond.value));
            }
        }
    }

    if !spec.global.is_empty() {
        output.push('\n');
        output.push_str("global:\n");
        for cond in &spec.global {
            if cond.op == "!" {
                output.push_str(&format!("  {} !\n", cond.field));
            } else {
                output.push_str(&format!("  {} {} {}\n", cond.field, cond.op, cond.value));
            }
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_methods_only() {
        let spec = PolicySpec {
            tag: None,
            id: None,
            methods: vec!["listfunds".into(), "xpay".into()],
            when: vec![],
            global: vec![],
        };
        let result = generate_policy(&spec).unwrap();
        assert_eq!(result, "allow methods: listfunds, xpay\n");
    }

    #[test]
    fn test_generate_with_tag() {
        let spec = PolicySpec {
            tag: Some(TagSpec {
                field: "operator_id".into(),
                value: "default-operator".into(),
            }),
            id: None,
            methods: vec!["listfunds".into()],
            when: vec![],
            global: vec![],
        };
        let result = generate_policy(&spec).unwrap();
        assert!(result.starts_with("tag: operator_id default-operator\n"));
    }

    #[test]
    fn test_generate_with_id() {
        let spec = PolicySpec {
            tag: None,
            id: Some("02abcdef".into()),
            methods: vec!["listfunds".into()],
            when: vec![],
            global: vec![],
        };
        let result = generate_policy(&spec).unwrap();
        assert!(result.starts_with("id: 02abcdef\n"));
    }

    #[test]
    fn test_generate_with_when() {
        let spec = PolicySpec {
            tag: None,
            id: None,
            methods: vec!["xpay".into()],
            when: vec![WhenSpec {
                method: "xpay".into(),
                conditions: vec![ConditionSpec {
                    field: "pnameamount_msat".into(),
                    op: "<".into(),
                    value: "1000000001".into(),
                }],
            }],
            global: vec![],
        };
        let result = generate_policy(&spec).unwrap();
        assert!(result.contains("when xpay:\n  pnameamount_msat < 1000000001\n"));
    }

    #[test]
    fn test_generate_with_missing_op() {
        let spec = PolicySpec {
            tag: None,
            id: None,
            methods: vec!["xpay".into()],
            when: vec![WhenSpec {
                method: "xpay".into(),
                conditions: vec![ConditionSpec {
                    field: "pnameamount_msat".into(),
                    op: "!".into(),
                    value: "".into(),
                }],
            }],
            global: vec![],
        };
        let result = generate_policy(&spec).unwrap();
        assert!(result.contains("when xpay:\n  pnameamount_msat !\n"));
    }

    #[test]
    fn test_generate_with_global() {
        let spec = PolicySpec {
            tag: None,
            id: None,
            methods: vec!["xpay".into()],
            when: vec![],
            global: vec![ConditionSpec {
                field: "per".into(),
                op: "=".into(),
                value: "1min".into(),
            }],
        };
        let result = generate_policy(&spec).unwrap();
        assert!(result.contains("global:\n  per = 1min\n"));
    }

    #[test]
    fn test_generate_full_roundtrip() {
        let spec = PolicySpec {
            tag: Some(TagSpec {
                field: "operator_id".into(),
                value: "default-operator".into(),
            }),
            id: None,
            methods: vec!["listfunds".into(), "xpay".into()],
            when: vec![WhenSpec {
                method: "xpay".into(),
                conditions: vec![ConditionSpec {
                    field: "pnameamount_msat".into(),
                    op: "<".into(),
                    value: "1000000001".into(),
                }],
            }],
            global: vec![],
        };
        let rf_source = generate_policy(&spec).unwrap();

        // Parse back and compile
        let policy = crate::parser::parse_policy(&rf_source).unwrap();
        let compiled = crate::compiler::compile(&policy).unwrap();
        assert_eq!(compiled.restrictions.len(), 3);
    }
}
