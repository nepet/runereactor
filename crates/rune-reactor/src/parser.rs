use crate::error::ParseError;
use crate::types::Op;

/// The parsed AST from an .rf policy file.
///
/// This is an intermediate representation — not the flat `RunePolicy`.
/// Pass it to `compiler::compile()` to get the final `RunePolicy`.
#[derive(Debug, Clone)]
pub struct Policy {
    pub directives: Vec<Directive>,
}

/// A single directive in an .rf policy file.
#[derive(Debug, Clone)]
pub enum Directive {
    /// `tag: field value` — non-enforcing metadata (Comment restriction)
    Tag { field: String, value: String },
    /// `id: hex` — restrict to a specific commando peer
    Id(String),
    /// `allow methods: a, b, c` — method whitelist
    AllowMethods(Vec<String>),
    /// `when method:` block — conditional restrictions
    When { method: String, body: Expr },
    /// `global:` block — restrictions applied to all methods
    Global(Expr),
}

/// An expression in a `when` or `global` block.
#[derive(Debug, Clone)]
pub enum Expr {
    /// A single condition: `field op value`
    Cond {
        field: String,
        op: Op,
        value: String,
    },
    /// `expr or expr` — disjunction
    Or(Vec<Expr>),
    /// `expr and expr` — conjunction (also implicit between lines)
    And(Vec<Expr>),
}

// ---------------------------------------------------------------------------
// Tokenizer for expression lines
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Word(String),
    OpChar(char),
    LParen,
    RParen,
    And,
    Or,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        if c == '(' {
            tokens.push(Token::LParen);
            chars.next();
            continue;
        }
        if c == ')' {
            tokens.push(Token::RParen);
            chars.next();
            continue;
        }
        if Op::from_char(c).is_some() && c != '#' {
            tokens.push(Token::OpChar(c));
            chars.next();
            continue;
        }
        // Collect a word
        let mut word = String::new();
        while let Some(&ch) = chars.peek() {
            if ch.is_whitespace() || ch == '(' || ch == ')' {
                break;
            }
            if Op::from_char(ch).is_some() && ch != '#' {
                break;
            }
            word.push(ch);
            chars.next();
        }
        if word == "and" {
            tokens.push(Token::And);
        } else if word == "or" {
            tokens.push(Token::Or);
        } else if !word.is_empty() {
            tokens.push(Token::Word(word));
        }
    }

    tokens
}

// ---------------------------------------------------------------------------
// Recursive-descent expression parser
// ---------------------------------------------------------------------------

struct ExprParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl ExprParser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn parse_or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut items = vec![self.parse_and_expr()?];
        while self.peek() == Some(&Token::Or) {
            self.next(); // consume 'or'
            items.push(self.parse_and_expr()?);
        }
        if items.len() == 1 {
            Ok(items.remove(0))
        } else {
            Ok(Expr::Or(items))
        }
    }

    fn parse_and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut items = vec![self.parse_atom()?];
        while self.peek() == Some(&Token::And) {
            self.next(); // consume 'and'
            items.push(self.parse_atom()?);
        }
        if items.len() == 1 {
            Ok(items.remove(0))
        } else {
            Ok(Expr::And(items))
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        if self.peek() == Some(&Token::LParen) {
            self.next(); // consume '('
            let expr = self.parse_or_expr()?;
            if self.peek() == Some(&Token::RParen) {
                self.next(); // consume ')'
            } else {
                return Err(ParseError::InvalidExpression(
                    "expected closing parenthesis".into(),
                ));
            }
            return Ok(expr);
        }
        self.parse_condition()
    }

    fn parse_condition(&mut self) -> Result<Expr, ParseError> {
        // field
        let field = match self.next() {
            Some(Token::Word(w)) => w.clone(),
            other => {
                return Err(ParseError::InvalidExpression(format!(
                    "expected field name, got {:?}",
                    other
                )));
            }
        };

        // op
        let op_char = match self.next() {
            Some(Token::OpChar(c)) => *c,
            other => {
                return Err(ParseError::InvalidExpression(format!(
                    "expected operator after '{}', got {:?}",
                    field, other
                )));
            }
        };

        let op = Op::from_char(op_char).ok_or_else(|| {
            ParseError::InvalidExpression(format!("unknown operator '{}'", op_char))
        })?;

        // value (optional for Missing)
        if op == Op::Missing {
            return Ok(Expr::Cond {
                field,
                op,
                value: String::new(),
            });
        }

        let value = match self.peek() {
            Some(Token::Word(_)) => {
                if let Some(Token::Word(w)) = self.next() {
                    w.clone()
                } else {
                    unreachable!()
                }
            }
            _ => {
                return Err(ParseError::InvalidExpression(format!(
                    "expected value after operator '{}' for field '{}'",
                    op_char, field
                )));
            }
        };

        Ok(Expr::Cond { field, op, value })
    }
}

