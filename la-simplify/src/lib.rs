// Many places in the code refer to Libre Algebra symbols.
// These symbols are in Pascal case, so we disable this warning.
#![allow(non_snake_case)]

#![no_std]
#![warn(missing_docs)]

pub use self::names::*;

use self::builtins::Builtins;

use hashbrown::HashMap;
use la_term::Term;
use la_term::View;
use la_term::symbol::Symbol;
use la_term::symbol::Symbols;

pub mod builtins;

mod names;

#[derive(Clone, Copy)]
pub struct Context<'a>
{
    pub recursion_limit: usize,
    pub builtins: &'a Builtins,
    pub names: &'a Names,
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
    c.recursion_limit -= 1;
    simplify(c, term)
}

pub fn simplify(c: Context, term: Term) -> Term
{
    if c.recursion_limit == 0 {
        // TODO: Emit warning about recursion depth reached.
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
    /// First simplify the function itself.
    let function = recurse(c, function.clone());

    match function.view() {

        // If the function is a symbol even after simplification,
        // then it can only be a builtin or not a function at all.
        // User-defined symbols should evaluate to their definitions.
        View::Symbol(builtin) =>
            c.builtins.get(builtin)
                .and_then(|b| b(c, arguments)),

        _ => todo!(),

    }
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
