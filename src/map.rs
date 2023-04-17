use std::fmt::Debug;
pub use std::net::Ipv4Addr;
pub use ipnet::Ipv4Net;

use crate::trie::common::{BitPrefix, BitSlot};
use crate::trie::patricia::RadixTrie;
use crate::trie::lctrie::LCTrie;
use crate::set::RTrieSet;

pub struct RTrieMap<K:BitPrefix,V>(RadixTrie<K,V>);

impl <K:BitPrefix,V:Default> RTrieMap<K,V>
{
    #[inline]
    pub fn new() -> Self { Self::with_root(V::default()) }

    #[inline]
    pub fn with_capacity(capacity:usize) -> Self { Self::with_root_and_capacity(V::default(), capacity)}
}

impl <K:BitPrefix,V> RTrieMap<K,V>
{
    #[inline]
    pub fn len(&self) -> usize { self.0.leaves.len() }

    #[inline]
    pub fn with_root(value: V) -> Self { Self::with_root_and_capacity(value, 1000) }

    #[inline]
    pub fn with_root_and_capacity(value: V, capacity: usize) -> Self {
        Self(RadixTrie::new(value, capacity))
    }

    #[inline]
    pub fn compress(self) -> LCTrieMap<K,V> { LCTrieMap(LCTrie::new(self.0)) }

    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    {
        self.0.insert(k, v)
    }

    #[inline]
    pub fn get<Q:BitPrefix<Slot=K::Slot>>(&self, k: &Q) -> Option<&V> { self.0.get(k) }

    #[inline]
    pub fn get_mut<Q:BitPrefix<Slot=K::Slot>>(&mut self, k: &Q) -> Option<&mut V>
    {
        self.0.get_mut(k)
    }

    #[inline]
    pub fn remove<Q:BitPrefix<Slot=K::Slot>>(&mut self, k: &Q) -> Option<V>
    {
        self.0.remove(k)
    }

    #[inline]
    pub fn lookup<Q:BitPrefix<Slot=K::Slot>>(&self, k: &Q) -> (&K, &V) { self.0.lookup(k) }

    #[inline]
    pub fn lookup_mut<Q:BitPrefix<Slot=K::Slot>>(&mut self, k: &Q) -> (&K, &mut V) { self.0.lookup_mut(k) }

    #[inline]
    pub fn drain(&mut self) -> impl Iterator<Item=(K,V)> + '_ {
        self.0.branching.clear();
        self.0.leaves.0.drain(1..).map(|l| (l.prefix,l.value))
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=(&K,&V)> + '_ {
        self.0.leaves.0.iter().skip(1).map(|x| (&x.prefix,&x.value))
    }

    #[inline]
    pub fn prefixes(&self) -> RTrieSet<K>
    {
        RTrieSet(self.0.map(|_| ()))
    }
}


impl<K:BitPrefix,V> Extend<(K,V)> for RTrieMap<K,V>
{
    fn extend<T: IntoIterator<Item=(K,V)>>(&mut self, iter: T)
    {
        iter.into_iter().for_each(|(k,v)| {self.insert(k,v);})
    }
}


pub struct LCTrieMap<K:BitPrefix,V>(LCTrie<K,V>);


impl<K:BitPrefix,V> LCTrieMap<K,V>
{
    #[inline]
    pub fn len(&self) -> usize { self.0.len() }

    #[inline]
    pub fn get<Q:BitPrefix<Slot=K::Slot>>(&self, k: &Q) -> Option<&V> { self.0.get(k) }

    #[inline]
    pub fn get_mut<Q:BitPrefix<Slot=K::Slot>>(&mut self, k: &Q) -> Option<&mut V>
    {
        self.0.get_mut(k)
    }

    #[inline]
    pub fn lookup<Q:BitPrefix<Slot=K::Slot>>(&self, k: &Q) -> (&K,&V) { self.0.lookup(k) }

    #[inline]
    pub fn lookup_mut<Q:BitPrefix<Slot=K::Slot>>(&mut self, k: &Q) -> (&K,&mut V) { self.0.lookup_mut(k) }

    #[inline]
    pub fn info(&self) { self.0.info() }
}


#[cfg(feature = "graphviz")] pub use crate::trie::graphviz::DotWriter;
#[cfg(feature = "graphviz")] use std::io;


#[cfg(feature= "graphviz")]
impl<K:BitPrefix,V> DotWriter for RTrieMap<K,V>
    where K: std::fmt::Display
{
    fn write_dot(&self, dot: &mut dyn io::Write) -> io::Result<()> {
        self.0.write_dot(dot)
    }
}

#[cfg(feature= "graphviz")]
impl<K:BitPrefix,V>  DotWriter for LCTrieMap<K,V>
    where K: std::fmt::Display
{
    fn write_dot(&self, dot: &mut dyn io::Write) -> io::Result<()> {
        self.0.write_dot(dot)
    }
}