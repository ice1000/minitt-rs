/// Normal form: when we read back, we get a normal form expression.
///
/// Depends on module `syntax`.
pub mod normal;

/// Read back: read back functions.
///
/// Converting terms to normal forms with de-bruijn indices so
/// we do not need to deal with alpha conversions.
///
/// Functions in this module are put into `impl for` blocks, their docs can be found in:
///
/// + [`ReadBack` of `Value`](../syntax/enum.Value.html#impl-ReadBack)
/// + [`ReadBack` of `Telescope`](../syntax/enum.Telescope.html#impl-ReadBack)
/// + [`ReadBack` of `Closure`](../syntax/enum.Closure.html#impl-ReadBack)
///
/// Depends on modules `syntax` and `normal`.
pub mod read_back;

/// Type-Checking Monad: context, state and error.
///
/// Typing context (`Gamma`) and its updater, the type-checking error and its pretty-printer
///
/// Depends on module `syntax`.
pub mod tcm;

/// Expression checker: infer, instance-of check, normal-form comparison, subtyping, etc.
///
/// Depends on modules `syntax`, `read_back` and `tcm`.
pub mod expr;

/// Declaration checker: for prefix parameters, simple declarations and recursive declarations.
///
/// Depends on modules `syntax`, `expr` and `tcm`.
pub mod decl;

use crate::ast::{Declaration, Expression, Telescope, Value};
use crate::check::decl::check_declaration;
use crate::check::expr::{check, check_infer};
use crate::check::tcm::{default_state, Gamma, TCM, TCS};

/// `checkMain` in Mini-TT.
pub fn check_main<'a>(expression: Expression) -> TCM<TCS<'a>> {
    check_contextual(default_state(), expression)
}

/// For REPL: check an expression under an existing context
pub fn check_contextual(tcs: TCS, expression: Expression) -> TCM<TCS> {
    check(0, tcs, expression, Value::One)
}

/// For REPL: infer the type of an expression under an existing context
pub fn check_infer_contextual(tcs: TCS, expression: Expression) -> TCM<Value> {
    check_infer(0, tcs, expression)
}

/// Similar to `checkMain` in Mini-TT, but for a declaration.
pub fn check_declaration_main<'a>(declaration: Declaration) -> TCM<(Gamma<'a>, Option<Telescope>)> {
    check_declaration(0, default_state(), declaration)
}

#[cfg(test)]
mod tests;
