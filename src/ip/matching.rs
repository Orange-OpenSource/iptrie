use crate::ip::*;
use std::net::{Ipv4Addr, Ipv6Addr};

pub trait IpPrefixMatch<IP:Ip>
{
    fn matched<P:IpPrefix<IP>>(&self, pfx: &P) -> bool;

    fn slot(&self) -> IP;
}


impl IpPrefixMatch<Ipv4> for Ipv4Addr
{
    #[inline]
    fn matched<P:IpPrefix<Ipv4>>(&self, pfx: &P) -> bool {
        pfx.matches(self)
    }

    #[inline]
    fn slot(&self) -> Ipv4 { Ipv4::from_addr(*self) }
}

impl IpPrefixMatch<Ipv6> for Ipv6Addr
{
    #[inline]
    fn matched<P:IpPrefix<Ipv6>>(&self, pfx: &P) -> bool {
        pfx.matches(self)
    }

    #[inline]
    fn slot(&self) -> Ipv6 { Ipv6::from_addr(*self) }
}

impl IpPrefixMatch<Ipv6s> for Ipv6Addr
{
    #[inline]
    fn matched<P:IpPrefix<Ipv6s>>(&self, pfx: &P) -> bool {
        pfx.matches(self)
    }

    #[inline]
    fn slot(&self) -> Ipv6s { Ipv6s::from_addr(*self) }
}


