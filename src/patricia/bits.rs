use crate::ip::{Ip, Ipv4, Ipv6, Ipv6s};
use std::ops::{Shr, Shl};
use std::marker::PhantomData;
use std::fmt::{Debug, Formatter, Display};
use std::cmp::Ordering;

pub(crate) trait BitMatch<T:Ip>
: Sized + Copy + Clone
+ From<u8> + Into<u8>
+ Shr<u8,Output=Self> + Shl<u8,Output=Self>
+ Eq + PartialEq + Ord + PartialOrd
+ Debug
{
    fn from_first_bit(slot: T) -> Self;
    fn is_set(&self, slot:&T) -> bool;
}

/// first impl: based on the index of a bit in an Ip slot
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct BitIndex<T:Ip>(u8,PhantomData<T>);

impl<T:Ip> Debug for BitIndex<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}


impl<T:Ip> From<u8> for BitIndex<T>
{
    #[inline]
    fn from(i: u8) -> Self {
       // debug_assert!( i > 0 );
        //debug_assert!( dbg!(i) as usize <= 8 * std::mem::size_of::<T>());
        Self(i, Default::default())
    }
}


impl<T:Ip> Into<u8> for BitIndex<T>
{
    #[inline]
    fn into(self) -> u8 { self.0 }
}

impl<T:Ip> Shr<u8> for BitIndex<T> {
    type Output = Self;
    #[inline]
    fn shr(self, rhs: u8) -> Self::Output {
        //debug_assert!( self.0 as usize + rhs as usize <= 8 * std::mem::size_of::<T>());
        BitIndex( self.0+rhs, self.1)
    }
}

impl<T:Ip> Shl<u8> for BitIndex<T> {
    type Output = Self;
    #[inline]
    fn shl(self, rhs: u8) -> Self::Output {
        debug_assert!( self.0 > rhs );
        BitIndex( self.0 - rhs , self.1)
    }
}

impl<T:Ip> BitMatch<T> for BitIndex<T>
{
    #[inline]
    fn is_set(&self, slot:&T) -> bool {
        *slot & T::single_bit(self.0) != T::default()
    }
    #[inline]
    fn from_first_bit(slot: T) -> Self { slot.first_bit().into() }
}

impl<T:Ip> Display for BitIndex<T>
{
    #[inline] fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { Display::fmt(&self.0, f) }
}

/// second impl: based on an ip mask (with only one bit set)
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) struct BitMask<T:Ip>(T);


// reverse order
impl<T: Ip> Ord for BitMask<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering { other.0.cmp(&self.0) }
}

// reverse order
impl<T: Ip> PartialOrd for BitMask<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { other.0.partial_cmp(&self.0) }
}



macro_rules! bitmask
{
    ($X:ty) => {
        impl BitMatch<$X> for BitMask<$X> {
            #[inline]
            fn is_set(&self, slot:&$X) -> bool { *slot & self.0 != <$X>::default() }
            #[inline]
            fn from_first_bit(slot: $X) -> Self { slot.first_bit().into() }
        }
        impl Into<u8> for BitMask<$X>
        {
            #[inline]
            fn into(self) -> u8 { todo!() }
        }
        impl From<u8> for BitMask<$X> {
            #[inline]
            fn from(n: u8) -> Self {
                debug_assert!( n > 0);
                debug_assert!( n as usize <= 8*std::mem::size_of::<$X>());
                BitMask(<$X>::single_bit(n))
            }
        }
        impl Shr<u8> for BitMask<$X> {
            type Output = Self;
            #[inline]
            fn shr(self, rhs: u8) -> Self {
                debug_assert!( (rhs as usize) < { 8*std::mem::size_of::<$X>() });
                let mask = self.0.shr(rhs);
                debug_assert_ne!(mask, Default::default());
                Self(mask)
            }
        }
        impl Shl<u8> for BitMask<$X> {
            type Output = Self;
            #[inline]
            fn shl(self, rhs: u8) -> Self {
                debug_assert!( (rhs as usize) < { 8*std::mem::size_of::<$X>() });
                let mask = self.0.shl(rhs);
                debug_assert_ne!(mask, Default::default());
                Self(mask)
            }
        }
    };
}

bitmask!(Ipv4);
bitmask!(Ipv6);
bitmask!(Ipv6s);
