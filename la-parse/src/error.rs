/// Result type for the parser.
pub type Result<T> = core::result::Result<T, Error>;

/// Error type for the parser.
#[derive(Debug)]
pub struct Error;
