#![feature(unchecked_math)]

pub mod ip;
mod trie;
mod patricia;
mod lctrie;

use std::io::Write;

pub use ip::*;
use crate::patricia::*;
use crate::lctrie::*;

#[cfg(feature = "graphviz")] mod graphviz;
#[cfg(feature = "graphviz")] pub use graphviz::DotWriter;

pub struct IpPrefixMap<IP:Ip,K:IpPrefix<IP>,V>(RadixTrie<IP,K,V>);

impl <IP:Ip,K:IpPrefix<IP>,V:Default> IpPrefixMap<IP,K,V>
{
    #[inline]
    pub fn new() -> Self { Self::with_root(V::default()) }

    #[inline]
    pub fn with_capacity(capacity:usize) -> Self { Self::with_root_and_capacity(V::default(), capacity)}
}

impl<IP:Ip,K:IpPrefix<IP>,V> IpPrefixMap<IP,K,V>
{
    #[inline]
    pub fn len(&self) -> usize { self.0.leaves.len() }

    #[inline]
    pub fn with_root(value:V) -> Self { Self::with_root_and_capacity(value, 1000) }

    #[inline]
    pub fn with_root_and_capacity(value: V, capacity: usize) -> Self {
        Self(RadixTrie::new(value, capacity))
    }

    #[inline]
    pub fn compile(self) -> LCTrie<IP,K,V> { LCTrie::new(self.0) }

    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    {
        self.0.insert(k,v)
    }

    #[inline]
    pub fn get<P: IpPrefix<IP>>(&self, k: &P) -> Option<&V> { self.0.get(k) }

    #[inline]
    pub fn get_mut<P: IpPrefix<IP>>(&mut self, k: &P) -> Option<&mut V>
    {
        self.0.get_mut(k)
    }

    #[inline]
    pub fn remove<P: IpPrefix<IP>>(&mut self, k: &P) -> Option<V>
    {
        self.0.remove(k)
    }

    #[inline]
    pub fn lookup<Q: IpPrefixMatch<IP>>(&self, k: &Q) -> (&K,&V) { self.0.lookup(k) }

    #[inline]
    pub fn lookup_mut<Q: IpPrefixMatch<IP>>(&mut self, k: &Q) -> (&K,&mut V) { self.0.lookup_mut(k) }
}

#[cfg(feature= "graphviz")]
impl<IP:Ip,K:IpPrefix<IP>,V> graphviz::DotWriter for IpPrefixMap<IP,K,V>
{
    fn write_dot(&self, dot: &mut dyn Write) -> std::io::Result<()> {
        self.0.write_dot(dot)
    }
}

pub struct IpPrefixSet<IP:Ip,K:IpPrefix<IP>>(RadixTrie<IP,K,()>);

impl <IP:Ip, K:IpPrefix<IP>> IpPrefixSet<IP,K>
{
    #[inline]
    pub fn new() -> Self { Self::with_capacity(1000) }

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

}

pub struct IpPrefixCompiledMap<IP:Ip,K:IpPrefix<IP>,V>(LCTrie<IP,K,V>);


impl<IP:Ip,K:IpPrefix<IP>,V> IpPrefixCompiledMap<IP,K,V>
{
    #[inline]
    pub fn new(trie: IpPrefixMap<IP,K,V>) -> Self { Self(LCTrie::new(trie.0)) }

    #[inline]
    pub fn get<P: IpPrefix<IP>>(&self, k: &P) -> Option<&V> { self.0.get(k) }

    #[inline]
    pub fn get_mut<P: IpPrefix<IP>>(&mut self, k: &P) -> Option<&mut V>
    {
        self.0.get_mut(k)
    }

    #[inline]
    pub fn lookup<Q: IpPrefixMatch<IP>>(&self, k: &Q) -> (&K,&V) { self.0.lookup(k) }

    #[inline]
    pub fn lookup_mut<Q: IpPrefixMatch<IP>>(&mut self, k: &Q) -> (&K,&mut V) { self.0.lookup_mut(k) }
}
