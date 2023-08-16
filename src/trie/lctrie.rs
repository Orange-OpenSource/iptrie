use std::fmt;
use std::mem::size_of;
use std::num::NonZeroUsize;
use std::ops::{Index, IndexMut};

use super::patricia::*;

use crate::prefix::*;

pub(crate) struct LevelCompressedTrie<K,V> {
    branching: CompressedTree,
    pub(crate) leaves: TrieLeaves<Leaf<K,V>>
}

impl<K,V> LevelCompressedTrie<K,V>
{
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> NonZeroUsize {
        unsafe {
            NonZeroUsize::new_unchecked(self.leaves.len())
        }
    }
}

impl<K:IpPrefix,V>  LevelCompressedTrie<K,V> {
    pub(crate) fn new(trie: RadixTrie<K, V>) -> Self
    {
        let mut lctrie = Self {
            branching: CompressedTree::with_capacity(trie.branching.0.len()),
            leaves: trie.leaves
        };
        // compiling...
        let comp = 0;
        let trie = trie.branching;
        let mut done = Vec::<Option<BranchingIndex>>::new();
        done.resize(trie.0.len(), None);

        lctrie.compress(&trie, BranchingIndex::root(), BranchingIndex::root(), &mut done, comp);
        lctrie.skip_redundant_parent(BranchingIndex::root(), LeafIndex::root_leaf(), BranchingIndex::root());
        lctrie
    }


    pub fn map<W, F: FnMut(&V) -> W>(&self, mut f: F) -> LevelCompressedTrie<K, W>
    {
        LevelCompressedTrie {
            branching: self.branching.clone(),
            leaves: TrieLeaves(
                self.leaves.0.iter()
                    .map(|leaf| Leaf::new(leaf.prefix, f(&leaf.value)))
                    .collect()
            )
        }
    }

    fn skip_redundant_parent(&mut self, b: BranchingIndex, esc: LeafIndex, up: BranchingIndex)
    {
        (0..self[b].children())
            .for_each(|i| {
                if self[b].child(i).is_branching() {
                    let bb = BranchingIndex::from(*self[b].child(i));
                    if self[bb].escape == esc {
                        self[bb].parent = up;
                        self.skip_redundant_parent(bb, esc, up);
                    } else {
                        self.skip_redundant_parent(bb, self[bb].escape, self[bb].parent);
                    }
                }
            });
    }

