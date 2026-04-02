use crate::error::ParseError;

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
        op: crate::types::Op,
        value: String,
    },
    /// `expr or expr` — disjunction
    Or(Vec<Expr>),
    /// `expr and expr` — conjunction (also implicit between lines)
    And(Vec<Expr>),
}

/// Parse an `.rf` policy file into an AST.
pub fn parse_policy(input: &str) -> Result<Policy, ParseError> {
    todo!("Task 2 implements this")
}
