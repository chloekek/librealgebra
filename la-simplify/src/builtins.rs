use la_term::AllocError;
use la_term::symbol::Symbol;
use la_term::symbol::Symbols;

macro_rules! builtins
{
    ($($name:ident)*) => {
        /// Each symbol that needs special treatment in the simplifier.
        #[allow(missing_docs)]
        #[allow(non_snake_case)]
        pub struct Builtins
        {
            $(pub $name: Symbol,)*
        }

        impl Builtins
        {
            /// Obtain symbols for all builtins.
            pub fn new(symbols: &Symbols) -> Result<Self, AllocError>
            {
                let this = Self{
                    $($name: symbols.get(stringify!($name).as_bytes())?,)*
                };
                Ok(this)
            }
        }
    };
}

builtins! {
    Antiderivative Derivative
    Add Ln Multiply Power
    Cos Sin Tan
    E Pi
}
