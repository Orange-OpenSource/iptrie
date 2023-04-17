use std::ops::{Shr, Shl, BitAnd, Not, BitOr, BitXor};
use std::fmt;
use std::hash::Hash;

/// A fixed-length slot of bits.
///
/// This is the slot on which all the prefix
/// (and so the tries) computations are made.
/// As an example, `u32` or `u128` are good candidates
/// for Ipv4 or Ipv6 prefixes.
pub trait BitSlot
: 'static
+ Clone + Copy + Default
+ fmt::Debug + fmt::Binary
+ Eq + PartialEq + Ord + PartialOrd + Hash
+ Not<Output=Self> + BitAnd<Output=Self> + BitOr<Output=Self> + BitXor<Output=Self>
+ Shl<u8,Output=Self> + Shr<u8,Output=Self>
+ BitPrefix<Slot=Self>
{
    const LEN: u8;
    fn single_bit(pos: u8) -> Self;
    fn bitmask(len: u8) -> Self;

    fn first_bit(&self) -> u8;
    fn is_set(&self, pos: u8) -> bool;

    fn as_u16(&self) -> u16;
}

macro_rules! bitslot {
    ($X:ty) => {
        impl BitSlot for $X {
            const LEN: u8 = std::mem::size_of::<$X>() as u8 * 8;
            #[inline] fn first_bit(&self) -> u8 {
                self.leading_zeros() as u8 + 1
            }
            #[inline] fn single_bit(pos: u8) -> Self {
                debug_assert!(pos > 0); debug_assert!( pos <= Self::LEN);
                1 as $X << (Self::LEN-pos)
            }
            #[inline] fn is_set(&self, pos: u8) -> bool {
                debug_assert!(pos > 0); debug_assert!( pos <= Self::LEN);
                (self >> (Self::LEN-pos)) & 1 != 0
            }
            #[inline] fn bitmask(len:u8) -> Self {
                debug_assert!( len <= Self::LEN);
                if len == 0 { 0 } else { (!0 as $X) << (Self::LEN-len) }
            }
            #[inline] fn as_u16(&self) -> u16 {
                *self as u16
            }
        }
        // each slot is a prefix with its maximal length
        impl BitPrefix for $X {
            type Slot = $X;
            #[inline] fn root() -> Self { 0 }
            #[inline] fn bitslot(&self) -> Self::Slot { *self }
            #[inline] fn len(&self) -> u8 { <$X as BitSlot>::LEN }
        }
    };
}

bitslot!(u32);
bitslot!(u64);
bitslot!(u128);


/// Inner bit prefix
pub trait BitPrefix: fmt::Debug+Clone+Eq
{
    type Slot: BitSlot;

    fn root() -> Self; // root prefix, of len =0

    fn bitslot(&self) -> Self::Slot;

    /// Gets the number of significant bits
    fn len(&self) -> u8;
}



use std::ops::{Index, IndexMut};

#[derive(Clone)]
pub(crate) struct TrieLeaves<L>(pub(crate) Vec<L>);


#[derive(Clone)]
pub(crate) struct Leaf<K,V> {
    pub(crate) prefix: K,
    pub(crate) value: V
}

impl<K,V> Leaf<K,V> {
    pub(crate) fn new(k:K, v:V) -> Self { Self { prefix: k, value: v}}
}


impl<K:BitPrefix, V> TrieLeaves<Leaf<K, V>>
{
    pub fn new(capacity: usize, k:K, v:V) -> Self {
        let mut leaves = Vec::with_capacity(capacity);
        leaves.push(Leaf::new(k,v));
        Self(leaves)
    }
}

impl<L> TrieLeaves<L>
{
    // returns the index of the added leaf
    pub fn push(&mut self, leaf: L) -> LeafIndex {
        let index = self.0.len().into();
        self.0.push(leaf);
        index
    }

    pub fn remove_last(&mut self) -> Option<L>
    {
        debug_assert!(self.0.len() > 1);
        self.0.pop()
    }

    pub fn remove(&mut self, i: LeafIndex) -> L
    {
        debug_assert!( !i.is_root_leaf() );
        self.0.swap_remove(i.index())
    }

