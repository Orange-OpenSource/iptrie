use std::ops::{Index, IndexMut};
use std::marker::PhantomData;

use crate::trie::*;
use crate::ip::*;
use super::bits::*;

#[derive(Debug)]
pub(crate) struct Branching<T:Ip, B:BitMatch<T>> {
    pub(crate) escape: LeafIndex, // leaf associated to this branching node
    pub(crate) parent: BranchingIndex, // to climb up the trie (>=0)
    pub(crate) child: [NodeIndex;2], // negative if leaf, positive if branching
    pub(crate) bit: B, // position of the relevant bit
    ip: PhantomData<T>
}

impl<T:Ip,B:BitMatch<T>> Branching<T,B> {

    fn child(&self, slot:&T) -> NodeIndex {
        if self.bit.is_set(slot) { self.child[1] } else { self.child[0] }
    }

    fn child_mut(&mut self, slot:&T) -> &mut NodeIndex {
        if self.bit.is_set(slot) { &mut self.child[1] } else { &mut self.child[0] }
    }
}

pub(crate) struct BranchingTree<T:Ip, B:BitMatch<T>>(pub(crate) Vec<Branching<T,B>>);

impl<T:Ip,B:BitMatch<T>> BranchingTree<T,B>
{
    pub fn new(capacity: usize) -> Self
    {
        let mut branching = Vec::with_capacity(capacity);
        branching.push(Branching {
            escape: LeafIndex::root_leaf(),
            parent: BranchingIndex::root(),
            child: [LeafIndex::root_leaf().into(); 2],
            bit: 1.into(),
            ip: PhantomData::default()
        });
        Self(branching)
    }

    // returns the index of the added node
    pub fn push(&mut self, parent: BranchingIndex, escape: LeafIndex, bit: B) -> BranchingIndex {
        let index = self.0.len().into();
        self.0.push(Branching {
            escape,
            parent,
            child: [escape.into(); 2],
            bit,
            ip: Default::default()
        });
        index
    }

    pub fn remove_last(&mut self)
    {
        debug_assert!(self.0.len() > 1);
        self.0.pop();
    }

    pub fn remove(&mut self, i: BranchingIndex)
    {
        debug_assert!(!i.is_root());
        self.0.swap_remove(i.index());
    }

    pub fn search_deepest_candidate(&self, slot: &T) -> (BranchingIndex, LeafIndex)
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
                return bb.child[0].into();
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

