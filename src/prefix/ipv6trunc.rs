use super::*;

/// An Ipv6 prefix limited to 56 bits and encoded as `u64` (EXPERIMENTAL)
///
/// In many applications, managed Ipv6 prefixes are never longer than 56 bits.
/// In theses cases, it is possible to save memory space by encoding it in
/// 64 bits with its length encoded in the last byte.
/// ```text
/// |------------ 56 bits ---------------|--8 bits--|
///            ip prefix slot                length
/// ```
/// The resulting prefix is 4 times shorter that the corresponding Ipv6 generic prefix.
#[derive(Copy, Clone, Default, Eq, PartialEq, Hash)]
pub struct Ipv6Prefix56 { addr: u64 }

impl IpPrefix for Ipv6Prefix56
{
    type Slot = u64;
    #[inline] fn root() -> Self { Self { addr: 0 } }
    #[inline] fn bitslot(&self) -> Self::Slot { self.addr }
    #[inline] fn bitslot_trunc(&self) -> Self::Slot { self.addr & !255 }
    #[inline] fn len(&self) -> u8 { self.addr as u8 }

    const MAX_LEN: u8 = 56;
    type Addr = Ipv6Addr;
    #[inline] fn network(&self) -> Self::Addr { ((self.bitslot_trunc() as u128) << 64).into() }
}

/// An Ipv6 prefix limited to 120 bits and encoded as `u128` (EXPERIMENTAL)
///
/// In many applications, managed Ipv6 prefixes are never longer than 120 bits.
/// In theses cases, it is possible to save memory space by encoding it in
/// 128 bits with its length encoded in last byte.
/// ```text
/// |---------------------- 120 bits -----------------------|--8 bits--|
///                      ip prefix slot                         length
/// ```
/// The resulting prefix is twice as short as the corresponding Ipv6 generic prefix.
#[derive(Copy, Clone, Default, Eq, PartialEq, Hash)]
pub struct Ipv6Prefix120 { addr: u128 }

impl IpPrefix for Ipv6Prefix120
{
    type Slot = u128;
    #[inline] fn root() -> Self { Self { addr: 0 } }
    #[inline] fn bitslot(&self) -> Self::Slot { self.addr }
    #[inline] fn bitslot_trunc(&self) -> Self::Slot { self.addr & !255 }
    #[inline] fn len(&self) -> u8 { self.addr as u8 }

    const MAX_LEN: u8 = 120;
    type Addr = Ipv6Addr;
    #[inline] fn network(&self) -> Self::Addr { (self.addr & !255).into() }
}


macro_rules! ipv6prefix {
    ($prefix:ident, $slot:ty) => {
        impl From<$prefix> for Ipv6Net
        {
            #[inline] fn from(value: $prefix) -> Self {
                Ipv6Net::new(value.network(), value.len()).unwrap()
            }
        }
        impl From<$prefix> for Ipv6Prefix
        {
            #[inline] fn from(value: $prefix) -> Self {
                Self { addr: value.network().into(), len: value.len() }
            }
        }
        impl TryFrom<Ipv6Net> for $prefix
        {
            type Error = IpPrefixError;
            #[inline]
            fn try_from(value: Ipv6Net) -> Result<Self, Self::Error> {
                Self::new(value.addr(), value.prefix_len())
            }
        }
        impl TryFrom<Ipv6Prefix> for $prefix
        {
            type Error = IpPrefixError;
            #[inline]
            fn try_from(value: Ipv6Prefix) -> Result<Self, Self::Error> {
                Self::new(value.addr.into(), value.len())
            }
        }
        impl Display for $prefix {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let ip = (*self).into();
                <Ipv6Net as fmt::Display>::fmt(&ip, f)
            }
        }
        impl Debug for $prefix {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let ip = (*self).into();
                <Ipv6Net as fmt::Display>::fmt(&ip, f)
            }
        }
        impl FromStr for $prefix {
            type Err = IpPrefixError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <$prefix>::try_from(Ipv6Net::from_str(s)?)
            }
        }
    }
}
ipv6prefix!(Ipv6Prefix56, u64);
ipv6prefix!(Ipv6Prefix120, u128);

impl Ipv6Prefix56
{
    #[inline]
    pub fn new(addr: Ipv6Addr, len: u8) -> Result<Self, IpPrefixError>
    {
        if len > Self::MAX_LEN {
            Err(IpPrefixError::PrefixLenError)
        } else {
            let addr = ((u128::from(addr) >> 64) as u64 & u64::bitmask(len)) | u64::from(len);
            Ok(Self { addr })
        }
    }
}

impl Ipv6Prefix120
{
    #[inline]
    pub fn new(addr: Ipv6Addr, len: u8) -> Result<Self, IpPrefixError>
    {
        if len > Self::MAX_LEN {
            Err(IpPrefixError::PrefixLenError)
        } else {
            let addr = (u128::from(addr) & u128::bitmask(len)) | u128::from(len);
            Ok(Self { addr })
        }
    }
}


impl From<Ipv6Prefix56> for Ipv6Prefix120
{
    #[inline]
    fn from(value: Ipv6Prefix56) -> Self {
        Ipv6Prefix120 { addr: u128::from(value.network()) | (value.len() as u128) }
    }
}

impl TryFrom<Ipv6Prefix120> for Ipv6Prefix56
{
    type Error = IpPrefixError;

    fn try_from(value: Ipv6Prefix120) -> Result<Self, Self::Error> {
        if value.len() <= 56 {
            let addr = (value.addr >> 64) as u64 | (value.len() as u64);
            Ok(Self { addr })
        } else {
            Err(IpPrefixError::PrefixLenError)
        }
    }
}
