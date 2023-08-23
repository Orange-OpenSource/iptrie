//! Standard Ip prefixes.
//!
//! This module implements [`IpPrefix`] trait for:
//! * [`Ipv4Prefix`], generic Ipv4 prefix, similar to [`Ipv4Net`] but with trailing bits
//!   guaranteed to equal `0`
//! * [`Ipv6Prefix`], generic Ipv6 prefix, similar to [`Ipv6Net`] but with trailing bits
//!   guaranteed to equal `0`
//! * [`Ipv4Addr`], Ipv4 address viewed as a prefix with length of 32 bits
//! * [`Ipv6Addr`], Ipv6 address viewed as a prefix with length of 128 bits
//! * [`Ipv4Net`] with a small extra cost to deal with non null trailing bits
//! * [`Ipv6Net`] with a small extra cost to deal with non null trailing bits

use super::*;

/// An Ipv4 prefix similar to [`Ipv4Net`] but with trailing bits
///  guaranteed to equal `0`
#[repr(C)]
#[derive(Copy, Clone, Default, Eq, PartialEq, Hash)]
pub struct Ipv4Prefix {
    pub(super) addr: u32,
    pub(super) len: u8
}

impl Ipv4Prefix {
    /// Build a new prefix.
    ///
    /// All the bits greater than the prefix length are set to `0``.
    /// If the specified length is greater than the maximum allowed,
    /// an error is returned.
    ///
    /// # Example
    /// ```
    /// use std::net::{Ipv4Addr, Ipv6Addr};
    ///     use ipnet::PrefixLenError;
    /// #     use iptrie::*;
    ///     let ipv4 = "1.2.3.4".parse::<Ipv4Addr>().unwrap();
    ///     let ipv6 = "1:2:3:4::".parse::<Ipv6Addr>().unwrap();
    ///
    ///     assert!( Ipv4Prefix::new(ipv4, 12).is_ok());
    ///
    ///     assert_eq!( Ipv4Prefix::new(ipv4, 64), Err(IpPrefixError::PrefixLenError));
    /// ```
    #[inline]
    pub fn new(addr: Ipv4Addr, len: u8) -> Result<Self, IpPrefixError>
    {
        if len > Self::MAX_LEN {
            Err(IpPrefixError::PrefixLenError)
        } else {
            Ok( Self { addr: u32::from(addr) & u32::bitmask(len), len })
        }
    }
}

/// An Ipv4 prefix similar to [`Ipv6Net`] but with trailing bits
///  guaranteed to equal `0`
#[repr(C)]
#[derive(Copy, Clone, Default, Eq, PartialEq, Hash)]
pub struct Ipv6Prefix {
    pub(super) addr: u128,
    pub(super) len: u8
}


impl Ipv6Prefix {
    /// Build a new prefix.
    ///
    /// All the bits greater than the prefix length are set to `0``.
    /// If the specified length is greater than the maximum allowed,
    /// an error is returned.
    ///
    /// # Example
    /// ```
    /// use std::net::{Ipv4Addr, Ipv6Addr};
    ///     use ipnet::PrefixLenError;
    /// #     use iptrie::*;
    ///     let ipv4 = "1.2.3.4".parse::<Ipv4Addr>().unwrap();
    ///     let ipv6 = "1:2:3:4::".parse::<Ipv6Addr>().unwrap();
    ///
    ///     assert!( Ipv6Prefix::new(ipv6, 78).is_ok());
    ///
    ///     assert_eq!( Ipv6Prefix::new(ipv6, 133), Err(IpPrefixError::PrefixLenError));
    /// ```
    #[inline]
    pub fn new(addr: Ipv6Addr, len: u8) -> Result<Self, IpPrefixError>
    {
        if len > Self::MAX_LEN {
            Err(IpPrefixError::PrefixLenError)
        } else {
            Ok( Self { addr: u128::from(addr) & u128::bitmask(len), len })
        }
    }
}



macro_rules! ipprefix {

    ($ipaddr:ty, $ipnet:ty, $prefix:ident, $slot:ty) => {

        impl IpPrefix for $prefix
        {
            type Slot = $slot;
            #[inline] fn root() -> Self { Self { addr: 0, len: 0 } }
            #[inline] fn bitslot(&self) -> Self::Slot { self.addr }
            #[inline] fn bitslot_trunc(&self) -> Self::Slot { self.addr }
            #[inline] fn len(&self) -> u8 { self.len }

            const MAX_LEN: u8 = <Self as IpPrefix>::Slot::LEN;
            type Addr = $ipaddr;
            #[inline] fn network(&self) -> Self::Addr { self.addr.into() }
        }

        impl From<$ipnet> for $prefix
        {
            #[inline] fn from(value: $ipnet) -> Self {
                Self { addr: value.trunc().addr().into(), len: value.prefix_len() }
            }
        }
        impl From<$prefix> for $ipnet
        {
            #[inline] fn from(value: $prefix) -> Self {
                <$ipnet>::new(value.addr.into(), value.len()).unwrap()
            }
        }
        impl Display for $prefix {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let ip = (*self).into();
                <$ipnet as fmt::Display>::fmt(&ip, f)
            }
        }
        impl Debug for $prefix {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let ip = (*self).into();
                <$ipnet as fmt::Display>::fmt(&ip, f)
            }
        }
        impl FromStr for $prefix {
            type Err = IpPrefixError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(<$prefix>::from(<$ipnet>::from_str(s)?))
            }
        }
        impl From<$ipaddr> for $prefix
        {
            #[inline] fn from(value: $ipaddr) -> Self {
                Self { addr: value.into(), len: Self::MAX_LEN }
            }
        }

        impl IpPrefix for $ipnet
        {
            type Slot = $slot;
            #[inline] fn root() -> Self { Self::default() }
            #[inline] fn bitslot(&self) -> Self::Slot { <$slot>::from(self.addr()) }
            #[inline] fn bitslot_trunc(&self) -> Self::Slot { <$slot>::from(self.network()) }
            #[inline] fn len(&self) -> u8 { self.prefix_len() }

            const MAX_LEN: u8 = <Self as IpPrefix>::Slot::LEN;
            type Addr = $ipaddr;
            #[inline] fn network(&self) -> Self::Addr { self.network() }
        }

        impl IpPrefix for $ipaddr
        {
            type Slot = $slot;
            #[inline] fn root() -> Self { 0.into() }
            #[inline] fn bitslot(&self) -> Self::Slot { Self::Slot::from(*self) }
            #[inline] fn bitslot_trunc(&self) -> Self::Slot { self.bitslot() }
            #[inline] fn len(&self) -> u8 { Self::Slot::LEN }

            const MAX_LEN: u8 = <Self as IpPrefix>::Slot::LEN;
            type Addr = $ipaddr;
            #[inline] fn network(&self) -> Self::Addr { *self }
        }
    }
}

ipprefix!(Ipv4Addr, Ipv4Net, Ipv4Prefix, u32);
ipprefix!(Ipv6Addr, Ipv6Net, Ipv6Prefix, u128);

