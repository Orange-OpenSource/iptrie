use std::num::NonZeroUsize;
use crate::trie::patricia::RadixTrie;
use crate::trie::lctrie::LevelCompressedTrie;
use crate::prefix::*;

#[cfg(feature = "graphviz")] pub use crate::trie::graphviz::DotWriter;
#[cfg(feature = "graphviz")] use std::fmt::Display;
use crate::trie::common::Leaf;

/// A set of Ip prefixes based on a radix binary trie
#[derive(Clone)]
pub struct RTrieSet<P: IpPrefix>(pub(crate) RadixTrie<P,()>);

/// Convenient alias for radix trie set of Ipv4 prefixes
pub type Ipv4RTrieSet = RTrieSet<Ipv4Prefix>;
/// Convenient alias for radix trie set of Ipv6 prefixes
pub type Ipv6RTrieSet = RTrieSet<Ipv6Prefix>;

/// A set of Ip prefixes based on a level-compressed trie
pub struct LCTrieSet<P: IpPrefix>(pub(crate) LevelCompressedTrie<P,()>);

/// Convenient alias for LC-Trie set of Ipv4 prefixes
pub type Ipv4LCTrieSet = LCTrieSet<Ipv4Prefix>;
/// Convenient alias for LC-Trie set of Ipv6 prefixes
pub type Ipv6LCTrieSet = LCTrieSet<Ipv6Prefix>;

impl<P:IpPrefix> RTrieSet<P>
{
    /// Creates a new set which contains the root prefix.
    #[inline]
    pub fn new() -> Self { Self::with_capacity(1000) }

    /// Creates a new set with an initial capacity.
    ///
    /// The returned set already contains the root prefix.
    #[inline]
    pub fn with_capacity(capacity:usize) -> Self { Self(RadixTrie::new((), capacity)) }

    /// Returns the size of the set.
    ///
    /// Notice that it never equals zero since the top prefix is
    /// always present in the set.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> NonZeroUsize { self.0.len() }

    /// Compress this Patricia trie in a LC-Trie.
    ///
    /// For lookup algorithms, a Patricia trie performs unit bit checking and LC-Trie
    /// performs multi bits checking. So the last one is more performant but it
    /// cannot be modified (no insertion or removal operations are provided).
    #[inline]
    pub fn compress(self) -> LCTrieSet<P> { LCTrieSet(LevelCompressedTrie::new(self.0)) }

    #[inline]
    pub fn shrink_to_fit(&mut self) { self.0.shrink_to_fit() }

    /// Inserts a new element in the set.
    ///
    /// If the specified element already exists in the set, `false` is returned.
    ///
    /// # Example
    /// ```
    /// #  use iptrie::*;
    /// use std::net::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::new(1,1,1,1);
    /// let mut trie = Ipv4RTrieSet::new();
    ///
    /// let ip20 = Ipv4Prefix::new(addr, 20).unwrap();
    /// let ip22 = Ipv4Prefix::new(addr, 22).unwrap();
    ///
    /// assert_eq!( trie.insert(ip20), true);
    /// assert_eq!( trie.insert(ip22), true);
    /// assert_eq!( trie.insert(ip20), false);
    /// ```
    #[inline]
    pub fn insert(&mut self, k: P) -> bool
    {
        self.0.insert(k,()).is_none()
    }

    /// Checks if an element is present (exact match).
    ///
    /// # Example
    /// ```
    /// #  use iptrie::*;
    /// use std::net::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::new(1,1,1,1);
    ///
    /// let ip20 = Ipv4Prefix::new(addr, 20).unwrap();
    /// let ip22 = Ipv4Prefix::new(addr, 22).unwrap();
    /// let ip24 = Ipv4Prefix::new(addr, 24).unwrap();
    ///
    /// let trie = Ipv4RTrieSet::from_iter([ip20,ip24]);
    ///
    /// assert_eq!( trie.contains(&ip20), true);
    /// assert_eq!( trie.contains(&ip22), false);
    /// assert_eq!( trie.contains(&ip24), true);
    /// ```
    #[inline]
    pub fn contains<Q>(&self, k: &Q) -> bool
        where
            Q: IpPrefix<Addr=P::Addr>,
            P: IpPrefixCovering<Q>
    {
        self.0.get(k).is_some()
    }