    // compress the node b as child of parent
    fn compress(&mut self,
                tree: &BranchingTree,
                b: BranchingIndex, parent: BranchingIndex,
                done: &mut Vec<Option<BranchingIndex>>, // the already known nodes (branching in radix trie => compressed in LC-trie)
                comp: u8)
                -> BranchingIndex
    {
        let compression = tree.compression_level(&tree[b], comp);
        let shift: u8 = tree[b].bit;
        let current = self.branching.push(parent, tree[b].escape, shift - 1, compression + 1);
        done[b.index()] = current.into();
        let bb = &mut self[current];
        (0..bb.children()).for_each(|i| self.compute_compressed_child(tree, current, i, 1, b, b, done, comp));
        current
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_compressed_child(&mut self,
                                tree: &BranchingTree,
                                current: BranchingIndex, // the compressed node index (in the LC-trie)
                                currchild: u16, // the current child index to compute (relative to the compressed node)
                                depth: u8, // the current depth of the analysis
                                start: BranchingIndex, // the start point of the analysis (in the radix trie)
                                mut b: BranchingIndex, // the current point of the analysis (in the radix trie)
                                done: &mut Vec<Option<BranchingIndex>>, // the already known nodes (branching in radix trie => compressed in LC-trie)
                                comp: u8) // the compression level: 0=>1bit (no compression), N=>N+1 bits
    //-> NodeIndex
    {
        debug_assert_eq!(tree[start].escape, tree[b].escape);

        let c = &self[current];
        let thechild = if currchild & (1 << (c.size - depth)) == 0 { tree[b].child[0] } else { tree[b].child[1] };
        if thechild.is_leaf() {
            let mut thechild = LeafIndex::from(thechild);
            // il faut tester si le prefixe de la feuille est correct sinon ce sera escape
            // on ne teste que sur les bits identifiant le fils (les autres sont ok)
            // mais attention, il se peut qu'il y ait une «pile» d'escape a tester
            let shft = K::Slot::LEN - c.shift - c.size;
            let mut matching = (self[thechild].bitslot() >> shft).last_16_bits() & c.mask;
            let mut child = (self[thechild].bitmask() >> shft).last_16_bits() & currchild;
            while matching != child {
                thechild = tree[b].escape;
                matching = (self[thechild].bitslot() >> shft).last_16_bits() & c.mask;
                child &= (self[thechild].bitmask() >> shft).last_16_bits();
                b = tree[b].parent;
            }
            *self[current].child_mut(currchild) = thechild.into();
        } else {
            let thechild = BranchingIndex::from(thechild);
            if let Some(n) = done[thechild.index()] {
                // cas on tombe sur un noeud de branchement deja compresse...
                *self[current].child_mut(currchild) = n.into();
            } else {
                let mut depth = tree[thechild].bit;
                depth -= c.shift;
                if depth > c.size {
                    // ce fils est au dela du niveau de compression en cours...
                    // on passe donc a un nouveau noeud de branchement compresse
                    *self[current].child_mut(currchild) = self.compress(tree, thechild, current, done, comp).into();
                } else {
                    //assert (start.escape == trie.branching[thechild].escape);
                    self.compute_compressed_child(tree, current, currchild, depth, start, thechild, done, comp);
                }
            }
        }
    }
}

impl<K:IpPrefix,V> LevelCompressedTrie<K,V>
{
    #[inline]
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q> + PartialEq<Q>
    {
        let mut b = BranchingIndex::root();
        let l: LeafIndex; // = LeafIndex::root_leaf();
        loop {
            match self[b].lookup(&k.bitslot()) {
                n if n.is_branching() => b = (*n).into(),
                n => { // leaf
                    l = (*n).into();
                    break;
                }
            }
        }
        let leaf = &self.leaves[l];
        if leaf.prefix == *k {
            Some(&leaf.value)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q> + PartialEq<Q>
    {
        let mut b = BranchingIndex::root();
        let l : LeafIndex; // = LeafIndex::root_leaf();
        loop {
            match self[b].lookup(&k.bitslot()) {
                n if n.is_branching() => b = (*n).into(),
                n => { // leaf
                    l = (*n).into();
                    break;
                }
            }
        }        let leaf = &mut self.leaves[l];
        if leaf.prefix == *k {
            Some(&mut leaf.value)
        } else {
            None
        }
    }

    #[inline]
    pub fn lookup<Q>(&self, k: &Q) -> (&K, &V)
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    {
        let l = self.inner_lookup(k);
        let result = &self.leaves[l];
        (&result.prefix, &result.value)
    }

    #[inline]
    pub fn lookup_mut<Q>(&mut self, k: &Q) -> (&K, &mut V)
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    {
        let l = self.inner_lookup(k);
        let result = &mut self.leaves[l];
        (&result.prefix, &mut result.value)
    }

    #[inline]
    fn inner_lookup<Q>(&self, k: &Q) -> LeafIndex
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    {
        let mut b = BranchingIndex::root();
        let mut l : LeafIndex; // = LeafIndex::root_leaf();
        loop {
            match self[b].lookup(&k.bitslot()) {
                n if n.is_branching() => b = (*n).into(),
                n => { // leaf
                    l = (*n).into();
                    break;
                }
            }
        }
        let mut bb = &self[b];
        if l != bb.escape {
            if self[l].covers(k) {
                return l;
            }
            l = bb.escape;
        }
        while !self[l].covers(k)  {
            b = bb.parent;
            bb = &self[b];
            l = bb.escape;
        }
        l
    }

    pub fn info(&self)
    {
        let mut counts = [0;128];
        self.branching.iter()
            .for_each(|(_,c)| counts[c.size as usize] += 1 );

        println!("{} branching, {} leaves", self.branching.iter().count(), self.leaves.len());
        println!("root: {} children (2^{}), {} shift", self.branching[0.into()].children(), self.branching[0.into()].size, self.branching[0.into()].shift);

        let mut counts = [0;128];
        self.branching.iter()
            .for_each(|(_,c)| counts[c.size as usize] += 1 );
        println!("size: {:?}", counts);

        let mut counts = [0;128];
        self.branching.iter()
            .for_each(|(_,c)| counts[c.shift as usize] += 1 );
        println!("shift: {:?}", counts);

        let mut counts = [0;128];
        self.branching.iter()
            .skip(1)
            .for_each(|(_,c)| {
                let p = self.branching[c.parent];
                counts[(c.shift - p.shift - p.size) as usize] += 1
            } );
        println!("shift: {:?}", counts);

        println!();
    }
}



impl<K: IpPrefix, V> Index<LeafIndex> for LevelCompressedTrie<K,V>
{
    type Output = K;
    #[inline]
    fn index(&self, i: LeafIndex) -> &Self::Output { &self.leaves[i].prefix }
}

impl<K: IpPrefix, V> IndexMut<LeafIndex> for LevelCompressedTrie<K,V>
{
    #[inline]
    fn index_mut(&mut self, i: LeafIndex) -> &mut Self::Output { &mut self.leaves[i].prefix }
}



impl<K: IpPrefix, V> Index<BranchingIndex> for LevelCompressedTrie<K,V>
{
    type Output = Compressed;
    #[inline]
    fn index(&self, i: BranchingIndex) -> &Self::Output { &self.branching[i] }
}

impl<K: IpPrefix, V> IndexMut<BranchingIndex> for LevelCompressedTrie<K,V>
{
    #[inline]
    fn index_mut(&mut self, i: BranchingIndex) -> &mut Self::Output { &mut self.branching[i] }
}


#[repr(C)]
#[derive(Copy, Clone)]
pub struct Compressed {
    pub(crate) shift: u8,
    pub(crate) size: u8,
    pub(crate) mask: u16,
    pub(crate) escape: LeafIndex,
    pub(crate) parent: BranchingIndex
}


impl fmt::Debug for Compressed
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Compressed<...> node")?;
        writeln!(f, "  - shift:{}, size:{}, bitmask:{:b}", self.shift, self.size, self.mask)?;
        writeln!(f, "  - escape leaf:{:?}, parent:{:?}", self.escape, self.parent)?;
        (0..self.children())
            .try_for_each(|i| writeln!(f, "   - child[{}]: {:?}", i, self.child(i)))
    }
}

impl Compressed {

