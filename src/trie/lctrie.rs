use std::fmt;
use std::fmt::Display;
use std::mem::size_of;
use std::ops::{Index, IndexMut};

use super::common::BitSlot;
use super::*;
use super::patricia::*;

#[cfg(feature= "graphviz")] use std::io;
use crate::{bitmask, bitslot_trunc, covers};

pub(crate) struct LCTrie<K:BitPrefix, V> {
    branching: CompressedTree,
    leaves: TrieLeaves<Leaf<K,V>>
}

impl<K:BitPrefix,V>  LCTrie<K,V> {

    pub(crate) fn new(trie: RadixTrie<K,V>) -> Self
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

    fn skip_redundant_parent(&mut self, b:BranchingIndex, esc: LeafIndex, up: BranchingIndex)
    {
        (0..self[b].children())
            .into_iter()
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
        let shift : u8 = tree[b].bit.into();
        let current = self.branching.push(parent, tree[b].escape,  shift - 1, compression + 1);
        done[b.index()] = current.into();
        let bb = &mut self[current];
        (0..bb.children()).into_iter()
            .for_each(|i| self.compute_compressed_child(&tree, current, i, 1, b, b, done, comp));
        current
    }

    fn compute_compressed_child(&mut self,
                                tree: &BranchingTree,
                                current : BranchingIndex, // the compressed node index (in the LC-trie)
                                currchild: u16, // the current child index to compute (relative to the compressed node)
                                depth: u8, // the current depth of the analysis
                                start: BranchingIndex, // the start point of the analysis (in the radix trie)
                                mut b: BranchingIndex, // the current point of the analysis (in the radix trie)
                                done: &mut Vec<Option<BranchingIndex>>, // the already known nodes (branching in radix trie => compressed in LC-trie)
                                comp: u8) // the compression level: 0=>1bit (no compression), N=>N+1 bits
    //-> NodeIndex
    {
        debug_assert_eq!( tree[start].escape , tree[b].escape );

        let c = &self[current];
        let thechild = if currchild & (1 << (c.size - depth)) == 0 { tree[b].child[0]} else { tree[b].child[1] };
        if thechild.is_leaf() {
            let mut thechild = LeafIndex::from(thechild);
            // il faut tester si le prefixe de la feuille est correct sinon ce sera escape
            // on ne teste que sur les bits identifiant le fils (les autres sont ok)
            // mais attention, il se peut qu'il y ait une «pile» d'escape a tester
            let shft = K::Slot::LEN - c.shift - c.size;
            let mut mattch = (self[thechild].bitslot() >> shft).as_u16() & c.mask;
            let mut child = (bitmask(&self[thechild]) >> shft).as_u16() & currchild;
            while mattch != child {
                thechild = tree[b].escape;
                mattch = (self[thechild].bitslot() >> shft).as_u16() & c.mask;
                child = (bitmask(&self[thechild]) >> shft).as_u16() & child;
                b = tree[b].parent;
            }
            *self[current].child_mut(currchild) = thechild.into();
        } else {
            let thechild = BranchingIndex::from(thechild);
            if let Some(n) = done[thechild.index()] {
                // cas on tombe sur un noeud de branchement deja compresse...
                *self[current].child_mut(currchild) = n.into();
            } else {
                let mut depth = tree[thechild].bit.into();
                depth -= c.shift;
                if depth > c.size {
                    // ce fils est au dela du niveau de compression en cours...
                    // on passe donc a un nouveau noeud de branchement compresse
                    *self[current].child_mut(currchild)  = self.compress(tree, thechild.into(), current, done, comp).into();
                } else {
                    //assert (start.escape == trie.branching[thechild].escape);
                    self.compute_compressed_child(tree, current, currchild, depth, start, thechild, done, comp);
                }
            }
        }
    }

