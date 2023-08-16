use std::cmp::Ordering;
use ipnet::{Ipv4Net,Ipv6Net};
use std::net::{Ipv4Addr,Ipv6Addr};
use crate::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum IpPrefixCoverage { NoCover, WiderRange, SameRange }

impl IpPrefixCoverage {
    #[inline] fn is_none(&self) -> bool { *self == IpPrefixCoverage::NoCover }
    #[inline] fn is_wider(&self) -> bool { *self == IpPrefixCoverage::WiderRange }
    #[inline] fn is_same(&self) -> bool { *self == IpPrefixCoverage::SameRange }
    #[inline] fn is_covering(&self) -> bool { !self.is_none() }
}

/// A trait to check if the prefix covers the specified data
pub trait IpPrefixCovering<P>
{
    /// Checks the coverage of this prefix against a set of adress
    ///
    /// * `SameRange` means that the two prefixes are equivalent
    /// * `WiderRange` means that this prefix includes the other
    /// * `NoCover` groups all the other cases
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

impl<P:IpPrefix> IpPrefixCovering<Self> for P
{
    #[inline]
    fn covering(&self, other: &Self) -> IpPrefixCoverage {
        if other.bitslot() & self.bitmask() != self.bitslot_trunc() {
            IpPrefixCoverage::NoCover
        } else {
            match self.len().cmp(&other.len()) {
                Ordering::Less => IpPrefixCoverage::WiderRange,
                Ordering::Equal => IpPrefixCoverage::SameRange,
                Ordering::Greater => IpPrefixCoverage::NoCover
            }
        }
    }
}

// coverage impl. between prefixes with same slots and same addr type
macro_rules! ipcover {
    ($self:ty, $other:ty) => {
        impl IpPrefixCovering<$other> for $self {
            #[inline]
            fn covering(&self, other: &$other) -> IpPrefixCoverage {
                if other.bitslot() & self.bitmask() != self.bitslot_trunc() {
                    IpPrefixCoverage::NoCover
                } else {
                    match self.len().cmp(&other.len()) {
                        Ordering::Less => IpPrefixCoverage::WiderRange,
                        Ordering::Equal => IpPrefixCoverage::SameRange,
                        Ordering::Greater => IpPrefixCoverage::NoCover
                    }
                }
            }
        }
    }
}

ipcover!(Ipv4Prefix, Ipv4Net);
ipcover!(Ipv4Prefix, Ipv4Addr);

ipcover!(Ipv4Net, Ipv4Prefix);
ipcover!(Ipv4Net, Ipv4Addr);

ipcover!(Ipv6Prefix, Ipv6Net);
ipcover!(Ipv6Prefix, Ipv6Prefix120);
ipcover!(Ipv6Prefix, Ipv6Addr);

ipcover!(Ipv6Prefix120, Ipv6Net);
ipcover!(Ipv6Prefix120, Ipv6Prefix);
ipcover!(Ipv6Prefix120, Ipv6Addr);

ipcover!(Ipv6Net, Ipv6Prefix);
ipcover!(Ipv6Net, Ipv6Prefix120);
ipcover!(Ipv6Net, Ipv6Addr);




// coverage impl. relative to IpPrefix56
// (which has a slot shorter than the others)
macro_rules! ipcover_for_56 {
    ($other:ty) => {
        impl IpPrefixCovering<$other> for Ipv6Prefix56 {
            #[inline]
            fn covering(&self, other: &$other) -> IpPrefixCoverage {
                if ((other.bitslot() >> 64) as u64) & self.bitmask() != self.bitslot_trunc() {
                    IpPrefixCoverage::NoCover
                } else {
                    match self.len().cmp(&other.len()) {
                        Ordering::Less => IpPrefixCoverage::WiderRange,
                        Ordering::Equal => IpPrefixCoverage::SameRange,
                        Ordering::Greater => IpPrefixCoverage::NoCover
                    }
                }
            }
        }
    }
}

ipcover_for_56!(Ipv6Prefix);
ipcover_for_56!(Ipv6Prefix120);
ipcover_for_56!(Ipv6Net);
ipcover_for_56!(Ipv6Addr);


macro_rules! ipcover_of_56 {
    ($self:ty) => {
        impl IpPrefixCovering<Ipv6Prefix56> for $self {
            #[inline]
            fn covering(&self, other: &Ipv6Prefix56) -> IpPrefixCoverage {
                if ((other.bitslot() as u128) << 64) & self.bitmask() != self.bitslot_trunc() {
                    IpPrefixCoverage::NoCover
                } else {
                    match self.len().cmp(&other.len()) {
                        Ordering::Less => IpPrefixCoverage::WiderRange,
                        Ordering::Equal => IpPrefixCoverage::SameRange,
                        Ordering::Greater => IpPrefixCoverage::NoCover
                    }
                }
            }
        }
    }
}

ipcover_of_56!(Ipv6Prefix);
ipcover_of_56!(Ipv6Prefix120);
ipcover_of_56!(Ipv6Net);
