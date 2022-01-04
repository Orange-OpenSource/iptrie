use crate::ip::{Ipv6s, IpPrefix, IpPrefixMatch, IpMask, Ip};
use std::fmt::{Debug, Formatter, Display};
use std::str::FromStr;
use crate::ip::error::PrefixError;
use std::ops::{Shl, Shr};
use std::cmp::Ordering;
use std::net::Ipv6Addr;
use std::convert::TryFrom;

#[derive(Copy, Clone, Default, Hash, PartialEq, Eq)]
pub struct Ipv6Prefix64(Ipv6s);

impl IpPrefix<Ipv6s> for Ipv6Prefix64
{
    type Mask = Ipv6Mask64;

    #[inline]
    fn new(addr: Ipv6Addr, mask: Self::Mask) -> Self {
        let addr : u128 = addr.into();
        Self ( ((addr >> 64) as u64).into() )
    }

    #[inline]
    fn mask(&self) -> Self::Mask { Ipv6Mask64 {} }

    #[inline]
    fn len(&self) -> u8 { 64 }

    #[inline]
    fn bitmask(&self) -> Ipv6s { (!0u64).into() }

    #[inline]
    fn matches(&self, addr: &Ipv6Addr) -> bool {
        self.0 == Ipv6s::from_addr(*addr)
    }
}

impl IpPrefixMatch<Ipv6s> for Ipv6Prefix64
{
    #[inline]
    fn matched<P: IpPrefix<Ipv6s>>(&self, pfx: &P) -> bool { pfx.overlaps(self) }

    #[inline]
    fn slot(&self) -> Ipv6s { self.0 }
}


impl Shl<u8> for Ipv6Prefix64
{
    type Output = Self;
    #[inline]
    fn shl(self, rhs: u8) -> Self {
        panic!("can’t shift a fixed prefix")
    }
}

impl PartialOrd for Ipv6Prefix64
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        // todo : improve it !!
        if self == other {
            Some(Ordering::Equal)
        } else if self.overlaps(other) {
            Some(Ordering::Greater)
        } else if other.overlaps(self) {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}

impl From<Ipv6s> for Ipv6Prefix64
{
    #[inline]
    fn from(slot: Ipv6s) -> Self { Self(slot) }
}

impl FromStr for Ipv6Prefix64
{
    type Err = PrefixError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl Debug for Ipv6Prefix64
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { Display::fmt(self, f) }
}


impl Display for Ipv6Prefix64
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/64", self.addr())
    }
}


#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ipv6Mask64;

impl IpMask<Ipv6s> for Ipv6Mask64
{
    const MAX_LEN: u8 = 64;
    #[inline] fn len(&self) -> u8 { 64 }
    #[inline] fn bitmask(&self) -> Ipv6s { (!0u64).into() }
}

impl TryFrom<u8> for Ipv6Mask64
{
    type Error = PrefixError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error>
    {
        if value == 64 {
            Ok(Self{})
        } else {
            Err(PrefixError::InvalidMask)
        }
    }
}


impl Shr<u8> for Ipv6Mask64
{
    type Output = Self;
    #[inline]
    fn shr(self, rhs: u8) -> Self::Output { panic!("can’t change length of a fixed mask") }
}

impl Shl<u8> for Ipv6Mask64
{
    type Output = Self;
    #[inline]
    fn shl(self, rhs: u8) -> Self::Output {
        panic!("can’t change length of a fixed mask")
    }
}
