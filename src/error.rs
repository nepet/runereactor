use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("line {line}: {message}")]
    Syntax { line: usize, message: String },

    #[error("{0}")]
    InvalidExpression(String),
}

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("{0}")]
    InvalidPolicy(String),
}
