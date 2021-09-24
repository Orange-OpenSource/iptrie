use std::str::FromStr;
use std::ops::*;
use std::fmt;
use std::hash::Hash;
use crate::ip::{IpPrefix,IpPrefixMatch};

/// Generic tag to specify the IP slot to use
pub trait Ip
: Clone + Copy + Default
+ fmt::Debug + fmt::Binary
+ Eq + PartialEq + Ord + PartialOrd + Hash
+ Not<Output=Self> + BitAnd<Output=Self> + BitOr<Output=Self> + BitXor<Output=Self>
+ Shl<u8,Output=Self> + Shr<u8,Output=Self>
+ From<u8> + Into<u8> + From<u16> + Into<u16>
+ IpPrefixMatch<Self>
{
    type Addr : Clone + Copy + Eq + PartialEq + FromStr + fmt::Display + fmt::Debug;

    fn from_addr(addr: Self::Addr) -> Self;
    fn to_addr(&self) -> Self::Addr;
    fn leading_ones(&self) -> u8;
    fn first_bit(&self) -> u8;
    fn single_bit(pos: u8) -> Self;
 }

/// Tag to use IPv4 prefixes (/32 or /27 for ltd)
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Ord, PartialOrd, Hash)]
pub struct Ipv4(u32);
impl Ip for Ipv4 {
    type Addr = std::net::Ipv4Addr;
    #[inline]
    fn from_addr(addr: Self::Addr) -> Self { Self(addr.into()) }
    #[inline]
    fn to_addr(&self) -> Self::Addr { self.0.into() }
    #[inline]
    fn leading_ones(&self) -> u8 { self.0.leading_ones() as u8 }
    #[inline]
    fn first_bit(&self) -> u8 { self.0.leading_zeros() as u8 + 1}
    #[inline]
    fn single_bit(pos: u8) -> Self {
        debug_assert!( pos > 0);
        debug_assert!( pos <= 32);
        Self( 1 << (32-pos))
    }
}

/// Tag to use short IPv6 prefixes (/64 or /58 for ltd)
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Ord, PartialOrd, Hash)]
pub struct Ipv6s(u64);
impl Ip for Ipv6s {
    type Addr = std::net::Ipv6Addr;
    #[inline]
    fn from_addr(addr: Self::Addr) -> Self {
        let addr : u128 = addr.into();
        Self((addr >> 64) as u64)
    }
    #[inline]
    fn to_addr(&self) -> Self::Addr {
        let addr : u128 = (self.0 as u128) << 64;
        addr.into()
    }
    #[inline]
    fn leading_ones(&self) -> u8 { self.0.leading_ones() as u8 }
    #[inline]
    fn first_bit(&self) -> u8 { self.0.leading_zeros() as u8 + 1}
    #[inline]
    fn single_bit(pos: u8) -> Self {
        debug_assert!( pos > 0);
        debug_assert!( pos <= 64);
        Self( 1 << (64-pos))
    }
}

/// Tag to use IPv6 prefixes (/128 or /121 for ltd)
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Ord, PartialOrd, Hash)]
pub struct Ipv6(u128);
impl Ip for Ipv6 {
    type Addr = std::net::Ipv6Addr;
    #[inline]
    fn from_addr(addr: Self::Addr) -> Self { Self(addr.into()) }
    #[inline]
    fn to_addr(&self) -> Self::Addr { self.0.into() }
    #[inline]
    fn leading_ones(&self) -> u8 { self.0.leading_ones() as u8 }
    #[inline]
    fn first_bit(&self) -> u8 { self.0.leading_zeros() as u8 + 1}
    #[inline]
    fn single_bit(pos: u8) -> Self {
        debug_assert!( pos > 0);
        debug_assert!( pos <= 128);
        Self( 1 << (128-pos))
    }
}


macro_rules! ipslot {
    ($X:ty, $B:ty) => {
        impl Shl<u8> for $X {
            type Output = Self;
            #[inline] fn shl(self, rhs: u8) -> Self { Self(self.0.checked_shl(rhs.into()).unwrap_or(0)) }
        }
        impl Shr<u8> for $X {
            type Output = Self;
            #[inline] fn shr(self, rhs: u8) -> Self { Self(self.0.checked_shr(rhs.into()).unwrap_or(0)) }
        }
        impl BitXor for $X {
            type Output = Self;
            #[inline] fn bitxor(self, rhs: Self) -> Self { Self(self.0 ^ rhs.0) }
        }
        impl BitOr for $X {
            type Output = Self;
            #[inline] fn bitor(self, rhs: Self) -> Self { Self(self.0 | rhs.0) }
        }
        impl BitAnd for $X {
            type Output = Self;
            #[inline] fn bitand(self, rhs: Self) -> Self { Self(self.0 & rhs.0) }
        }
        impl Not for $X {
            type Output = Self;
            #[inline] fn not(self) -> Self { Self(!self.0) }
        }
        impl From<u8> for $X {
            #[inline] fn from(v: u8) -> Self { Self ( v.into() ) }
        }
        impl From<u16> for $X {
            #[inline] fn from(v: u16) -> Self { Self ( v.into() ) }
        }
        impl Into<u8> for $X {
            #[inline] fn into(self) -> u8 { debug_assert!(self.0 <= u8::MAX.into()); self.0 as u8 }
        }
        impl Into<u16> for $X {
            #[inline] fn into(self) -> u16 { /*debug_assert!(self.0 <= u16::MAX.into());*/ self.0 as u16 }
        }
        impl fmt::Binary for $X {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { fmt::Binary::fmt(&self.0, f) }
        }
        impl Into<$B> for $X {
            #[inline] fn into(self) -> $B { self.0 }
        }
        impl From<$B> for $X {
            #[inline] fn from(v:$B) -> Self { Self(v) }
        }
        impl IpPrefixMatch<Self> for $X {
            #[inline]
            fn matched<P:IpPrefix<$X>>(&self, pfx: &P) -> bool {
                pfx.matches(&self.to_addr())
            }
            #[inline] fn slot(&self) -> $X { *self }
        }
    };
}

ipslot!(Ipv4, u32);
ipslot!(Ipv6, u128);
ipslot!(Ipv6s,u64);

