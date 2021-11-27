use crate::Context;
use crate::recurse;

use la_term::Term;

/// Simplify an application of `Cos`.
pub fn simplify(c: Context, arguments: &[Term]) -> Option<Term>
{
    if arguments.len() != 1 {
        // TODO: Warn about arity of Cos.
        return None;
    }

    let original_operand = &arguments[0];

    let operand = recurse(c, original_operand.clone());

    if operand.eq_integer_i32(0) {
        return Some(Term::integer_i32(1));
    }

    if operand.eq_symbol(&c.names.Pi) {
        return Some(Term::integer_i32(-1));
    }

    if operand.ptr_eq(&original_operand) {
        None
    } else {
        let Cos = Term::symbol(c.names.Cos.clone());
        Some(Term::application(Cos, [operand]))
    }
}
