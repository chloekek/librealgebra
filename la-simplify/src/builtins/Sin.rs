use crate::Context;
use crate::recurse;

use la_term::Term;

/// Simplify an application of `Sin`.
pub fn simplify(c: &Context, arguments: &[Term]) -> Option<Term>
{
    if arguments.len() != 1 {
        // TODO: Warn about arity of Sin.
        return None;
    }

    let original_operand = &arguments[0];

    let operand = recurse(c, original_operand.clone());

    if operand.eq_integer_i32(0) {
        return Some(c.constants.integer_0.clone());
    }

    if operand.eq_symbol(&c.constants.Pi) {
        return Some(c.constants.integer_0.clone());
    }

    if operand.ptr_eq(&original_operand) {
        None
    } else {
        let Sin = c.constants.Sin.term();
        Some(Term::application(Sin, [operand]))
    }
}
