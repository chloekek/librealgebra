//! Working with integer terms.
//!
//! The payload of an integer term contains one word,
//! which is the value of the integer as an `i32`.
//! In the future this type should support integers of arbitrary size.

use crate::Header;
use crate::Kind;
use crate::Payload;
use crate::Term;
use crate::View;

/// Pointers to the words in the payload of an integer term.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct UnsafeView
{
    pub value: *mut i32,
}

impl UnsafeView
{
    /// Obtain the pointers to the words in the payload of an integer term.
    ///
    /// # Safety
    ///
    /// This function calls [`pointer::add`].
    pub unsafe fn new(payload: *mut Payload) -> Self
    {
        Self{value: payload as *mut i32}
    }
}

impl Term
{
    /// Create an integer term.
    pub fn integer_i32(value: i32) -> Self
    {
        // A word is always at least 32 bits.
        let payload_words = 1;
        unsafe {
            Self::new(payload_words, |payload| {
                let view = UnsafeView::new(payload);
                view.value.write(value);
                Header::new(Kind::Integer)
            })
        }
    }

    /// View an integer term.
    ///
    /// # Safety
    ///
    /// The term must be an integer term.
    pub unsafe fn as_integer_unchecked(&self) -> i32
    {
        let payload = self.payload();
        let view = UnsafeView::new(payload);
        *view.value
    }

    /// Whether this is that specific integer term.
    pub fn eq_integer_i32(&self, value: i32) -> bool
    {
        match self.view() {
            View::Integer(val) => val == value,
            _ => false,
        }
    }
}
