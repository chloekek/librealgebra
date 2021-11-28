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
use core::ops::BitOr;
use core::ops::BitOrAssign;
use core::ops::Shr;
use core::ops::ShrAssign;

////////////////////////////////////////////////////////////////////////////////
// Variable terms

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

                // A variable appears free in itself.
                let de_bruijn_cache = DeBruijnCache::EMPTY.insert(de_bruijn);

                let view = UnsafeView::new(payload);
                view.de_bruijn.write(de_bruijn);

                Header::new(Kind::Variable, de_bruijn_cache)

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

////////////////////////////////////////////////////////////////////////////////
// De Bruijn indices

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

////////////////////////////////////////////////////////////////////////////////
// De Bruijn caches

#[derive(Clone, Copy)]
pub struct DeBruijnCache
{
    bits: u16,
}

impl DeBruijnCache
{
    pub const EMPTY: Self = Self{bits: 0};

    pub const UNKNOWN: Self = Self{bits: 0xFFFF};

    pub fn contains(self, de_bruijn: DeBruijn) -> Option<bool>
    {
        if self.bits == 0xFFFF {
            None
        } else if de_bruijn.0 >= 16 {
            Some(false)
        } else {
            Some(self.bits & 1 << de_bruijn.0 != 0)
        }
    }

    #[must_use]
    pub fn insert(self, de_bruijn: DeBruijn) -> Self
    {
        if de_bruijn.0 >= 16 {
            Self::UNKNOWN
        } else {
            Self{bits: self.bits | 1 << de_bruijn.0}
        }
    }
}

impl BitOr for DeBruijnCache
{
    type Output = Self;

    fn bitor(self, rhs: DeBruijnCache) -> Self::Output
    {
        Self{bits: self.bits | rhs.bits}
    }
}

impl BitOrAssign for DeBruijnCache
{
    fn bitor_assign(&mut self, rhs: DeBruijnCache)
    {
        *self = *self | rhs;
    }
}

impl Shr<u32> for DeBruijnCache
{
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output
    {
        if self.bits == 0xFFFF {
            self
        } else {
            Self{bits: self.bits.checked_shr(rhs).unwrap_or(0)}
        }
    }
}

impl ShrAssign<u32> for DeBruijnCache
{
    fn shr_assign(&mut self, rhs: u32)
    {
        *self = *self >> rhs;
    }
}