    pub fn replace_escade_leaf(&mut self, n: BranchingIndex, l1: LeafIndex, l2: LeafIndex)
    {
        debug_assert!(self[n].escape == l1);
        self[n].escape = l2;
        for i in 0..=1 {
            let c = self[n].child[i];
            if c.is_leaf() {
                if c == l1 { *&mut self[n].child[i] = l2.into(); }
            } else {
                if self[c].escape == l1 {
                    self.replace_escade_leaf(c.into(), l1, l2);
                }
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
    pub fn insert_prefix_branching(&mut self, n: BranchingIndex, e: LeafIndex, x: NodeIndex, p: B, slot: &T) -> BranchingIndex
    {
        debug_assert!(self[n].bit < p);
        let nn = self.push(n, e, p);

        // reste a le connecter comme il faut
        *self[nn].child_mut(slot) = x;

        if x.is_branching() {
            debug_assert!(self[x].bit > p);
            self[x].parent = nn;
            if self[x].escape == self[n].escape {
                self.replace_escade_leaf(x.into(), self[n].escape, e);
            }
        }
        *self[n].child_mut(slot) = nn.into();
        return nn;
    }

    /*
     * REQUIREMENT: le prefixe ajoute n'est pas deja present dans le trie
     */
    pub fn insert_prefix(&mut self,
                         addedindex: LeafIndex, addedslot: &T, addedlen: u8,
                         mut n: BranchingIndex,
                         deepestindex: LeafIndex, deepestslot: &T, deepestlen: u8)
    {
        // attention, cette feuille peut etre plus profonde que le prefixe insere
        // mais le fait de faire un xor verifie egalement la comparaison (sauf la longueur
        // qu'il faudra comparer ensuite).
        let cmp = *addedslot ^ *deepestslot;

        // position discriminante du noeud de branchement du nouveau prefixe
        let pos = B::from_first_bit(cmp);

        if (pos > deepestlen.into()) && (deepestlen < addedlen) {
            // tout se joue au dela du prefixe le plus long dans le trie
            if (self[n].child(addedslot) == self[n].escape) {
                *self[n].child_mut(addedslot) = addedindex.into();
            } else {
                self.insert_prefix_branching(n, deepestindex, addedindex.into(), (deepestlen + 1).into(), addedslot);
            }
        } else if pos > addedlen.into() {
            // on sait que le deepest est plus long (il est plus long que pos donc de addedlength), donc on sait que
            // le prefixe ajoute est un prefixe de deepest (sinon pos serait plus petite)
            // reste a l'inserer s'il n'est pas deja present
            let pos = addedlen + 1;
            while self[n].bit > pos.into() {
                n = self[n].parent;
            }

            // ici, sauf erreur, la longueur du prefixe de b->escape n'est pas egale
            // a addedlength sinon, cela voudrait dire que le prefixe ajoute etati
            if (self[n].bit < pos.into()) {
                // il faut inserer un branchement avec la bonne position
                self.insert_prefix_branching(n, addedindex, self[n].child(deepestslot), pos.into(), deepestslot);
            } else {
                debug_assert_eq!(self[n].bit, pos.into());
                self.replace_escade_leaf(n, self[n].escape, addedindex);
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
            if (self[n].bit < pos) {
                n = self.insert_prefix_branching(n, self[n].escape, self[n].child(deepestslot), pos, deepestslot);
            }
            debug_assert_eq!(self[n].bit, pos);
            debug_assert_ne!(self[n].child(addedslot), self[n].child(deepestslot)); // la position est bien discriminante
            *self[n].child_mut(addedslot) = addedindex.into();
        }
    }

    pub(crate) fn count_descendants(&self, b: &Branching<T,B>, p: B, stop: LeafIndex) -> usize
    {
        b.child.iter()
            .filter(|c| c.is_branching())
            .map(|c| &self[BranchingIndex::from(*c)])
            .try_fold(1, |count,b|
                if b.escape != stop {
                    Err(()) // prefix change
                } else if b.bit <= p {
                    Ok(count + self.count_descendants(b, p, stop))
                } else {
                    Ok(count)
                }
            )
            .unwrap_or(0) // when prefix changes
    }



    pub(crate) fn compression_level(&self, b: &Branching<T,B>, comp: u8 ) -> u8
    {
        match (1..=15).into_iter()
            .try_fold((0u8,0), |(compression_level, compressed_children), j| {
                let cc = self.count_descendants(b, b.bit >> j, b.escape);
                if cc < (1<<j)/(1<<comp) {
                    Err(compression_level) // on ne trouvera pas mieux...
                } else if (cc > compressed_children) {
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


impl<T:Ip, B:BitMatch<T>> Index<BranchingIndex> for BranchingTree<T,B>
{
    type Output = Branching<T,B>;

    #[inline]
    fn index(&self, i: BranchingIndex) -> &Self::Output
    {
        debug_assert!( i.index() < self.0.len());
        unsafe { self.0.get_unchecked(i.index()) }
    }
}

impl<T:Ip, B:BitMatch<T>> IndexMut<BranchingIndex> for BranchingTree<T,B>
{
    #[inline]
    fn index_mut(&mut self, i: BranchingIndex) -> &mut Self::Output
    {
        debug_assert!( i.index() < self.0.len());
        unsafe { self.0.get_unchecked_mut(i.index())}
    }
}


impl<T:Ip, B:BitMatch<T>> Index<NodeIndex> for BranchingTree<T,B>
{
    type Output = Branching<T,B>;

    fn index(&self, i: NodeIndex) -> &Self::Output
    {
        debug_assert!( i.is_branching() );
        let i: BranchingIndex = i.into();
        debug_assert!( i.index() < self.0.len());
        unsafe { self.0.get_unchecked(i.index()) }
    }
}

impl<T:Ip, B:BitMatch<T>> IndexMut<NodeIndex> for BranchingTree<T,B>
{
    fn index_mut(&mut self, i: NodeIndex) -> &mut Self::Output
    {
        debug_assert!( i.is_branching() );
        let i: BranchingIndex = i.into();
        debug_assert!( i.index() < self.0.len());
        unsafe { self.0.get_unchecked_mut(i.index())}
    }
}

