//! Simplification of applications of builtins.

use crate::Constants;
use crate::Context;

use hashbrown::HashMap;
use la_term::Term;
use la_term::symbol::Symbol;

macro_rules! builtins
{
    ($($name:ident)*) => {

        $(
            #[doc = "Simplification of applications of"]
            #[doc = concat!("`", stringify!($name), "`.")]
            pub mod $name;
        )*

        /// Implementation of a builtin.
        ///
        /// This simplifies an application of the builtin.
        /// The interface is similar to that of [`simplify_application`].
        ///
        /// [`simplify_application`]: `crate::simplify_application`
        pub type Builtin = fn(c: Context, arguments: &[Term]) -> Option<Term>;

        /// Lookup table for implementations of builtins.
        ///
        /// This is a hash table that associates some symbols
        /// with functions that simplify applications of those symbols.
        pub struct Builtins
        {
            jump_table: HashMap<Symbol, Builtin>,
        }

        impl Builtins
        {
            /// Collect all builtins into the table.
            pub fn new(constants: &Constants) -> Self
            {
                let mut jump_table = HashMap::<_, Builtin>::new();
                $(jump_table.insert(constants.$name.clone(), $name::simplify);)*
                Self{jump_table}
            }

            /// Get the implementation for a particular builtin.
            ///
            /// If the symbol does not refer to a builtin,
            /// this method returns [`None`].
            pub fn get(&self, symbol: &Symbol) -> Option<Builtin>
            {
                self.jump_table.get(symbol).copied()
            }
        }

    };
}

builtins! {
    Derivative
    Cos Sin
}