    pub fn len(&self) -> usize { self.0.len() }
}


impl<L> Index<LeafIndex> for TrieLeaves<L>
{
    type Output = L;

    fn index(&self, i: LeafIndex) -> &Self::Output
    {
        debug_assert!( i.index() < self.0.len());
        unsafe { self.0.get_unchecked(i.index()) }
    }
}

impl<L> IndexMut<LeafIndex> for TrieLeaves<L>
{
    fn index_mut(&mut self, i: LeafIndex) -> &mut Self::Output
    {
        debug_assert!( i.index() < self.0.len());
        unsafe { self.0.get_unchecked_mut(i.index())}
    }
}



pub static INDEX_MAX:usize = i32::MAX as usize;

/// index of any node (leaf or branching, work as union)
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct NodeIndex(pub(crate) i32);

/// index of a leaf of the trie
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct LeafIndex(i32);

/// index of a branching of the trie
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct BranchingIndex(i32);


impl NodeIndex {
    #[inline] pub(crate) fn root() -> Self { Self(0) }
    #[inline] pub(crate) fn root_leaf() -> Self { Self(!0) }
    #[inline] pub(crate) fn is_root(&self) -> bool { self.0 == 0 }
    #[inline] pub(crate) fn is_branching(&self) -> bool { self.0 >= 0 }
    #[inline] pub(crate) fn is_leaf(&self) -> bool { self.0 < 0 }
    #[inline] pub(crate) fn as_leaf(&self) -> LeafIndex { (*self).into() }
    #[inline] pub(crate) fn as_branching(&self) -> BranchingIndex { (*self).into() }
}

impl LeafIndex {
    #[inline] pub(crate) fn root_leaf() -> Self { Self(!0) }
    #[inline] pub(crate) fn is_root_leaf(&self) -> bool { !self.0 == 0 }
    #[inline] pub(crate) fn index(&self) -> usize { !self.0 as usize }
}

impl BranchingIndex {
    #[inline] pub(crate) fn root() -> Self { Self(0) }
    #[inline] pub(crate) fn is_root(&self) -> bool { self.0 == 0 }
    #[inline] pub(crate) fn index(&self) -> usize { self.0 as usize }
}

impl From<NodeIndex> for LeafIndex
{
    #[inline]
    fn from(i: NodeIndex) -> Self {
        debug_assert!( i.is_leaf() );
        Self(i.0)
    }
}

impl From<usize> for LeafIndex
{
    #[inline]
    fn from(i: usize) -> Self {
        debug_assert!( i <= i );
        Self(!(i as i32))
    }
}

impl From<usize> for BranchingIndex
{
    #[inline]
    fn from(i: usize) -> Self {
        debug_assert!( i <= INDEX_MAX);
        Self(i as i32)
    }
}

impl From<NodeIndex> for BranchingIndex
{
    #[inline]
    fn from(i: NodeIndex) -> Self {
        debug_assert!( i.is_branching() );
        Self(i.0)
    }
}

impl From<LeafIndex> for NodeIndex {
    #[inline] fn from(i: LeafIndex) -> Self { Self(i.0) }
}

impl From<BranchingIndex> for NodeIndex {
    #[inline] fn from(i: BranchingIndex) -> Self { Self(i.0) }
}

impl PartialEq<LeafIndex> for NodeIndex {
    #[inline] fn eq(&self, other: &LeafIndex) -> bool { self.0 == other.0 }
}

impl PartialEq<BranchingIndex> for NodeIndex {
    #[inline] fn eq(&self, other: &BranchingIndex) -> bool { self.0 == other.0 }
}

impl PartialEq<NodeIndex> for BranchingIndex {
    #[inline] fn eq(&self, other: &NodeIndex) -> bool { self.0 == other.0 }
}

impl PartialEq<NodeIndex> for LeafIndex {
    #[inline] fn eq(&self, other: &NodeIndex) -> bool { self.0 == other.0 }
}



impl fmt::Debug for LeafIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}

impl fmt::Debug for BranchingIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}

impl fmt::Debug for NodeIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}


