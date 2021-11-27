use crate::Context;
use crate::recurse;

use la_term::Term;

/// Simplify an application of `Derivative`.
pub fn simplify(c: Context, arguments: &[Term]) -> Option<Term>
{
    if arguments.len() != 1 {
        // TODO: Warn about arity of Derivative.
        return None;
    }

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

fn function_derivative(c: Context, function: Term) -> Option<Term>
{
    if function.eq_symbol(&c.constants.Sin) {
        return Some(c.constants.Cos.term());
    }

    if function.eq_symbol(&c.constants.Cos) {
        return Some(c.constants.lambda_neg_Sin.clone());
    }

    None
}