    /// Removes a previously inserted prefix (exact match).
    ///
    /// Returns `false` is the element was not present in the set
    /// and `true` if the removal is effective.
    ///
    /// # Example
    /// ```
    /// #  use iptrie::*;
    /// use std::net::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::new(1,1,1,1);
    ///
    /// let ip20 = Ipv4Prefix::new(addr, 20).unwrap();
    /// let ip22 = Ipv4Prefix::new(addr, 22).unwrap();
    ///
    /// let mut trie = Ipv4RTrieSet::from_iter([ip20,ip22]);
    ///
    /// assert_eq!( trie.contains(&ip20), true);
    /// assert_eq!(trie.remove(&ip20), true);
    /// assert_eq!(trie.remove(&ip20), false);
    ///
    /// assert_eq!( trie.contains(&ip22), true);
    /// assert_eq!( trie.contains(&ip20), false);
    /// ```
    #[inline]
    pub fn remove<Q>(&mut self, k: &Q) -> bool
        where
            Q: IpPrefix<Addr=P::Addr>,
            P: IpPrefixCovering<Q>
    {
        self.0.remove(k).is_some()
    }

    /// Adds a prefix to the set, replacing the existing one, if any (exact match performed).
    /// Returns the replaced value.
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// use std::net::Ipv4Addr;
    /// use ipnet::Ipv4Net;
    /// let mut trie = RTrieSet::new();
    ///
    /// let addr1 = Ipv4Addr::new(1,1,1,1);;
    /// let addr2 = Ipv4Addr::new(1,1,1,2);;
    /// let ip20 = Ipv4Net::new(addr1, 20).unwrap();
    /// let ip20b = Ipv4Net::new(addr2, 20).unwrap();
    ///
    /// trie.insert(ip20);
    /// assert_eq!(trie.get(&ip20).unwrap().to_string(), "1.1.1.1/20".to_string());
    ///
    /// assert_eq!(trie.insert(ip20b), false);
    /// assert_eq!(trie.get(&ip20).unwrap().to_string(), "1.1.1.1/20".to_string());
    ///
    /// assert_eq!(trie.replace(ip20b), Some(ip20));
    /// assert_eq!(trie.get(&ip20).unwrap().to_string(), "1.1.1.2/20".to_string());
    /// ```
    #[inline]
    pub fn replace(&mut self, k: P) -> Option<P>
    {
        self.0.replace(k,()).map(|l| *l.prefix())
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
    /// use ipnet::Ipv4Net;
    /// let mut trie = RTrieSet::new();
    ///
    /// let addr1 = Ipv4Addr::new(1,1,1,1);;
    /// let addr2 = Ipv4Addr::new(1,1,1,2);;
    /// let ip20 = Ipv4Net::new(addr1, 20).unwrap();
    /// let ip20b = Ipv4Net::new(addr2, 20).unwrap();
    ///
    /// trie.insert(ip20);
    /// assert_eq!(trie.get(&ip20).unwrap().to_string(), "1.1.1.1/20".to_string());
    ///
    /// trie.insert(ip20b);
    /// assert_eq!(trie.get(&ip20).unwrap().to_string(), "1.1.1.1/20".to_string());
    /// ```
    #[inline]
    pub fn get<Q>(&self, k: &Q) -> Option<&P>
        where
            Q: IpPrefix<Addr=P::Addr>,
            P: IpPrefixCovering<Q>
    { self.0.get(k).map(|(k,_)| k) }


    /// Gets the longest prefix which matches the given key.
    ///
    /// As the top prefix always matches, it never fails.
    ///
    /// To access to the exact prefix match, use [`Self::get`].
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// use std::net::Ipv4Addr;
    /// let mut trie = Ipv4RTrieSet::new();
    ///
    /// let addr = Ipv4Addr::new(1,1,1,1);;
    /// let ip20 = Ipv4Prefix::new(addr, 20).unwrap();
    /// let ip22 = Ipv4Prefix::new(addr, 22).unwrap();
    /// let ip24 = Ipv4Prefix::new(addr, 24).unwrap();
    ///
    /// trie.insert(ip20);
    /// trie.insert(ip24);
    ///
    /// assert_eq!( trie.lookup(&ip20), &ip20);
    /// assert_eq!( trie.lookup(&ip22), &ip20);
    /// assert_eq!( trie.lookup(&ip24), &ip24);
    ///
    /// assert_eq!( trie.lookup(&addr), &ip24);
    /// ```
    #[inline]
    pub fn lookup<Q>(&self, k: &Q) -> &P
        where
            Q: IpPrefix<Addr=P::Addr>,
            P: IpPrefixCovering<Q>
    {
        self.0.lookup(k).0
    }

    /// Iterates over all the prefixes of this set.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=&P> + '_ {
        self.0.leaves.0.iter().map(Leaf::prefix)
    }

    #[inline]
    pub fn info(&self) { self.0.info() }
}

