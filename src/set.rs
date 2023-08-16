use std::num::NonZeroUsize;
use crate::trie::patricia::RadixTrie;
use crate::trie::lctrie::LevelCompressedTrie;
use crate::prefix::*;

#[cfg(feature = "graphviz")] pub use crate::trie::graphviz::DotWriter;
#[cfg(feature = "graphviz")] use std::fmt::Display;

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
    #[inline]
    pub fn new() -> Self { Self::with_capacity(1000) }

    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> NonZeroUsize { self.0.len() }

    #[inline]
    pub fn compress(self) -> LCTrieSet<P> { LCTrieSet(LevelCompressedTrie::new(self.0)) }

    #[inline]
    pub fn with_capacity(capacity:usize) -> Self { Self(RadixTrie::new((), capacity)) }

    #[inline]
    pub fn insert(&mut self, k: P) -> bool
    {
        self.0.insert(k,()).is_none()
    }

    #[inline]
    pub fn contains<Q>(&self, k: &Q) -> bool
        where
            Q: IpPrefix<Addr=P::Addr>,
            P: IpPrefixCovering<Q>
    {
        self.0.get(k).is_some()
    }

    #[inline]
    pub fn remove<Q>(&mut self, k: &Q) -> bool
        where
            Q: IpPrefix<Addr=P::Addr>,
            P: IpPrefixCovering<Q>
    {
        self.0.remove(k).is_some()
    }

    #[inline]
    pub fn replace(&mut self, k: P) -> Option<P>
    {
        self.0.replace(k,()).map(|l| l.prefix)
    }

    #[inline]
    pub fn lookup<Q>(&self, k: &Q) -> &P
        where
            Q: IpPrefix<Addr=P::Addr>,
            P: IpPrefixCovering<Q>
    {
        self.0.lookup(k).0
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=&P> + '_ {
        self.0.leaves.0.iter().map(|l| &l.prefix )
    }
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
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> NonZeroUsize { self.0.len() }

    #[inline]
    pub fn contains<Q>(&self, k: &Q) -> bool
        where
            Q: IpPrefix<Addr=P::Addr>,
            P: IpPrefixCovering<Q>+PartialEq<Q>
    {
        self.0.get(k).is_some()
    }

    #[inline]
    pub fn lookup<Q>(&self, k: &Q) -> &P
        where
            Q: IpPrefix<Addr=P::Addr>,
            P: IpPrefixCovering<Q>
    {
        self.0.lookup(k).0
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=&P> + '_ {
        self.0.leaves.0.iter().map(|l| &l.prefix )
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


