
mod branching;
mod bits;

use std::ops::{Index, IndexMut};
use std::io;
use std::io::Write;

use crate::trie::*;
use crate::ip::*;
pub(crate) use bits::*;
pub(crate) use branching::*;
use std::marker::PhantomData;
use crate::lctrie::LCTrie;


pub(crate) struct RadixTrie<IP:Ip, K:IpPrefix<IP>, V>
{
    pub(crate) branching: BranchingTree<IP,BitIndex<IP>>,
    pub(crate) leaves: TrieLeaves<Leaf<K,V>>,
    phantom: PhantomData<IP>
}

impl<IP:Ip, K:IpPrefix<IP>, V> RadixTrie<IP,K,V>
{
    pub(crate) fn new(value: V, capacity: usize) -> Self
    {
        Self {
            branching: BranchingTree::new(capacity.clone() / 2),
            leaves: TrieLeaves::new(capacity, value),
            phantom: Default::default()
        }
    }

    #[inline]
    pub(crate) fn compile(self) -> LCTrie<IP,K,V> {
        LCTrie::new(self)
    }


    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    {
        let addedleaf = self.leaves.push(Leaf::new(k, v));
        let addedpfx = self[addedleaf].clone();

        let (deepestbranching, deepestleaf) = self.branching.search_deepest_candidate(&addedpfx.slot());
        let mut l = deepestleaf;
        let mut b = deepestbranching;
        if l != self[b].escape {
            if !addedpfx.matched(&self[l]) {
                l = self[b].escape;
            }
        }
        while !addedpfx.matched(&self[l]) {
            assert!(!l.is_root_leaf());
            b = self[b].parent;
            l = self[b].escape;
        }
        if self[l] == addedpfx {
            self.leaves.remove_last().map(|l| l.value)
        } else {
            self.branching.insert_prefix(addedleaf, &addedpfx.slot(), addedpfx.len(),
                                         deepestbranching, deepestleaf,
                                         &self[deepestleaf].slot(), self[deepestleaf].len());
            None
        }
    }

    #[inline]
    pub fn get<P: IpPrefix<IP>>(&self, k: &P) -> Option<&V>
    {
        let (_, l) = self.branching.search_deepest_candidate(&k.slot());
        let leaf = &self.leaves[l];
        if leaf.prefix.slot() == k.slot() && leaf.prefix.len() == k.len() {
            Some(&leaf.value)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut<P: IpPrefix<IP>>(&mut self, k: &P) -> Option<&mut V>
    {
        let (_, l) = self.branching.search_deepest_candidate(&k.slot());
        let leaf = &mut self.leaves[l];
        if leaf.prefix.slot() == k.slot() && leaf.prefix.len() == k.len() {
            Some(&mut leaf.value)
        } else {
            None
        }
    }

    #[inline]
    pub fn remove<P: IpPrefix<IP>>(&mut self, k: &P) -> Option<V>
    {
        todo!()
    }

    #[inline]
    fn inner_lookup<Q: IpPrefixMatch<IP>>(&self, k: &Q) -> LeafIndex
    {
        let (mut n, mut l) = self.branching.search_deepest_candidate(&k.slot());

        if l != self[n].escape {
            if k.matched(&self[l]) { return l; }
            l = self[n].escape;
        }
        while !k.matched(&self[l]) {
            debug_assert!( !l.is_root_leaf() );
            n = self[n].parent;
            l = self[n].escape;
        }
        l
    }

    #[inline]
    pub fn lookup<Q: IpPrefixMatch<IP>>(&self, k: &Q) -> (&K, &V)
    {
        let l = self.inner_lookup(k);
        let result = &self.leaves[l];
        return (&result.prefix, &result.value)
    }

    #[inline]
    pub fn lookup_mut<Q: IpPrefixMatch<IP>>(&mut self, k: &Q) -> (&K, &mut V)
    {
        let l = self.inner_lookup(k);
        let result = &mut self.leaves[l];
        return (&result.prefix, &mut result.value)
    }
}

#[cfg(feature= "graphviz")]
impl<IP:Ip, K:IpPrefix<IP>, V> crate::graphviz::DotWriter for RadixTrie<IP,K,V>
{
    fn write_dot(&self, dot: &mut dyn Write) -> io::Result<()>
    {
        writeln!(dot, "digraph patricia {{")?;
        writeln!(dot, "    rankdir=LR")?;

        // writing branching nodes
        writeln!(dot, "node[shape=box]")?;
        self.branching.0.iter()
            .enumerate()
            .try_for_each(|(i,b)|
                writeln!(dot, "{0} [label=\"[{0}] bit={1}\n[{2:?}] {3}\"]", i, b.bit, b.escape, self[b.escape])
            )?;
        // display the relevant leaves (i.e. not escaped)
        writeln!(dot, "node[shape=none]")?;
        self.branching.0.iter()
            .try_for_each(|b|
                b.child.iter()
                    .filter(|&c| *c != b.escape && c.is_leaf())
                    .try_for_each(|c|
                        writeln!(dot, "{0:?} [label=\"[{0:?}] {1}\"]", c, self[c.as_leaf()])
                    ))?;

        writeln!(dot, "edge[headport=w,colorscheme=dark28]")?;
        self.branching.0.iter()
            .enumerate()
            .try_for_each(|(i, b)|
                b.child.iter()
                    .enumerate()
                    .filter(|(_, &c)| c != b.escape) // avoid redundant link
                    .try_for_each(|(j, _)|
                        writeln!(dot, "{0} -> {1:?} [fontcolor={2},color={2},label={3}]", i, b.child[j], j+1, j)
                    )
            )?;

        writeln!(dot,"}}")?;
        dot.flush()
    }
}


/*
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
struct BitIndex(u8);

impl BitIndex {
    fn get<T:OneBitMatch>(&self, addr:&T) -> bool {
        addr.is_set(self.0)
    }
}


pub(super) trait OneBitMatch {
    fn is_set(&self, bit: u8) -> bool;
}


macro_rules! one_bit_match {
    ($X:ty) => {
        impl OneBitMatch for $X {
            #[inline] fn is_set(&self, bit: u8) -> bool { (self >> bit) & 1 != 0 }
        }
    };
}

one_bit_match!(u32);
one_bit_match!(u64);
one_bit_match!(u128);

*/

impl<IP:Ip,K:IpPrefix<IP>,V> Index<BranchingIndex> for RadixTrie<IP,K,V>
{
    type Output = Branching<IP,BitIndex<IP>>;
    #[inline]
    fn index(&self, i: BranchingIndex) -> &Self::Output { &self.branching[i] }
}

impl<IP:Ip,K:IpPrefix<IP>,V> IndexMut<BranchingIndex> for RadixTrie<IP,K,V>
{
    #[inline]
    fn index_mut(&mut self, i: BranchingIndex) -> &mut Self::Output { &mut self.branching[i] }
}

impl<IP:Ip,K:IpPrefix<IP>,V> Index<LeafIndex> for RadixTrie<IP,K,V>
{
    type Output = K;
    #[inline]
    fn index(&self, i: LeafIndex) -> &Self::Output { &self.leaves[i].prefix }
}

impl<IP:Ip,K:IpPrefix<IP>,V> IndexMut<LeafIndex> for RadixTrie<IP,K,V>
{
    #[inline]
    fn index_mut(&mut self, i: LeafIndex) -> &mut Self::Output { &mut self.leaves[i].prefix }
}