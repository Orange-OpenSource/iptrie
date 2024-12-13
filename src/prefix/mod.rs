//! IP Prefixes types and utilities
//!

mod slot;
mod ipstd;
mod cover;

#[cfg(test)] mod tests;
mod private;
mod shorten;
mod routing;

use std::error::Error;
pub use slot::*;
pub use ipstd::*;
pub use cover::*;
pub use shorten::*;
pub use routing::Ipv6NetRouting;

use std::fmt;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use ipnet::{Ipv4Net, Ipv6Net};
pub use crate::prefix::private::IpPrivatePrefix;

pub trait IpRootPrefix: IpPrefix {
    /// Root prefix has a length of 0
    fn root() -> Self; // root prefix, of len =0
}

/// Ip prefix (as bit prefix)
#[allow(clippy::len_without_is_empty)]
pub trait IpPrefix: IpPrivatePrefix+Debug+Clone+Copy
{
    /// The slot manipulated inside this prefix
    type Slot: BitSlot;

    /// The inner slot _as is_
    ///
    /// Depending of the implementation, there is no warranty that
    /// the masked bits (i.e. with position greater than the length)
    /// are set to 0.
    ///
    /// For a version of a slot with masked bits set to 0,
    /// see [`Self::bitslot_trunc`].
    fn bitslot(&self) -> Self::Slot;

    /// The inner slot with all the masked bits set to 0.
    ///
    /// The result always equals `self.bitslot() & self.bitmask()`
    /// but it is usually faster to compute (depending on the prefix structure)
    fn bitslot_trunc(&self) -> Self::Slot;

    /// Length of the prefix.
    ///
    /// This is the number of significant first bits.
    /// Others should be considered as 0 (event if not)
    fn len(&self) -> u8;

    /// Mask of the prefix.
    ///
    /// The n (prefix length) first bits are set to 1 and the last ones are set to 0.
    #[inline]
    fn bitmask(&self) -> Self::Slot { <Self::Slot as BitSlot>::bitmask(self.len()) }

    /// The maximum allowed length for this prefix
    const MAX_LEN: u8;

    /// The underlying ip address (usually Ipv4Addr or Ipv6Addr)
    type Addr: Display+Clone+Copy+Eq+Hash;

    /// The address of the network defined by the prefixv
    ///
    /// All the bits greater than the prefix length are set to `0`
    fn network(&self) -> Self::Addr;
}



/// Error generated when building an Ip prefix
#[derive(Debug,PartialEq,Eq,Copy, Clone)]
pub enum IpPrefixError {
    /// The specified length of the prefix is not valid.
    ///
    /// For Ipv4, this error is generated if the specified length
    /// is greater than 32 for an  [`Ipv4Prefix`] or [`Ipv4Net`].
    ///
    /// For Ipv6, this error is generated if the specified length
    /// is greater than 128 for an  [`Ipv6Prefix`] or [`Ipv6Net`]
    /// or greater than 120 for an [`Ipv6Prefix120`]
    /// or greater than 56 for an [`Ipv6Prefix56`]
    /// or not equal to 64 for an [`Ipv6NetAddr`]
    PrefixLenError,

    /// The parsed string does not contains a valid Ip address.
    ///
    /// It occurs also if when parsing an Ipv4 (resp. Ipv6) address on a string
    /// which contains an Ipv6 (resp. Ipv4) syntax.
    AddrParseError,
}

impl Display for IpPrefixError
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IpPrefixError::PrefixLenError => {
                fmt.write_str("invalid IP prefix length")
            }
            IpPrefixError::AddrParseError => {
                fmt.write_str("invalid IP address syntax")
            }
        }
    }
}

impl Error for IpPrefixError {}

impl From<ipnet::AddrParseError> for IpPrefixError {
    fn from(_: ipnet::AddrParseError) -> Self {
        IpPrefixError::AddrParseError
    }
}

impl From<std::net::AddrParseError> for IpPrefixError {
    fn from(_: std::net::AddrParseError) -> Self {
        IpPrefixError::AddrParseError
    }
}

impl From<ipnet::PrefixLenError> for IpPrefixError {
    fn from(_: ipnet::PrefixLenError) -> Self {
        IpPrefixError::PrefixLenError
    }
}

