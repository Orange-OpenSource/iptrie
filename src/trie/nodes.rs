use std::fmt;

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


