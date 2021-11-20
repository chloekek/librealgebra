use alloc::vec::Vec;
use core::ascii;
use core::fmt;

/// Text-like `Debug` impl for `Vec<u8>`.
#[derive(Clone)]
pub struct ByteString(pub Vec<u8>);

impl From<Vec<u8>> for ByteString
{
    fn from(other: Vec<u8>) -> Self
    {
        Self(other)
    }
}

impl fmt::Debug for ByteString
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "\"")?;
        for &byte in &self.0 {
            let ascii = ascii::escape_default(byte).map(|c| c as char);
            for character in ascii {
                write!(f, "{}", character)?;
            }
        }
        write!(f, "\"")?;
        Ok(())
    }
}
