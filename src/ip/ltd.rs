use std::cmp::Ordering;
use std::marker::PhantomData;
use std::fmt;

use super::*;
use crate::ip::whole::IpCidrMask;


/// Mask use by limited prefix mask
///
/// See [`IpPrefixLtd`] for more explanations about what _limited_ means.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct IpMaskLtd<IP:Ip>(u8, PhantomData<IP>);

impl<IP:Ip> IpMaskLtd<IP>
{
    #[inline]
    fn new(m: u8) -> Self {
        debug_assert!( m <= Self::MAX_LEN);
        Self(m, Default::default())
    }
}


impl<IP:Ip> TryFrom<u8> for IpMaskLtd<IP>
{
    type Error = PrefixError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error>
    {
        if value <= Self::MAX_LEN {
            Ok(Self::new(value))
        } else {
            Err(PrefixError::TooLongMask)
        }
    }
}

impl<IP:Ip> Shr<u8> for IpMaskLtd<IP>
{
    type Output = Self;
    #[inline]
    fn shr(self, rhs: u8) -> Self::Output
    {
        if Self::MAX_LEN - self.0 <= self.0 {
            Self(Self::MAX_LEN, Default::default())
        } else {
            Self(self.0 + rhs, Default::default())
        }
    }
}

impl<IP:Ip> Shl<u8> for IpMaskLtd<IP>
{
    type Output = Self;
    #[inline]
    fn shl(self, rhs: u8) -> Self::Output
    {
        if self.0 <= rhs {
            Self(0, Default::default())
        } else {
            Self(self.0 - rhs, Default::default())
        }
    }
}

impl<IP:Ip> fmt::Debug for IpMaskLtd<IP>
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "/{}", self.len())
    }
}

impl<IP:Ip> fmt::Display for IpMaskLtd<IP>
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "/{}", self.len())
    }
}

impl<IP:Ip> IpMask<IP> for IpMaskLtd<IP>
{
    const MAX_LEN: u8 = IpCidrMask::<IP>::MAX_LEN - IpCidrMask::<IP>::MAX_LEN.trailing_zeros() as u8;

    #[inline] fn len(&self) -> u8 { self.0 }

    #[inline]
    fn bitmask(&self) -> IP
    {
        let ip = !IP::default();
        ip << IpCidrMask::<IP>::MAX_LEN - self.0
    }
}

/// Limited IP prefix (prefix which save memory space)
///
/// Limited IP prefix are prefix that save memory spaces by using trailing
/// bits to store the prefix length. By the way, not all the prefix space
/// could be represented but, in many cases, it is sufficient.
/// If it is not the case, one should consider [`IpWholePrefix`]
///
/// - For IPv4, the prefix is stored in 32 bits and is limited to /27
/// - For IPv6, the prefix is stored in 128 bits and is limited to /121
/// - For shorten IPv6, the prefix is stored in 64 bits and is limited to /58
#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct IpPrefixLtd<IP:Ip>(IP);

impl<IP:Ip> IpPrefix<IP> for IpPrefixLtd<IP> {

    type Mask = IpMaskLtd<IP>;

    fn new(addr: IP::Addr, mask: Self::Mask) -> Self {
        Self ((IP::from_addr(addr) & mask.bitmask()) | mask.0.into())
    }

    #[inline]
    fn mask(&self) -> Self::Mask {
        Self::Mask::new(self.len())
    }

    #[inline]
    fn len(&self) -> u8 {
        ((!IP::default() >> Self::Mask::MAX_LEN)  & self.0).into()
    }
}


impl<IP:Ip> IpPrefixMatch<IP> for IpPrefixLtd<IP>
{
    #[inline]
    fn matched<P:IpPrefix<IP>>(&self, pfx: &P) -> bool {
        pfx.overlaps(self)
    }
    #[inline]
    fn slot(&self) -> IP { self.0 & !(!IP::default() >> IpMaskLtd::<IP>::MAX_LEN) }
}

impl<IP:Ip> FromStr for IpPrefixLtd<IP>
{
    type Err = PrefixError;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> { super::parse_prefix(s) }
}

impl<IP:Ip> Shl<u8> for IpPrefixLtd<IP>
{
    type Output = Self;
    #[inline]
    fn shl(self, rhs: u8) -> Self {
        let mask = self.mask().shl(rhs);
        Self((self.slot() & mask.bitmask()) | mask.0.into())
    }
}

impl<IP:Ip> PartialOrd for IpPrefixLtd<IP>
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


impl<IP:Ip> fmt::Display for IpPrefixLtd<IP>
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.addr(), self.len())
    }
}

impl<IP:Ip> fmt::Debug for IpPrefixLtd<IP>
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { <Self as fmt::Display>::fmt(self, f) }
}