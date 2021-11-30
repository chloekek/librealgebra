use crate::Context;
use crate::recurse;

use la_term::Term;
use la_term::View;
use la_term::variable::DeBruijn;
use smallvec::SmallVec;
use std::iter::TrustedLen;

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

    if let Some(derivative) = of_function(c, function.clone()) {
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
pub fn of_function(c: &Context, function: Term) -> Option<Term>
{
    if function.eq_symbol(&c.constants.Sin) {
        return Some(c.constants.Cos.term());
    }

    if function.eq_symbol(&c.constants.Cos) {
        return Some(c.constants.lambda_neg_Sin.clone());
    }

    if let View::Lambda(parameters, body) = function.view() {
        if parameters.len() == 1 {
            let body_derivative = of_term(c, DeBruijn(0), body.clone())?;
            return Some(Term::lambda(parameters.clone(), body_derivative));
        }
    }

    None
}

/// Find the derivative of `term` with respect to `parameter`.
pub fn of_term(c: &Context, parameter: DeBruijn, term: Term)
    -> Option<Term>
{
    if is_constant(parameter, &term) == Some(true) {
        return Some(c.constants.integer_0.clone());
    }

    if term.eq_variable(parameter) {
        return Some(c.constants.integer_1.clone());
    }

    if let View::Application(function, arguments) = term.view() {
        if function.eq_symbol(&c.constants.Add) {
            return of_add(c, parameter, arguments);
        }
    }

    None
}

/// Find the derivative of the sum of `terms` with respect to `parameter`.
pub fn of_add(c: &Context, parameter: DeBruijn, terms: &[Term]) -> Option<Term>
{
    let terms = (
        terms.iter()

        // Let’s skip constant terms as differentiating them results in zero,
        // which will then end up in our addition and contribute nothing.
        .filter(|term| is_constant(parameter, term) == Some(false))

        .map(|term| of_term(c, parameter, term.clone()))
        .collect::<Option<SmallVec<[_; 8]>>>()?
    );
    let result = make_add(c, terms);
    return Some(recurse(c, result));
}

/// Whether `term` is a constant with respect to `parameter`
/// for the purpose of differentiation.
///
/// If this method returns `Some`, the answer is correct.
/// Otherwise, it is unknown whether `term` is a constant.
pub fn is_constant(parameter: DeBruijn, term: &Term) -> Option<bool>
{
    term.header().de_bruijn_cache
        .contains(parameter)
        .map(|c| c == false)
}

fn make_add<I, J>(c: &Context, terms: I) -> Term
    where I: IntoIterator<IntoIter=J>
        , J: Iterator<Item=Term> + ExactSizeIterator + TrustedLen
{
    let mut terms = terms.into_iter();
    match terms.len() {
        0 => c.constants.integer_0.clone(),
        1 => terms.next().unwrap(),
        _ => Term::application(c.constants.Add.term(), terms),
    }
}
