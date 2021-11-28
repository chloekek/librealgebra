use la_term::symbol::Symbol;
use la_term::variable::DeBruijn;
use std::collections::HashMap;

/// Mapping from identifiers to De Bruijn indices.
///
/// When the parser encounters an identifier token,
/// it must decide whether it is a symbol or a variable.
/// To do this, scopes record which variables are in scope.
/// The parser creates a new scope each time a set of variables is bound.
pub struct Scope<'a>
{
    parent: Option<&'a Scope<'a>>,
    variables: HashMap<Symbol, DeBruijn>,
}

impl<'a> Scope<'a>
{
    /// Create a scope with the given variables.
    ///
    /// The new scope is derived from the given parent scope, if any.
    /// The variables are assigned De Bruijn indices starting at zero.
    /// Variables in the parent scope are renumbered on demand by `get`.
    pub fn new<I>(parent: Option<&'a Scope>, variables: I) -> Self
        where I: IntoIterator<Item=Symbol>
    {
        let variables =
            variables
            .into_iter()
            .enumerate()
            .map(|(i, name)| (name, DeBruijn(i as u32)))
            .collect();
        Self{parent, variables}
    }

    /// Find the De Bruijn index corresponding to the given name.
    ///
    /// If there is no variable with that name in scope,
    /// then this method will return `None`.
    /// This should be interpreted as there not being
    /// a bound variable with that name in scope.
    pub fn get(mut self: &Self, name: &Symbol) -> Option<DeBruijn>
    {
        // The compiler would not optimize the tail recursive implementation.
        // So we write an iterative version manually (hence mut self).

        // The shift is added to the De Bruijn index.
        // It is incremented when entering into a parent scope.
        let mut shift = 0;

        loop {
            if let Some(&variable) = self.variables.get(name) {
                break Some(variable + shift);
            } else if let Some(parent) = self.parent {
                self = parent;
                shift += self.variables.len() as u32;
                continue;
            } else {
                break None;
            }
        }
    }
}
