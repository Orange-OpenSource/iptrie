pub use std::net::{Ipv4Addr, Ipv6Addr};
pub use ipnet::{Ipv4Net, Ipv6Net};
use crate::trie::common::BitPrefix;

mod trie;
mod map;
mod set;

pub use map::*;
pub use set::*;

impl BitPrefix for Ipv4Addr {
    type Slot = u32;
    #[inline] fn root() -> Self { Ipv4Addr::new(0,0,0,0)  }
    #[inline] fn bitslot(&self) -> Self::Slot { (*self).into() }
    #[inline] fn len(&self) -> u8 { if self.is_unspecified() {0} else {32} }
}

impl BitPrefix for Ipv4Net {
    type Slot = u32;
    #[inline] fn root() -> Self { Ipv4Net::default()  }
    #[inline] fn bitslot(&self) -> Self::Slot { self.addr().into()}
    #[inline] fn len(&self) -> u8 { self.prefix_len() }
}


impl BitPrefix for Ipv6Addr {
    type Slot = u128;
    #[inline] fn root() -> Self { Ipv6Addr::new(0,0,0,0,0,0,0,0) }
    #[inline] fn bitslot(&self) -> Self::Slot { (*self).into() }
    #[inline] fn len(&self) -> u8 { if self.is_unspecified() {0} else {128} }
}

impl BitPrefix for Ipv6Net {
    type Slot = u128;
    #[inline] fn root() -> Self { Ipv6Net::default()  }
    #[inline] fn bitslot(&self) -> Self::Slot { self.addr().into()}
    #[inline] fn len(&self) -> u8 { self.prefix_len() }
}
