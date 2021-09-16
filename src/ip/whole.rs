
use super::*;

use std::marker::PhantomData;
use std::ops::{Shl, Shr};
use std::fmt;
use std::cmp::Ordering;


#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct IpWholePrefix<IP, MASK>
    where IP: Ip, MASK: IpMask<IP>
{
    addr: IP,
    mask: MASK
}


impl <IP,MASK> IpPrefix<IP> for IpWholePrefix<IP,MASK>
    where IP: Ip, MASK: IpMask<IP>
{
    type Mask = MASK;

    #[inline]
    fn new(addr: IP::Addr, mask: MASK) -> Self {
        Self { addr: IP::from_addr(addr) & mask.bitmask(), mask }
    }

    #[inline]
    fn mask(&self) -> Self::Mask { self.mask }

}

impl <IP,MASK> IpPrefixMatch<IP> for IpWholePrefix<IP,MASK>
    where IP: Ip, MASK: IpMask<IP>
{
    #[inline]
    fn matched<P:IpPrefix<IP>>(&self, pfx: &P) -> bool {
        pfx.overlaps(self)
    }
    #[inline]
    fn slot(&self) -> IP { self.addr }
}

impl<IP,MASK> FromStr for IpWholePrefix<IP, MASK>
    where IP: Ip, MASK: IpMask<IP>
{
    type Err = PrefixError;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> { super::parse_prefix(s) }
}

impl<IP,MASK> Shl<u8> for IpWholePrefix<IP, MASK>
    where IP: Ip, MASK: IpMask<IP>
{
    type Output = Self;
    #[inline]
    fn shl(self, rhs: u8) -> Self {
        let mask = self.mask.shl(rhs);
        Self {
            addr: self.addr & mask.bitmask(),
            mask
        }
    }
}

impl<IP,MASK> PartialOrd for IpWholePrefix<IP, MASK>
    where IP: Ip, MASK: IpMask<IP>
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

impl<IP,MASK> fmt::Display for IpWholePrefix<IP, MASK>
    where IP: Ip, MASK: IpMask<IP>
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.addr.to_addr(), self.len())
    }
}

impl<IP,MASK> fmt::Debug for IpWholePrefix<IP, MASK>
    where IP: Ip, MASK: IpMask<IP>
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { <Self as fmt::Display>::fmt(self, f) }
}


/// A mask implementation based on the length.
///
/// Only the length is stored: small memory use but
/// more computation to match the prefix which uses it.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct IpCidrMask<IP:Ip>(u8, PhantomData<IP>);
// note: we store the number of trailing zeroes, not the cidr
// (the purpose is to compute the bitmask more easily: is it correct ?)

impl<IP:Ip> IpCidrMask<IP>
{
    #[inline]
    fn new(m: u8) -> Self {
        debug_assert!( m <= Self::MAX_LEN);
        Self(Self::MAX_LEN - m, Default::default())
    }
}

impl <IP:Ip> Default for IpCidrMask<IP>
{
    #[inline]
    fn default() -> Self { Self::new(0) }
}

impl<IP:Ip> Ord for IpCidrMask<IP>
{
    #[inline] // reordering is reversed due to internal repr.
    fn cmp(&self, other: &Self) -> Ordering { other.0.cmp(&self.0) }
}

impl<IP:Ip> PartialOrd for IpCidrMask<IP>
{
    #[inline] // reordering is reversed due to internal repr.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { other.0.partial_cmp(&self.0) }
}


impl <IP:Ip> IpMask<IP> for IpCidrMask<IP>
{
    const MAX_LEN: u8 = 8 * std::mem::size_of::<IP>() as u8;

    #[inline]
    fn len(&self) -> u8 { Self::MAX_LEN - self.0 }

    #[inline]
    fn bitmask(&self) -> IP
    {
        let ip = !IP::default();
        if self.0 >= Self::MAX_LEN { IP::default() } else { ip << self.0 }
    }
}

impl<IP:Ip> TryFrom<u8> for IpCidrMask<IP>
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

impl<IP:Ip> Shr<u8> for IpCidrMask<IP>
{
    type Output = Self;
    #[inline]
    fn shr(self, rhs: u8) -> Self::Output
    {
        if rhs >= self.0 {
            Self(0, Default::default())
        } else {
            Self(self.0 - rhs, Default::default())
        }
    }
}

impl<IP:Ip> Shl<u8> for IpCidrMask<IP>
{
    type Output = Self;
    #[inline]
    fn shl(self, rhs: u8) -> Self::Output
    {
        if Self::MAX_LEN - self.0 <= rhs {
            Self(Self::MAX_LEN, Default::default())
        } else {
            Self(self.0 + rhs, Default::default())
        }
    }
}

impl<IP:Ip> fmt::Display for IpCidrMask<IP>
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "/{}", self.len())
    }
}



/// A mask implementation based on bitmask.
///
/// Needs more memory (especially for Ipv6) but ready
/// to compute prefix matching.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub(crate) struct IpBitMask<IP:Ip>(IP);
// note: we store the bitmask, more memory... but ready to use

impl<IP:Ip> IpBitMask<IP>
{
    fn new(m: u8) -> Self {
        debug_assert!( m <= Self::MAX_LEN);
        Self ( (!IP::default()) << (Self::MAX_LEN - m) )
    }
}

impl <IP:Ip> IpMask<IP> for IpBitMask<IP>
{
    const MAX_LEN: u8 = 8 * std::mem::size_of::<IP>() as u8;

    #[inline]
    fn len(&self) -> u8 { self.0.leading_ones() }

    #[inline]
    fn bitmask(&self) -> IP { self.0 }
}

impl<IP:Ip> TryFrom<u8> for IpBitMask<IP>
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

impl<IP:Ip> Shr<u8> for IpBitMask<IP>
{
    type Output = Self;
    #[inline]
    fn shr(self, rhs: u8) -> Self::Output { Self( !((!self.0) >> rhs)) }
}

impl<IP:Ip> Shl<u8> for IpBitMask<IP>
{
    type Output = Self;
    #[inline]
    fn shl(self, rhs: u8) -> Self::Output {
        Self(self.0.shl(rhs))
    }
}



impl<IP:Ip> fmt::Display for IpBitMask<IP>
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "/{}", self.len())
    }
}
