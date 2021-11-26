use la_term::symbol::Symbol;
use la_term::symbol::Symbols;

macro_rules! names
{
    ($($name:ident)*) => {

        /// Each symbol that needs special treatment in the simplifier.
        ///
        /// In many places the simplifier treats some symbols specially.
        /// For example when looking up implementations of builtins,
        /// or to recognize certain identities involving constants.
        /// This struct contains a field for each such symbol.
        /// No hash table lookups are required to access them!
        #[allow(missing_docs)]
        pub struct Names
        {
            $(pub $name: Symbol,)*
        }

        impl Names
        {
            /// Look up each special symbol and construct the table.
            pub fn new(symbols: &Symbols) -> Self
            {
                Self{
                    $($name: symbols.get(stringify!($name).as_bytes()),)*
                }
            }
        }

    };
}

names! {
    Antiderivative Derivative
    Add Ln Multiply Power
    Cos Sin Tan
    E Pi
}
