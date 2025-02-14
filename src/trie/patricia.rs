

#[cfg(feature= "graphviz")] use std::io;
use std::num::NonZeroUsize;
use std::ops::{Index, IndexMut};
use crate::prefix::*;
use super::common::*;

#[derive(Clone)]
pub(crate) struct RadixTrie<K,V>
{
    pub(crate) branching: BranchingTree,
    pub(crate) leaves: TrieLeaves<Leaf<K,V>>
}

impl<K,V> RadixTrie<K,V>
{
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=&Leaf<K,V>> + '_ {
        self.leaves.0.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut Leaf<K,V>> + '_ {
        self.leaves.0.iter_mut()
    }

    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> NonZeroUsize {
        unsafe {
            NonZeroUsize::new_unchecked(self.leaves.len())
        }
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.leaves.0.shrink_to_fit();
        self.branching.0.shrink_to_fit();
    }
}

impl<K:IpRootPrefix,V> RadixTrie<K,V>
{
    pub(crate) fn new(value: V, capacity: usize) -> Self
    {
        Self {
            branching: BranchingTree::new(capacity / 2),
            leaves: TrieLeaves::new(capacity, K::root(), value)
        }
    }
}

impl<K:IpPrefix,V> RadixTrie<K,V>
{
    pub fn map<W, F: FnMut(&V) -> W>(&self, mut f: F) -> RadixTrie<K, W>
    {
        RadixTrie {
            branching: self.branching.clone(),
            leaves: TrieLeaves(
                self.leaves.0.iter()
                    .map(|leaf| Leaf::new(*leaf.prefix(), f(leaf.get().1)))
                    .collect()
            )
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    {
        let addedleaf = self.leaves.push(Leaf::new(k, v));
        let addedpfx = self[addedleaf];

        let (deepestbranching, deepestleaf) = self.branching.search_deepest_candidate(&addedpfx.bitslot());
        let mut l = deepestleaf;
        let mut b = deepestbranching;
        if l != self[b].escape && !self[l].covers(&addedpfx) {
            l = self[b].escape;
        }
        // will stop since the top prefix always matches
        loop {
            match self[l].covering(&addedpfx) {
                IpPrefixCoverage::NoCover => {
                    assert!(!l.is_root_leaf());
                    b = self[b].parent;
                    l = self[b].escape;
                }
                IpPrefixCoverage::WiderRange => {
                    self.branching.insert_prefix(addedleaf, &addedpfx.bitslot(), addedpfx.len(),
                                                 deepestbranching, deepestleaf,
                                                 &self[deepestleaf].bitslot(), self[deepestleaf].len());
                    return None
                }
                IpPrefixCoverage::SameRange => {
                    let leaf = self.leaves.remove_last().unwrap();
                    let mut v = <Leaf<K,V> as Into<(K,V)>>::into(leaf).1;
                    std::mem::swap(&mut v, self.leaves[l].get_mut().1);
                    return Some(v);
                }
            }
        }
    }

    pub fn replace(&mut self, k: K, v: V) -> Option<Leaf<K,V>>
    {
        let addedleaf = self.leaves.push(Leaf::new(k, v));
        let addedpfx = self[addedleaf];

        let (deepestbranching, deepestleaf) = self.branching.search_deepest_candidate(&addedpfx.bitslot());
        let mut l = deepestleaf;
        let mut b = deepestbranching;
        if l != self[b].escape && !self[l].covers(&addedpfx) {
            l = self[b].escape;
        }
        // will stop since the top prefix always matches
        loop {
            match self[l].covering(&addedpfx) {
                IpPrefixCoverage::NoCover => {
                    assert!(!l.is_root_leaf());
                    b = self[b].parent;
                    l = self[b].escape;
                }
                IpPrefixCoverage::WiderRange => {
                    self.branching.insert_prefix(addedleaf, &addedpfx.bitslot(), addedpfx.len(),
                                                 deepestbranching, deepestleaf,
                                                 &self[deepestleaf].bitslot(), self[deepestleaf].len());
                    return None;
                }
                IpPrefixCoverage::SameRange => {
                    let mut v = self.leaves.remove_last().unwrap();
                    std::mem::swap(&mut v, &mut self.leaves[l]);
                    return Some(v);
                }
            }
        }
    }
}

impl<K:IpPrefix,V> RadixTrie<K,V>
{
    pub fn get<Q>(&self, k: &Q) -> Option<(&K,&V)>
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    {
        let (_,l) = self.inner_lookup(k);
        if k.len() == self[l].len() { Some(self.leaves[l].get()) } else { None }
    }

    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<(&K,&mut V)>
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    {
        let (_,l) = self.inner_lookup(k);
        if k.len() == self[l].len() { Some(self.leaves[l].get_mut()) } else { None }
    }

    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q> + IpPrefixCovering<K>
    {
        let (mut b,l) = self.inner_lookup(k);
        if k.len() != self[l].len() {
            None
        } else {
            if l == self[b].escape {
                if l == LeafIndex::root_leaf() {
                    panic!("can’t remove root prefix");
                }
                // the node to suppress is an escape node
                // so we should climb to its first appearance
                while self[self[b].parent].escape == l {
                    b = self[b].parent;
                }
                // and we propagate the removal (i.e. the escape change)
                self.branching.replace_escape_leaf(b, l, self[self[b].parent].escape);
            } else {
                // we suppress a leaf of the tree... so easy... (redirect to escape)
                *self[b].child_mut(&k.bitslot()) = self[b].escape.into();
            }

            // todo: some branching possibly becomes useless and should be removed here

            // reindex the leaf which will be swapped with the removed one
            let lastleaf = LeafIndex::from(self.leaves.len()-1);
            let (mut bb,_ll) = self.inner_lookup(&self[lastleaf]);
            debug_assert_eq!( self[lastleaf].len(), self[_ll].len() );
            if self[bb].child[0] == lastleaf { self[bb].child[0] = l.into(); }
            if self[bb].child[1] == lastleaf { self[bb].child[1] = l.into(); }
            while self[bb].escape == lastleaf {
                self[bb].escape = l;
                bb = self[bb].parent; // climb up the escape chain
            }
            // effective removal of the leaf
            let removed = self.leaves.0.swap_remove(l.index());
            Some(<Leaf<K,V> as Into<(K,V)>>::into(removed).1)
        }
    }

    #[inline]
    fn inner_lookup<Q>(&self, k: &Q) -> (BranchingIndex, LeafIndex)
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    {
        let (mut n, mut l) = self.branching.search_deepest_candidate(&k.bitslot_trunc());

        if l != self[n].escape {
            if self[l].covers(k) { return (n,l); }
            l = self[n].escape;
        }
        while !self[l].covers(k) {
            debug_assert!( !l.is_root_leaf() );
            n = self[n].parent;
            l = self[n].escape;
        }
        (n,l)
    }


    #[inline]
    pub fn lookup<Q>(&self, k: &Q) -> (&K, &V)
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    {
        let (_,l) = self.inner_lookup(k);
        self.leaves[l].get()
    }

    #[inline]
    pub fn lookup_mut<Q>(&mut self, k: &Q) -> (&K, &mut V)
        where
            Q: IpPrefix<Addr=K::Addr>,
            K: IpPrefixCovering<Q>
    {
        let (_,l) = self.inner_lookup(k);
        self.leaves[l].get_mut()
    }

    pub fn info(&self)
    {
        println!("PATRICIA TRIE info");
        println!("{} branching, {} leaves", self.branching.0.len(), self.leaves.len());

        let branching =    self.branching.0.len() * std::mem::size_of::<Branching>()/1000;
        let leaves = self.leaves.len() * std::mem::size_of::<Leaf<K,V>>()/1000;
        println!("memory: {:?}k + {:?}k = {:?}k", branching, leaves, branching+leaves);

        println!();
    }
}


#[cfg(feature= "graphviz")]
impl<K:std::fmt::Display, V> crate::graphviz::DotWriter for RadixTrie<K,V>
{
    fn write_dot(&self, dot: &mut dyn io::Write) -> io::Result<()>
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
                    .filter(|&&c| c.is_leaf())
                    .filter(|&&c| c != b.escape) // avoid redundant link
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


impl<K,V> Index<BranchingIndex> for RadixTrie<K,V>
{
    type Output = Branching;
    #[inline]
    fn index(&self, i: BranchingIndex) -> &Self::Output { &self.branching[i] }
}

impl<K,V> IndexMut<BranchingIndex> for RadixTrie<K,V>
{
    #[inline]
    fn index_mut(&mut self, i: BranchingIndex) -> &mut Self::Output { &mut self.branching[i] }
}

impl<K,V> Index<LeafIndex> for RadixTrie<K,V>
{
    type Output = K;
    #[inline]
    fn index(&self, i: LeafIndex) -> &Self::Output { self.leaves[i].prefix() }
}


#[derive(Debug, Copy, Clone)]
pub(crate) struct Branching {
    pub(crate) escape: LeafIndex, // leaf associated to this branching node
    pub(crate) parent: BranchingIndex, // to climb up the trie (>=0)
    pub(crate) child: [NodeIndex;2], // negative if leaf, positive if branching
    pub(crate) bit: u8, // position of the relevant bit
}

impl Branching {

    #[inline]
    fn child<B:BitSlot>(&self, slot:&B) -> NodeIndex {
        if slot.is_set(self.bit) { self.child[1] } else { self.child[0] }
    }

    #[inline]
    pub(crate) fn child_mut<B:BitSlot>(&mut self, slot:&B) -> &mut NodeIndex {
        if slot.is_set(self.bit) { &mut self.child[1] } else { &mut self.child[0] }
    }
}

#[derive(Clone)]
pub(crate) struct BranchingTree(pub(crate) Vec<Branching>);

impl BranchingTree
{
    pub fn new(capacity: usize) -> Self
    {
        let mut branching = Vec::with_capacity(capacity);
        branching.push(Branching {
            escape: LeafIndex::root_leaf(),
            parent: BranchingIndex::root(),
            child: [LeafIndex::root_leaf().into(); 2],
            bit: 1,
        });
        Self(branching)
    }

    #[allow(dead_code)]
    pub fn clear(&mut self)
    {
        self.0.clear();
        self.0.push(Branching {
            escape: LeafIndex::root_leaf(),
            parent: BranchingIndex::root(),
            child: [LeafIndex::root_leaf().into(); 2],
            bit: 1
        });
    }

    // returns the index of the added node
    pub fn push(&mut self, parent: BranchingIndex, escape: LeafIndex, bit: u8) -> BranchingIndex
    {
        let index = self.0.len().into();
        self.0.push(Branching {
            escape,
            parent,
            child: [escape.into(); 2],
            bit,
        });
        index
    }

    #[inline]
    #[allow(dead_code)]
    pub fn remove_last(&mut self)
    {
        debug_assert!(self.0.len() > 1);
        self.0.pop();
    }

    #[inline]
    #[allow(dead_code)]
    pub fn remove(&mut self, i: BranchingIndex)
    {
        debug_assert!(!i.is_root());
        self.0.swap_remove(i.index());
    }

    #[inline]
    pub fn search_deepest_candidate<B:BitSlot>(&self, slot: &B) -> (BranchingIndex, LeafIndex)
    {
        let mut b = BranchingIndex::root();
        loop {
            let n = self[b].child(slot);
            if n.is_leaf() {
                return (b, n.into());
            }
            b = n.into();
        }
    }

    #[allow(dead_code)]
    pub fn search_one_matching_leaf(&self, mut b: BranchingIndex) -> LeafIndex
    {
        loop {
            let bb = &self[b];
            if self[bb.parent].escape != bb.escape {
                return bb.escape;
            }
            if bb.child[0].is_leaf() && bb.child[0] != bb.escape {
                return bb.child[0].into();
            }
            if bb.child[1].is_leaf() && bb.child[1] != bb.escape {
                return bb.child[1].into();
            }
            if bb.child[0].is_branching() {
                b = bb.child[0].into();
            } else if bb.child[1].is_branching() {
                b = bb.child[1].into();
            } else {
                debug_assert!(b.is_root()); // empty trie...
                return LeafIndex::root_leaf();
            }
        }
    }

    pub fn replace_escape_leaf(&mut self, n: BranchingIndex, l1: LeafIndex, l2: LeafIndex)
    {
        debug_assert!(self[n].escape == l1);
        self[n].escape = l2;
        for i in 0..=1 {
            let c = self[n].child[i];
            if c.is_leaf() {
                if c == l1 { self[n].child[i] = l2.into(); }
            } else if self[c].escape == l1 {
                self.replace_escape_leaf(c.into(), l1, l2);
            }
        }
    }

    /*
        * insertion d'un branchement juste apres n
        * x est le noeud pour la valeur du bit p dans le slot s
        * e est la valeur escape a utiliser pour ce noeud
        // NOTE : ca rebelote potentiellement les pointeurs...
        // DONC apres un appel a insertSuffixBranching, on ne peut se fier a aucun poiteur
        */
    pub fn insert_prefix_branching<B:BitSlot>(&mut self, n: BranchingIndex, e: LeafIndex, x: NodeIndex, p: u8, slot: &B) -> BranchingIndex
    {
        debug_assert!(self[n].bit <= p);
        let nn = self.push(n, e, p);

        // reste a le connecter comme il faut
        *self[nn].child_mut(slot) = x;

        if x.is_branching() {
            debug_assert!(self[x].bit > p);
            self[x].parent = nn;
            if self[x].escape == self[n].escape {
                self.replace_escape_leaf(x.into(), self[n].escape, e);
            }
        }
        *self[n].child_mut(slot) = nn.into();
        nn
    }

    /*
     * REQUIREMENT: le prefixe ajoute n'est pas deja present dans le trie
     */
    #[allow(clippy::too_many_arguments)]
    pub fn insert_prefix<B:BitSlot>(&mut self,
                                    addedindex: LeafIndex, addedslot: &B, addedlen: u8,
                                    mut n: BranchingIndex,
                                    deepestindex: LeafIndex, deepestslot: &B, deepestlen: u8)
    {
        // attention, cette feuille peut etre plus profonde que le prefixe insere
        // mais le fait de faire un xor verifie egalement la comparaison (sauf la longueur
        // qu'il faudra comparer ensuite).
        let cmp:B = *addedslot ^ *deepestslot;

        // position discriminante du noeud de branchement du nouveau prefixe
        let pos = cmp.first_bit();

        if (pos > deepestlen) && (deepestlen < addedlen) {
            // tout se joue au dela du prefixe le plus long dans le trie
            if self[n].child(addedslot) == self[n].escape {
                *self[n].child_mut(addedslot) = addedindex.into();
            } else {
                self.insert_prefix_branching(n, deepestindex, addedindex.into(), deepestlen+1, addedslot);
            }
        } else if pos > addedlen {
            // on sait que le deepest est plus long (il est plus long que pos donc de addedlength), donc on sait que
            // le prefixe ajoute est un prefixe de deepest (sinon pos serait plus petite)
            // reste a l'inserer s'il n'est pas deja present
            let pos = addedlen + 1;
            while self[n].bit > pos {
                n = self[n].parent;
            }

            // ici, sauf erreur, la longueur du prefixe de b->escape n'est pas egale
            // a addedlength sinon, cela voudrait dire que le prefixe ajoute etati
            if self[n].bit < pos {
                // il faut inserer un branchement avec la bonne position
                self.insert_prefix_branching(n, addedindex, self[n].child(deepestslot), pos, deepestslot);
            } else {
                debug_assert_eq!(self[n].bit, pos);
                self.replace_escape_leaf(n, self[n].escape, addedindex);
            }
        } else {
            // bon, la, on sait que la position discriminante est inferieure a la longueur
            // de chacun des prefixes donc ils sont bien concurrents
            // (et on sait aussi que le prefixe ajoute est bien nouveau)

            // on recherche maintenant le point d'insertion de cette position
            // (on remonte suffisamment pour que la position retenue soit valide)
            while self[n].bit > pos {
                n = self[n].parent;
            }

            // il faut maintenant s'assurer qu'on teste bien la bonne position dans le
            // branchement courant (si ce n'est pas le cas, on ajoute le branchement idoine)
            if self[n].bit < pos {
                n = self.insert_prefix_branching(n, self[n].escape, self[n].child(deepestslot), pos, deepestslot);
            }
            debug_assert_eq!(self[n].bit, pos);
            debug_assert_ne!(self[n].child(addedslot), self[n].child(deepestslot)); // la position est bien discriminante
            *self[n].child_mut(addedslot) = addedindex.into();
        }
    }

    // this is the number of suppressed branching if compression is done
    // note: this node is counted also
    pub(crate) fn count_compressed_branching(&self, b: &Branching, p: u8) -> usize
    {
        b.child
            .iter()
            .filter(|c| c.is_branching())
            .map(|c| &self[BranchingIndex::from(*c)])
            .fold(1, |count, b|
                if b.bit <= p {
                    count + self.count_compressed_branching(b, p)
                } else {
                    count
                }
            )
    }

    fn compression_level_max(&self, b: &Branching, max: u8, stop: LeafIndex) -> u8
    {
        if max == 0 { return 0; }

        // NOTE : on ne peut pas compresser des niveaux avec des prefixes differents
        // donc on s'arrete quand le prefixe differe ou quand on atteint max
        if b.escape != stop { return 0; }

        if b.child[0].is_branching() {
            if b.child[1].is_branching() {
                let l0 = self.compression_level_max(&self[BranchingIndex::from(b.child[0])], max-1, stop);
                let l1 = self.compression_level_max(&self[BranchingIndex::from(b.child[1])], max-1, stop);
                max.min(1 + l0.min(l1))
            } else {
                1+self.compression_level_max(&self[BranchingIndex::from(b.child[0])], max-1, stop)
            }
        } else if b.child[1].is_branching() {
            1+self.compression_level_max(&self[BranchingIndex::from(b.child[1])], max-1, stop)
        } else {
            // two leaves... no branching
            1
        }
    }

    pub(crate) fn compression_level(&self, b: &Branching, comp: u8 ) -> u8
    {
        let compression_max = self.compression_level_max(b, 15, b.escape);
        match (1..compression_max)
            .try_fold((0u8, self.count_compressed_branching(b, b.bit)),
                      |(compression_level, compressed_children), j| {
                          let cc = self.count_compressed_branching(b, b.bit + j);
                          if cc < (1<<j)/(1<<comp)/2 {
                              Err(compression_level) // on ne trouvera pas mieux...
                          } else if cc > compressed_children {
                              Ok((j, cc))
                          } else {
                              Ok((compression_level, compressed_children))
                          }
                      })
        {
            Err(n)  => n,
            Ok((n, _)) => n
        }
    }
}


impl Index<BranchingIndex> for BranchingTree
{
    type Output = Branching;

    #[inline]
    fn index(&self, i: BranchingIndex) -> &Self::Output
    {
        debug_assert!( i.index() < self.0.len());
        unsafe { self.0.get_unchecked(i.index()) }
    }
}

impl IndexMut<BranchingIndex> for BranchingTree
{
    #[inline]
    fn index_mut(&mut self, i: BranchingIndex) -> &mut Self::Output
    {
        debug_assert!( i.index() < self.0.len());
        unsafe { self.0.get_unchecked_mut(i.index())}
    }
}


impl Index<NodeIndex> for BranchingTree
{
    type Output = Branching;

    fn index(&self, i: NodeIndex) -> &Self::Output
    {
        debug_assert!( i.is_branching() );
        let i: BranchingIndex = i.into();
        debug_assert!( i.index() < self.0.len());
        unsafe { self.0.get_unchecked(i.index()) }
    }
}

impl IndexMut<NodeIndex> for BranchingTree
{
    fn index_mut(&mut self, i: NodeIndex) -> &mut Self::Output
    {
        debug_assert!( i.is_branching() );
        let i: BranchingIndex = i.into();
        debug_assert!( i.index() < self.0.len());
        unsafe { self.0.get_unchecked_mut(i.index())}
    }
}

