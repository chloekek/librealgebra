use crate::Context;
use crate::recurse;

use la_term::Term;
use la_term::View;
use la_term::variable::DeBruijn;

/// Simplify an application of `Derivative`.
pub fn simplify(c: &Context, arguments: &[Term]) -> Option<Term>
{
    if arguments.len() != 1 {
        // TODO: Warn about arity of Derivative.
        return None;
    }

    // TODO: Support multi-parameter functions.
    // If arguments are given to Derivative,
    // check that they are a symbol and a lambda.

    let original_function = &arguments[0];

    let function = recurse(c, original_function.clone());

    if let Some(derivative) = function_derivative(c, function.clone()) {
        return Some(derivative);
    }

    if function.ptr_eq(&original_function) {
        None
    } else {
        let Derivative = c.constants.Derivative.term();
        Some(Term::application(Derivative, [function]))
    }
}

/// Find the derivative of `function`, which must be a unary function.
fn function_derivative(c: &Context, function: Term) -> Option<Term>
{
    if function.eq_symbol(&c.constants.Sin) {
        return Some(c.constants.Cos.term());
    }

    if function.eq_symbol(&c.constants.Cos) {
        return Some(c.constants.lambda_neg_Sin.clone());
    }

    if let View::Lambda(parameters, body) = function.view() {
        if parameters.len() == 1 {
            let body_derivative = term_derivative(c, DeBruijn(0), body.clone())?;
            return Some(Term::lambda(parameters.clone(), body_derivative));
        }
    }

    None
}

/// Find the derivative of `term` with respect to `parameter`.
fn term_derivative(c: &Context, parameter: DeBruijn, term: Term)
    -> Option<Term>
{
    if term.header().de_bruijn_cache.contains(parameter) == Some(false) {
        return Some(c.constants.integer_0.clone());
    }

    if term.eq_variable(parameter) {
        return Some(c.constants.integer_1.clone());
    }

    None
}
