
use crate::ip::{Ip, IpPrefix, IpPrefixMatch};
use crate::patricia::*;
use crate::lctrie::*;
use crate::IpPrefixSet;


#[cfg(feature = "graphviz")] pub use crate::graphviz::DotWriter;
#[cfg(feature = "graphviz")] use std::io;

#[derive(Clone)]
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
    pub fn with_root(value: V) -> Self { Self::with_root_and_capacity(value, 1000) }

    #[inline]
    pub fn with_root_and_capacity(value: V, capacity: usize) -> Self {
        Self(RadixTrie::new(value, capacity))
    }

    #[inline]
    pub fn compile(self) -> LCTrie<IP, K, V> { LCTrie::new(self.0) }

    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    {
        self.0.insert(k, v)
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
    pub fn lookup<Q: IpPrefixMatch<IP>>(&self, k: &Q) -> (&K, &V) { self.0.lookup(k) }

    #[inline]
    pub fn lookup_mut<Q: IpPrefixMatch<IP>>(&mut self, k: &Q) -> (&K, &mut V) { self.0.lookup_mut(k) }

    #[inline]
    pub fn drain(&mut self) -> impl Iterator<Item=(K,V)> + '_ {
        self.0.leaves.0.drain(1..).map(|l| (l.prefix,l.value))
    }
/*
    #[inline]
    pub fn drain_filter<'a,F>(&'a mut self, mut pred: F) -> impl Iterator + 'a
        where F: 'a + FnMut(&K, &mut V) -> bool
    {
        unimplemented!();
        self.0.leaves.0.drain_filter(move |l| (pred)(&l.prefix,&mut l.value))
    }*/

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=(&K,&V)> + '_ {
        self.0.leaves.0.iter().skip(1).map(|x| (&x.prefix,&x.value))
    }

    #[inline]
    pub fn prefixes(&self) -> IpPrefixSet<IP,K>
    {
        IpPrefixSet(self.0.map(|_| ()))
    }
}

impl<IP:Ip,K:IpPrefix<IP>,V> Extend<(K,V)> for IpPrefixMap<IP,K,V>
{
    fn extend<T: IntoIterator<Item=(K, V)>>(&mut self, iter: T)
    {
        iter.into_iter().for_each(|(k,v)| {self.insert(k,v);})
    }
}

#[cfg(feature= "graphviz")]
impl<IP:Ip,K:IpPrefix<IP>,V> crate::graphviz::DotWriter for IpPrefixMap<IP,K,V>
{
    fn write_dot(&self, dot: &mut dyn io::Write) -> io::Result<()> {
        self.0.write_dot(dot)
    }
}

pub struct IpPrefixLCMap<IP:Ip,K:IpPrefix<IP>,V>(LCTrie<IP,K,V>);


impl<IP:Ip,K:IpPrefix<IP>,V> IpPrefixLCMap<IP,K,V>
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

    #[inline]
    pub fn info(&self) { self.0.info() }
}


#[cfg(feature= "graphviz")]
impl<IP:Ip,K:IpPrefix<IP>,V> crate::graphviz::DotWriter for IpPrefixLCMap<IP,K,V>
{
    fn write_dot(&self, dot: &mut dyn io::Write) -> io::Result<()> {
        self.0.write_dot(dot)
    }
}
