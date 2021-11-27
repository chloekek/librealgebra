//! Data type for terms.

#![feature(core_intrinsics)]
#![feature(extern_types)]
#![feature(intra_doc_pointers)]
#![feature(option_result_unwrap_unchecked)]
#![feature(trusted_len)]
#![no_std]
#![warn(missing_docs)]

extern crate alloc;

use self::guard::Guard;
use self::object::*;

use alloc::alloc::alloc;
use alloc::alloc::dealloc;
use alloc::alloc::handle_alloc_error;
use alloc::rc::Rc;
use core::alloc::Layout;
use core::fmt;
use core::intrinsics::abort;
use core::mem::align_of;
use core::mem::size_of;
use core::ptr::NonNull;

pub mod application;
pub mod integer;
pub mod lambda;
pub mod string;
pub mod symbol;
pub mod variable;

mod guard;

#[cold]
#[inline(never)]
fn panic_layout() -> !
{
    panic!("layout")
}

/// Convenience function that adds two integers and panics on overflow.
fn add(a: usize, b: usize) -> usize
{
    a.checked_add(b).unwrap_or_else(|| panic_layout())
}

/// Convenience function that multiplies two integers and panics on overflow.
fn mul(a: usize, b: usize) -> usize
{
    a.checked_mul(b).unwrap_or_else(|| panic_layout())
}

/// Handle to a term of any type.
pub struct Term
{
    ptr: NonNull<Object>,
}

/// Borrowed view into a term.
///
/// There is a variant for each kind of term.
/// The fields of these variants point into the term.
/// This is how you generally inspect terms,
/// if not through the `as_*` methods on [`Term`].
#[allow(missing_docs)]
#[derive(Debug)]
pub enum View<'a>
{
    Application(&'a Term, &'a [Term]),
    Integer(i32),
    Lambda(&'a Rc<[lambda::Parameter]>, &'a Term),
    String(&'a [u8]),
    Symbol(&'a symbol::Symbol),
    Variable(variable::DeBruijn),
}

impl Term
{
    /// Compute the layout for a term that has a specified number of words.
    fn layout(payload_words: usize) -> Layout
    {
        let payload_size = mul(payload_words, size_of::<usize>());
        let size = add(size_of::<Header>(), payload_size);
        Layout::from_size_align(size, align_of::<Header>())
            .unwrap_or_else(|_| panic_layout())
    }

    /// Allocate memory for a new term and initialize it.
    ///
    /// The `payload_words` argument specifies
    /// how many words the term payload occupies.
    /// After allocation, the `init` function is called
    /// which must initialize the term payload
    /// and return the term header.
    ///
    /// # Safety
    ///
    /// The `init` function must initialize the payload and return a header
    /// such that the term can be used safely when this operation is complete.
    pub unsafe fn new<F>(payload_words: usize, init: F) -> Self
        where F: FnOnce(*mut Payload) -> Header
    {
        let layout = Self::layout(payload_words);
        let ptr = alloc(layout) as *mut Object;
        let ptr = NonNull::new(ptr).unwrap_or_else(|| handle_alloc_error(layout));

        // If init panics then we want to deallocate the memory ...
        let guard = Guard::new(|| dealloc(ptr.as_ptr() as *mut u8, layout));
        (*ptr.as_ptr()).header = init(&mut (*ptr.as_ptr()).payload);
        guard.skip(); // ... but not if init returns.

        Self{ptr}
    }

    /// Access the term as a pointer.
    pub fn as_ptr(&self) -> *mut Object
    {
        self.ptr.as_ptr()
    }

    /// Whether two terms are the same object.
    pub fn ptr_eq(&self, other: &Term) -> bool
    {
        self.as_ptr() == other.as_ptr()
    }

    /// Access the header of the term.
    pub fn header(&self) -> Header
    {
        unsafe {
            (*self.as_ptr()).header
        }
    }

    /// Access the payload of the term.
    pub fn payload(&self) -> *mut Payload
    {
        unsafe {
            &mut (*self.as_ptr()).payload
        }
    }

    /// Borrow the components of the term.
    pub fn view(&self) -> View
    {
        // SAFETY: The calls to `as_*_unchecked` correspond
        //         to the kinds of the match arms.
        unsafe {
            match self.header().kind {
                Kind::Application => {
                    let (function, arguments) = self.as_application_unchecked();
                    View::Application(function, arguments)
                },
                Kind::Integer => View::Integer(self.as_integer_unchecked()),
                Kind::Lambda => {
                    let (parameters, body) = self.as_lambda_unchecked();
                    View::Lambda(parameters, body)
                },
                Kind::String => View::String(self.as_string_unchecked()),
                Kind::Symbol => View::Symbol(self.as_symbol_unchecked()),
                Kind::Variable => View::Variable(self.as_variable_unchecked()),
            }
        }
    }
}

impl Clone for Term
{
    fn clone(&self) -> Self
    {
        unsafe {
            let ref_count: *mut u32 =
                &mut (*self.as_ptr()).header.ref_count;
            if *ref_count == u32::MAX {
                abort();
            } else {
                *ref_count += 1;
            }
        }
        Self{ptr: self.ptr}
    }
}

impl Drop for Term
{
    fn drop(&mut self)
    {
        // TODO: Call correct destructor depending on kind.
        // TODO: Deallocate memory after obtaining layout.
    }
}

impl fmt::Debug for Term
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        self.view().fmt(f)
    }
}

/// In-memory representation of terms.
///
/// This is exposed only for documentation purposes.
/// If you find yourself using these interfaces outside this crate,
/// consider adding the required functionality to this crate instead.
pub mod object
{
    /// In-memory representation of a term.
    #[allow(missing_docs)]
    pub struct Object
    {
        pub header: Header,
        pub payload: Payload,
    }

    /// Information common to all terms.
    #[derive(Clone, Copy)]
    #[repr(C, align(8))]
    pub struct Header
    {
        /// Number of references to the term.
        pub ref_count: u32,

        /// Which kind of term this is.
        pub kind: Kind,
    }

    impl Header
    {
        /// Create a header with a reference count of one.
        pub fn new(kind: Kind) -> Self
        {
            Self{kind, ref_count: 1}
        }
    }

    /// Different kinds of terms.
    ///
    /// The payload layout depends on this field.
    /// There is also one module for each kind,
    /// that implements working with terms of that kind.
    #[allow(missing_docs)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum Kind
    {
        Application,
        Integer,
        Lambda,
        String,
        Symbol,
        Variable,
    }

    extern
    {
        /// Kind-specific data for the term.
        pub type Payload;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Test that the `Term` type has the same size and alignment as a word.
    /// This is an assumption that is made throughout the `la_term` crate.
    #[test]
    fn term_size_align()
    {
        assert_eq!(size_of::<Term>(), size_of::<usize>());
        assert_eq!(align_of::<Term>(), align_of::<usize>());
    }

    /// Test that the `Header` type has a size of 8.
    /// This is the expected size of the header type,
    /// and if it is larger then something went wrong.
    #[test]
    fn header_size()
    {
        assert_eq!(size_of::<Header>(), 8);
    }

    /// Test that the `Header` type has an alignment of 8.
    /// This makes it fast to fetch the whole header on 64-bit systems.
    /// It is also a multiple of the word size, which is convenient.
    #[test]
    fn header_align()
    {
        assert_eq!(align_of::<Header>(), 8);
    }
}
