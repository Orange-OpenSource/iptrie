use std::num::NonZeroUsize;
use crate::trie::patricia::RadixTrie;
use crate::trie::lctrie::LevelCompressedTrie;
use crate::set::*;

use crate::prefix::*;

#[cfg(feature = "ipnet")] use ipnet::IpNet;
#[cfg(feature = "graphviz")] pub use crate::trie::graphviz::DotWriter;
#[cfg(feature = "graphviz")] use std::fmt::Display;
use crate::trie::common::Leaf;

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
    /// Notice that it never equals zero since the top prefix is
    /// always present in the map.
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// use std::net::Ipv4Addr;
    /// let trie = Ipv4RTrieMap::with_root(42);
    ///
    /// assert_eq!(trie.len().get(), 1);
    /// ```
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> NonZeroUsize { self.0.len() }

    /// Creates a new trie map with the specified value associated to the
    /// root prefix.
    #[inline]
    pub fn with_root(root: V) -> Self { Self::with_root_and_capacity(root, 1000) }

    /// Creates a new trie map with a initial capacity.
    #[inline]
    pub fn with_root_and_capacity(root: V, capacity: usize) -> Self {
        Self(RadixTrie::new(root, capacity))
    }

    /// Compress this Patricia trie in a LC-Trie.
    ///
    /// For lookup algorithms, a Patricia trie performs unit bit checking and LC-Trie
    /// performs multi bits checking. So the last one is more performant but it
    /// cannot be modified (no insertion or removal operations are provided).
    #[inline]
    pub fn compress(self) -> LCTrieMap<K,V> { LCTrieMap(LevelCompressedTrie::new(self.0)) }

    #[inline]
    pub fn shrink_to_fit(&mut self) { self.0.shrink_to_fit() }

    /// Inserts a new entry in the map.
    ///
    /// If the specified key already exists in the map, then the previous associated
    /// value is replaced by the new one and is returned.
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// use std::net::Ipv4Addr;
    ///
    /// let mut trie = RTrieMap::with_root(42);
    /// let addr = Ipv4Addr::new(1,1,1,1);
    /// let ip = Ipv4Prefix::new(addr, 20).unwrap();
    ///
    /// assert_eq!( trie.insert(ip, 45), None);
    /// assert_eq!( trie.insert(ip, 50), Some(45));
    ///
    /// assert_eq!( trie.insert(Ipv4Prefix::default(), 12), Some(42));
    /// ```
    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.0.insert(k, v)
    }

    /// Gets the value associated with an exact match of the key.
    ///
    /// To access to the longest prefix match, use [`Self::lookup`].
    ///
    /// To get a mutable access to a value, use [`Self::get_mut`].
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// use std::net::Ipv4Addr;
    /// let mut trie = RTrieMap::with_root(42);
    ///
    /// let addr = Ipv4Addr::new(1,1,1,1);;
    /// let ip20 = Ipv4Prefix::new(addr, 20).unwrap();
    /// let ip22 = Ipv4Prefix::new(addr, 22).unwrap();
    /// let ip24 = Ipv4Prefix::new(addr, 24).unwrap();
    ///
    /// trie.insert(ip24, 24);
    /// trie.insert(ip20, 20);
    ///
    /// assert_eq!( trie.get(&ip24), Some(&24));
    /// assert_eq!( trie.get(&ip22), None);
    /// assert_eq!( trie.get(&ip20), Some(&20));
    /// ```
    #[inline]
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    {
        self.0.get(k).map(|(_,v)| v)
    }

    /// Gets a mutable access to the value associated with an exact match of the key.
    ///
    /// To access to the longest prefix match, use [`Self::lookup_mut`].
    ///
    /// To get a mutable access to a value, use [`Self::get_mut`].
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// use std::net::Ipv4Addr;
    /// let mut trie = RTrieMap::with_root(42);
    ///
    /// let addr = Ipv4Addr::new(1,1,1,1);;
    /// let ip20 = Ipv4Prefix::new(addr, 20).unwrap();
    /// let ip22 = Ipv4Prefix::new(addr, 22).unwrap();
    ///
    /// trie.insert(ip20, 20);
    /// assert_eq!( trie.get(&ip20), Some(&20));
    ///
    /// assert_eq!( trie.get_mut(&ip22), None);
    /// *trie.get_mut(&ip20).unwrap() = 42;
    /// assert_eq!( trie.get(&ip20), Some(&42));
    /// ```
    #[inline]
    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    {
        self.0.get_mut(k).map(|(_,v)| v)
    }

    /// Removes a previously inserted prefix (exact match).
    /// # Panic
    /// Panics if trying to remove the root prefix.
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// use std::net::Ipv4Addr;
    /// let mut trie = RTrieMap::with_root(42);
    ///
    /// let addr = Ipv4Addr::new(1,1,1,1);;
    /// let ip20 = Ipv4Prefix::new(addr, 20).unwrap();
    /// let ip22 = Ipv4Prefix::new(addr, 22).unwrap();
    ///
    /// trie.insert(ip22, 22);
    /// trie.insert(ip20, 20);
    /// assert_eq!( trie.get(&ip22), Some(&22));
    ///
    /// trie.remove(&ip22);
    /// assert_eq!( trie.get(&ip22), None);
    /// ```
    #[inline]
    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    {
        self.0.remove(k)
    }

    /// Gets the entry associated with the longest prefix match of the key.
    ///
    /// As the top prefix always matches, it never fails.
    ///
    /// To access to the exact prefix match, use [`Self::get`].
    ///
    /// To get a mutable access to a value, use [`Self::lookup_mut`].
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// use std::net::Ipv4Addr;
    /// let mut trie = RTrieMap::with_root(42);
    ///
    /// let addr = Ipv4Addr::new(1,1,1,1);;
    /// let ip20 = Ipv4Prefix::new(addr, 20).unwrap();
    /// let ip22 = Ipv4Prefix::new(addr, 22).unwrap();
    /// let ip24 = Ipv4Prefix::new(addr, 24).unwrap();
    ///
    /// trie.insert(ip20, 20);
    /// trie.insert(ip24, 24);
    ///
    /// assert_eq!( trie.lookup(&ip20), (&ip20, &20));
    /// assert_eq!( trie.lookup(&ip22), (&ip20, &20));
    /// assert_eq!( trie.lookup(&ip24), (&ip24, &24));
    ///
    /// assert_eq!( trie.lookup(&addr), (&ip24, &24));
    /// ```
    #[inline]
    pub fn lookup<Q>(&self, k: &Q) -> (&K, &V)
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    { self.0.lookup(k) }

    /// Gets a mutable access to the value associated with a longest prefix match of the key.
    ///
    /// To access to the exact prefix match, use [`Self::get_mut`].
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// let mut trie = RTrieMap::with_root(42);
    ///
    /// let ip20 = "1.1.1.1/20".parse::<Ipv4Prefix>().unwrap();
    /// let ip22 = "1.1.1.1/22".parse::<Ipv4Prefix>().unwrap();
    ///
    /// trie.insert(ip20, 20);
    /// assert_eq!( trie.get(&ip20), Some(&20));
    ///
    /// *trie.lookup_mut(&ip22).1 = 42;
    /// assert_eq!( trie.get(&ip20), Some(&42));
    /// ```
    #[inline]
    pub fn lookup_mut<Q>(&mut self, k: &Q) -> (&K, &mut V)
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    { self.0.lookup_mut(k) }

    /// Iterates over all the entries.
    ///
    /// For a mutable access of values, use [`Self::iter_mut`]
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=(&K,&V)> + '_ {
        self.0.iter().map(Leaf::get)
    }

    /// Iterates over all the entries with a mutable access to values.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item=(&K,&mut V)> + '_ {
        self.0.iter_mut().map(Leaf::get_mut)
    }

    /// Gets a set of copy of all the keys in a trie set.
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
    /// Returns the size of the map.
    ///
    /// Notice that it never equals zero since the top prefix is
    /// always present in the map.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> NonZeroUsize {
        self.0.len()
    }


    /// Gets the value associated with an exact match of the key.
    ///
    /// To access to the longest prefix match, use [`Self::lookup`].
    ///
    /// To get a mutable access to a value, use [`Self::get_mut`].
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// use std::net::Ipv4Addr;
    /// let mut trie = RTrieMap::with_root(42);
    ///
    /// let addr = Ipv4Addr::new(1,1,1,1);;
    /// let ip20 = Ipv4Prefix::new(addr, 20).unwrap();
    /// let ip22 = Ipv4Prefix::new(addr, 22).unwrap();
    /// let ip24 = Ipv4Prefix::new(addr, 24).unwrap();
    ///
    /// trie.insert(ip24, 24);
    /// trie.insert(ip20, 20);
    ///
    /// let lctrie = trie.compress();
    /// assert_eq!( lctrie.get(&ip20), Some(&20));
    /// assert_eq!( lctrie.get(&ip22), None);
    /// assert_eq!( lctrie.get(&ip24), Some(&24));
    /// ```
    #[inline]
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>+PartialEq<Q>
    { self.0.get(k).map(|(_,v)| v) }

    /// Gets a mutable access to the value associated with an exact match of the key.
    ///
    /// To access to the longest prefix match, use [`Self::lookup_mut`].
    ///
    /// To get a mutable access to a value, use [`Self::get_mut`].
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// use std::net::Ipv4Addr;
    /// let mut trie = RTrieMap::new();
    ///
    /// let addr = Ipv4Addr::new(1,1,1,1);;
    /// let ip20 = Ipv4Prefix::new(addr, 20).unwrap();
    /// let ip22 = Ipv4Prefix::new(addr, 22).unwrap();
    ///
    /// trie.insert(ip20, 20);
    /// let mut lctrie = trie.compress();
    ///
    /// assert_eq!( lctrie.get(&ip20), Some(&20));
    ///
    /// assert_eq!( lctrie.get_mut(&ip22), None);
    /// *lctrie.get_mut(&ip20).unwrap() = 42;
    /// assert_eq!( lctrie.get(&ip20), Some(&42));
    /// ```
    #[inline]
    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>+PartialEq<Q>
    {
        self.0.get_mut(k).map(|(_,v)| v)
    }

    /// Gets the value associated with the longest prefix match of the key.
    ///
    /// As the top prefix always matches, the lookup never fails.
    ///
    /// To access to the exact prefix match, use [`Self::get`].
    ///
    /// To get a mutable access to a value, use [`Self::lookup_mut`].
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// use std::net::Ipv4Addr;
    /// let mut trie = RTrieMap::with_root(42);
    ///
    /// let addr = Ipv4Addr::new(1,1,1,1);;
    /// let ip20 = Ipv4Prefix::new(addr, 20).unwrap();
    /// let ip22 = Ipv4Prefix::new(addr, 22).unwrap();
    /// let ip24 = Ipv4Prefix::new(addr, 24).unwrap();
    ///
    /// trie.insert(ip20, 20);
    /// trie.insert(ip24, 24);
    ///
    /// let lctrie = trie.compress();
    ///
    /// assert_eq!( lctrie.lookup(&ip20), (&ip20, &20));
    /// assert_eq!( lctrie.lookup(&ip22), (&ip20, &20));
    /// assert_eq!( lctrie.lookup(&ip24), (&ip24, &24));
    ///
    /// assert_eq!( lctrie.lookup(&addr), (&ip24, &24));
    /// ```
    #[inline]
    pub fn lookup<Q>(&self, k: &Q) -> (&K,&V)
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    { self.0.lookup(k) }


    /// Gets a mutable access to the value associated with a longest prefix match of the key.
    ///
    /// To access to the exact prefix match, use [`Self::get_mut`].
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// let mut trie = RTrieMap::with_root(42);
    ///
    /// let ip20 = "1.1.1.1/20".parse::<Ipv4Prefix>().unwrap();
    /// let ip22 = "1.1.1.1/22".parse::<Ipv4Prefix>().unwrap();
    ///
    /// trie.insert(ip20, 20);
    ///
    /// let mut lctrie = trie.compress();
    ///
    /// assert_eq!( lctrie.get(&ip20), Some(&20));
    ///
    /// *lctrie.lookup_mut(&ip22).1 = 42;
    /// assert_eq!( lctrie.get(&ip20), Some(&42));
    /// ```
    #[inline]
    pub fn lookup_mut<Q>(&mut self, k: &Q) -> (&K,&mut V)
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    { self.0.lookup_mut(k) }

    #[inline]
    pub fn info(&self) { self.0.info() }

    /// Iterates over all the entries.
    ///
    /// As the root prefix always exists, this iterator is never empty.
    ///
    /// For a mutable access of values, use [`Self::iter_mut`]
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// let trie = Ipv4RTrieMap::with_root(42);
    /// let lctrie = trie.compress();
    /// assert_eq!( lctrie.len().get(), lctrie.iter().count());
    /// ```
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=(&K,&V)> + '_ {
        self.0.leaves.0.iter().map(Leaf::get)
    }

    /// Iterates over all the entries with a mutable access to values.
    /// As the root prefix always exists, this iterator is never empty.
    ///
    /// For a mutable access of values, use [`Self::iter_mut`]
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// let trie = Ipv4RTrieMap::with_root(42);
    /// let mut lctrie = trie.compress();
    /// lctrie.iter_mut().for_each(|(_,v)| *v += 1 );
    /// assert_eq!( lctrie.lookup(&Ipv4Prefix::root()).1, &43);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item=(&K,&mut V)> + '_ {
        self.0.leaves.0.iter_mut().map(Leaf::get_mut)
    }

    /// Gets a set of copy of all the keys in a trie set.
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// let trie = Ipv4RTrieMap::with_root(42);
    /// let mut lctrie = trie.compress();
    /// assert_eq!( lctrie.len(), lctrie.prefixes().len());
    /// ```
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