    fn new(shift:u8, size:u8, escape: LeafIndex, parent: BranchingIndex) -> Self
    {
        assert!( size <= 16 );
        Self {
            shift,
            size,
            mask: !(!0 << size),
            escape,
            parent
        }
    }

    pub(crate) fn children(&self) -> u16 { 1 << self.size }

    fn letter<B:BitSlot>(&self, slot:&B) -> u16 {
        let slot : u16 = (*slot >> (B::LEN-self.shift-self.size)).last_16_bits();
        self.mask & slot
    }

    fn offset(children: u16) -> usize { children as usize + size_of::<Compressed>()/size_of::<NodeIndex>() }

    pub(crate) fn child(&self, n:u16) -> &NodeIndex
    {
        debug_assert!( n < self.children() );
        unsafe {
            &* (self as * const Compressed).add(1)
                .cast::<NodeIndex>().add(n as usize)
        }
    }

    pub(crate) fn child_mut(&mut self, n:u16) -> &mut NodeIndex
    {
        debug_assert!( n < self.children() );
        unsafe {
            &mut *(self as * mut Compressed).add(1)
                .cast::<NodeIndex>().add(n as usize)
        }
    }

    #[inline]
    pub(crate) fn lookup<B:BitSlot>(&self, slot: &B) -> &NodeIndex {
        self.child(self.letter(slot))
    }
}

#[derive(Clone)]
pub(crate) struct CompressedTree {
    memzone: Vec<NodeIndex>
}

impl CompressedTree {

    pub fn with_capacity(n: usize) -> Self
    {
        let mut memzone = Vec::new();
        // todo: (n+1) ou n ?? ou autre chose ? comment est-ce calculé ?
        memzone.resize((n+1) * (2 * size_of::<Compressed>() / size_of::<NodeIndex>()), NodeIndex::root());
        unsafe { memzone.set_len(0) };
        Self { memzone }
    }

    pub fn push(&mut self, parent: BranchingIndex, escape: LeafIndex, shift:u8, size:u8) -> BranchingIndex
    {
        assert!( self.memzone.capacity() >= self.memzone.len() + Compressed::offset(1<<size));

        let index = self.memzone.len().into();
        unsafe { self.memzone.set_len( self.memzone.len() + Compressed::offset(1<<size)); }

        self[index] = Compressed::new(shift, size, escape, parent);
        (0..self[index].children())
            .for_each(|i| *self[index].child_mut(i) = self[index].escape.into());
        index
    }

