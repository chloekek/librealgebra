use la_term::Term;
use la_term::lambda::Parameter;
use la_term::lambda::Strictness;
use la_term::symbol::Symbol;
use la_term::symbol::Symbols;
use la_term::variable::DeBruijn;
use std::rc::Rc;

macro_rules! parameter
{
    ( $name:ident ) => {
        Parameter{
            strictness: Strictness::Strict,
            name: $name.clone(),
        }
    };

    ( ~ $name:ident ) => {
        Parameter{
            strictness: Strictness::NonStrict,
            name: $name.clone(),
        }
    };
}

macro_rules! constants
{
    (

        integers! {
            $($integer_name:ident = $integer_value:literal;)*
        }

        parameters! {
            $(
                $parameters_name:ident = |
                    $(
                        $parameter_1st_token:tt
                        $($parameter_2nd_token:ident)?
                    ),*
                    $(,)?
                |;
            )*
        }

        symbols! {
            $($symbol_name:ident)*
        }

        variables! {
            $($variable_name:ident = $variable_de_bruijn:literal;)*
        }

        custom! {
            $(
                $(#[doc = $custom_doc:tt])*
                $custom_name:ident = $custom_init:expr;
            )*
        }

    ) => {

        /// Heap-allocated objects that are used throughout the simplifier.
        ///
        /// The objects are created once and then reused whenever needed.
        /// This significantly cuts down on allocations and memory usage.
        ///
        /// Not all fields of this struct are individually documented.
        /// Integers are named after their values (with `neg_` if negative).
        /// Parameter lists are named after the names of the parameters,
        /// with each parameter prefixed `s` or `n` for strict or non-strict.
        /// Symbols are named after the symbol they contain.
        /// Variables are named after their De Bruijn indices.
        #[allow(missing_docs)]
        pub struct Constants
        {
            $(pub $integer_name: Term,)*
            $(pub $parameters_name: Rc<[Parameter]>,)*
            $(pub $symbol_name: Symbol,)*
            $(pub $variable_name: Term,)*
            $(
                $(#[doc = $custom_doc])*
                pub $custom_name: Term,
            )*
        }

        impl Constants
        {
            /// Create the table of constants.
            pub fn new(symbols: &Symbols) -> Self
            {
                $(
                    let $symbol_name = symbols.get(
                        stringify!($symbol_name).as_bytes(),
                    );
                )*
                $(
                    let $parameters_name = Rc::from([
                        $(
                            parameter!(
                                $parameter_1st_token
                                $($parameter_2nd_token)?
                            ),
                        )*
                    ]);
                )*
                $(
                    let $integer_name = Term::integer_i32($integer_value);
                )*
                $(
                    let $variable_name = Term::variable(
                        DeBruijn($variable_de_bruijn),
                    );
                )*
                $(
                    let $custom_name = $custom_init;
                )*
                Self{
                    $($integer_name,)*
                    $($parameters_name,)*
                    $($symbol_name,)*
                    $($variable_name,)*
                    $($custom_name,)*
                }
            }
        }

    };
}

constants! {

    integers! {
        integer_neg_1 = -1;
        integer_0 = 0;
        integer_1 = 1;
    }

    parameters! {
        parameters_sx = |x|;
        parameters_nx = |~x|;
    }

    symbols! {
        Antiderivative Derivative
        Add Ln Multiply Power
        Cos Sin Tan
        E Pi
        x
    }

    variables! {
        variable_0 = 0;
    }

    custom! {
        /// ```librealgebra
        /// |x| Multiply(-1, Sin(x))
        /// ```
        lambda_neg_Sin = Term::lambda(
            parameters_sx.clone(),
            Term::application(
                Multiply.term(),
                [
                    integer_neg_1.clone(),
                    Term::application(Sin.term(), [variable_0.clone()]),
                ],
            ),
        );
    }

}
