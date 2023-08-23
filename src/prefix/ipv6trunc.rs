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
pub struct Ipv6Prefix56 { slot: u64 }

impl IpPrefix for Ipv6Prefix56
{
    type Slot = u64;
    #[inline] fn root() -> Self { Self { slot: 0 } }
    #[inline] fn bitslot(&self) -> Self::Slot { self.slot }
    #[inline] fn bitslot_trunc(&self) -> Self::Slot { self.slot & !255 }
    #[inline] fn len(&self) -> u8 { self.slot as u8 }

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
pub struct Ipv6Prefix120 { slot: u128 }

impl IpPrefix for Ipv6Prefix120
{
    type Slot = u128;
    #[inline] fn root() -> Self { Self { slot: 0 } }
    #[inline] fn bitslot(&self) -> Self::Slot { self.slot }
    #[inline] fn bitslot_trunc(&self) -> Self::Slot { self.slot & !255 }
    #[inline] fn len(&self) -> u8 { self.slot as u8 }

    const MAX_LEN: u8 = 120;
    type Addr = Ipv6Addr;
    #[inline] fn network(&self) -> Self::Addr { (self.slot & !255).into() }
}


macro_rules! ipv6prefix {
    ($prefix:ident, $slot:ty) => {
        impl $prefix {

            /// Creates a prefix from a well structured slot.
            ///
            /// A well-structured slot is structured as follows where length is
            /// encoded on the last byte (8 bits) and this length should not
            /// exceed the prefix limit [`Self::MAX_LEN`].
            /// ```text
            /// |------------ ip prefix slot ------------|-- length --|
            /// ```
            /// Results are unpredictable if the last byte is not a valid length.
            ///
            /// A safe version of this function exists as [`Self::from_slot`]
            /// or also [`Self::try_from_slot] which doesn’t panic.
            ///
            /// # Example
            /// ```
            /// # use iptrie::*;
            /// let prefix = "1:1::/48".parse::<Ipv6Prefix56>().unwrap();
            /// let slot : u64 = prefix.into_slot();
            /// assert_eq!( prefix, unsafe { Ipv6Prefix56::from_slot_unchecked(slot)});
            ///
            /// let prefix = "1:1::/48".parse::<Ipv6Prefix120>().unwrap();
            /// let slot : u128 = prefix.into_slot();
            /// assert_eq!( prefix, unsafe { Ipv6Prefix120::from_slot_unchecked(slot)});
            ///
            #[inline]
            pub unsafe fn from_slot_unchecked(slot: $slot) -> Self { Self { slot } }


            /// Creates a prefix from a well structured slot.
            ///
            /// A well-structured slot is structured as follows where length is
            /// encoded on the last byte (8 bits) and this length should not
            /// exceed the prefix limit [`Self::MAX_LEN`].
            /// ```text
            /// |------------ ip prefix slot ------------|-- length --|
            /// ```
            /// It returns an error if the last byte is not a valid length.
            ///
            /// If you are sure that the slot is well-structured, an unchecked
            /// (and unsafe) version of this function is provided by [`Self::from_slot_unchecked`].
            ///
            /// # Example
            /// ```
            /// # use iptrie::*;
            /// let prefix = "1:1::/48".parse::<Ipv6Prefix56>().unwrap();
            /// let slot : u64 = prefix.into_slot();
            /// assert_eq!( Ok(prefix), Ipv6Prefix56::from_slot(slot));
            ///
            /// let prefix = "1:1::/48".parse::<Ipv6Prefix120>().unwrap();
            /// let slot : u128 = prefix.into_slot();
            /// assert_eq!( Ok(prefix), Ipv6Prefix120::from_slot(slot));
            ///
            #[inline]
            pub fn from_slot(slot: $slot) -> Result<Self, IpPrefixError> {
                if slot as u8  <= Self::MAX_LEN {
                    Ok( Self { slot } )
                } else {
                    Err(IpPrefixError::PrefixLenError)
                }
            }


            /// Gets the raw value of the inner slot.
            ///
            /// This slot is structured as follows where length is
            /// encoded on the last byte (8 bits).
            /// ```text
            /// |------------ ip prefix slot ------------|-- length --|
            /// ```
            /// The returned slot could be safely used with [`Self::from_slot_unchecked`].
            #[inline]
            pub fn into_slot(self) -> $slot { self.slot }
        }

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
            let slot = ((u128::from(addr) >> 64) as u64 & u64::bitmask(len)) | u64::from(len);
            Ok(Self { slot })
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
            let slot = (u128::from(addr) & u128::bitmask(len)) | u128::from(len);
            Ok(Self { slot })
        }
    }
}


impl From<Ipv6Prefix56> for Ipv6Prefix120
{
    #[inline]
    fn from(value: Ipv6Prefix56) -> Self {
        Ipv6Prefix120 { slot: u128::from(value.network()) | (value.len() as u128) }
    }
}

impl TryFrom<Ipv6Prefix120> for Ipv6Prefix56
{
    type Error = IpPrefixError;

    fn try_from(value: Ipv6Prefix120) -> Result<Self, Self::Error> {
        if value.len() <= 56 {
            let addr = (value.slot >> 64) as u64 | (value.len() as u64);
            Ok(Self { slot: addr })
        } else {
            Err(IpPrefixError::PrefixLenError)
        }
    }
}
