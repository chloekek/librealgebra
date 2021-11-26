//! Working with application terms.
//!
//! The payload of an application term contains 2 + _n_ words,
//! where _n_ is the number of arguments in the application.
//! The first word records the number of arguments.
//! The second word records the function being applied.
//! The remaining words record the arguments, in order.

use crate::Header;
use crate::Kind;
use crate::Payload;
use crate::Term;
use crate::add;

use core::iter::TrustedLen;
use core::slice;

/// Pointers to the words in the payload of an application term.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct UnsafeView
{
    pub argument_count: *mut usize,
    pub function: *mut Term,
    pub arguments: *mut Term,
}

impl UnsafeView
{
    /// Obtain the pointers to the words in the payload of an application term.
    ///
    /// # Safety
    ///
    /// This function calls [`pointer::add`].
    pub unsafe fn new(payload: *mut Payload) -> Self
    {
        let payload = payload as *mut usize;
        Self{
            argument_count: payload,
            function: payload.add(1) as *mut Term,
            arguments: payload.add(2) as *mut Term,
        }
    }
}

impl Term
{
    /// Create an application term.
    pub fn application<I, J>(function: Term, arguments: I) -> Self
        where I: IntoIterator<IntoIter=J>
            , J: Iterator<Item=Term> + ExactSizeIterator + TrustedLen
    {
        let arguments = arguments.into_iter();
        let payload_words = add(2, arguments.len());
        unsafe {
            Self::new(payload_words, |payload| {
                let view = UnsafeView::new(payload);
                view.argument_count.write(arguments.len());
                view.function.write(function);
                // BUG: Memory leak if iterator panics.
                for (i, argument) in arguments.enumerate() {
                    view.arguments.add(i).write(argument);
                }
                Header::new(Kind::Application)
            })
        }
    }

    /// View an application term.
    ///
    /// Returns the function being applied and
    /// the list of arguments it is applied to.
    ///
    /// # Safety
    ///
    /// The term must be an application term.
    pub unsafe fn as_application_unchecked(&self) -> (&Term, &[Term])
    {
        let payload = self.payload();
        let view = UnsafeView::new(payload);
        let function = &*view.function;
        let argument_count = *view.argument_count;
        let arguments = slice::from_raw_parts(view.arguments, argument_count);
        (function, arguments)
    }
}
