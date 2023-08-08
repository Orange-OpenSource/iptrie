use std::num::NonZeroUsize;
use crate::trie::patricia::RadixTrie;
use crate::trie::lctrie::LCTrie;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};

#[cfg(feature = "graphviz")] pub use crate::trie::graphviz::DotWriter;

/// A set of Ipv4 prefixes based on a radix binary trie
#[derive(Clone)]
pub struct Ipv4RTrieSet(pub(crate) RadixTrie<Ipv4Net,()>);
/// A set of Ipv6 prefixes based on a radix binary trie
#[derive(Clone)]
pub struct Ipv6RTrieSet(pub(crate) RadixTrie<Ipv6Net,()>);
/// A merge of two sets based on radix binary tries (Ipv4 and Ipv6)
#[derive(Clone,Default)]
pub struct IpRTrieSet { pub v4: Ipv4RTrieSet, pub v6: Ipv6RTrieSet }

/// A set of Ipv4 prefixes based on a level-compressed trie
pub struct Ipv4LCTrieSet(pub(crate) LCTrie<Ipv4Net,()>);
/// A set of Ipv6 prefixes based on a level-compressed trie
pub struct Ipv6LCTrieSet(pub(crate) LCTrie<Ipv6Net,()>);
/// A merge of two sets based on level-compressed tries (Ipv4 and Ipv6)
pub struct IpLCTrieSet { pub v4: Ipv4LCTrieSet, pub v6: Ipv6LCTrieSet }

macro_rules! trieset {
    ($rtrie:ident, $lctrie:ident, $ipnet:ty) => {

        impl $rtrie
        {
            #[inline]
            pub fn new() -> Self { Self::with_capacity(1000) }

            #[inline]
            #[allow(clippy::len_without_is_empty)]
            pub fn len(&self) -> NonZeroUsize {
                unsafe {
                    NonZeroUsize::new_unchecked(self.0.leaves.len())
                }
            }

            #[inline]
            pub fn compress(self) -> $lctrie { $lctrie(LCTrie::new(self.0)) }

            #[inline]
            pub fn with_capacity(capacity:usize) -> Self { Self(RadixTrie::new((), capacity)) }

            #[inline]
            pub fn insert<K:Into<$ipnet>>(&mut self, k: K) -> bool
            {
                self.0.insert(k.into().trunc(),()).is_none()
            }

            #[inline]
            pub fn contains<K:Into<$ipnet>>(&self, k: K) -> bool
            {
                self.0.get(&k.into().trunc()).is_some()
            }

            #[inline]
            pub fn remove<K:Into<$ipnet>>(&mut self, k: K) -> bool
            {
                self.0.remove(&k.into().trunc()).is_some()
            }

            #[inline]
            pub fn lookup<K:Into<$ipnet>>(&self, k: K) -> $ipnet {
                self.0.lookup(&k.into().trunc()).0
            }

            #[inline]
            pub fn iter(&self) -> impl Iterator<Item=$ipnet> + '_ {
                self.0.leaves.0.iter().map(|l| l.prefix )
            }
        }

        impl Default for $rtrie
        {
            #[inline]
            fn default() -> Self {
                Self::new()
            }
        }

        impl<T:Into<$ipnet>> Extend<T> for $rtrie
        {
            fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I)
            {
                iter.into_iter().for_each(| item | { self.insert(item); } )
            }
        }

        impl<T:Into<$ipnet>> FromIterator<T> for $rtrie
        {
            fn from_iter<I:IntoIterator<Item=T>>(iter: I) -> Self
            {
                let mut trieset = Self::default();
                trieset.extend(iter);
                trieset
            }
        }

        impl $lctrie
        {
            #[inline]
            #[allow(clippy::len_without_is_empty)]
            pub fn len(&self) -> NonZeroUsize {
                unsafe {
                    NonZeroUsize::new_unchecked(self.0.len())
                }
            }

            #[inline]
            pub fn contains<K:Into<$ipnet>>(&self, k: K) -> bool
            {
                self.0.get(&k.into()).is_some()
            }

            #[inline]
            pub fn lookup<K:Into<$ipnet>>(&self, k: K) -> $ipnet
            {
                self.0.lookup(&k.into()).0
            }

            #[inline]
            pub fn iter(&self) -> impl Iterator<Item=$ipnet> + '_ {
                self.0.leaves.0.iter().map(|l| l.prefix )
            }
        }

        impl<T:Into<$ipnet>> FromIterator<T> for $lctrie
        {
            fn from_iter<I:IntoIterator<Item=T>>(iter: I) -> Self
            {
                $rtrie::from_iter(iter).compress()
            }
        }


        #[cfg(feature= "graphviz")]
        impl DotWriter for $rtrie
        {
            fn write_dot(&self, dot: &mut dyn std::io::Write) -> std::io::Result<()> {
                self.0.write_dot(dot)
            }
        }

        #[cfg(feature= "graphviz")]
        impl DotWriter for $lctrie
        {
            fn write_dot(&self, dot: &mut dyn std::io::Write) -> std::io::Result<()> {
                self.0.write_dot(dot)
            }
        }
    };
}


