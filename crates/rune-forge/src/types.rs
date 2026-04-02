use serde::Serialize;

/// Rune condition operator — maps 1:1 to futhark's `Condition` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Op {
    Eq,
    Ne,
    Missing,
    Lt,
    Gt,
    LexLt,
    LexGt,
    StartsWith,
    EndsWith,
    Contains,
    Comment,
}

impl Op {
    /// Parse a single character into an operator.
    pub fn from_char(c: char) -> Option<Op> {
        match c {
            '=' => Some(Op::Eq),
            '/' => Some(Op::Ne),
            '!' => Some(Op::Missing),
            '<' => Some(Op::Lt),
            '>' => Some(Op::Gt),
            '{' => Some(Op::LexLt),
            '}' => Some(Op::LexGt),
            '^' => Some(Op::StartsWith),
            '$' => Some(Op::EndsWith),
            '~' => Some(Op::Contains),
            '#' => Some(Op::Comment),
            _ => None,
        }
    }

    /// Return a human-readable name for this operator.
    pub fn name(&self) -> &'static str {
        match self {
            Op::Eq => "equal",
            Op::Ne => "not equal",
            Op::Missing => "missing",
            Op::Lt => "less than",
            Op::Gt => "greater than",
            Op::LexLt => "lex less than",
            Op::LexGt => "lex greater than",
            Op::StartsWith => "starts with",
            Op::EndsWith => "ends with",
            Op::Contains => "contains",
            Op::Comment => "comment",
        }
    }

    /// Return the single-character symbol for this operator.
    pub fn as_char(&self) -> char {
        match self {
            Op::Eq => '=',
            Op::Ne => '/',
            Op::Missing => '!',
            Op::Lt => '<',
            Op::Gt => '>',
            Op::LexLt => '{',
            Op::LexGt => '}',
            Op::StartsWith => '^',
            Op::EndsWith => '$',
            Op::Contains => '~',
            Op::Comment => '#',
        }
    }
}

/// A single condition: field, operator, value.
///
/// For `Op::Missing`, value is empty.
/// For `Op::Comment`, field is the tag name, value is the tag content.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Condition {
    pub field: String,
    pub op: Op,
    pub value: String,
}

/// A disjunction (OR) of conditions. At least one must pass.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Restriction {
    pub alternatives: Vec<Condition>,
}

/// A conjunction (AND) of restrictions. All must pass.
///
/// This is the compiled output — a flat CNF representation ready for
/// conversion to a CLN rune or Biscuit token.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunePolicy {
    pub restrictions: Vec<Restriction>,
}