    pub(crate) fn iter(&self) -> BranchingIterator<'_>
    {
        BranchingIterator {
            curs: 0,
            tree: self
        }
    }
}

impl Index<BranchingIndex> for CompressedTree
{
    type Output = Compressed;

    fn index(&self, i: BranchingIndex) -> &Self::Output {
        debug_assert!(i.index() < self.memzone.len());
        let branching = unsafe {
            (self.memzone.as_ptr().add(i.index()) as *const Compressed).as_ref().unwrap()
        };
        debug_assert!( branching.size <= 16);
        debug_assert_eq!(branching.mask, !(!0u16 << branching.size)); // to check misalign
        branching
    }
}

impl IndexMut<BranchingIndex> for CompressedTree
{
    fn index_mut(&mut self, i: BranchingIndex) -> &mut Self::Output {
        debug_assert!(i.index() < self.memzone.len());
        let branching = unsafe {
            (self.memzone.as_ptr().add(i.index()) as * mut Compressed).as_mut().unwrap()
        };
        debug_assert!( branching.size <= 16);
        debug_assert_eq!(branching.mask, !(!0u16 << branching.size)); // to check misalign
        branching
    }
}


pub(crate) struct BranchingIterator<'a> {
    curs: usize,
    tree: &'a CompressedTree
}

impl<'a> Iterator for BranchingIterator<'a>
{
    type Item = (BranchingIndex,&'a Compressed);

    fn next(&mut self) -> Option<Self::Item> {
        if self.curs < self.tree.memzone.len() {
            let n: BranchingIndex = self.curs.into();
            let node = &self.tree[n];
            self.curs += Compressed::offset(node.children());
            Some((n,node))
        } else {
            None
        }
    }
}


#[cfg(feature= "graphviz")]
impl<K: IpPrefix, V> crate::trie::graphviz::DotWriter for LevelCompressedTrie<K,V>
    where K: std::fmt::Display
{
    fn write_dot(&self, dot: &mut dyn std::io::Write) -> std::io::Result<()>
    {
        use std::collections::BTreeSet;

        writeln!(dot, "digraph lctrie {{")?;
        writeln!(dot, "    rankdir=LR")?;
        writeln!(dot, "    node[shape=box]")?;
        writeln!(dot, "    edge[headport=w,colorscheme=dark28]")?;
        writeln!(dot, "    labelloc=top")?;
        writeln!(dot, "    labeljust=l")?;
        writeln!(dot, "    label=\"BITS LC-TRIE\\l - {} leaves\\l - {} branching nodes\\l\"", self.leaves.len(), self.branching.iter().count())?;

        // writing branching nodes
        writeln!(dot, "node[shape=box]")?;
        self.branching.iter()
            .try_for_each(|(i,b)| {
                if b.size == 1 {
                    writeln!(dot, "{0:?} [label=\"[{0:?}] bit={1}\n[{2:?}] {3}\"]", i, b.shift + 1, b.escape, self[b.escape])
                } else {
                    writeln!(dot, "{0:?} [label=\"[{0:?}] bits=[{1}..{2}]\n[{3:?}] {4}\"]", i, b.shift + 1, b.shift + b.size, b.escape, self[b.escape])
                }
            })?;

        // and the edges...
        writeln!(dot, "node[shape=none]")?;
        self.branching.iter()
            .try_for_each(|(i,b)| {
                let mut done = BTreeSet::<u32>::default();
                (0..b.children())
                    .try_for_each(|c|
                        if !done.contains(&(c as u32)) && *b.child(c) != b.escape {
                            let group = ((c+1)..b.children())
                                .filter(|cc| b.child(c) == b.child(*cc))
                                .fold(BTreeSet::from_iter([c as u32;1]), |mut group, cc| { group.insert(cc as u32); group } );
                            let label = group.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(",");
                            done.extend(group);
                            if b.child(c).is_leaf() {
                                writeln!(dot, "{0:?} [label=\"[{0:?}] {1}\"]", b.child(c), self[LeafIndex::from(*b.child(c))])?;
                            }
                            writeln!(dot, "{0:?} -> {1:?} [fontcolor={2},color={2},label=\"{3}\"]", i, b.child(c), 1+(c%8), label)
                        } else { Ok(()) })
            })?;

        writeln!(dot,"}}")?;
        dot.flush()
    }
}