fn parse_expr_line(line: &str) -> Result<Expr, ParseError> {
    let tokens = tokenize(line);
    let mut parser = ExprParser::new(tokens);
    let expr = parser.parse_or_expr()?;
    Ok(expr)
}

// ---------------------------------------------------------------------------
// Line-level parser
// ---------------------------------------------------------------------------

/// Parse an `.rf` policy file into an AST.
pub fn parse_policy(input: &str) -> Result<Policy, ParseError> {
    let mut directives = Vec::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Skip blank lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            i += 1;
            continue;
        }

        // tag: field value
        if let Some(rest) = trimmed.strip_prefix("tag:") {
            let rest = rest.trim();
            let (field, value) = rest
                .split_once(char::is_whitespace)
                .ok_or_else(|| ParseError::Syntax {
                    line: i + 1,
                    message: "tag directive requires 'field value'".into(),
                })?;
            directives.push(Directive::Tag {
                field: field.trim().to_string(),
                value: value.trim().to_string(),
            });
            i += 1;
            continue;
        }

        // id: hex_string
        if let Some(rest) = trimmed.strip_prefix("id:") {
            let hex = rest.trim().to_string();
            directives.push(Directive::Id(hex));
            i += 1;
            continue;
        }

        // allow methods: method_list
        if let Some(rest) = trimmed.strip_prefix("allow methods:") {
            let methods: Vec<String> = rest
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            directives.push(Directive::AllowMethods(methods));
            i += 1;
            continue;
        }

        // when method:
        if let Some(rest) = trimmed.strip_prefix("when ") {
            let method = rest
                .strip_suffix(':')
                .ok_or_else(|| ParseError::Syntax {
                    line: i + 1,
                    message: "when directive must end with ':'".into(),
                })?
                .trim()
                .to_string();
            i += 1;
            let body = collect_indented_body(&lines, &mut i)?;
            directives.push(Directive::When { method, body });
            continue;
        }

        // global:
        if trimmed == "global:" {
            i += 1;
            let body = collect_indented_body(&lines, &mut i)?;
            directives.push(Directive::Global(body));
            continue;
        }

        return Err(ParseError::Syntax {
            line: i + 1,
            message: format!("unrecognized directive: {}", trimmed),
        });
    }

    Ok(Policy { directives })
}

