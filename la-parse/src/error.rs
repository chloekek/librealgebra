use la_term::AllocError;

/// Result type for the parser.
pub type Result<T> = core::result::Result<T, Error>;

/// Error type for the parser.
#[derive(Debug)]
pub struct Error;

impl From<AllocError> for Error
{
    fn from(_other: AllocError) -> Self
    {
        Self
    }
}
