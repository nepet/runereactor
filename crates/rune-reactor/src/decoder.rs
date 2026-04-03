use crate::error::ParseError;
use crate::types::{Condition, Op, Restriction, RunePolicy};

pub fn decode_rune(raw: &str) -> Result<RunePolicy, ParseError> {
    if raw.is_empty() {
        return Err(ParseError::InvalidExpression(
            "empty rune string".to_string(),
        ));
    }
    let mut restrictions = Vec::new();
    for restriction_str in raw.split('&') {
        let mut alternatives = Vec::new();
        for alt_str in restriction_str.split('|') {
            let condition = parse_condition(alt_str)?;
            alternatives.push(condition);
        }
        restrictions.push(Restriction { alternatives });
    }
    Ok(RunePolicy { restrictions })
}

fn parse_condition(s: &str) -> Result<Condition, ParseError> {
    for (i, c) in s.char_indices() {
        if let Some(op) = Op::from_char(c) {
            let field = s[..i].to_string();
            if field.is_empty() {
                return Err(ParseError::InvalidExpression(format!(
                    "missing field name before operator '{}'",
                    c
                )));
            }
            let value = if op == Op::Missing {
                String::new()
            } else {
                s[i + c.len_utf8()..].to_string()
            };
            return Ok(Condition { field, op, value });
        }
    }
    Err(ParseError::InvalidExpression(format!(
        "no operator found in condition '{}'",
        s
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_single_condition() {
        let result = decode_rune("method=listfunds").unwrap();
        assert_eq!(result.restrictions.len(), 1);
        assert_eq!(result.restrictions[0].alternatives.len(), 1);
        let cond = &result.restrictions[0].alternatives[0];
        assert_eq!(cond.field, "method");
        assert_eq!(cond.op, Op::Eq);
        assert_eq!(cond.value, "listfunds");
    }

    #[test]
    fn decode_or_alternatives() {
        let result = decode_rune("method=listfunds|method=xpay").unwrap();
        assert_eq!(result.restrictions.len(), 1);
        assert_eq!(result.restrictions[0].alternatives.len(), 2);
        assert_eq!(result.restrictions[0].alternatives[0].field, "method");
        assert_eq!(result.restrictions[0].alternatives[0].op, Op::Eq);
        assert_eq!(result.restrictions[0].alternatives[0].value, "listfunds");
        assert_eq!(result.restrictions[0].alternatives[1].field, "method");
        assert_eq!(result.restrictions[0].alternatives[1].op, Op::Eq);
        assert_eq!(result.restrictions[0].alternatives[1].value, "xpay");
    }

    #[test]
    fn decode_and_restrictions() {
        let result = decode_rune("method=listfunds&pnameamount_msat<1000000001").unwrap();
        assert_eq!(result.restrictions.len(), 2);
        assert_eq!(result.restrictions[0].alternatives[0].field, "method");
        assert_eq!(result.restrictions[0].alternatives[0].op, Op::Eq);
        assert_eq!(result.restrictions[0].alternatives[0].value, "listfunds");
        assert_eq!(result.restrictions[1].alternatives[0].field, "pnameamount_msat");
        assert_eq!(result.restrictions[1].alternatives[0].op, Op::Lt);
        assert_eq!(result.restrictions[1].alternatives[0].value, "1000000001");
    }

    #[test]
    fn decode_missing_operator() {
        let result = decode_rune("rate!").unwrap();
        assert_eq!(result.restrictions.len(), 1);
        let cond = &result.restrictions[0].alternatives[0];
        assert_eq!(cond.field, "rate");
        assert_eq!(cond.op, Op::Missing);
        assert_eq!(cond.value, "");
    }

    #[test]
    fn decode_comment() {
        let result = decode_rune("note#this is a comment").unwrap();
        assert_eq!(result.restrictions.len(), 1);
        let cond = &result.restrictions[0].alternatives[0];
        assert_eq!(cond.field, "note");
        assert_eq!(cond.op, Op::Comment);
        assert_eq!(cond.value, "this is a comment");
    }

    #[test]
    fn decode_complex_multi_restriction() {
        let raw = "method=listfunds|method=xpay&method/xpay|pnameamount_msat<1000000001";
        let result = decode_rune(raw).unwrap();
        assert_eq!(result.restrictions.len(), 2);
        // First restriction: method=listfunds OR method=xpay
        assert_eq!(result.restrictions[0].alternatives.len(), 2);
        assert_eq!(result.restrictions[0].alternatives[0].value, "listfunds");
        assert_eq!(result.restrictions[0].alternatives[1].value, "xpay");
        // Second restriction: method/xpay OR pnameamount_msat<1000000001
        assert_eq!(result.restrictions[1].alternatives.len(), 2);
        assert_eq!(result.restrictions[1].alternatives[0].field, "method");
        assert_eq!(result.restrictions[1].alternatives[0].op, Op::Ne);
        assert_eq!(result.restrictions[1].alternatives[0].value, "xpay");
        assert_eq!(result.restrictions[1].alternatives[1].field, "pnameamount_msat");
        assert_eq!(result.restrictions[1].alternatives[1].op, Op::Lt);
    }

    #[test]
    fn decode_empty_string_error() {
        let result = decode_rune("");
        assert!(result.is_err());
    }
}
