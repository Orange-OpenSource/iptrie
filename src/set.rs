use crate::ip::{Ip, IpPrefix, IpPrefixMatch};
use crate::patricia::*;
use crate::lctrie::*;


#[cfg(feature = "graphviz")] pub use crate::graphviz::DotWriter;
#[cfg(feature = "graphviz")] use std::io;

#[derive(Clone)]
pub struct IpPrefixSet<IP:Ip,K:IpPrefix<IP>>(pub(crate) RadixTrie<IP,K,()>);

impl <IP:Ip, K:IpPrefix<IP>> IpPrefixSet<IP,K>
{
    #[inline]
    pub fn new() -> Self { Self::with_capacity(1000) }

    #[inline]
    pub fn len(&self) -> usize { self.0.leaves.len() }

    #[inline]
    pub fn compile(self) -> LCTrie<IP,K,()> { LCTrie::new(self.0) }

    #[inline]
    pub fn with_capacity(capacity:usize) -> Self { Self(RadixTrie::new((), capacity)) }

    #[inline]
    pub fn insert(&mut self, k: K) -> bool
    {
        self.0.insert(k,()).is_none()
    }

    #[inline]
    pub fn contains<P: IpPrefix<IP>>(&self, k: &P) -> bool
    {
        self.0.get(k).is_some()
    }

    #[inline]
    pub fn remove<P: IpPrefix<IP>>(&mut self, k: &P) -> bool
    {
        self.0.remove(k).is_some()
    }

    #[inline]
    pub fn lookup<Q: IpPrefixMatch<IP>>(&self, k: &Q) -> &K
    {
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

#[cfg(feature= "graphviz")]
impl<IP:Ip,K:IpPrefix<IP>> crate::graphviz::DotWriter for IpPrefixSet<IP,K>
{
    fn write_dot(&self, dot: &mut dyn io::Write) -> io::Result<()> {
        self.0.write_dot(dot)
    }
}

pub struct IpPrefixLCSet<IP:Ip,K:IpPrefix<IP>>(LCTrie<IP,K,()>);

impl <IP:Ip, K:IpPrefix<IP>> IpPrefixLCSet<IP,K>
{
    #[inline]
    pub fn new(trie: IpPrefixSet<IP,K>) -> Self { Self(LCTrie::new(trie.0)) }

    #[inline]
    pub fn contains<P: IpPrefix<IP>>(&self, k: &P) -> bool
    {
        self.0.get(k).is_some()
    }

    #[inline]
    pub fn lookup<Q: IpPrefixMatch<IP>>(&self, k: &Q) -> &K
    {
        &self.0.lookup(k).0
    }
}

