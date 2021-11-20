//! Lexing and parsing of terms.

#![feature(decl_macro)]
#![no_std]
#![warn(missing_docs)]

extern crate alloc;

pub use self::error::*;
pub use self::lex::*;
pub use self::parse::*;
pub use self::scope::*;

mod error;
mod lex;
mod parse;
mod scope;
