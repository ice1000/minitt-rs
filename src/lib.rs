/*!
Reading the [README](https://github.com/owo-lang/minitt-rs/blob/master/README.md) is recommended
before reading the documentation.
*/

/// Syntax: term, expression, context.
///
/// Methods are defined in `reduce`/`read_back` modules but their documents are here.
///
/// No dependency.
pub mod ast;
/// Reduction: eval and eval's friends.
///
/// Functions in this module are put into `impl` blocks, their docs can be found in:
///
/// + [Methods of `Pattern`](../syntax/enum.Pattern.html#methods)
/// + [Methods of `Value`](../syntax/enum.Value.html#methods)
/// + [Methods of `Telescope`](../syntax/enum.Telescope.html#methods)
/// + [Methods of `Closure`](../syntax/enum.Closure.html#methods)
/// + [Methods of `Expression`](../syntax/enum.Expression.html#methods)
///
/// Depends on module `syntax`.
pub mod eval;

/// Type checking: everything related to type-checking, including:<br/>
/// + Normal form and read-back functions
/// + The four type checking functions -- `checkI`, `checkD`, `check` and `checkT`.
/// + (extended) (sub)typing rules
///
/// Depends on module `syntax`.
pub mod check;

/// Pretty print utilities
#[cfg(feature = "pretty")]
pub mod pretty;

/// Parser, from text to AST and a bunch of utilities
#[cfg(feature = "parser")]
pub mod parser;
