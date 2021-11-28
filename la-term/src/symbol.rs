//! Working with symbol terms.
//!
//! The payload of a symbol term contains 1 + ⌈_n_ / _s_⌉ words,
//! where _n_ is the number of bytes in the name, and _s_ is the word size.
//! The first word records the number of bytes in the name.
//! The remaining words record the bytes of the name.

use crate::Header;
use crate::Kind;
use crate::Payload;
use crate::Term;
use crate::View;
use crate::add;
use crate::variable::DeBruijnCache;

use core::borrow::Borrow;
use core::cell::RefCell;
use core::fmt;
use core::hash::Hash;
use core::hash::Hasher;
use core::mem::size_of;
use core::ptr::copy;
use core::slice;
use hashbrown::HashSet;

////////////////////////////////////////////////////////////////////////////////
// Symbol terms

/// Pointers to the words in the payload of a symbol term.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct UnsafeView
{
    pub name_len: *mut usize,
    pub name: *mut u8,
}

impl UnsafeView
{
    /// Obtain the pointers to the words in the payload of a symbol term.
    ///
    /// # Safety
    ///
    /// This function calls [`pointer::add`].
    pub unsafe fn new(payload: *mut Payload) -> Self
    {
        let payload = payload as *mut usize;
        Self{
            name_len: payload,
            name: payload.add(1) as *mut u8,
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
    /// Unwrap a symbol term.
    pub fn symbol(symbol: Symbol) -> Self
    {
        symbol.inner
    }

    /// Create a symbol term.
    fn symbol_uninterned(name: &[u8]) -> Term
    {
        let name_words = round_to_words(name.len());
        let payload_words = add(1, name_words);
        unsafe {
            Self::new(payload_words, |payload| {
                let view = UnsafeView::new(payload);
                view.name_len.write(name.len());
                copy(name.as_ptr(), view.name, name.len());
                Header::new(Kind::Symbol, DeBruijnCache::EMPTY)
            })
        }
    }

    /// Whether this is a symbol term.
    pub fn is_symbol(&self) -> bool
    {
        self.header().kind == Kind::Symbol
    }

    /// Whether this is that specific symbol term.
    pub fn eq_symbol(&self, symbol: &Symbol) -> bool
    {
        self.as_ptr() == symbol.inner.as_ptr()
    }

    /// Wrap a symbol term, if it is one.
    pub fn as_symbol(&self) -> Option<&Symbol>
    {
        match self.view() {
            View::Symbol(symbol) => Some(symbol),
            _ => None,
        }
    }

    /// Wrap a symbol term.
    ///
    /// # Safety
    ///
    /// The term must be a symbol term.
    pub unsafe fn as_symbol_unchecked(&self) -> &Symbol
    {
        // SAFETY: Term and Symbol have the same representation.
        &*(self as *const Term as *const Symbol)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Symbol type

/// Handle to a symbol term.
///
/// As symbols are interned, the `PartialEq` and `Hash` impls for this type
/// operate on the addresses of the terms (as opposed to their payloads).
/// As a result we can check symbol equality in _O(1)_ time.
///
/// You cannot create symbols directly.
/// Symbol creation happens only through [`Symbols::get`].
#[derive(Clone)]
#[repr(transparent)]
pub struct Symbol
{
    // INVARIANT: This is a symbol term.
    inner: Term,
}

impl Symbol
{
    /// Unwrap the symbol term.
    ///
    /// Unlike [`Term::symbol`], this clones the term for you.
    /// So you can write `s.term()` instead of `Term::symbol(s.clone())`.
    pub fn term(&self) -> Term
    {
        Term::symbol(self.clone())
    }

    /// The name of the symbol.
    pub fn name(&self) -> &[u8]
    {
        let payload = self.inner.payload();
        unsafe {
            let view = UnsafeView::new(payload);
            slice::from_raw_parts(view.name, *view.name_len)
        }
    }
}

impl PartialEq for Symbol
{
    fn eq(&self, rhs: &Symbol) -> bool
    {
        self.inner.as_ptr() == rhs.inner.as_ptr()
    }
}

impl Eq for Symbol
{
}

impl Hash for Symbol
{
    fn hash<H>(&self, state: &mut H)
        where H: Hasher
    {
        self.inner.as_ptr().hash(state)
    }
}

impl fmt::Debug for Symbol
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        f.debug_tuple("Symbol")
            .field(&self.name())
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Symbol interning

/// Table of interned symbols.
pub struct Symbols
{
    symbols: RefCell<HashSet<Entry>>,
}

/// Entry of the table of interned symbols.
///
/// This uses name equality rather than pointer equality
/// in its impls of `PartialEq` and `Hash`.
struct Entry
{
    inner: Symbol,
}

impl PartialEq for Entry
{
    fn eq(&self, rhs: &Entry) -> bool
    {
        self.inner.name() == rhs.inner.name()
    }
}

impl Eq for Entry
{
}

impl Hash for Entry
{
    fn hash<H>(&self, state: &mut H)
        where H: Hasher
    {
        self.inner.name().hash(state)
    }
}

impl Borrow<[u8]> for Entry
{
    fn borrow(&self) -> &[u8]
    {
        self.inner.name()
    }
}

impl Symbols
{
    /// Create a new table with no symbols.
    pub fn new() -> Self
    {
        Self{symbols: RefCell::new(HashSet::new())}
    }

    /// Get or create the symbol with the given name.
    ///
    /// When a symbol with the same name was already created,
    /// this method will return that same symbol term.
    pub fn get(&self, name: &[u8]) -> Symbol
    {
        let mut symbols = self.symbols.borrow_mut();
        symbols.get_or_insert_with(name, |name| {
            let symbol_term = Term::symbol_uninterned(name);
            Entry{inner: Symbol{inner: symbol_term}}
        }).inner.clone()
    }
}
