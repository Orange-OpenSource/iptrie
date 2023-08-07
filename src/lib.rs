mod trie;
mod map;
mod set;

use ipnet::{Ipv4Net, Ipv6Net};

pub use map::*;
pub use set::*;

#[cfg(feature = "graphviz")]
pub use trie::graphviz::DotWriter;

use crate::trie::common::{BitPrefix, BitSlot};

impl BitPrefix for Ipv4Net {
    type Slot = u32;
    #[inline] fn root() -> Self { Ipv4Net::default()  }
    #[inline] fn bitslot(&self) -> Self::Slot  { u32::from(self.addr()) & u32::bitmask(self.len()) }
    #[inline] fn len(&self) -> u8 { self.prefix_len() }
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
