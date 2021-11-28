use crate::Context;
use crate::recurse;

use la_term::Term;

/// Simplify an application of `Cos`.
pub fn simplify(c: &Context, arguments: &[Term]) -> Option<Term>
{
    if arguments.len() != 1 {
        // TODO: Warn about arity of Cos.
        return None;
    }

    let original_operand = &arguments[0];

    let operand = recurse(c, original_operand.clone());

    if operand.eq_integer_i32(0) {
        return Some(c.constants.integer_1.clone());
    }

    if operand.eq_symbol(&c.constants.Pi) {
        return Some(c.constants.integer_neg_1.clone());
    }

    if operand.ptr_eq(&original_operand) {
        None
    } else {
        let Cos = c.constants.Cos.term();
        Some(Term::application(Cos, [operand]))
    }
}
