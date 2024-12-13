use std::net::{Ipv4Addr, Ipv6Addr};
use ipnet::{Ipv4Net, Ipv6Net};
use crate::*;

/// Checks if the range of IP addresses are all for private use.
///
/// The IPv4 and IPv6 ranges are those specified by the IANA and IETF.
/// The following tables sums up the ranges which are considered as private use by this trait:
///
/// | RFC | Protocol | Range | Description |
/// |---------|---------|---------|--------|
/// | [RFC1918](https://www.rfc-editor.org/rfc/rfc1918.html) |v4    | 10.0.0.0/8     | Private Use |
/// | [RFC1918](https://www.rfc-editor.org/rfc/rfc1918.html)  |v4   | 172.16.0.0/12     | Private Use |
/// | [RFC1918](https://www.rfc-editor.org/rfc/rfc1918.html)  |v4  | 192.168.0.0/16     | Private Use |
/// |  |  |  |
/// | [RFC4193](https://www.rfc-editor.org/rfc/rfc8215.html)  |v6   | 64:ff9b:1::/48     | Private Use |
/// | [RFC4193](https://www.rfc-editor.org/rfc/rfc4193.html), [RFC8190](https://www.rfc-editor.org/rfc/rfc8190.html) |v6    | fc00::/7     | Private Use |
///
// Some extra ranges...
// | [RFC6598](https://www.rfc-editor.org/rfc/rfc6598.html)  |v4   | 100.64.0.0/10     | Shared Address Space |
// | [RFC6890](https://www.rfc-editor.org/rfc/rfc6890.html), section 2.1 |v4  | 192.0.0.0/24     | IETF Protocol Assignments |
// | [RFC2544](https://www.rfc-editor.org/rfc/rfc2544.html) |v4  | 198.18.0.0/15     | Benchmarking |
pub trait IpPrivatePrefix {
    fn is_private(&self) -> bool;
}


impl IpPrivatePrefix for Ipv6Prefix
{
    #[inline]
    fn is_private(&self) -> bool {
        (self.bitslot() >> 121 == 0xfc >> 1 && self.len() >= 7) // fc00::/7
            || (self.bitslot() >> 80 == 0x64ff9b0001 && self.len() >= 48) // 64:ff9b:1::/48
    }
}

impl IpPrivatePrefix for Ipv4Prefix
{
    #[inline]
    fn is_private(&self) -> bool {
        Ipv4Net::from(*self).is_private()
    }
}

impl IpPrivatePrefix for Ipv6Net
{
    #[inline]
    fn is_private(&self) -> bool {
        match self.addr().octets() {
            [0xfc, ..] | [0xfd, ..] => self.len() >= 7, // fc00::/7
            [0,0x64,0xff,0x9b,0,1,..] => self.len() >= 48, // 64:ff9b:1::/48
            _ => false,
        }
    }
}

impl IpPrivatePrefix for Ipv4Net
{
    #[inline]
    fn is_private(&self) -> bool {
        match self.addr().octets() {
            [10, ..] => self.len() >= 8, // 10.0.0.0/8
            //[100, b, ..] if b >= 64 && b <= 127 => self.len() >= 10, // 100.64.0.0/10 (Shared Address Space)
            [172, b, ..] if b >= 16 && b <= 31 => self.len() >= 12, // 172.16.0.0/12
            //[192, 0, 0, ..] => self.len() >= 24, // 192.0.0.0/24 (IETF Protocol Assignments)
            [192, 168, ..] => self.len() >= 16, // 192.168.0.0/16
            //[198, 18, ..]|[198, 19, ..] => self.len() >= 15, // 198.18.0.0/15 (Benchmarking)
            _ => false,
        }
    }
}

impl IpPrivatePrefix for Ipv4Addr
{
    #[inline]
    fn is_private(&self) -> bool {
        match self.octets() {
            [10, ..] => true, // 10.0.0.0/8
            //[100, b, ..] if b >= 64 && b <= 127 => true, // 100.64.0.0/10  (Shared Address Space)
            [172, b, ..] if b >= 16 && b <= 31 => true, // 172.16.0.0/12
            //[192, 0, 0, ..] => true, // 192.0.0.0/24 (IETF Protocol Assignments)
            [192, 168, ..] => true, // 192.168.0.0/16
            //[198, 18, ..]|[198, 19, ..] => true, // 198.18.0.0/15 (Benchmarking)
            _ => false,
        }
    }
}
impl IpPrivatePrefix for Ipv6Addr
{
    #[inline]
    fn is_private(&self) -> bool {
        match self.octets() {
            [0xfc, ..] | [0xfd, ..] => true, // fc00::/7
            [0,0x64,0xff,0x9b,0,1,..] => true, // 64:ff9b:1::/48
            _ => false,
        }
    }
}

#[cfg(test)] mod tests {
    use std::str::FromStr;
    use std::net::*;
    use ipnet::*;
    use crate::*;

    #[test]
    fn private_ipv4() {
        assert!(Ipv4Addr::from_str("172.31.255.255").unwrap().is_private());
        assert!(Ipv4Prefix::from_str("172.28.0.0/14").unwrap().is_private());
        assert!(Ipv4Net::from_str("172.29.0.0/12").unwrap().is_private());
    }

    #[test]
    fn private_ipv6()
    {
        assert!(Ipv6Addr::from_str("64:ff9b:1::42").unwrap().is_private());
        assert!(Ipv6Prefix::from_str("64:ff9b:1:42::/96").unwrap().is_private());
        assert!(Ipv6Net::from_str("64:ff9b:1:42::/96").unwrap().is_private());
        assert!(Ipv6NetRouting::from_str("64:ff9b:1::42/55").unwrap().is_private());
        assert!(Ipv6NetRouting::from_str("64:ff9b:1:42::/64").unwrap().is_private());

        assert!(Ipv6Addr::from_str("fcc0:ff9b:1::42").unwrap().is_private());
        assert!(Ipv6Prefix::from_str("fcc0:ff9b:1:42::/96").unwrap().is_private());
        assert!(Ipv6Net::from_str("fcc0:ff9b:1:42::/96").unwrap().is_private());
        assert!(Ipv6NetRouting::from_str("fcc0:ff9b:1::42/55").unwrap().is_private());
        assert!(Ipv6NetRouting::from_str("fcc0:ff9b:1:42::/64").unwrap().is_private());
    }
}
