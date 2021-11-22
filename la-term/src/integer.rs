//! Working with integer terms.
//!
//! TODO: Document layout of integer payloads.

use crate::AllocError;
use crate::Term;

use num_bigint::BigUint;

impl Term
{
    /// Create an integer term.
    pub fn integer(_value: BigUint) -> Result<Self, AllocError>
    {
        todo!()
    }
}