trieset!(Ipv4RTrieSet,Ipv4LCTrieSet,Ipv4Net);
trieset!(Ipv6RTrieSet,Ipv6LCTrieSet,Ipv6Net);



impl IpRTrieSet
{
    #[inline]
    pub fn new() -> Self { IpRTrieSet { v4: Ipv4RTrieSet::new(), v6: Ipv6RTrieSet::new() } }

    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> NonZeroUsize {
        self.v4.len().checked_add(self.v6.len().get()).expect("overflow")
    }

    #[inline]
    pub fn compress(self) -> IpLCTrieSet {
        IpLCTrieSet {
            v4: self.v4.compress(),
            v6: self.v6.compress(),
        }
    }

    #[inline]
    pub fn with_capacity(ipv4:usize, ipv6:usize) -> Self {
        Self {
            v4: Ipv4RTrieSet::with_capacity(ipv4),
            v6: Ipv6RTrieSet::with_capacity(ipv6),
        }
    }

    #[inline]
    pub fn insert<K:Into<IpNet>>(&mut self, k: K) -> bool
    {
        match k.into() {
            IpNet::V4(ip) => self.v4.insert(ip),
            IpNet::V6(ip) => self.v6.insert(ip)
        }
    }

    #[inline]
    pub fn contains<K:Into<IpNet>>(&self, k: K) -> bool
    {
        match k.into() {
            IpNet::V4(ip) => self.v4.contains(ip),
            IpNet::V6(ip) => self.v6.contains(ip)
        }
    }

    #[inline]
    pub fn remove<K:Into<IpNet>>(&mut self, k: K) -> bool
    {
        match k.into() {
            IpNet::V4(ip) => self.v4.remove(ip),
            IpNet::V6(ip) => self.v6.remove(ip)
        }
    }

    #[inline]
    pub fn lookup<K:Into<IpNet>>(&self, k: K) -> IpNet {
        match k.into() {
            IpNet::V4(ip) => IpNet::V4(self.v4.lookup(ip)),
            IpNet::V6(ip) => IpNet::V6(self.v6.lookup(ip))
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=IpNet> + '_ {
        self.v4.iter().map(IpNet::V4).chain(self.v6.iter().map(IpNet::V6))
    }
}


impl<T:Into<IpNet>> Extend<T> for IpRTrieSet
{
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I)
    {
        iter.into_iter().for_each(| item | { self.insert(item); } )
    }
}

impl<T:Into<IpNet>> FromIterator<T> for IpRTrieSet
{
    fn from_iter<I:IntoIterator<Item=T>>(iter: I) -> Self
    {
        let mut trieset = Self::default();
        trieset.extend(iter);
        trieset
    }
}

impl IpLCTrieSet
{
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> NonZeroUsize {
        self.v4.len().checked_add(self.v6.len().get()).expect("overflow")
    }

    #[inline]
    pub fn contains<K:Into<IpNet>>(&self, k: K) -> bool
    {
        match k.into() {
            IpNet::V4(ip) => self.v4.contains(ip),
            IpNet::V6(ip) => self.v6.contains(ip)
        }
    }

    #[inline]
    pub fn lookup<K:Into<IpNet>>(&self, k: K) -> IpNet
    {
        match k.into() {
            IpNet::V4(ip) => IpNet::V4(self.v4.lookup(ip)),
            IpNet::V6(ip) => IpNet::V6(self.v6.lookup(ip))
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=IpNet> + '_ {
        self.v4.iter().map(IpNet::V4).chain(self.v6.iter().map(IpNet::V6))
    }
}

impl<T:Into<IpNet>> FromIterator<T> for IpLCTrieSet
{
    fn from_iter<I:IntoIterator<Item=T>>(iter: I) -> Self
    {
        IpRTrieSet::from_iter(iter).compress()
    }
}