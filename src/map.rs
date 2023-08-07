use std::num::NonZeroUsize;
use crate::trie::patricia::RadixTrie;
use crate::trie::lctrie::LCTrie;
use crate::set::*;

use ipnet::{IpNet, Ipv4Net, Ipv6Net};

#[cfg(feature = "graphviz")] pub use crate::trie::graphviz::DotWriter;

/// A map of Ipv4 prefixes based on a radix binary trie
#[derive(Clone)]
pub struct Ipv4RTrieMap<V>(pub(crate) RadixTrie<Ipv4Net,V>);
/// A map of Ipv6 prefixes based on a radix binary trie
#[derive(Clone)]
pub struct Ipv6RTrieMap<V>(pub(crate) RadixTrie<Ipv6Net,V>);
/// A merge of two maps based on radix binary tries (Ipv4 and Ipv6)
#[derive(Clone,Default)]
pub struct IpRTrieMap<V> { pub v4: Ipv4RTrieMap<V>, pub v6: Ipv6RTrieMap<V> }

/// A map of Ipv4 prefixes based on a level-compressed trie
pub struct Ipv4LCTrieMap<V>(pub(crate) LCTrie<Ipv4Net,V>);
/// A map of Ipv6 prefixes based on a level-compressed trie
pub struct Ipv6LCTrieMap<V>(pub(crate) LCTrie<Ipv6Net,V>);
/// A merge of two maps based on level-compressed tries (Ipv4 and Ipv6)
pub struct IpLCTrieMap<V> { pub v4: Ipv4LCTrieMap<V>, pub v6: Ipv6LCTrieMap<V> }


macro_rules! triemap {
    ($rtrie:ident, $rtrieset:ident, $lctrie:ident, $lctrieset:ident, $ipnet:ty) => {

        impl <V:Default> $rtrie<V>
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

        impl<V:Default> Default for $rtrie<V>
        {
            #[inline] fn default() -> Self { Self::with_root(V::default()) }
        }

        impl<V> $rtrie<V>
        {
            /// Returns the size of the map.
            ///
            /// Notice that it is never null since the top prefix is
            /// always present in the map.
            #[inline]
            #[allow(clippy::len_without_is_empty)]
            pub fn len(&self) -> NonZeroUsize {
                unsafe {
                    NonZeroUsize::new_unchecked(self.0.leaves.len())
                }
            }

            #[inline]
            pub fn with_root(value: V) -> Self { Self::with_root_and_capacity(value, 1000) }

            #[inline]
            pub fn with_root_and_capacity(value: V, capacity: usize) -> Self {
                Self(RadixTrie::new(value, capacity))
            }

            #[inline]
            pub fn compress(self) -> $lctrie<V> { $lctrie(LCTrie::new(self.0)) }

            #[inline]
            pub fn insert<K:Into<$ipnet>>(&mut self, k: K, v: V) -> Option<V>
            {
                self.0.insert(k.into(), v)
            }

            #[inline]
            pub fn get<K:Into<$ipnet>>(&self, k: K) -> Option<&V> { self.0.get(&k.into()) }

            #[inline]
            pub fn get_mut<K:Into<$ipnet>>(&mut self, k: K) -> Option<&mut V> {
                self.0.get_mut(&k.into())
            }

            #[inline]
            pub fn remove<K:Into<$ipnet>>(&mut self, k: K) -> Option<V>
            {
                self.0.remove(&k.into())
            }

            #[inline]
            pub fn lookup<K:Into<$ipnet>>(&self, k: K) -> ($ipnet, &V) { self.0.lookup(&k.into()) }

            #[inline]
            pub fn lookup_mut<K:Into<$ipnet>>(&mut self, k: K) -> ($ipnet, &mut V) { self.0.lookup_mut(&k.into()) }

            #[inline]
            pub fn iter(&self) -> impl Iterator<Item=($ipnet,&V)> + '_ {
                self.0.leaves.0.iter().map(|x| (x.prefix,&x.value))
            }

            #[inline]
            pub fn prefixes(&self) -> $rtrieset
            {
                $rtrieset(self.0.map(|_| ()))
            }
        }


        impl<K:Into<$ipnet>,V> Extend<(K,V)> for $rtrie<V>
        {
            fn extend<I: IntoIterator<Item=(K,V)>>(&mut self, iter: I)
            {
                iter.into_iter().for_each(|(k,v)| {self.insert(k,v);})
            }
        }

        impl<K:Into<$ipnet>,V:Default> FromIterator<(K,V)> for $rtrie<V>
        {
            fn from_iter<I:IntoIterator<Item=(K,V)>>(iter: I) -> Self
            {
                let mut triemap = Self::default();
                triemap.extend(iter);
                triemap
            }
        }

        impl<V> $lctrie<V>
        {
            #[inline]
            #[allow(clippy::len_without_is_empty)]
            pub fn len(&self) -> NonZeroUsize {
                unsafe {
                    NonZeroUsize::new_unchecked(self.0.len())
                }
            }

            #[inline]
            pub fn get<K:Into<$ipnet>>(&self, k: K) -> Option<&V> { self.0.get(&k.into()) }

            #[inline]
            pub fn get_mut<K:Into<$ipnet>>(&mut self, k: K) -> Option<&mut V>
            {
                self.0.get_mut(&k.into())
            }

            #[inline]
            pub fn lookup<K:Into<$ipnet>>(&self, k: K) -> ($ipnet,&V) { self.0.lookup(&k.into()) }

            #[inline]
            pub fn lookup_mut<K:Into<$ipnet>>(&mut self, k: K) -> ($ipnet,&mut V) { self.0.lookup_mut(&k.into()) }

            #[inline]
            pub fn info(&self) { self.0.info() }

            #[inline]
            pub fn iter(&self) -> impl Iterator<Item=($ipnet,&V)> + '_ {
                self.0.leaves.0.iter().map(|x| (x.prefix,&x.value))
            }

            #[inline]
            pub fn prefixes(&self) -> $lctrieset
            {
                $lctrieset(self.0.map(|_| ()))
            }
        }

        impl<K:Into<$ipnet>,V:Default> FromIterator<(K,V)> for $lctrie<V>
        {
            fn from_iter<I:IntoIterator<Item=(K,V)>>(iter: I) -> Self
            {
                $rtrie::from_iter(iter).compress()
            }
        }

        #[cfg(feature= "graphviz")]
        impl<V> DotWriter for $rtrie<V>
        {
            fn write_dot(&self, dot: &mut dyn std::io::Write) -> std::io::Result<()> {
                self.0.write_dot(dot)
            }
        }

        #[cfg(feature= "graphviz")]
        impl<V> DotWriter for $lctrie<V>
        {
            fn write_dot(&self, dot: &mut dyn std::io::Write) -> std::io::Result<()> {
                self.0.write_dot(dot)
            }
        }
    }
}


