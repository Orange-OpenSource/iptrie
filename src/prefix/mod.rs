//! IP Prefixes types and utilities
//!

mod slot;
mod ipv6trunc;
mod ipstd;

#[cfg(test)] mod tests;

use std::cmp::Ordering;
use std::error::Error;
pub use slot::*;
pub use ipv6trunc::*;
pub use ipstd::*;

use std::fmt;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use ipnet::{Ipv4Net, Ipv6Net};

/// Bit prefix
#[allow(clippy::len_without_is_empty)]
pub trait IpPrefix: Debug+Clone+Copy
{
    /// The slot manipulated inside this prefix
    type Slot: BitSlot;

    /// Root prefix has a length of 0
    fn root() -> Self; // root prefix, of len =0

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

    /// The address of the network defined by the prefixe.
    ///
    /// All the bits greater than the prefix length are set to `0`
    fn network(&self) -> Self::Addr;
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum IpPrefixCoverage { NoCoverage, WiderRange, SameRange }

impl IpPrefixCoverage {
    #[inline] fn is_none(&self) -> bool { *self == IpPrefixCoverage::NoCoverage }
    #[inline] fn is_wider(&self) -> bool { *self == IpPrefixCoverage::WiderRange }
    #[inline] fn is_same(&self) -> bool { *self == IpPrefixCoverage::SameRange }
    #[inline] fn is_covering(&self) -> bool { !self.is_none() }
}

/// A trait to check if the prefix covers the specified data
pub trait IpPrefixCovering<P>:IpPrefix
{
    /// Checks the coverage of this prefix against the specified one.
    ///
    /// * `SameRange` means that the two prefixes are equivalent
    /// * `WiderRange` means that this prefix includes the other
    /// * `NoCoverage` groups all the other cases
    ///
    /// # Difference with `PartialEq`
    /// Two prefixes could be different but equivalent regarding to
    /// the range of addresses they cover.
    /// ```
    /// # use iptrie::*;
    /// use ipnet::Ipv4Net;
    /// let a = "1.1.1.1/16".parse::<Ipv4Net>().unwrap();
    /// let b = "1.1.2.2/16".parse::<Ipv4Net>().unwrap();
    /// assert!( a != b ); // since host addr are different
    /// assert! (a.covers_equally(&b) ); // but prefixes are equivalent
    /// ```
    fn covering(&self, other: &P) -> IpPrefixCoverage;

    #[inline] fn covers(&self, other: &P) -> bool{ self.covering(other).is_covering() }
    #[inline] fn covers_striclty(&self, other: &P) -> bool{ self.covering(other).is_wider() }
    #[inline] fn covers_equally(&self, other: &P) -> bool{ self.covering(other).is_same() }
}

impl<P:IpPrefix> IpPrefixCovering<Self> for P {
    #[inline]
    fn covering(&self, other: &Self) -> IpPrefixCoverage {
        if other.bitslot() & self.bitmask() != self.bitslot_trunc() {
            IpPrefixCoverage::NoCoverage
        } else {
            match self.len().cmp(&other.len()) {
                Ordering::Less => IpPrefixCoverage::WiderRange,
                Ordering::Equal => IpPrefixCoverage::SameRange,
                Ordering::Greater => IpPrefixCoverage::NoCoverage
            }
        }
    }
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
    /// or greater than 56 for an [`Ipv6Prefix56`].
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