impl<P:IpPrefix> Default for RTrieSet<P>
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<P:IpPrefix> Extend<P> for RTrieSet<P>
{
    fn extend<I: IntoIterator<Item=P>>(&mut self, iter: I)
    {
        iter.into_iter().for_each(| item | { self.insert(item); } )
    }
}

impl<P:IpPrefix> FromIterator<P> for RTrieSet<P>
{
    fn from_iter<I:IntoIterator<Item=P>>(iter: I) -> Self
    {
        let mut trieset = Self::default();
        trieset.extend(iter);
        trieset
    }
}

impl<P:IpPrefix> LCTrieSet<P>
{
    /// Returns the size of the set.
    ///
    /// Notice that it never equals zero since the top prefix is
    /// always present in the set.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> NonZeroUsize { self.0.len() }

    #[inline]
    pub fn info(&self) { self.0.info() }

    /// Checks if an element is present (exact match).
    ///
    /// # Example
    /// ```
    /// #  use iptrie::*;
    /// use std::net::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::new(1,1,1,1);
    ///
    /// let ip20 = Ipv4Prefix::new(addr, 20).unwrap();
    /// let ip22 = Ipv4Prefix::new(addr, 22).unwrap();
    /// let ip24 = Ipv4Prefix::new(addr, 24).unwrap();
    ///
    /// let trie = Ipv4RTrieSet::from_iter([ip20,ip24]);
    /// let lctrie = trie.compress();
    ///
    /// assert_eq!( lctrie.contains(&ip20), true);
    /// assert_eq!( lctrie.contains(&ip22), false);
    /// assert_eq!( lctrie.contains(&ip24), true);
    /// ```
    #[inline]
    pub fn contains<Q>(&self, k: &Q) -> bool
        where
            Q: IpPrefix<Addr=P::Addr>,
            P: IpPrefixCovering<Q>+PartialEq<Q>
    {
        self.0.get(k).is_some()
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
    ///
    /// assert_eq!( lctrie.get(&ip24), Some(&24));
    /// assert_eq!( lctrie.get(&ip22), None);
    /// assert_eq!( lctrie.get(&ip20), Some(&20));
    /// ```
    #[inline]
    pub fn get<Q>(&self, k: &Q) -> Option<&P>
        where
            Q: IpPrefix<Addr=P::Addr>,
            P: IpPrefixCovering<Q>
    { self.0.get(k).map(|(k,_)| k) }


    /// Gets the longest prefix which matches the given key.
    ///
    /// As the top prefix always matches, it never fails.
    ///
    /// To access to the exact prefix match, use [`Self::get`].
    ///
    /// # Example
    /// ```
    /// # use iptrie::*;
    /// use std::net::Ipv4Addr;
    /// let mut trie = Ipv4RTrieSet::new();
    ///
    /// let addr = Ipv4Addr::new(1,1,1,1);;
    /// let ip20 = Ipv4Prefix::new(addr, 20).unwrap();
    /// let ip22 = Ipv4Prefix::new(addr, 22).unwrap();
    /// let ip24 = Ipv4Prefix::new(addr, 24).unwrap();
    ///
    /// trie.insert(ip20);
    /// trie.insert(ip24);
    ///
    /// let lctrie = trie.compress();
    ///
    /// assert_eq!( lctrie.lookup(&ip20), &ip20);
    /// assert_eq!( lctrie.lookup(&ip22), &ip20);
    /// assert_eq!( lctrie.lookup(&ip24), &ip24);
    ///
    /// assert_eq!( lctrie.lookup(&addr), &ip24);
    /// ```
    #[inline]
    pub fn lookup<Q>(&self, k: &Q) -> &P
        where
            Q: IpPrefix<Addr=P::Addr>,
            P: IpPrefixCovering<Q>
    {
        self.0.lookup(k).0
    }

    /// Iterates over all the prefixes of this set.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=&P> + '_ {
        self.0.leaves.0.iter().map(Leaf::prefix)
    }
}

impl<P:IpPrefix> FromIterator<P> for LCTrieSet<P>
{
    fn from_iter<I:IntoIterator<Item=P>>(iter: I) -> Self
    {
        RTrieSet::from_iter(iter).compress()
    }
}


#[cfg(feature= "graphviz")]
impl<P:IpPrefix+Display> DotWriter for RTrieSet<P>
{
    fn write_dot(&self, dot: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.0.write_dot(dot)
    }
}

#[cfg(feature= "graphviz")]
impl<P:IpPrefix+Display> DotWriter for LCTrieSet<P>
{
    fn write_dot(&self, dot: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.0.write_dot(dot)
    }
}