triemap!(Ipv4RTrieMap,Ipv4RTrieSet,Ipv4LCTrieMap,Ipv4LCTrieSet,Ipv4Net);
triemap!(Ipv6RTrieMap,Ipv6RTrieSet,Ipv6LCTrieMap,Ipv6LCTrieSet,Ipv6Net);


impl<V> IpRTrieMap<V>
{
    #[inline]
    pub fn new() -> Self where V:Default { Self::default() }

    #[inline]
    pub fn with_capacity(ipv4: usize, ipv6: usize) -> Self
        where V: Default
    {
        Self {
            v4: Ipv4RTrieMap::with_root_and_capacity(V::default(), ipv4),
            v6: Ipv6RTrieMap::with_root_and_capacity(V::default(), ipv6),
        }
    }

    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> NonZeroUsize {
        self.v4.len().checked_add(self.v6.len().get()).expect("overflow")
    }

    #[inline]
    pub fn with_root(ipv4: V, ipv6: V) -> Self {
        Self {
            v4: Ipv4RTrieMap::with_root(ipv4),
            v6: Ipv6RTrieMap::with_root(ipv6),
        }
    }

    #[inline]
    pub fn with_root_and_capacity(rootv4: V, ipv4: usize, rootv6: V, ipv6: usize) -> Self {
        Self {
            v4: Ipv4RTrieMap::with_root_and_capacity(rootv4, ipv4),
            v6: Ipv6RTrieMap::with_root_and_capacity(rootv6, ipv6),
        }
    }

    #[inline]
    pub fn compress(self) -> IpLCTrieMap<V> {
        IpLCTrieMap {
            v4: self.v4.compress(),
            v6: self.v6.compress(),
        }
    }

    #[inline]
    pub fn insert<K: Into<IpNet>>(&mut self, k: K, v: V) -> Option<V>
    {
        match k.into() {
            IpNet::V4(ip) => self.v4.insert(ip, v),
            IpNet::V6(ip) => self.v6.insert(ip, v)
        }
    }

    #[inline]
    pub fn get<K: Into<IpNet>>(&self, k: K) -> Option<&V> {
        match k.into() {
            IpNet::V4(ip) => self.v4.get(ip),
            IpNet::V6(ip) => self.v6.get(ip)
        }
    }

    #[inline]
    pub fn get_mut<K: Into<IpNet>>(&mut self, k: K) -> Option<&mut V> {
        match k.into() {
            IpNet::V4(ip) => self.v4.get_mut(ip),
            IpNet::V6(ip) => self.v6.get_mut(ip)
        }
    }

