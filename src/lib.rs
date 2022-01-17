mod ip;
mod trie;
mod patricia;
mod lctrie;
mod map;
mod set;

#[cfg(feature = "graphviz")] pub mod graphviz;

pub use map::{IpPrefixMap,IpPrefixLCMap};
pub use set::{IpPrefixSet,IpPrefixLCSet};

pub use crate::ip::{Ip,Ipv4,Ipv6,IpPrefixLtd,IpWholePrefix};

pub type Ipv4Prefix = IpWholePrefix<Ipv4>;
pub type Ipv6Prefix = IpWholePrefix<Ipv6>;

pub type IpWholePrefixMap<IP,V> = IpPrefixMap<IP,IpWholePrefix<IP>,V>;
pub type IpWholePrefixSet<IP> = IpPrefixSet<IP,IpWholePrefix<IP>>;
pub type IpWholePrefixLCMap<IP,V> = IpPrefixLCMap<IP,IpWholePrefix<IP>,V>;
pub type IpWholePrefixLCSet<IP> = IpPrefixLCSet<IP,IpWholePrefix<IP>>;


pub type IpPrefixLtdMap<IP,V> = IpPrefixMap<IP,IpPrefixLtd<IP>,V>;
pub type IpPrefixLtdSet<IP> = IpPrefixSet<IP,IpPrefixLtd<IP>>;
pub type IpPrefixLtdLCMap<IP,V> = IpPrefixLCMap<IP,IpPrefixLtd<IP>,V>;
pub type IpPrefixLtdLCSet<IP> = IpPrefixLCSet<IP,IpPrefixLtd<IP>>;


#[derive(Clone,Copy,Debug,Hash,Eq,PartialEq)]
pub enum IpPrefix {
    V4(Ipv4Prefix),
    V6(Ipv6Prefix)
}


use std::fmt;
impl fmt::Display for IpPrefix {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpPrefix::V4(pfx) => write!(f, "{}", pfx),
            IpPrefix::V6(pfx) => write!(f, "{}", pfx),
        }
    }
}

use std::str::FromStr;
impl FromStr for IpPrefix
{
    type Err = ip::PrefixError;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.find(':') {
            None => Ok(IpPrefix::V4(s.parse()?)),
            Some(_) => Ok(IpPrefix::V6(s.parse()?))
        }
    }
}