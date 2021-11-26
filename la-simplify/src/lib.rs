#![no_std]
#![warn(missing_docs)]

pub use self::builtins::*;

use hashbrown::HashMap;
use la_term::Term;
use la_term::View;
use la_term::symbol::Symbol;
use la_term::symbol::Symbols;

mod builtins;

pub struct Context<'a>
{
    pub depth: usize,
    pub builtins: &'a Builtins,
    pub session: &'a Session,
    pub symbols: &'a Symbols,
    pub warner: &'a dyn Warner,
}

pub struct Session
{
    pub definitions: HashMap<Symbol, Term>,
}

impl Session
{
    pub fn new() -> Self
    {
        Self{definitions: HashMap::new()}
    }
}

pub trait Warner
{
}

pub fn recurse(mut c: Context, term: Term) -> Term
{
    c.depth += 1;
    simplify(c, term)
}

pub fn simplify(c: Context, term: Term) -> Term
{
    if c.depth >= 16 {
        return term;
    }

    match term.view() {

        View::Application(function, arguments) =>
            simplify_application(c, function, arguments)
            .unwrap_or(term),

        View::Symbol(symbol) =>
            simplify_symbol(c, symbol),

        // Variable substitution is performed during lambda application.
        // Lone variables encountered during simplification remain.
        View::Variable(..) => term,

        // Other types of terms do not simplify,
        // as they are already simple enough.
        View::Integer(..) => term,
        View::Lambda(..) => term,
        View::String(..) => term,

    }
}

pub fn simplify_application(c: Context, function: &Term, arguments: &[Term])
    -> Option<Term>
{
    if function.eq_symbol(&c.builtins.Sin) {
        return simplify_Sin(c, arguments);
    }

    None
}

#[allow(non_snake_case)]
pub fn simplify_Sin(c: Context, arguments: &[Term]) -> Option<Term>
{
    if arguments.len() != 1 {
        // TODO: Warn about arity of Sin.
        return None;
    }

    let operand = &arguments[0];

    if operand.eq_symbol(&c.builtins.Pi) {
        return Some(Term::integer_i32(0).unwrap());
    }

    None
}

pub fn simplify_symbol(c: Context, symbol: &Symbol) -> Term
{
    // Look up the definition of the symbol.
    let definition = c.session.definitions.get(symbol);

    match definition {

        // If the symbol is not defined, then there is nothing to do.
        None => Term::symbol(symbol.clone()),

        // A definition like `E := E` indicates that `E` is a reserved symbol.
        // In this case we should not recurse, and retain the symbol itself.
        Some(definition) if definition.eq_symbol(symbol) =>
            Term::symbol(symbol.clone()),

        // Otherwise the definition is simplified.
        Some(definition) => recurse(c, definition.clone()),

    }
}
