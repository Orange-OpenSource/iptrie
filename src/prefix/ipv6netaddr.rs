
use std::fmt;
use std::fmt::{Debug, Display};
use std::net::Ipv6Addr;
use std::str::FromStr;
use ipnet::Ipv6Net;
use crate::{IpPrefix, IpPrefixError, Ipv6Prefix, Ipv6Prefix120};

/// An Ipv6 prefix of fixed length of 64 bits
///
/// This prefix is commonly used to carry the network
/// address of a unicast Ipv6 address.
/// It takes exactly 64 bits.
/// ```text
/// |------------ 64 bits ---------------|
///            ip prefix slot
/// ```
/// To deal with a short prefix but without a fixed length,
/// consider [`Ipv6Prefix56`] which use one char to
/// store its length.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Ipv6NetAddr {
    addr: u64
}

impl IpPrefix for Ipv6NetAddr {
    type Slot = u64;

    #[inline] fn bitslot(&self) -> Self::Slot { self.addr }
    #[inline] fn bitslot_trunc(&self) -> Self::Slot { self.addr }
    #[inline] fn len(&self) -> u8 { 64 }
    #[inline] fn bitmask(&self) -> Self::Slot { !0 }

    const MAX_LEN: u8 = 64;
    type Addr = Ipv6Addr;

    #[inline] fn network(&self) -> Self::Addr {
        ((self.addr as u128) << 64).into()
    }
}

impl Ipv6NetAddr
{
    #[inline]
    pub fn new(addr: Ipv6Addr) -> Self
    {
        Self { addr: (u128::from(addr) >> 64) as u64 }
    }

    #[inline]
    pub fn into_slot(self) -> u64 { self.addr }

    #[inline]
    pub fn from_slot(addr: u64) -> Self { Self { addr } }
}

impl From<Ipv6NetAddr> for Ipv6Net
{
    #[inline] fn from(value: Ipv6NetAddr) -> Self {
        Ipv6Net::new(value.network(), value.len()).unwrap()
    }
}

impl From<Ipv6NetAddr> for Ipv6Prefix
{
    #[inline] fn from(value: Ipv6NetAddr) -> Self {
        Ipv6Prefix::new(value.network(), value.len()).unwrap()
    }
}

impl From<Ipv6NetAddr> for Ipv6Prefix120
{
    #[inline] fn from(value: Ipv6NetAddr) -> Self {
        let slot= ((value.addr as u128) << 64) | 64;
        unsafe { Ipv6Prefix120::from_slot_unchecked(slot) }
    }
}

impl TryFrom<Ipv6Net> for Ipv6NetAddr
{
    type Error = IpPrefixError;
    #[inline]
    fn try_from(value: Ipv6Net) -> Result<Self, Self::Error> {
        if value.len() == 64 {
            Ok(Self { addr: (value.bitslot() >> 64) as u64 })
        } else {
            Err(IpPrefixError::PrefixLenError)
        }
    }
}

impl TryFrom<Ipv6Prefix> for Ipv6NetAddr
{
    type Error = IpPrefixError;
    #[inline]
    fn try_from(value: Ipv6Prefix) -> Result<Self, Self::Error> {
        if value.len() == 64 {
            Ok(Self { addr: (value.bitslot() >> 64) as u64 })
        } else {
            Err(IpPrefixError::PrefixLenError)
        }
    }
}

impl TryFrom<Ipv6Prefix120> for Ipv6NetAddr
{
    type Error = IpPrefixError;
    #[inline]
    fn try_from(value: Ipv6Prefix120) -> Result<Self, Self::Error> {
        if value.len() == 64 {
            Ok(Self { addr: (value.bitslot() >> 64) as u64 })
        } else {
            Err(IpPrefixError::PrefixLenError)
        }
    }
}

impl From<u64> for Ipv6NetAddr
{
    #[inline]
    fn from(addr: u64) -> Self { Self { addr } }
}


impl From<Ipv6NetAddr> for u64
{
    #[inline]
    fn from(addr: Ipv6NetAddr) -> u64 { addr.addr }
}


impl Display for Ipv6NetAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ip = (*self).into();
        <Ipv6Net as fmt::Display>::fmt(&ip, f)
    }
}
impl Debug for Ipv6NetAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ip = (*self).into();
        <Ipv6Net as fmt::Display>::fmt(&ip, f)
    }
}
impl FromStr for Ipv6NetAddr {
    type Err = IpPrefixError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(Ipv6Net::from_str(s)?)
    }
}