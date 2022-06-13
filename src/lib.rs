mod trie;
mod map;
mod set;

pub use std::net::{Ipv4Addr, Ipv6Addr};
pub use ipnet::{Ipv4Net, Ipv6Net};
pub use crate::trie::common::BitPrefix;

pub use map::*;
pub use set::*;
use crate::trie::common::BitSlot;

impl BitPrefix for Ipv4Addr {
    type Slot = u32;
    #[inline] fn root() -> Self { Ipv4Addr::new(0,0,0,0)  }
    #[inline] fn bitslot(&self) -> Self::Slot { u32::from(*self) }
    #[inline] fn len(&self) -> u8 { if self.is_unspecified() {0} else {32} }
}

impl BitPrefix for Ipv4Net {
    type Slot = u32;
    #[inline] fn root() -> Self { Ipv4Net::default()  }
    #[inline] fn bitslot(&self) -> Self::Slot  { u32::from(self.addr()) & u32::bitmask(self.len()) }
    #[inline] fn len(&self) -> u8 { self.prefix_len() }
}


impl BitPrefix for Ipv6Addr {
    type Slot = u128;
    #[inline] fn root() -> Self { Ipv6Addr::new(0,0,0,0,0,0,0,0) }
    #[inline] fn bitslot(&self) -> Self::Slot { u128::from(*self) }
    #[inline] fn len(&self) -> u8 { if self.is_unspecified() {0} else {128} }
}

impl BitPrefix for Ipv6Net {
    type Slot = u128;
    #[inline] fn root() -> Self { Ipv6Net::default()  }
    #[inline] fn bitslot(&self) -> Self::Slot { u128::from(self.addr()) & u128::bitmask(self.len())}
    #[inline] fn len(&self) -> u8 { self.prefix_len() }
}


#[inline]
fn bitmask<B:BitPrefix>(pfx: &B) -> B::Slot {
    <B::Slot as BitSlot>::bitmask(pfx.len())
}

#[inline]
fn bitslot_trunc<B:BitPrefix>(pfx: &B) -> B::Slot {
    pfx.bitslot() & bitmask(pfx)
}

#[inline]
fn covers<P1: BitPrefix, P2: BitPrefix<Slot=P1::Slot>>(pfx1: &P1, pfx2: &P2) -> bool {
    pfx2.bitslot() & bitmask(pfx1) == bitslot_trunc(pfx1) && pfx1.len() <= pfx2.len()
}
