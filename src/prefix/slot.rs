
use std::ops::{Shr, Shl, BitAnd, Not, BitOr, BitXor};
use std::fmt::{Binary, Debug};
use std::hash::Hash;

/// A fixed-length slot of bits.
///
/// This is the slot on which all the prefix
/// (and so the tries) computations are made.
///
/// As an example, `u32` or `u128` are good candidates
/// for Ipv4 or Ipv6 prefixes.
/// Moreover, `u64` could be used to save memory
/// for truncated Ipv6 prefixes.
#[doc(hidden)]
pub trait BitSlot :
  Clone + Copy + Default + Debug + Binary + Eq + PartialEq + Hash
+ Not<Output=Self> + BitAnd<Output=Self> + BitOr<Output=Self> + BitXor<Output=Self>
+ Shl<u8,Output=Self> + Shr<u8,Output=Self>
{
    /// The length of the slot (number of bits)
    /// # Example
    /// ```
    /// # use iptrie::BitSlot;
    /// assert_eq!( <u32 as BitSlot>::LEN, 32);
    /// assert_eq!( <u64 as BitSlot>::LEN, 64);
    /// assert_eq!( <u128 as BitSlot>::LEN, 128);
    /// ```
    const LEN: u8;

    /// Returns a slot with the bit of the specified position set to 1.
    ///
    /// As this is a prefix, the first bits are the highest ones.
    /// # Example
    /// ```
    /// # use iptrie::BitSlot;
    /// assert_eq!( <u32 as BitSlot>::single_bit(30), 4);
    /// assert_eq!( <u32 as BitSlot>::single_bit(24), 256);
    /// assert_eq!( <u64 as BitSlot>::single_bit(60), 16);
    /// assert_eq!( <u64 as BitSlot>::single_bit(1), u64::MAX/2 + 1);
    /// assert_eq!( <u128 as BitSlot>::single_bit(120), 256);
    /// ```
    fn single_bit(pos: u8) -> Self;

    /// Returns a slot with the `len` first bits set to 1 and
    /// the others set to 0
    fn bitmask(len: u8) -> Self;

    /// Returns the position of the first bit set to 1
    ///
    /// If all the bits are set to 0, the position `LEN+1` is returned
    /// (i.e. 33 for 32bit slot or 65 for a 64bit slot).
    /// # Example
    /// ```
    /// # use iptrie::BitSlot;
    /// assert_eq!( <u32 as BitSlot>::first_bit(&0), 33);
    /// assert_eq!( <u32 as BitSlot>::first_bit(&1024), 22);
    /// assert_eq!( <u64 as BitSlot>::first_bit(&1024), 54);
    /// assert_eq!( <u128 as BitSlot>::first_bit(&0), 129);
    /// ```
    fn first_bit(&self) -> u8;

    /// Checks if the bit at the given position is set to 1 or not
    /// # Example
    /// ```
    /// # use iptrie::BitSlot;
    /// assert!( ! <u32 as BitSlot>::is_set(&0, 12));
    /// assert!( <u32 as BitSlot>::is_set(&!0, 12));
    /// assert!( <u32 as BitSlot>::is_set(&1024, 22));
    /// ```
    fn is_set(&self, pos: u8) -> bool;

    /// Truncates the end of the slot to the last 16 bits.
    ///
    /// This is used to compress the trie (LC-Trie) and implicitly
    /// defines the maximum compression level (i.e. 16 bits)
    /// # Example
    /// ```
    /// # use iptrie::BitSlot;
    /// assert_eq!( <u32 as BitSlot>::last_16_bits(&!0), 65535);
    /// assert_eq!( <u32 as BitSlot>::last_16_bits(&65), 65);
    /// ```
    fn last_16_bits(&self) -> u16;
}


macro_rules! bitslot {
    ($slot:ty) => {
        impl BitSlot for $slot {
            const LEN: u8 = std::mem::size_of::<$slot>() as u8 * 8;
            fn first_bit(&self) -> u8 {
                self.leading_zeros() as u8 + 1
            }
            fn single_bit(pos: u8) -> Self {
                debug_assert!(pos > 0); debug_assert!( pos <= Self::LEN);
                1 as $slot << (Self::LEN-pos)
            }
            fn is_set(&self, pos: u8) -> bool {
                debug_assert!(pos > 0); debug_assert!( pos <= Self::LEN);
                (self >> (Self::LEN-pos)) & 1 != 0
            }
            fn bitmask(len:u8) -> Self {
                debug_assert!( len <= Self::LEN);
                if len == 0 { 0 } else { (!0 as $slot) << (Self::LEN-len) }
            }
            fn last_16_bits(&self) -> u16 {
                *self as u16
            }
        }
    };
}

bitslot!(u32);
bitslot!(u64);
bitslot!(u128);

