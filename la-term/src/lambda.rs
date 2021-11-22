//! Working with lambda terms.
//!
//! The payload of a lambda term contains three words.
//! The first two words record an `Rc` to the parameters of the lambda.
//! The third word records the body of the lambda, which is another term.

use crate::AllocError;
use crate::Header;
use crate::Kind;
use crate::Payload;
use crate::Term;
use crate::symbol::Symbol;

use alloc::rc::Rc;

/// Information about a lambda parameter.
#[allow(missing_docs)]
#[derive(Debug)]
pub struct Parameter
{
    pub strictness: Strictness,
    pub name: Symbol,
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

/// Pointers to the words in the payload of a lambda term.
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub struct UnsafeView
{
    pub parameters: *mut Rc<[Parameter]>,
    pub body: *mut Term,
}

impl UnsafeView
{
    /// Obtain the pointers to the words in the payload of a lambda term.
    ///
    /// # Safety
    ///
    /// This function calls [`pointer::add`].
    pub unsafe fn new(payload: *mut Payload) -> Self
    {
        let payload = payload as *mut usize;
        Self{
            parameters: payload as *mut Rc<[Parameter]>,
            body: payload.add(2) as *mut Term,
        }
    }
}

impl Term
{
    /// Create a lambda term.
    pub fn lambda(parameters: Rc<[Parameter]>, body: Term)
        -> Result<Self, AllocError>
    {
        // `Rc<[Parameter]>` is two words, as it is a fat pointer.
        // Maybe we will optimize this in the future, but not now.
        let payload_words = 3;
        unsafe {
            Self::new(payload_words, |payload| {
                let view = UnsafeView::new(payload);
                view.parameters.write(parameters);
                view.body.write(body);
                Header::new(Kind::Lambda)
            })
        }
    }

    /// View a lambda term.
    ///
    /// Returns the parameters and the body of the lambda.
    ///
    /// # Safety
    ///
    /// The term must be a lambda term.
    pub unsafe fn as_lambda_unchecked(&self) -> (&Rc<[Parameter]>, &Term)
    {
        let payload = self.payload();
        let view = UnsafeView::new(payload);
        let parameters = &*view.parameters;
        let body = &*view.body;
        (parameters, body)
    }
}
