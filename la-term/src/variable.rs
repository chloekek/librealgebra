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

/// Set of De Bruijn indices used to locate free variables.
///
/// Each object contains a De Bruijn cache in its header.
/// The purpose of the De Bruijn cache is to speed up answering the question:
/// “does a particular variable appear free in a given term?”
/// When the De Bruijn cache is able to answer the question,
/// then the answer can be found in constant time
/// (the term does not have to be traversed).
/// Due to the limited amount of available space in the object header,
/// the De Bruijn cache only has space for variables
/// with small De Bruijn indices.
/// Attempting to insert a variable with a higher De Bruijn index
/// causes the De Bruijn cache to enter a special “unknown” state,
/// from which it is no longer able to answer the question
/// (requiring traversal of the term).
#[derive(Clone, Copy)]
pub struct DeBruijnCache
{
    bits: u16,
}

impl DeBruijnCache
{
    /// The De Bruijn cache with no variables in it.
    pub const EMPTY: Self = Self{bits: 0};

    /// The De Bruijn cache in the “unknown” state.
    pub const UNKNOWN: Self = Self{bits: 0xFFFF};

    /// Whether the De Bruijn cache is in the “unknown” state.
    pub fn is_unknown(self) -> bool
    {
        self.bits == Self::UNKNOWN.bits
    }

    /// Whether the De Bruijn cache contains a given variable.
    ///
    /// If this method returns `Some`, the answer is correct.
    /// Otherwise, the De Bruijn cache is in the “unknown” state,
    /// and the answer should be taken to be neither true nor false.
    pub fn contains(self, de_bruijn: DeBruijn) -> Option<bool>
    {
        if self.is_unknown() {
            None
        } else if de_bruijn.0 >= 16 {
            Some(false)
        } else {
            Some(self.bits & 1 << de_bruijn.0 != 0)
        }
    }

    /// Insert a variable into the De Bruijn cache.
    ///
    /// If the De Bruijn index of the variable is too large,
    /// the cache will enter the “unknown” state.
    /// It will also enter the “unknown” state if this completes
    /// the set of variables with small De Bruijn indices.
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

/// Take the set union of two De Bruijn caches.
impl BitOr for DeBruijnCache
{
    type Output = Self;

    fn bitor(self, rhs: DeBruijnCache) -> Self::Output
    {
        Self{bits: self.bits | rhs.bits}
    }
}

/// See the impl for `BitOr`.
impl BitOrAssign for DeBruijnCache
{
    fn bitor_assign(&mut self, rhs: DeBruijnCache)
    {
        *self = *self | rhs;
    }
}

/// Decrement each De Bruijn index in the cache by the given amount.
///
/// This is used for computing the De Bruijn cache of a lambda term.
/// Lambda terms are a point where the De Bruijn index of each variable
/// is increased by the number of lambda parameters.
/// Thus, to compute the De Bruijn cache of the lambda term itself,
/// the De Bruijn index of each variable must be decreased.
/// De Bruijn indices that would be decreased beyond zero are discarded,
/// as they do not appear free in the lambda term itself
/// (they are bound by the lambda term as parameters).
impl Shr<u32> for DeBruijnCache
{
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output
    {
        if self.is_unknown() {
            self
        } else {
            Self{bits: self.bits.checked_shr(rhs).unwrap_or(0)}
        }
    }
}

/// See the impl for `Shr`.
impl ShrAssign<u32> for DeBruijnCache
{
    fn shr_assign(&mut self, rhs: u32)
    {
        *self = *self >> rhs;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    use proptest::arbitrary::Arbitrary;
    use proptest::proptest;
    use proptest::strategy::Map;
    use proptest::strategy::Strategy;

    impl Arbitrary for DeBruijn
    {
        type Parameters = <u32 as Arbitrary>::Parameters;

        type Strategy = Map<<u32 as Arbitrary>::Strategy, fn(u32) -> DeBruijn>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy
        {
            <u32 as Arbitrary>::arbitrary_with(args)
                .prop_map(DeBruijn)
        }
    }

    proptest!
    {
        #[test]
        fn empty_answers_false(de_bruijn: DeBruijn)
        {
            let cache = DeBruijnCache::EMPTY;
            assert_eq!(cache.contains(de_bruijn), Some(false));
        }

        #[test]
        fn unknown_answers_none(de_bruijn: DeBruijn)
        {
            let cache = DeBruijnCache::UNKNOWN;
            assert_eq!(cache.contains(de_bruijn), None);
        }

        #[test]
        fn small_answers_true(de_bruijn in 0 .. 16u32)
        {
            let de_bruijn = DeBruijn(de_bruijn);
            let cache = DeBruijnCache::EMPTY.insert(de_bruijn);
            assert_eq!(cache.contains(de_bruijn), Some(true));
        }

        #[test]
        fn large_answers_false(small in 0 .. 16u32, large in 16u32 ..)
        {
            let small = DeBruijn(small);
            let large = DeBruijn(large);
            let cache = DeBruijnCache::EMPTY.insert(small);
            assert_eq!(cache.contains(large), Some(false));
        }

        #[test]
        fn insert_all_small_enters_unknown(de_bruijn: DeBruijn)
        {
            let cache =
                (0 .. 16)
                .map(DeBruijn)
                .fold(DeBruijnCache::EMPTY,
                      DeBruijnCache::insert);
            assert_eq!(cache.contains(de_bruijn), None);
        }

        #[test]
        fn insert_any_large_enters_unknown(de_bruijn in 16u32 ..)
        {
            let de_bruijn = DeBruijn(de_bruijn);
            let cache = DeBruijnCache::EMPTY.insert(de_bruijn);
            assert_eq!(cache.contains(de_bruijn), None);
        }

        #[test]
        fn shift_shifts(shift in 0u32 .. 16)
        {
            let cache = DeBruijnCache::EMPTY.insert(DeBruijn(15)) >> shift;
            assert_eq!(cache.contains(DeBruijn(15 - shift)), Some(true));
        }

        #[test]
        fn shift_retains_unknown(shift: u32)
        {
            let cache = DeBruijnCache::UNKNOWN >> shift;
            assert!(cache.is_unknown());
        }
    }
}
