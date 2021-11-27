//! Working with variable terms.
//!
//! The payload of a variable term contains one word,
//! which records the De Bruijn index of the variable.

use super::Header;
use super::Kind;
use super::Payload;
use super::Term;
use super::View;

use core::ops::Add;

/// A De Bruijn index references a variable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeBruijn(pub u32);

impl Add<u32> for DeBruijn
{
    type Output = DeBruijn;

    fn add(self, rhs: u32) -> Self::Output
    {
        DeBruijn(self.0 + rhs)
    }
}

/// Pointers to the words in the payload of a variable term.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct UnsafeView
{
    pub de_bruijn: *mut DeBruijn,
}

impl UnsafeView
{
    /// Obtain the pointers to the words in the payload of a variable term.
    pub fn new(payload: *mut Payload) -> Self
    {
        Self{de_bruijn: payload as *mut DeBruijn}
    }
}

impl Term
{
    /// Create a variable term.
    pub fn variable(de_bruijn: DeBruijn) -> Self
    {
        // `DeBruijn` is 32 bits. A word is always at least 32 bits.
        let payload_words = 1;
        unsafe {
            Self::new(payload_words, |payload| {
                let view = UnsafeView::new(payload);
                view.de_bruijn.write(de_bruijn);
                Header::new(Kind::Variable)
            })
        }
    }

    /// View a variable term.
    ///
    /// # Safety
    ///
    /// The term must be a variable term.
    pub unsafe fn as_variable_unchecked(&self) -> DeBruijn
    {
        let payload = self.payload();
        let view = UnsafeView::new(payload);
        *view.de_bruijn
    }

    /// Whether this is that specific variable term.
    pub fn eq_variable(&self, de_bruijn: DeBruijn) -> bool
    {
        match self.view() {
            View::Variable(var) => var == de_bruijn,
            _ => false,
        }
    }
}
