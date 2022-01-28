use std::fmt::Debug;
pub use std::net::Ipv4Addr;
pub use ipnet::Ipv4Net;

use crate::trie::common::{BitPrefix, BitSlot};
use crate::trie::patricia::RadixTrie;
use crate::trie::lctrie::LCTrie;



#[derive(Clone)]
pub struct RTrieSet<P:BitPrefix>(pub(crate) RadixTrie<P,()>);


impl<P:BitPrefix> RTrieSet<P>
{
    #[inline]
    pub fn new() -> Self { Self::with_capacity(1000) }

    #[inline]
    pub fn len(&self) -> usize { self.0.leaves.len() }

    #[inline]
    pub fn compress(self) -> LCTrieSet<P> { LCTrieSet(LCTrie::new(self.0)) }

    #[inline]
    pub fn with_capacity(capacity:usize) -> Self { Self(RadixTrie::new((), capacity)) }

    #[inline]
    pub fn insert(&mut self, k: P) -> bool
    {
        self.0.insert(k,()).is_none()
    }

    #[inline]
    pub fn contains<Q: BitPrefix<Slot=P::Slot>>(&self, k: &Q) -> bool
    {
        self.0.get(k).is_some()
    }

    #[inline]
    pub fn remove<Q: BitPrefix<Slot=P::Slot>>(&mut self, k: &Q) -> bool
    {
        self.0.remove(k).is_some()
    }

    #[inline]
    pub fn lookup<Q:BitPrefix<Slot=P::Slot>>(&self, k: &Q) -> &P {
        &self.0.lookup(k).0
    }

    #[inline]
    pub fn drain(&mut self) -> impl Iterator + '_ {
        self.0.leaves.0.drain(1..).map(|l| l.prefix )
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator + '_ {
        self.0.leaves.0.iter().skip(1).map(|l| &l.prefix )
    }
}


pub struct LCTrieSet<P:BitPrefix>(LCTrie<P,()>);

impl <P:BitPrefix> LCTrieSet<P>
{

    #[inline]
    pub fn contains<Q:BitPrefix<Slot=P::Slot>>(&self, k: &Q) -> bool
    {
        self.0.get(k).is_some()
    }

    #[inline]
    pub fn lookup<Q:BitPrefix<Slot=P::Slot>>(&self, k: &Q) -> &P
    {
        &self.0.lookup(k).0
    }
}


#[cfg(feature = "graphviz")] pub use crate::trie::graphviz::DotWriter;
#[cfg(feature = "graphviz")] use std::io;

#[cfg(feature= "graphviz")]
impl<P:BitPrefix> DotWriter for RTrieSet<P>
    where P: std::fmt::Display
{
    fn write_dot(&self, dot: &mut dyn io::Write) -> io::Result<()> {
        self.0.write_dot(dot)
    }
}

#[cfg(feature= "graphviz")]
impl<P:BitPrefix> DotWriter for LCTrieSet<P>
    where P: std::fmt::Display
{
    fn write_dot(&self, dot: &mut dyn io::Write) -> io::Result<()> {
        self.0.write_dot(dot)
    }
}

