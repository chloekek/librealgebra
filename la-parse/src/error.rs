/// Result type for the parser.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for the parser.
#[derive(Debug)]
pub struct Error;