    #[inline]
    pub fn remove<K: Into<IpNet>>(&mut self, k: K) -> Option<V>
    {
        match k.into() {
            IpNet::V4(ip) => self.v4.remove(ip),
            IpNet::V6(ip) => self.v6.remove(ip)
        }
    }

    #[inline]
    pub fn lookup<K: Into<IpNet>>(&self, k: K) -> (IpNet, &V)
    {
        match k.into() {
            IpNet::V4(ip) => {
                let (ip, v) = self.v4.lookup(ip);
                (IpNet::V4(ip), v)
            }
            IpNet::V6(ip) => {
                let (ip, v) = self.v6.lookup(ip);
                (IpNet::V6(ip), v)
            }
        }
    }

    #[inline]
    pub fn lookup_mut<K: Into<IpNet>>(&mut self, k: K) -> (IpNet, &mut V) {
        match k.into() {
            IpNet::V4(ip) => {
                let (ip, v) = self.v4.lookup_mut(ip);
                (IpNet::V4(ip), v)
            }
            IpNet::V6(ip) => {
                let (ip, v) = self.v6.lookup_mut(ip);
                (IpNet::V6(ip), v)
            }
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=(IpNet, &V)> + '_
    {
        self.v4.iter().map(|(ip, v)| (IpNet::V4(ip), v))
            .chain(self.v6.iter().map(|(ip, v)| (IpNet::V6(ip), v)))
    }

    #[inline]
    pub fn prefixes(&self) -> IpRTrieSet
    {
        IpRTrieSet {
            v4: self.v4.prefixes(),
            v6: self.v6.prefixes(),
        }
    }
}


impl<K:Into<IpNet>,V> Extend<(K,V)> for IpRTrieMap<V>
{
    fn extend<I: IntoIterator<Item=(K,V)>>(&mut self, iter: I)
    {
        iter.into_iter().for_each(|(k,v)| { self.insert(k,v); })
    }
}

impl<K:Into<IpNet>,V:Default> FromIterator<(K,V)> for IpRTrieMap<V>
{
    fn from_iter<I:IntoIterator<Item=(K,V)>>(iter: I) -> Self
    {
        let mut triemap = Self::default();
        triemap.extend(iter);
        triemap
    }
}

impl<V> IpLCTrieMap<V>
{
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> NonZeroUsize {
        self.v4.len().checked_add(self.v6.len().get()).expect("overflow")
    }

    #[inline]
    pub fn get<K: Into<IpNet>>(&self, k: K) -> Option<&V>
    {
        match k.into() {
            IpNet::V4(ip) => self.v4.get(ip),
            IpNet::V6(ip) => self.v6.get(ip)
        }
    }

    #[inline]
    pub fn get_mut<K: Into<IpNet>>(&mut self, k: K) -> Option<&mut V>
    {
        match k.into() {
            IpNet::V4(ip) => self.v4.get_mut(ip),
            IpNet::V6(ip) => self.v6.get_mut(ip)
        }
    }

    #[inline]
    pub fn lookup<K: Into<IpNet>>(&self, k: K) -> (IpNet, &V)
    {
        match k.into() {
            IpNet::V4(ip) => {
                let (ip, v) = self.v4.lookup(ip);
                (IpNet::V4(ip), v)
            }
            IpNet::V6(ip) => {
                let (ip, v) = self.v6.lookup(ip);
                (IpNet::V6(ip), v)
            }
        }
    }

    #[inline]
    pub fn lookup_mut<K: Into<IpNet>>(&mut self, k: K) -> (IpNet, &mut V)
    {
        match k.into() {
            IpNet::V4(ip) => {
                let (ip, v) = self.v4.lookup_mut(ip);
                (IpNet::V4(ip), v)
            }
            IpNet::V6(ip) => {
                let (ip, v) = self.v6.lookup_mut(ip);
                (IpNet::V6(ip), v)
            }
        }
    }

    #[inline]
    pub fn info(&self) {
        self.v4.info();
        self.v6.info()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=(IpNet, &V)> + '_ {
        self.v4.iter().map(|(ip, v)| (IpNet::V4(ip), v))
            .chain(self.v6.iter().map(|(ip, v)| (IpNet::V6(ip), v)))
    }

    #[inline]
    pub fn prefixes(&self) -> IpLCTrieSet
    {
        IpLCTrieSet {
            v4: self.v4.prefixes(),
            v6: self.v6.prefixes(),
        }
    }
}

impl<K:Into<IpNet>,V:Default> FromIterator<(K,V)> for IpLCTrieMap<V>
{
    fn from_iter<I:IntoIterator<Item=(K,V)>>(iter: I) -> Self
    {
        IpRTrieMap::from_iter(iter).compress()
    }
}
