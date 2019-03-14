use crate::ast::{up_var_rc, Branch, Closure, Expression, Pattern, Value};
use crate::check::decl::check_declaration;
use crate::check::read_back::generate_value;
use crate::check::subtype::check_subtype;
use crate::check::tcm::{update_gamma, update_gamma_borrow, TCE, TCM, TCS};
use std::collections::BTreeMap;

/// `checkI` in Mini-TT.<br/>
/// Type inference rule. More inferences are added here (maybe it's useful?).
pub fn check_infer(index: u32, tcs: TCS, expression: Expression) -> TCM<Value> {
    use crate::ast::Expression::*;
    match expression {
        Unit => Ok(Value::One),
        Type | Void | One => Ok(Value::Type),
        Var(name) => tcs
            .0
            .get(&name)
            .cloned()
            .ok_or_else(|| TCE::UnresolvedName(name)),
        Constructor(name, expression) => {
            let mut map = BTreeMap::new();
            map.insert(name, Box::new(check_infer(index, tcs, *expression)?));
            Ok(Value::InferredSum(Box::new(map)))
        }
        Pair(left, right) => {
            let left = check_infer(index, tcs_borrow!(tcs), *left)?;
            let right = check_infer(index, tcs_borrow!(tcs), *right)?;
            Ok(Value::Sigma(
                Box::new(left),
                Closure::Value(Box::new(right)),
            ))
        }
        First(pair) => match check_infer(index, tcs, *pair)? {
            Value::Sigma(first, _) => Ok(*first),
            e => Err(TCE::WantSigmaBut(e)),
        },
        Second(pair) => {
            let (gamma, context) = tcs;
            match check_infer(index, (gamma, context.clone()), *pair.clone())? {
                Value::Sigma(_, second) => Ok(second.instantiate(pair.eval(context).first())),
                e => Err(TCE::WantSigmaBut(e)),
            }
        }
        Sum(branches) => {
            for (_name, branch) in branches.into_iter() {
                check_type(index, tcs_borrow!(tcs), *branch)?;
            }
            Ok(Value::Type)
        }
        Pi((pattern, input), output) | Sigma((pattern, input), output) => {
            let (gamma, context) = check_type(index, tcs, *input.clone())?;
            let input_type = input.eval(context.clone());
            let generated = generate_value(index);
            let gamma = update_gamma(gamma, &pattern, input_type, generated)?;
            check_type(index + 1, (gamma, context), *output)?;
            Ok(Value::Type)
        }
        Application(function, argument) => match *function {
            Lambda(pattern, Some(parameter_type), return_value) => {
                let parameter_type = *parameter_type.internal;
                check(index, tcs_borrow!(tcs), *argument, parameter_type.clone())?;
                let generated = generate_value(index + 1);
                let gamma = update_gamma_borrow(tcs.0, &pattern, parameter_type, &generated)?;
                let context = up_var_rc(tcs.1, pattern, generated);
                check_infer(index + 1, (gamma, context), *return_value)
            }
            f => match check_infer(index, tcs_borrow!(tcs), f)? {
                Value::Pi(input, output) => {
                    let (gamma, context) = tcs;
                    check(index, (gamma, context.clone()), *argument.clone(), *input)?;
                    Ok(output.instantiate(argument.eval(context)))
                }
                e => Err(TCE::WantPiBut(e, *argument)),
            },
        },
        Declaration(_, _) | Constant(_, _, _) => Err(tce_unreachable!()),
        e => Err(TCE::CannotInfer(e)),
    }
}

/// `checkT` in Mini-TT.<br/>
/// Check if an expression is a well-typed type expression.
pub fn check_type(index: u32, tcs: TCS, expression: Expression) -> TCM<TCS> {
    use crate::ast::Expression::*;
    match expression {
        Sum(constructors) => check_sum_type(index, tcs, constructors),
        Pi((pattern, first), second) | Sigma((pattern, first), second) => {
            check_telescoped(index, tcs, pattern, *first, *second)
        }
        Type | Void | One => Ok(tcs),
        expression => check(index, tcs, expression, Value::Type),
    }
}

