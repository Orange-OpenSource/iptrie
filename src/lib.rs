#![feature(unchecked_math)]
#![feature(drain_filter)]

mod ip;
mod trie;
mod patricia;
mod lctrie;
mod map;
mod set;

#[cfg(feature = "graphviz")] mod graphviz;

pub use map::{IpPrefixMap,IpPrefixLCMap};
pub use set::{IpPrefixSet,IpPrefixLCSet};

pub use ip::{Ip,Ipv4,Ipv6,Ipv6s};
pub use ip::{IpPrefix,IpPrefixMatch,IpWholePrefix,IpPrefixLtd};

pub type IpWholePrefixMap<IP,V> = IpPrefixMap<IP,IpWholePrefix<IP>,V>;
pub type IpWholePrefixSet<IP> = IpPrefixSet<IP,IpWholePrefix<IP>>;
pub type IpWholePrefixLCMap<IP,V> = IpPrefixLCMap<IP,IpWholePrefix<IP>,V>;
pub type IpWholePrefixLCSet<IP> = IpPrefixLCSet<IP,IpWholePrefix<IP>>;


pub type IpPrefixLtdMap<IP,V> = IpPrefixMap<IP,IpPrefixLtd<IP>,V>;
pub type IpPrefixLtdSet<IP> = IpPrefixSet<IP,IpPrefixLtd<IP>>;
pub type IpPrefixLtdLCMap<IP,V> = IpPrefixLCMap<IP,IpPrefixLtd<IP>,V>;
pub type IpPrefixLtdLCSet<IP> = IpPrefixLCSet<IP,IpPrefixLtd<IP>>;
