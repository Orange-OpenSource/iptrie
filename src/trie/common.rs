use std::fmt;
use std::ops::{Index, IndexMut};
use crate::prefix::IpPrefix;

#[derive(Clone)]
pub(crate) struct TrieLeaves<L>(pub(crate) Vec<L>);


#[derive(Clone)]
pub(crate) struct Leaf<K,V> {
    prefix: K,
    value: V
}

impl<K,V> Leaf<K,V> {
    pub fn new(k:K, v:V) -> Self { Self { prefix: k, value: v}}
    pub fn prefix(&self) -> &K { &self.prefix }
    pub fn get(&self) -> (&K,&V) { (&self.prefix, &self.value) }
    pub fn get_mut(&mut self) -> (&K,&mut V) { (&self.prefix, &mut self.value) }
}

impl<K,V> Into<(K,V)> for Leaf<K,V> {
    fn into(self) -> (K, V) { (self.prefix, self.value) }
}


impl<K: IpPrefix, V> TrieLeaves<Leaf<K, V>>
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

    #[allow(dead_code)]
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

#[allow(dead_code)]
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
        debug_assert!( i <= INDEX_MAX );
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