    #[inline]
    pub fn get<P: BitPrefix<Slot=K::Slot>>(&self, k: &P) -> Option<&V>
    {
        let mut b = BranchingIndex::root();
        let mut l = LeafIndex::root_leaf();
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
        if bitslot_trunc(&leaf.prefix) == bitslot_trunc(k) && leaf.prefix.len() == k.len() {
            Some(&leaf.value)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut<P: BitPrefix<Slot=K::Slot>>(&mut self, k: &P) -> Option<&mut V>
    {
        let mut b = BranchingIndex::root();
        let mut l = LeafIndex::root_leaf();
        loop {
            match self[b].lookup(&k.bitslot()) {
                n if n.is_branching() => b = (*n).into(),
                n => { // leaf
                    l = (*n).into();
                    break;
                }
            }
        }        let leaf = &mut self.leaves[l];
        if bitslot_trunc(&leaf.prefix) == bitslot_trunc(k) && leaf.prefix.len() == k.len() {
            Some(&mut leaf.value)
        } else {
            None
        }
    }

    #[inline]
    pub fn lookup<Q: BitPrefix<Slot=K::Slot>>(&self, k: &Q) -> (&K, &V)
    {
        let l = self.inner_lookup(k);
        let result = &self.leaves[l];
        return (&result.prefix, &result.value)
    }

    #[inline]
    pub fn lookup_mut<Q: BitPrefix<Slot=K::Slot>>(&mut self, k: &Q) -> (&K, &mut V)
    {
        let l = self.inner_lookup(k);
        let result = &mut self.leaves[l];
        return (&result.prefix, &mut result.value)
    }

    #[inline]
    fn inner_lookup<Q: BitPrefix<Slot=K::Slot>>(&self, k: &Q) -> LeafIndex
    {
        let mut b = BranchingIndex::root();
        let mut l = LeafIndex::root_leaf();
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
            if covers(&self[l], k) {
                return l;
            }
            l = bb.escape;
        }
        while !covers(&self[l],k) {
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



impl<K:BitPrefix, V> Index<LeafIndex> for LCTrie<K,V>
{
    type Output = K;
    #[inline]
    fn index(&self, i: LeafIndex) -> &Self::Output { &self.leaves[i].prefix }
}

impl<K:BitPrefix, V> IndexMut<LeafIndex> for LCTrie<K,V>
{
    #[inline]
    fn index_mut(&mut self, i: LeafIndex) -> &mut Self::Output { &mut self.leaves[i].prefix }
}



impl<K:BitPrefix, V> Index<BranchingIndex> for LCTrie<K,V>
{
    type Output = Compressed;
    #[inline]
    fn index(&self, i: BranchingIndex) -> &Self::Output { &self.branching[i] }
}

impl<K:BitPrefix, V> IndexMut<BranchingIndex> for LCTrie<K,V>
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
        (0..self.children()).into_iter()
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
        let slot : u16 = (*slot >> (B::LEN-self.clone().shift-self.clone().size)).as_u16();
        self.mask & slot
    }

    fn offset(children: u16) -> usize { children as usize + size_of::<Compressed>()/size_of::<NodeIndex>() }

    pub(crate) fn child(&self, n:u16) -> &NodeIndex
    {
        debug_assert!( n < self.children() );
        unsafe {
            (self as * const Compressed).add(1)
                .cast::<NodeIndex>().add(n as usize)
                .as_ref().unwrap()
        }
    }

    pub(crate) fn child_mut(&mut self, n:u16) -> &mut NodeIndex
    {
        debug_assert!( n < self.children() );
        unsafe {
            (self as * mut Compressed).add(1)
                .cast::<NodeIndex>().add(n as usize)
                .as_mut().unwrap()
        }
    }

    #[inline]
    pub(crate) fn lookup<B:BitSlot>(&self, slot: &B) -> &NodeIndex {
        self.child(self.letter(slot))
    }
}


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
        (0..self[index].children()).into_iter()
            .for_each(|i| *self[index].child_mut(i) = self[index].escape.into());
        index
    }


    pub(crate) fn iter<'a>(&'a self) -> BranchingIterator<'a>
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
            let n: BranchingIndex = self.curs.clone().into();
            let node = &self.tree[n];
            self.curs += Compressed::offset(node.children());
            Some((n,node))
        } else {
            None
        }
    }
}


#[cfg(feature= "graphviz")]
impl<K:BitPrefix, V> crate::trie::graphviz::DotWriter for LCTrie<K,V>
    where K:Display
{
    fn write_dot(&self, dot: &mut dyn io::Write) -> io::Result<()>
    {
        use lux::bits::BitVec;

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
                let mut done = BitVec::new();
                (0..b.children()).into_iter()
                    .try_for_each(|c|
                        if !done[c as u32] && *b.child(c) != b.escape {
                            let group = ((c+1)..b.children()).into_iter()
                                .filter(|cc| b.child(c) == b.child(*cc))
                                .fold(BitVec::singleton(c as u32), |mut group, cc| { group.set(cc as u32); group } );
                            done |= &group;
                            if b.child(c).is_leaf() {
                                writeln!(dot, "{0:?} [label=\"[{0:?}] {1}\"]", b.child(c), self[LeafIndex::from(*b.child(c))])?;
                            }
                            writeln!(dot, "{0:?} -> {1:?} [fontcolor={2},color={2},label=\"{3}\"]", i, b.child(c), 1+(c%8), group)
                        } else { Ok(()) })
            })?;

        writeln!(dot,"}}")?;
        dot.flush()
    }
}