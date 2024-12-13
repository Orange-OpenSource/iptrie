use std::fmt::{Debug, Display, Formatter};
use std::net::Ipv6Addr;
use std::str::FromStr;
use ipnet::{Ipv6Net, PrefixLenError};
use crate::{BitSlot, IpPrefix, IpPrefixError, IpPrefixShortening, IpPrivatePrefix, IpRootPrefix, Ipv6Prefix};

/// An Ipv6 prefix limited to 64 bits (EXPERIMENTAL)
///
/// In many applications, managed Ipv6 prefixes are never longer than 64 bits
/// which contains any routing prefix.
/// In these cases, it is possible to save memory space by encoding it in
/// 64 bits with one more byte to encode the prefix length.
/// ```text
/// |------------ 64 bits ----------------|--8 bits--|
///            ip prefix slot                length
/// ```
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Ipv6NetRouting {
    slot: u64,
    len: u8
}

impl Ipv6NetRouting {

    pub const fn new(ip: Ipv6Addr, len: u8) -> Result<Self, PrefixLenError>
    {
        if len > 64 {
            Err(PrefixLenError)
        } else {
            let bitmask = if len == 0 { 0 } else { (!0) << (64-len) };
            Ok(Self { slot: ((ip.to_bits() >> 64) as u64) & bitmask, len })
        }
    }

    pub const fn new_assert(ip: Ipv6Addr, len: u8) -> Self
    {
        assert!(len <= 64);
        let bitmask = if len == 0 { 0 } else { (!0) << (64-len) };
        Self { slot: ((ip.to_bits() >> 64) as u64) & bitmask, len }
    }

}

impl IpPrefix for Ipv6NetRouting {
    type Slot = u64;

    #[inline]
    fn bitslot(&self) -> Self::Slot {
        self.slot
    }

    #[inline]
    fn bitslot_trunc(&self) -> Self::Slot {
        self.slot & u64::bitmask(self.len)
    }

    #[inline]
    fn len(&self) -> u8 {
        self.len
    }

    const MAX_LEN: u8 = 64;
    type Addr = Ipv6Addr;

    fn network(&self) -> Self::Addr {
        Ipv6Addr::from_bits((self.slot as u128) << 64)
    }
}

impl IpPrivatePrefix for Ipv6NetRouting {
    #[inline]
    fn is_private(&self) -> bool {
        (self.bitslot() >> 57 == 0xfc >> 1 && self.len() >= 7) // fc00::/7
            || (self.bitslot() >> 16 == 0x64ff9b0001 && self.len() >= 48) // 64:ff9b:1::/48
    }
}

impl Debug for Ipv6NetRouting {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Ipv6NetRouting as Display>::fmt(self, f)
    }
}

impl Display for Ipv6NetRouting {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Ipv6Net as Display>::fmt(&(*self).into(), f)
    }
}

impl IpRootPrefix for Ipv6NetRouting
{
    fn root() -> Self {
        Ipv6NetRouting::new_assert(Ipv6Addr::UNSPECIFIED, 0)
    }
}


impl IpPrefixShortening for Ipv6NetRouting
{
    #[inline]
    fn shorten(&mut self, maxlen: u8) {
        if maxlen < self.len() {
            self.slot = self.slot & u64::bitmask(maxlen);
            self.len = maxlen;
        }
    }
}

impl From<Ipv6NetRouting> for Ipv6Net
{
    #[inline]
    fn from(value: Ipv6NetRouting) -> Self {
        Ipv6Net::new(value.network(), value.len()).unwrap()
    }
}

impl From<Ipv6NetRouting> for Ipv6Prefix
{
    #[inline] fn from(value: Ipv6NetRouting) -> Self {
        Self { addr: value.network().into(), len: value.len() }
    }
}

impl TryFrom<Ipv6Net> for Ipv6NetRouting
{
    type Error = IpPrefixError;
    #[inline]
    fn try_from(value: Ipv6Net) -> Result<Self, Self::Error> {
        Ok(Self::new(value.addr(), value.prefix_len())?)
    }
}

impl TryFrom<Ipv6Prefix> for Ipv6NetRouting
{
    type Error = IpPrefixError;
    #[inline]
    fn try_from(value: Ipv6Prefix) -> Result<Self, Self::Error> {
        Ok(Self::new(value.addr.into(), value.len())?)
    }
}

impl FromStr for Ipv6NetRouting {
    type Err = IpPrefixError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ipv6NetRouting::try_from(Ipv6Net::from_str(s)?)
    }
}