/// Collect indented continuation lines and parse them into an expression.
/// Multiple lines are implicitly AND'd together.
fn collect_indented_body(lines: &[&str], i: &mut usize) -> Result<Expr, ParseError> {
    let mut exprs = Vec::new();

    while *i < lines.len() {
        let line = lines[*i];
        // An indented line starts with whitespace and is not blank
        if line.starts_with(' ') || line.starts_with('\t') {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                *i += 1;
                continue;
            }
            exprs.push(parse_expr_line(trimmed)?);
            *i += 1;
        } else {
            break;
        }
    }

    if exprs.is_empty() {
        return Err(ParseError::InvalidExpression(
            "expected indented body after block directive".into(),
        ));
    }

    if exprs.len() == 1 {
        Ok(exprs.remove(0))
    } else {
        Ok(Expr::And(exprs))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Op;

    #[test]
    fn test_parse_tag() {
        let policy = parse_policy("tag: operator_id default-operator\n").unwrap();
        assert_eq!(policy.directives.len(), 1);
        match &policy.directives[0] {
            Directive::Tag { field, value } => {
                assert_eq!(field, "operator_id");
                assert_eq!(value, "default-operator");
            }
            other => panic!("expected Tag, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_id() {
        let policy = parse_policy("id: 02abcdef1234567890\n").unwrap();
        assert_eq!(policy.directives.len(), 1);
        match &policy.directives[0] {
            Directive::Id(hex) => assert_eq!(hex, "02abcdef1234567890"),
            other => panic!("expected Id, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_allow_methods() {
        let policy = parse_policy("allow methods: listfunds, xpay, invoice\n").unwrap();
        assert_eq!(policy.directives.len(), 1);
        match &policy.directives[0] {
            Directive::AllowMethods(methods) => {
                assert_eq!(methods.len(), 3);
                assert_eq!(methods[0], "listfunds");
                assert_eq!(methods[1], "xpay");
                assert_eq!(methods[2], "invoice");
            }
            other => panic!("expected AllowMethods, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_when_simple() {
        let input = "when fundchannel:\n  pnameamount < 1000001\n";
        let policy = parse_policy(input).unwrap();
        assert_eq!(policy.directives.len(), 1);
        match &policy.directives[0] {
            Directive::When { method, body } => {
                assert_eq!(method, "fundchannel");
                match body {
                    Expr::Cond { field, op, value } => {
                        assert_eq!(field, "pnameamount");
                        assert_eq!(*op, Op::Lt);
                        assert_eq!(value, "1000001");
                    }
                    other => panic!("expected Cond, got {:?}", other),
                }
            }
            other => panic!("expected When, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_when_or() {
        let input = "when xpay:\n  pnameamount_msat < 1000000001 or pnameamount_msat !\n";
        let policy = parse_policy(input).unwrap();
        assert_eq!(policy.directives.len(), 1);
        match &policy.directives[0] {
            Directive::When { method, body } => {
                assert_eq!(method, "xpay");
                match body {
                    Expr::Or(items) => {
                        assert_eq!(items.len(), 2);
                        match &items[0] {
                            Expr::Cond { field, op, value } => {
                                assert_eq!(field, "pnameamount_msat");
                                assert_eq!(*op, Op::Lt);
                                assert_eq!(value, "1000000001");
                            }
                            other => panic!("expected Cond, got {:?}", other),
                        }
                        match &items[1] {
                            Expr::Cond { field, op, value } => {
                                assert_eq!(field, "pnameamount_msat");
                                assert_eq!(*op, Op::Missing);
                                assert_eq!(value, "");
                            }
                            other => panic!("expected Cond, got {:?}", other),
                        }
                    }
                    other => panic!("expected Or, got {:?}", other),
                }
            }
            other => panic!("expected When, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_when_paren_and() {
        let input =
            "when xpay:\n  (pnameamount_msat < 1000000001 or pnameamount_msat !) and rate = 10\n";
        let policy = parse_policy(input).unwrap();
        assert_eq!(policy.directives.len(), 1);
        match &policy.directives[0] {
            Directive::When { method, body } => {
                assert_eq!(method, "xpay");
                match body {
                    Expr::And(items) => {
                        assert_eq!(items.len(), 2);
                        // First item: Or
                        match &items[0] {
                            Expr::Or(or_items) => {
                                assert_eq!(or_items.len(), 2);
                            }
                            other => panic!("expected Or, got {:?}", other),
                        }
                        // Second item: Cond rate = 10
                        match &items[1] {
                            Expr::Cond { field, op, value } => {
                                assert_eq!(field, "rate");
                                assert_eq!(*op, Op::Eq);
                                assert_eq!(value, "10");
                            }
                            other => panic!("expected Cond, got {:?}", other),
                        }
                    }
                    other => panic!("expected And, got {:?}", other),
                }
            }
            other => panic!("expected When, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_when_multiline_implicit_and() {
        let input = "when xpay:\n  pnameamount_msat < 1000000001\n  rate = 10\n";
        let policy = parse_policy(input).unwrap();
        assert_eq!(policy.directives.len(), 1);
        match &policy.directives[0] {
            Directive::When { method, body } => {
                assert_eq!(method, "xpay");
                match body {
                    Expr::And(items) => {
                        assert_eq!(items.len(), 2);
                        match &items[0] {
                            Expr::Cond { field, op, value } => {
                                assert_eq!(field, "pnameamount_msat");
                                assert_eq!(*op, Op::Lt);
                                assert_eq!(value, "1000000001");
                            }
                            other => panic!("expected Cond, got {:?}", other),
                        }
                        match &items[1] {
                            Expr::Cond { field, op, value } => {
                                assert_eq!(field, "rate");
                                assert_eq!(*op, Op::Eq);
                                assert_eq!(value, "10");
                            }
                            other => panic!("expected Cond, got {:?}", other),
                        }
                    }
                    other => panic!("expected And, got {:?}", other),
                }
            }
            other => panic!("expected When, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_global() {
        let input = "global:\n  per = 1min\n";
        let policy = parse_policy(input).unwrap();
        assert_eq!(policy.directives.len(), 1);
        match &policy.directives[0] {
            Directive::Global(body) => match body {
                Expr::Cond { field, op, value } => {
                    assert_eq!(field, "per");
                    assert_eq!(*op, Op::Eq);
                    assert_eq!(value, "1min");
                }
                other => panic!("expected Cond, got {:?}", other),
            },
            other => panic!("expected Global, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_comments_and_blanks() {
        let input = "# This is a comment\n\n# Another comment\ntag: foo bar\n\n";
        let policy = parse_policy(input).unwrap();
        assert_eq!(policy.directives.len(), 1);
        match &policy.directives[0] {
            Directive::Tag { field, value } => {
                assert_eq!(field, "foo");
                assert_eq!(value, "bar");
            }
            other => panic!("expected Tag, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_operator_rf() {
        let input = r#"# Operator Rune Policy
#
# This policy defines the CLN methods an operator can access via commando,
# and the parameter constraints that apply to each method.
#
# Compile with: rune-biscuit to-biscuit operator.rf
# Signed token: rune-biscuit to-biscuit operator.rf --format token --key key.hex

tag: operator_id default-operator

allow methods: listfunds, listpeerchannels, fundchannel, close, invoice, xpay

# Fund channels up to 1,000,000 sats (strict less-than, so < 1000001)
when fundchannel:
  pnameamount < 1000001

# Pay invoices up to 1,000,000 sats.
# The `or pnameamount_msat !` handles the case where the invoice
# already specifies an amount and amount_msat is not passed explicitly.
when xpay:
  pnameamount_msat < 1000000001 or pnameamount_msat !

# Restrict channel closes to a known cold wallet address
# when close:
#   pnamedestination = bc1qYOUR_COLD_WALLET_ADDRESS_HERE
"#;
        let policy = parse_policy(input).unwrap();
        assert_eq!(policy.directives.len(), 4);

        assert!(matches!(&policy.directives[0], Directive::Tag { field, value }
            if field == "operator_id" && value == "default-operator"));
        assert!(matches!(&policy.directives[1], Directive::AllowMethods(m) if m.len() == 6));
        assert!(matches!(&policy.directives[2], Directive::When { method, .. } if method == "fundchannel"));
        assert!(matches!(&policy.directives[3], Directive::When { method, .. } if method == "xpay"));
    }

    #[test]
    fn test_parse_advanced_rf() {
        let input = r#"# Advanced Operator Policy — demonstrating AND-within-OR
#
# This showcases the negation bypass + CNF distribution that rune-reactor
# automates. Without the tool, constructing these by hand is error-prone.

# Restrict to a specific commando peer (enforcing — peer must match this pubkey)
id: 02abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab

# Audit metadata (comment restrictions — always pass, visible in showrunes)
tag: operator_id advanced-operator
tag: version 2

allow methods: listfunds, listpeerchannels, listchannels, listpays, listinvoices, getinfo, fundchannel, close, invoice, xpay, waitanyinvoice

# Fund channels: max 1M sats
when fundchannel:
  pnameamount < 1000001

# Pay: max 1M sats, OR invoice-embedded amount (missing param)
# Also rate-limit to 10 calls/minute
when xpay:
  (pnameamount_msat < 1000000001 or pnameamount_msat !) and rate = 10

# Close only to known address
when close:
  pnamedestination = bc1qexamplecoldwalletaddress

# Global: rate limit all calls
global:
  per = 1min
"#;
        let policy = parse_policy(input).unwrap();
        // id + 2 tags + allow + 3 when + global = 8
        assert_eq!(policy.directives.len(), 8);

        assert!(matches!(&policy.directives[0], Directive::Id(hex)
            if hex == "02abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab"));
        assert!(matches!(&policy.directives[1], Directive::Tag { field, .. } if field == "operator_id"));
        assert!(matches!(&policy.directives[2], Directive::Tag { field, .. } if field == "version"));
        assert!(matches!(&policy.directives[3], Directive::AllowMethods(m) if m.len() == 11));
        assert!(matches!(&policy.directives[4], Directive::When { method, .. } if method == "fundchannel"));
        assert!(matches!(&policy.directives[5], Directive::When { method, .. } if method == "xpay"));
        assert!(matches!(&policy.directives[6], Directive::When { method, .. } if method == "close"));
        assert!(matches!(&policy.directives[7], Directive::Global(_)));

        // Verify the xpay body is And([Or([..]), Cond{rate=10}])
        if let Directive::When { body, .. } = &policy.directives[5] {
            match body {
                Expr::And(items) => {
                    assert_eq!(items.len(), 2);
                    assert!(matches!(&items[0], Expr::Or(v) if v.len() == 2));
                    assert!(matches!(&items[1], Expr::Cond { field, op, value }
                        if field == "rate" && *op == Op::Eq && value == "10"));
                }
                other => panic!("expected And for xpay body, got {:?}", other),
            }
        }
    }
}
