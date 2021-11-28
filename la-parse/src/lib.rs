//! Lexing and parsing of terms.

#![feature(decl_macro)]
#![warn(missing_docs)]

pub use self::error::*;
pub use self::lex::*;
pub use self::parse::*;
pub use self::scope::*;

mod error;
mod lex;
mod parse;
mod scope;
