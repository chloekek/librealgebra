//! Data type for terms.

#![feature(trusted_len)]
#![no_std]
#![warn(missing_docs)]

extern crate alloc;

pub use self::de_bruijn::*;

use self::byte_string::ByteString;

use alloc::rc::Rc;
use alloc::vec::Vec;
use core::fmt;
use core::iter::TrustedLen;
use num_bigint::BigUint;

mod de_bruijn;
mod byte_string;

/// Reference-counted handle to a term.
#[derive(Clone)]
pub struct Term
{
    inner: Rc<Inner>,
}

struct Inner
{
    payload: Payload,
}

#[derive(Debug)]
enum Payload
{
    Application(Term, Vec<Term>),
    Integer(BigUint),
    Lambda(Vec<(Strictness, ByteString)>, Term),
    String(ByteString),
    Symbol(ByteString),
    Variable(DeBruijn),
}

/// When to evaluate a lambda argument.
#[derive(Clone, Copy, Debug)]
pub enum Strictness
{
    /// Immediately evaluate the argument when applying the lambda.
    Strict,

    /// Substitute the argument into the lambda body as-is.
    NonStrict,
}

impl Term
{
    /// Create an application term.
    pub fn application(function: Term, arguments: &[Term]) -> Self
    {
        let payload = Payload::Application(function, arguments.to_vec());
        Self{inner: Rc::new(Inner{payload})}
    }

    /// Create an integer term.
    pub fn integer(value: BigUint) -> Self
    {
        Self{inner: Rc::new(Inner{payload: Payload::Integer(value)})}
    }

    /// Create a lambda term.
    pub fn lambda<I, J, N>(parameters: I, body: Term) -> Self
        where I: IntoIterator<IntoIter=J>
            , J: Iterator<Item=(Strictness, N)> + ExactSizeIterator + TrustedLen
            , N: AsRef<[u8]>
    {
        let parameters =
            parameters
            .into_iter()
            .map(|(s, n)| (s, ByteString::from(n.as_ref().to_vec())))
            .collect();
        Self{inner: Rc::new(Inner{payload: Payload::Lambda(parameters, body)})}
    }

    /// Create a string term.
    pub fn string(value: &[u8]) -> Self
    {
        let value = ByteString::from(value.to_vec());
        Self{inner: Rc::new(Inner{payload: Payload::String(value)})}
    }

    /// Create a symbol term.
    pub fn symbol(name: &[u8]) -> Self
    {
        let name = ByteString::from(name.to_vec());
        Self{inner: Rc::new(Inner{payload: Payload::Symbol(name)})}
    }

    /// Create a variable term.
    pub fn variable(de_bruijn: DeBruijn) -> Self
    {
        Self{inner: Rc::new(Inner{payload: Payload::Variable(de_bruijn)})}
    }
}

impl fmt::Debug for Term
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        self.inner.fmt(f)
    }
}

impl fmt::Debug for Inner
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        self.payload.fmt(f)
    }
}
