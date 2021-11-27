//! Functions for simplifying terms.

// Many places in the code refer to Libre Algebra symbols.
// These symbols are in Pascal case, so we disable this warning.
#![allow(non_snake_case)]

#![no_std]
#![warn(missing_docs)]

extern crate alloc;

pub use self::constants::*;

use self::builtins::Builtins;

use hashbrown::HashMap;
use la_term::Term;
use la_term::View;
use la_term::symbol::Symbol;
use la_term::symbol::Symbols;

pub mod builtins;

mod constants;

/// Information threaded through the simplifier.
///
/// This provides access to commonly used objects.
/// See the documentation for the type of each field
/// to learn more about what they mean.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct Context<'a>
{
    /// Decremented on each call to [`recurse`].
    /// Once zero, simplification no longer descends.
    pub recursion_limit: usize,

    pub builtins: &'a Builtins,
    pub constants: &'a Constants,
    pub session: &'a Session,
    pub symbols: &'a Symbols,
    pub warner: &'a dyn Warner,
}

/// Per-session state such as global definitions.
pub struct Session
{
    /// Global definitions, as created with `:=`.
    pub definitions: HashMap<Symbol, Term>,
}

impl Session
{
    /// Create a session with no definitions.
    pub fn new() -> Self
    {
        Self{definitions: HashMap::new()}
    }
}

/// Object for emitting warnings.
pub trait Warner
{
}

/// Decrement [`recursion_limit`], then call [`simplify`].
///
/// [`recursion_limit`]: `Context::recursion_limit`
pub fn recurse(mut c: Context, term: Term) -> Term
{
    c.recursion_limit -= 1;
    simplify(c, term)
}

/// Apply simplification rules to a term.
///
/// This does not necessarily reduce the term to a “most simple” form.
/// The exact semantics can be found in the Libre Algebra manual.
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

/// Simplify an application term.
pub fn simplify_application(c: Context, function: &Term, arguments: &[Term])
    -> Option<Term>
{
    // First simplify the function itself.
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

/// Simplify a symbol term.
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
