mod slot;
mod error;
mod whole;
mod ltd;
mod matching;
mod ipv6prefix64;

use std::fmt;
use std::str::FromStr;
use std::ops::{Shr, Shl};
use std::convert::{TryInto, TryFrom};

pub use slot::*;
pub use ltd::*;
pub use matching::*;
pub use whole::*;
pub use ipv6prefix64::Ipv6Prefix64;

pub use error::PrefixError;


/// Trait for IP prefix mask
pub trait IpMask<IP: Ip>
: Copy + Clone
+ Eq + PartialEq
+ Ord + PartialOrd
+ TryFrom<u8,Error=PrefixError>
+ Shl<u8,Output=Self> + Shr<u8,Output=Self>
+ Default + fmt::Debug
{
    /// The max length allowed by the mask
    const MAX_LEN: u8;

    /// The length of the mask
    ///
    /// This is the number of leading bits of the
    /// associated bitmask
    fn len(&self) -> u8;

    /// The bit mask associated to this mask
    ///
    /// The returned slot starts with CIDR leading bits set to 1.
    /// The number of 1-bits is also the length of the mask (see [`Self::len`]).
    fn bitmask(&self) -> IP;
}

/// Trait for any IP prefix
pub trait IpPrefix<IP:Ip>
: Copy + Clone
+ Eq + PartialEq + PartialOrd
+ Shl<u8,Output=Self>
+ FromStr<Err=PrefixError> + Default
+ fmt::Display + fmt::Debug
+ IpPrefixMatch<IP>
{
    type Mask: IpMask<IP>;

    /// Creates a new prefix from an IP address and a mask
    ///
    /// ```
    /// # use iptrie::ip::*;
    ///
    /// use std::convert::TryInto;
    /// let addr = "121.2.2.2".parse().unwrap();
    /// let mask = 24.try_into().unwrap();
    /// let pfx = IpPrefixLtd::<Ipv4>::new(addr, mask);
    ///
    /// assert_eq!( pfx.to_string(), "121.2.2.0/24" );
    /// ```
    fn new(addr: IP::Addr, mask: Self::Mask) -> Self;

    /// Gets the mask of this prefix
    ///
    /// ```
    /// # use iptrie::ip::*;
    /// let pfx : IpPrefixLtd::<Ipv4> = "121.0.0.0/8".parse().unwrap();
    ///
    /// assert_eq!( pfx.len(), 8 );
    /// assert_eq!( format!("{:b}",pfx.bitmask()), "11111111000000000000000000000000");
    /// ```
    fn mask(&self) -> Self::Mask;

    /// Gets the length of this prefix
    ///
    /// This is a shorthand for `self.mask().len()`.
    ///
    /// ```
    /// # use iptrie::ip::*;
    /// let pfx : IpPrefixLtd::<Ipv4> = "121.2.2.0/25".parse().unwrap();
    ///
    /// assert_eq!( pfx.len(), 25 );
    /// ```
    #[inline]
    fn len(&self) -> u8 { self.mask().len() }

    /// Gets the bitmask of this prefix
    ///
    /// This is a shorthand for `self.mask().bitmask()`.
    ///
    /// ```
    /// # use iptrie::ip::*;
    /// let pfx : IpPrefixLtd::<Ipv4> = "121.0.0.0/8".parse().unwrap();
    ///
    /// assert_eq!( format!("{:b}",pfx.bitmask()), "11111111000000000000000000000000");
    /// ```
    #[inline]
    fn bitmask(&self) -> IP { self.mask().bitmask() }

    /// Gets the IP address of this prefix.
    ///
    /// ```
    /// # use iptrie::ip::*;
    /// let pfx : IpPrefixLtd::<Ipv4> = "121.2.2.0/25".parse().unwrap();
    ///
    /// assert_eq!( pfx.addr().to_string(), "121.2.2.0");
    /// ```
    #[inline]
    fn addr(&self) -> IP::Addr { self.slot().to_addr() }

    /// Checks if this prefix overlaps an other one.
    ///
    /// This function corresponds to a kind of containment relationship.
    /// It defines a partial order among the prefix set.
    /// ```
    /// # use iptrie::ip::*;
    /// let a : IpPrefixLtd::<Ipv4> = "121.2.2.0/24".parse().unwrap();
    /// let b : IpPrefixLtd::<Ipv4> = "121.2.0.0/18".parse().unwrap();
    /// let c : IpPrefixLtd::<Ipv4> = "121.1.2.0/20".parse().unwrap();
    ///
    /// assert!( ! a.overlaps( &b) );
    /// assert!( ! a.overlaps( &c) );
    ///
    /// assert!( b.overlaps( &a) );
    /// assert!( ! b.overlaps( &c) );
    ///
    /// assert!( ! c.overlaps( &a) );
    /// assert!( ! c.overlaps( &b) );
    /// ```
    #[inline]
    fn overlaps<P: IpPrefix<IP>>(&self, prefix: &P) -> bool {
        prefix.slot() & self.bitmask() == self.slot() && self.len() <= prefix.len()
    }

    /// Checks if this prefix matches an IP address.
    ///
    /// ```
    /// # use iptrie::ip::*;
    /// let pfx : IpPrefixLtd::<Ipv4> = "121.2.2.0/24".parse().unwrap();
    ///
    /// assert!( pfx.matches( &"121.2.2.2".parse().unwrap()) );
    /// assert!( ! pfx.matches( &"121.2.1.2".parse().unwrap()) );
    /// ```
    #[inline]
    fn matches(&self, addr: &IP::Addr) -> bool {
        IP::from_addr(*addr) & self.bitmask() == self.slot()
    }
}

