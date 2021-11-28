//! Working with string terms.
//!
//! The payload of a string term contains 1 + ⌈_n_ / _s_⌉ words,
//! where _n_ is the number of bytes in the string, and _s_ is the word size.
//! The first word records the number of bytes in the string.
//! The remaining words record the bytes.

use crate::Header;
use crate::Kind;
use crate::Payload;
use crate::Term;
use crate::add;
use crate::variable::DeBruijnCache;

use core::iter::TrustedLen;
use core::mem::size_of;
use core::slice;

/// Pointers to the words in the payload of a string term.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct UnsafeView
{
    pub byte_count: *mut usize,
    pub bytes: *mut u8,
}

impl UnsafeView
{
    /// Obtain the pointers to the words in the payload of a string term.
    ///
    /// # Safety
    ///
    /// This function calls [`pointer::add`].
    pub unsafe fn new(payload: *mut Payload) -> Self
    {
        let payload = payload as *mut usize;
        Self{
            byte_count: payload,
            bytes: payload.add(1) as *mut u8,
        }
    }
}

fn round_to_words(bytes: usize) -> usize
{
    let word = size_of::<usize>() as u128;
    ((bytes as u128 + word - 1) / word) as usize
}

impl Term
{
    /// Create a string term.
    pub fn string<I, J>(bytes: I) -> Self
        where I: IntoIterator<IntoIter=J>
            , J: Iterator<Item=u8> + ExactSizeIterator + TrustedLen
    {
        let bytes = bytes.into_iter();
        let bytes_words = round_to_words(bytes.len());
        let payload_words = add(1, bytes_words);
        unsafe {
            Self::new(payload_words, |payload| {
                let view = UnsafeView::new(payload);
                view.byte_count.write(bytes.len());
                for (i, byte) in bytes.enumerate() {
                    view.bytes.add(i).write(byte);
                }
                Header::new(Kind::String, DeBruijnCache::EMPTY)
            })
        }
    }

    /// View a string term.
    ///
    /// # Safety
    ///
    /// The term must be a string term.
    pub unsafe fn as_string_unchecked(&self) -> &[u8]
    {
        let payload = self.payload();
        let view = UnsafeView::new(payload);
        let byte_count = *view.byte_count;
        slice::from_raw_parts(view.bytes, byte_count)
    }
}
