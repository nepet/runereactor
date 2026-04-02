use crate::error::CompileError;
use crate::parser::Policy;
use crate::types::RunePolicy;

/// Compile a parsed policy AST into a flat `RunePolicy`.
///
/// This performs:
/// - Tag/Id → simple restrictions
/// - AllowMethods → single restriction with method=X alternatives
/// - When blocks → CNF normalization + negation bypass (method/M prepended)
/// - Global blocks → CNF normalization, restrictions applied directly
pub fn compile(policy: &Policy) -> Result<RunePolicy, CompileError> {
    todo!("Task 3 implements this")
}
