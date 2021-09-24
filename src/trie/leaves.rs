
use crate::trie::LeafIndex;
use std::ops::{Index, IndexMut};

#[derive(Clone)]
pub(crate) struct TrieLeaves<L>(pub(crate) Vec<L>);


pub(crate) struct Leaf<K,V> {
    pub(crate) prefix: K,
    pub(crate) value: V
}

impl<K,V> Leaf<K,V> {
    pub(crate) fn new(k:K, v:V) -> Self { Self { prefix: k, value: v}}
}


impl<K:Default, V> TrieLeaves<Leaf<K, V>>
{
    pub fn new(capacity: usize, value: V) -> Self {
        let mut leaves = Vec::with_capacity(capacity);
        leaves.push(Leaf { prefix: K::default(), value });
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