fn parse_prefix<IP:Ip,P:IpPrefix<IP>>(s: &str) -> Result<P,PrefixError>
{
    s.find('/')
        .ok_or(PrefixError::MissingMask)
        .and_then(|pos|
            Ok(P::new(
                s[0..pos]
                    .parse::<IP::Addr>()
                    .map_err(|_| PrefixError::InvalidAddress)?,
                s[pos + 1..]
                    .parse::<u8>()
                    .map_err(|_| PrefixError::InvalidMask)?
                    .try_into()?
            )))
}

/// Any IP prefix representation
///
/// This representation could be memory overconsuming if the necessary
/// prefix are limited (i.e. /24 for IPv4). If it is the case, one
/// should consider [`IpPrefixLtd`].
pub type IpWholePrefix<IP> = whole::IpWholePrefix<IP,whole::IpCidrMask<IP>>;




#[cfg(test)]
mod tests {
    use crate::ip::*;
    use crate::ip::whole::{IpCidrMask, IpBitMask};

    fn generic_ipv4_tests<P:IpPrefix<Ipv4>>()
    {
        let prefix = "127.1.1.3/24".parse::<P>().unwrap();
        assert_eq!(prefix, "127.1.1.0/24".parse().unwrap());
        assert_eq!(prefix << 1, "127.1.0.0/23".parse().unwrap());
        assert_eq!(prefix << 18, "124.0.0.0/6".parse().unwrap());
        assert_eq!(prefix << 36, "0.0.0.0/0".parse().unwrap());
    }

    fn generic_ipv6_tests<P:IpPrefix<Ipv6>>()
    {
        let prefix = "2021:2021:2021:2021::/121".parse::<P>().unwrap();
        assert_eq!(prefix, "2021:2021:2021:2021::/121".parse().unwrap());
        assert_eq!(prefix << 69, "2021:2021:2021:2000::/52".parse().unwrap());
        assert_eq!(prefix << 150, "::/0".parse().unwrap());
    }
    fn generic_ipv6s_tests<P:IpPrefix<Ipv6s>>()
    {
        let prefix = "2021:2021:2021:2021::/52".parse::<P>().unwrap();
        assert_eq!(prefix, "2021:2021:2021:2000::/52".parse().unwrap());
        assert_eq!(prefix << 1, "2021:2021:2021:2000::/51".parse().unwrap());
        assert_eq!(prefix << 4, "2021:2021:2021::/48".parse().unwrap());
        assert_eq!(prefix << 20, "2021:2021::/32".parse().unwrap());
        assert_eq!(prefix << 22, "2021:2020::/30".parse().unwrap());
        assert_eq!(prefix << 60, "::/0".parse().unwrap());
    }

    #[test]
    fn prefixes()
    {
        // here, we test all the derivations of IpWholePrefix
        // (in practice, only the ip::IpWholePrefix is used)
        generic_ipv4_tests::<IpPrefixLtd<Ipv4>>();
        generic_ipv4_tests::<whole::IpWholePrefix<Ipv4,IpCidrMask<Ipv4>>>();
        generic_ipv4_tests::<whole::IpWholePrefix<Ipv4,IpBitMask<Ipv4>>>();

        generic_ipv6_tests::<IpPrefixLtd<Ipv6>>();
        generic_ipv6_tests::<whole::IpWholePrefix<Ipv6,IpCidrMask<Ipv6>>>();
        generic_ipv6_tests::<whole::IpWholePrefix<Ipv6,IpBitMask<Ipv6>>>();

        generic_ipv6s_tests::<IpPrefixLtd<Ipv6s>>();
        generic_ipv6s_tests::<whole::IpWholePrefix<Ipv6s,IpCidrMask<Ipv6s>>>();
        generic_ipv6s_tests::<whole::IpWholePrefix<Ipv6s,IpBitMask<Ipv6s>>>();
    }
}