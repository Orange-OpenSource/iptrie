use std::cmp::Ordering;
use ipnet::{Ipv4Net,Ipv6Net};
use std::net::{Ipv4Addr,Ipv6Addr};
use crate::*;

#[doc(hidden)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum IpPrefixCoverage { NoCover, WiderRange, SameRange }

impl IpPrefixCoverage {
    #[inline] fn is_wider(&self) -> bool { *self == IpPrefixCoverage::WiderRange }
    #[inline] fn is_same(&self) -> bool { *self == IpPrefixCoverage::SameRange }
    #[inline] fn is_covering(&self) -> bool { *self != IpPrefixCoverage::NoCover }
}

/// A trait to check the prefix coverage
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
    #[doc(hidden)]
    fn covering(&self, other: &P) -> IpPrefixCoverage;

    /// Checks the coverage of this prefix against a set of addresses
    ///
    /// # Difference with `PartialEq`
    /// Two prefixes could be different but equivalent regarding to
    /// the range of addresses they cover.
    /// ```
    /// # use iptrie::*;
    /// use ipnet::Ipv4Net;
    /// let a = "1.1.1.1/16".parse::<Ipv4Net>().unwrap();
    /// let b = "1.1.2.2/16".parse::<Ipv4Net>().unwrap();
    /// assert!( a != b ); // host addr are different
    /// assert! (a.covers(&b) && b.covers(&a)); // but prefixes are equivalent
    /// ```
    #[inline]
    fn covers(&self, other: &P) -> bool{ self.covering(other).is_covering() }

    #[inline] #[doc(hidden)]
    fn covers_striclty(&self, other: &P) -> bool{ self.covering(other).is_wider() }
    #[inline] #[doc(hidden)]
    fn covers_equally(&self, other: &P) -> bool{ self.covering(other).is_same() }
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
ipcover!(Ipv6Prefix, Ipv6Addr);

ipcover!(Ipv6Net, Ipv6Prefix);
ipcover!(Ipv6Net, Ipv6Addr);




// coverage impl. relative to IpPrefix56 or IpNetAddr
// (which has a slot shorter than the others)
macro_rules! ipcover_for_ipv6_on_u64 {
    ($short:ty, $other:ty) => {
        impl IpPrefixCovering<$other> for $short {
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

ipcover_for_ipv6_on_u64!(Ipv6NetRouting, Ipv6Prefix);
ipcover_for_ipv6_on_u64!(Ipv6NetRouting, Ipv6Net);
ipcover_for_ipv6_on_u64!(Ipv6NetRouting, Ipv6Addr);


macro_rules! ipcover_of_ipv6_on_u64 {
    ($self:ty, $short:ty) => {
        impl IpPrefixCovering<$short> for $self {
            #[inline]
            fn covering(&self, other: &$short) -> IpPrefixCoverage {
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

ipcover_of_ipv6_on_u64!(Ipv6Prefix, Ipv6NetRouting);
ipcover_of_ipv6_on_u64!(Ipv6Net, Ipv6NetRouting);




// Equality between prefix...
macro_rules! ipprefix_eq {
    ($self:ty, $other:ty) => {
        impl PartialEq<$other> for $self {
            #[inline]
            fn eq(&self, other: &$other) -> bool {
                self.covers_equally(other)
            }
        }
    }
}

ipprefix_eq!(Ipv6Net,Ipv6NetRouting);
ipprefix_eq!(Ipv6Net,Ipv6Prefix);

ipprefix_eq!(Ipv6NetRouting,Ipv6Net);
ipprefix_eq!(Ipv6NetRouting,Ipv6Prefix);

ipprefix_eq!(Ipv6Prefix,Ipv6Net);
ipprefix_eq!(Ipv6Prefix,Ipv6NetRouting);