/// `check` in Mini-TT.<br/>
/// However, telescope and gamma are preserved for REPL use.
pub fn check(index: u32, tcs: TCS, expression: Expression, value: Value) -> TCM<TCS> {
    use crate::ast::Expression as E;
    use crate::ast::Value as V;
    match (expression, value) {
        (E::Unit, V::One) | (E::Type, V::Type) | (E::One, V::Type) => Ok(tcs),
        // There's nothing left to check.
        (E::Void, _) => Ok(tcs),
        (E::Lambda(pattern, _, body), V::Pi(signature, closure)) => {
            let (gamma, context) = tcs;
            let generated = generate_value(index);
            let gamma = update_gamma_borrow(gamma, &pattern, *signature, &generated)?;
            check(
                index + 1,
                (gamma, up_var_rc(context, pattern, generated.clone())),
                *body,
                closure.instantiate(generated),
            )
        }
        (E::Pair(first, second), V::Sigma(first_type, second_type)) => {
            check(index, tcs_borrow!(tcs), *first.clone(), *first_type)?;
            let (gamma, context) = tcs;
            check(
                index,
                (gamma, context.clone()),
                *second,
                second_type.instantiate(first.eval(context)),
            )
        }
        (E::Constructor(name, body), V::Sum((constructors, telescope))) => {
            let constructor = *constructors
                .get(&name)
                .ok_or_else(|| TCE::InvalidConstructor(name))?
                .clone();
            check(index, tcs, *body, constructor.eval(*telescope))
        }
        (E::Sum(constructors), V::Type) => check_sum_type(index, tcs, constructors),
        (E::Sigma((pattern, first), second), V::Type)
        | (E::Pi((pattern, first), second), V::Type) => {
            check_telescoped(index, tcs, pattern, *first, *second)
        }
        (E::Declaration(declaration, rest), rest_type) => {
            let tcs = check_declaration(index, tcs, *declaration)?;
            check(index, tcs, *rest, rest_type)
        }
        (E::Constant(pattern, body, rest), rest_type) => {
            let signature = check_infer(index, tcs_borrow!(tcs), *body.clone())?;
            let (gamma, context) = tcs;
            let body_val = body.eval(context.clone());
            let gamma = update_gamma_borrow(gamma, &pattern, signature, &body_val)?;
            let context = up_var_rc(context, pattern, body_val);
            check(index, (gamma, context), *rest, rest_type)
        }
        // I really wish to have box pattern here :(
        (E::Split(mut branches), V::Pi(sum, closure)) => match *sum {
            V::Sum((sum_branches, telescope)) => {
                for (name, branch) in sum_branches.into_iter() {
                    let pattern_match = match branches.remove(&name) {
                        Some(pattern_match) => *pattern_match,
                        None => return Err(TCE::MissingCase(name)),
                    };
                    check(
                        index,
                        tcs_borrow!(tcs),
                        pattern_match,
                        V::Pi(
                            Box::new(branch.eval(*telescope.clone())),
                            Closure::Choice(Box::new(closure.clone()), name.clone()),
                        ),
                    )?;
                }
                if branches.is_empty() {
                    Ok(tcs)
                } else {
                    let clauses: Vec<_> = branches.keys().map(|br| br.as_str()).collect();
                    Err(TCE::UnexpectedCases(clauses.join(" | ")))
                }
            }
            not_sum_so_fall_through => check_fallback(
                index,
                tcs,
                E::Split(branches),
                V::Pi(Box::new(not_sum_so_fall_through), closure),
            ),
        },
        (expression, value) => check_fallback(index, tcs, expression, value),
    }
}

/// Fallback rule of instance check.<br/>
/// First infer the expression type, then do subtyping comparison.
fn check_fallback(index: u32, tcs: TCS, body: Expression, signature: Value) -> TCM<TCS> {
    let inferred = check_infer(index, tcs_borrow!(tcs), body)?;
    check_subtype(index, tcs, inferred, signature)
}

/// To reuse code that checks if a sum type is well-typed between `check_type` and `check`
pub fn check_sum_type(index: u32, tcs: TCS, constructors: Branch) -> TCM<TCS> {
    for constructor in constructors.values().cloned() {
        check_type(index, tcs_borrow!(tcs), *constructor)?;
    }
    Ok(tcs)
}

/// To reuse code that checks if a sigma or a pi type is well-typed between `check_type` and `check`
pub fn check_telescoped(
    index: u32,
    tcs: TCS,
    pattern: Pattern,
    first: Expression,
    second: Expression,
) -> TCM<TCS> {
    check_type(index, tcs_borrow!(tcs), first.clone())?;
    let (gamma, context) = tcs;
    let generated = generate_value(index);
    let gamma = update_gamma(
        gamma,
        &pattern,
        first.eval(context.clone()),
        generated.clone(),
    )?;
    check_type(
        index + 1,
        (gamma, up_var_rc(context, pattern, generated)),
        second,
    )
}
