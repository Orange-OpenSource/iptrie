
use std::mem::size_of;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::fmt::{Debug, Formatter};

use crate::trie::*;
use crate::ip::*;


#[repr(C)]
#[derive(Copy, Clone)]
pub struct Compressed<IP:Ip> {
    pub(crate) shift: u8,
    pub(crate) size: u8,
    pub(crate) mask: u16,
    pub(crate) escape: LeafIndex,
    pub(crate) parent: BranchingIndex,
    phantom: PhantomData<IP>
}


impl<IP:Ip> Debug for Compressed<IP>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Compressed<...> node")?;
        writeln!(f, "  - shift:{}, size:{}, bitmask:{:b}", self.shift, self.size, self.mask)?;
        writeln!(f, "  - escape leaf:{:?}, parent:{:?}", self.escape, self.parent)?;
        (0..self.children()).into_iter()
            .try_for_each(|i| writeln!(f, "   - child[{}]: {:?}", i, self.child(i)))
    }
}

impl<IP:Ip> Compressed<IP> {

    fn new(shift:u8, size:u8, escape: LeafIndex, parent: BranchingIndex) -> Self
    {
        assert!( size <= 16 );
        Self {
            shift,
            size,
            mask: !(!0 << size),
            escape,
            parent,
            phantom: Default::default()
        }
    }

    pub(crate) fn children(&self) -> u16 { 1 << self.size }

    fn letter(&self, slot:&IP) -> u16 {
        let slot : u16 = (*slot >> (size_of::<IP>() as u8 *8-self.clone().shift-self.clone().size)).into();
        self.mask & slot
    }

    fn offset(children: u16) -> usize { children as usize + size_of::<Compressed<IP>>()/size_of::<NodeIndex>() }

    pub(crate) fn child(&self, n:u16) -> &NodeIndex
    {
        debug_assert!( n < self.children() );
        unsafe {
            (self as * const Compressed<IP>).add(1)
                .cast::<NodeIndex>().add(n as usize)
                .as_ref().unwrap()
        }
    }

    pub(crate) fn child_mut(&mut self, n:u16) -> &mut NodeIndex
    {
        debug_assert!( n < self.children() );
        unsafe {
            (self as * mut Compressed<IP>).add(1)
                .cast::<NodeIndex>().add(n as usize)
                .as_mut().unwrap()
        }
    }

    #[inline]
    pub(crate) fn lookup(&self, slot: &IP) -> &NodeIndex {
        self.child(self.letter(slot))
    }
}


pub(crate) struct CompressedTree<IP:Ip> {
    memzone: Vec<NodeIndex>,
    ip: PhantomData<IP>
}

impl<IP:Ip> CompressedTree<IP> {

    pub fn with_capacity(n: usize) -> Self
    {
        let mut memzone = Vec::new();
        // todo: (n+1) ou n ?? ou autre chose ? comment est-ce calculé ?
        memzone.resize((n+1) * (2 * size_of::<Compressed<IP>>() / size_of::<NodeIndex>()), NodeIndex::root());
        unsafe { memzone.set_len(0) };
        Self {
            memzone,
            ip: Default::default()
        }
    }

    pub fn push(&mut self, parent: BranchingIndex, escape: LeafIndex, shift:u8, size:u8) -> BranchingIndex
    {
        assert!( self.memzone.capacity() >= self.memzone.len() + Compressed::<IP>::offset(1<<size));

        let index = self.memzone.len().into();
        unsafe { self.memzone.set_len( self.memzone.len() + Compressed::<IP>::offset(1<<size)); }

        self[index] = Compressed::new(shift, size, escape, parent);
        (0..self[index].children()).into_iter()
            .for_each(|i| *self[index].child_mut(i) = self[index].escape.into());
        index
    }


    pub(crate) fn iter<'a>(&'a self) -> BranchingIterator<'a, IP>
    {
        BranchingIterator {
            curs: 0,
            tree: self
        }
    }

}

impl<IP:Ip> Index<BranchingIndex> for CompressedTree<IP>
{
    type Output = Compressed<IP>;

    fn index(&self, i: BranchingIndex) -> &Self::Output {
        debug_assert!(i.index() < self.memzone.len());
        let branching = unsafe {
            (self.memzone.as_ptr().add(i.index()) as *const Compressed<IP>).as_ref().unwrap()
        };
        debug_assert!( branching.size <= 16);
        debug_assert_eq!(branching.mask, !(!0u16 << branching.size)); // to check misalign
        branching
    }
}

impl<IP:Ip> IndexMut<BranchingIndex> for CompressedTree<IP>
{
    fn index_mut(&mut self, i: BranchingIndex) -> &mut Self::Output {
        debug_assert!(i.index() < self.memzone.len());
        let branching = unsafe {
            (self.memzone.as_ptr().add(i.index()) as * mut Compressed<IP>).as_mut().unwrap()
        };
        debug_assert!( branching.size <= 16);
        debug_assert_eq!(branching.mask, !(!0u16 << branching.size)); // to check misalign
        branching
    }
}


pub(crate) struct BranchingIterator<'a, IP:Ip> {
    curs: usize,
    tree: &'a CompressedTree<IP>
}

impl<'a, IP:Ip> Iterator for BranchingIterator<'a,IP>
{
    type Item = (BranchingIndex,&'a Compressed<IP>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.curs < self.tree.memzone.len() {
            let n: BranchingIndex = self.curs.clone().into();
            let node = &self.tree[n];
            self.curs += Compressed::<IP>::offset(node.children());
            Some((n,node))
        } else {
            None
        }
    }
}