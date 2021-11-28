/// General-purpose scope guard.
///
/// This is in the `la_term` crate for convenience, not because it makes sense.
/// Hence it is hidden from the documentation.
#[doc(hidden)]
pub struct Guard<F>
    where F: FnOnce()
{
    f: Option<F>,
}

impl<F> Guard<F>
    where F: FnOnce()
{
    /// Run the given function when this guard is dropped.
    pub fn new(f: F) -> Self
    {
        Self{f: Some(f)}
    }

    /// Drop the guard without running the function.
    pub fn skip(mut self)
    {
        self.f = None;
    }
}

impl<F> Drop for Guard<F>
    where F: FnOnce()
{
    fn drop(&mut self)
    {
        if let Some(f) = self.f.take() {
            f();
        }
    }
}
