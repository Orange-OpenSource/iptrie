use std::error::Error;
use std::net::AddrParseError;
use std::fmt;

/// An error to qualify why a prefix is wrong
#[derive(Debug)]
pub enum PrefixError {
    /// No mask when parsing the prefix.
    /// The slash (`/`) character is not found during parsing.
    MissingMask,

    /// Unparsable address, address in bad format.
    /// Notice that IPv4 can’t be parsed as IPv6 address (and vice-versa)
    InvalidAddress,

    /// The mask is unparsable or the parsed number is not valid.
    InvalidMask
}

impl Error for PrefixError {}

impl fmt::Display for PrefixError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrefixError::MissingMask => write!(f, "IP mask (CIDR) is missing"),
            PrefixError::InvalidAddress => write!(f, "Invalid IP address"),
            PrefixError::InvalidMask => write!(f, "Invalid mask (CIDR)"),
        }
    }
}

impl From<AddrParseError> for PrefixError {
    #[inline]
    fn from(_: AddrParseError) -> Self {
        PrefixError::InvalidAddress
    }
}

