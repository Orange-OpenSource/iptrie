use std::num::NonZeroUsize;
use crate::trie::patricia::RadixTrie;
use crate::trie::lctrie::LevelCompressedTrie;
use crate::set::*;

use crate::prefix::*;

#[cfg(feature = "ipnet")] use ipnet::IpNet;
#[cfg(feature = "graphviz")] pub use crate::trie::graphviz::DotWriter;
#[cfg(feature = "graphviz")] use std::fmt::Display;

/// A map of Ip prefixes based on a radix binary trie
#[derive(Clone)]
pub struct RTrieMap<K,V>(pub(crate) RadixTrie<K,V>);

/// Convenient alias for radix trie map of Ipv4 prefixes
pub type Ipv4RTrieMap<V> = RTrieMap<Ipv4Prefix,V>;
/// Convenient alias for radix trie map of Ipv6 prefixes
pub type Ipv6RTrieMap<V> = RTrieMap<Ipv6Prefix,V>;

/// A map of Ip prefixes based on a level-compressed trie
pub struct LCTrieMap<K,V>(pub(crate) LevelCompressedTrie<K,V>);

/// Convenient alias for LC-Trie map of Ipv4 prefixes
pub type Ipv4LCTrieMap<V> = LCTrieMap<Ipv4Prefix,V>;
/// Convenient alias for LC-Trie map of Ipv6 prefixes
pub type Ipv6LCTrieMap<V> = LCTrieMap<Ipv6Prefix,V>;


impl<K:IpPrefix,V:Default> RTrieMap<K,V>
{
    /// Create a new map.
    ///
    /// The root prefix is associated with the default value of `V`.
    #[inline]
    pub fn new() -> Self { Self::default() }

    /// Create a new map with a initial capacity.
    ///
    /// The root prefix is associated with the default value of `V`.
    #[inline]
    pub fn with_capacity(capacity:usize) -> Self { Self::with_root_and_capacity(V::default(), capacity)}
}

impl<K:IpPrefix,V:Default> Default for RTrieMap<K,V>
{
    #[inline] fn default() -> Self { Self::with_root(V::default()) }
}

impl<K:IpPrefix,V> RTrieMap<K,V>
{
    /// Returns the size of the map.
    ///
    /// Notice that it is never null since the top prefix is
    /// always present in the map.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> NonZeroUsize { self.0.len() }

    #[inline]
    pub fn with_root(value: V) -> Self { Self::with_root_and_capacity(value, 1000) }

    #[inline]
    pub fn with_root_and_capacity(value: V, capacity: usize) -> Self {
        Self(RadixTrie::new(value, capacity))
    }

    #[inline]
    pub fn compress(self) -> LCTrieMap<K,V> { LCTrieMap(LevelCompressedTrie::new(self.0)) }

    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.0.insert(k, v)
    }

    #[inline]
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    { self.0.get(k) }

    #[inline]
    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    {
        self.0.get_mut(k)
    }

    #[inline]
    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    {
        self.0.remove(k)
    }

    #[inline]
    pub fn lookup<Q>(&self, k: &Q) -> (&K, &V)
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    { self.0.lookup(k) }

    #[inline]
    pub fn lookup_mut<Q>(&mut self, k: &Q) -> (&K, &mut V)
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    { self.0.lookup_mut(k) }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=(&K,&V)> + '_ { self.0.iter().map(|l| (&l.prefix, &l.value)) }

    #[inline]
    pub fn drain(&mut self) -> impl Iterator<Item=(K,V)> + '_ { self.0.drain().map(|l| (l.prefix, l.value)) }

    #[inline]
    pub fn prefixes(&self) -> RTrieSet<K>
    {
        RTrieSet(self.0.map(|_| ()))
    }
}


impl<K:IpPrefix,V> Extend<(K, V)> for RTrieMap<K,V>
{
    fn extend<I: IntoIterator<Item=(K,V)>>(&mut self, iter: I)
    {
        iter.into_iter().for_each(|(k,v)| {self.insert(k,v);})
    }
}

impl<K:IpPrefix,V:Default> FromIterator<(K, V)> for RTrieMap<K,V>
{
    fn from_iter<I:IntoIterator<Item=(K,V)>>(iter: I) -> Self
    {
        let mut triemap = Self::default();
        triemap.extend(iter);
        triemap
    }
}

impl<K:IpPrefix,V> LCTrieMap<K,V>
{
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> NonZeroUsize {
        self.0.len()
    }

    #[inline]
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>+PartialEq<Q>
    { self.0.get(k) }

    #[inline]
    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>+PartialEq<Q>
    {
        self.0.get_mut(k)
    }

    #[inline]
    pub fn lookup<Q>(&self, k: &Q) -> (&K,&V)
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    { self.0.lookup(k) }

    #[inline]
    pub fn lookup_mut<Q>(&mut self, k: &Q) -> (&K,&mut V)
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    { self.0.lookup_mut(k) }

    #[inline]
    pub fn info(&self) { self.0.info() }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=(&K,&V)> + '_ {
        self.0.leaves.0.iter().map(|x| (&x.prefix,&x.value))
    }

    #[inline]
    pub fn prefixes(&self) -> LCTrieSet<K>
    {
        LCTrieSet(self.0.map(|_| ()))
    }
}

impl<K:IpPrefix,V:Default> FromIterator<(K,V)> for LCTrieMap<K,V>
{
    fn from_iter<I:IntoIterator<Item=(K,V)>>(iter: I) -> Self {
        RTrieMap::from_iter(iter).compress()
    }
}

#[cfg(feature= "graphviz")]
impl<P:IpPrefix+Display,V> DotWriter for RTrieMap<P,V>
{
    fn write_dot(&self, dot: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.0.write_dot(dot)
    }
}

#[cfg(feature= "graphviz")]
impl<P: IpPrefix +Display,V> DotWriter for LCTrieMap<P,V>
{
    fn write_dot(&self, dot: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.0.write_dot(dot)
    }
}

