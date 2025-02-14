mod trie;
pub mod map;
pub mod set;
mod prefix;
#[cfg(feature= "graphviz")] pub mod graphviz;

use std::num::NonZeroUsize;
use ipnet::IpNet;
use map::*;
use set::*;

pub use prefix::*;

/// Convenient alias for radix trie set of Ipv4 prefixes
pub type Ipv4RTrieSet = RTrieSet<Ipv4Prefix>;
/// Convenient alias for radix trie set of Ipv6 prefixes
pub type Ipv6RTrieSet = RTrieSet<Ipv6Prefix>;

/// Convenient alias for LC-Trie set of Ipv4 prefixes
pub type Ipv4LCTrieSet = LCTrieSet<Ipv4Prefix>;
/// Convenient alias for LC-Trie set of Ipv6 prefixes
pub type Ipv6LCTrieSet = LCTrieSet<Ipv6Prefix>;


/// A radix trie set that mix both Ipv4 and Ipv6 prefixes
#[derive(Clone,Default)]
pub struct IpRTrieSet {
    pub ipv4: Ipv4RTrieSet,
    pub ipv6: Ipv6RTrieSet,
}

/// A LC-trie set that mix both Ipv4 and Ipv6 prefixes
pub struct IpLCTrieSet {
    pub ipv4: Ipv4LCTrieSet,
    pub ipv6: Ipv6LCTrieSet,
}

impl IpRTrieSet {

    pub fn new() -> Self { Self { ipv4: Ipv4RTrieSet::new(), ipv6: Ipv6RTrieSet::new() } }
    pub fn compress(self) -> IpLCTrieSet { IpLCTrieSet { ipv4: self.ipv4.compress(), ipv6: self.ipv6.compress() } }
    pub fn shrink_to_fit(&mut self) { self.ipv4.shrink_to_fit(); self.ipv6.shrink_to_fit(); }

    /// Returns the size of the set.
    ///
    /// Notice that it always greater or equals two since two top prefixes are
    /// always present in the map (one for Ipv4 and the other for Ipv6)
    pub fn len(&self) -> NonZeroUsize { self.ipv4.len().saturating_add(self.ipv6.len().get()) }

    /// Checks if an element is present (exact match).
    pub fn contains(&self, ipnet: &IpNet) -> bool {
        match ipnet {
            IpNet::V4(net) => self.ipv4.contains(net),
            IpNet::V6(net) => self.ipv6.contains(net),
        }
    }

    /// Gets the value associated with an exact match of the key.
    ///
    /// To access to the longest prefix match, use [`Self::lookup`].
    pub fn get(&self, ipnet: &IpNet) -> Option<IpNet> {
        match ipnet {
            IpNet::V4(net) => self.ipv4.get(net).map(|ip| (*ip).into()),
            IpNet::V6(net) => self.ipv6.get(net).map(|ip| (*ip).into()),
        }
    }
    /// Gets the longest prefix which matches the given key.
    ///
    /// As the top prefix always matches, it never fails.
    ///
    /// To access to the exact prefix match, use [`Self::get`].
    pub fn lookup(&self, ipnet: &IpNet) -> IpNet {
        match ipnet {
            IpNet::V4(net) => (*self.ipv4.lookup(net)).into(),
            IpNet::V6(net) => (*self.ipv6.lookup(net)).into(),
        }
    }

    /// Inserts a new element in the set.
    ///
    /// If the specified element already exists in the set, `false` is returned.
    pub fn insert(&mut self, ipnet: IpNet) -> bool {
        match ipnet {
            IpNet::V4(net) => self.ipv4.insert(net.into()),
            IpNet::V6(net) => self.ipv6.insert(net.into()),
        }
    }
    /// Removes a previously inserted prefix (exact match).
    ///
    /// Returns `false` is the element was not present in the set
    /// and `true` if the removal is effective.
    pub fn remove(&mut self, ipnet: &IpNet) -> bool {
        match ipnet {
            IpNet::V4(net) => self.ipv4.remove(net),
            IpNet::V6(net) => self.ipv6.remove(net),
        }
    }
    /// Replace an existing prefix.
    ///
    /// Adds a prefix to the set, replacing the existing one, if any (exact match performed).
    /// Returns the replaced value.
    pub fn replace(&mut self, ipnet: IpNet) -> Option<IpNet> {
        match ipnet {
            IpNet::V4(net) => self.ipv4.replace(net.into()).map(IpNet::from),
            IpNet::V6(net) => self.ipv6.replace(net.into()).map(IpNet::from),
        }
    }

    /// Iterates over all the prefixes of this set.
    pub fn iter(&self) -> impl Iterator<Item=IpNet> + '_
    {
        self.ipv4.iter().map(|i| (*i).into())
            .chain(self.ipv6.iter().map(|i| (*i).into()))
    }
}

impl IpLCTrieSet {

    /// Returns the size of the set.
    ///
    /// Notice that it always greater or equals two since two top prefixes are
    /// always present in the map (one for Ipv4 and the other for Ipv6)
    pub fn len(&self) -> NonZeroUsize { self.ipv4.len().saturating_add(self.ipv6.len().get()) }

    /// Checks if an element is present (exact match).
    pub fn contains(&self, ipnet: &IpNet) -> bool {
        match ipnet {
            IpNet::V4(net) => self.ipv4.contains(net),
            IpNet::V6(net) => self.ipv6.contains(net),
        }
    }

    /// Gets the value associated with an exact match of the key.
    ///
    /// To access to the longest prefix match, use [`Self::lookup`].
    pub fn get(&self, ipnet: &IpNet) -> Option<IpNet> {
        match ipnet {
            IpNet::V4(net) => self.ipv4.get(net).map(|ip| (*ip).into()),
            IpNet::V6(net) => self.ipv6.get(net).map(|ip| (*ip).into()),
        }
    }
    /// Gets the longest prefix which matches the given key.
    ///
    /// As the top prefix always matches, it never fails.
    ///
    /// To access to the exact prefix match, use [`Self::get`].
    pub fn lookup(&self, ipnet: &IpNet) -> IpNet {
        match ipnet {
            IpNet::V4(net) => (*self.ipv4.lookup(net)).into(),
            IpNet::V6(net) => (*self.ipv6.lookup(net)).into(),
        }
    }

    /// Iterates over all the prefixes of this set.
    pub fn iter(&self) -> impl Iterator<Item=IpNet> + '_
    {
        self.ipv4.iter().map(|i| (*i).into())
            .chain(self.ipv6.iter().map(|i| (*i).into()))
    }
}


/// Convenient alias for radix trie map of Ipv4 prefixes
pub type Ipv4RTrieMap<V> = RTrieMap<Ipv4Prefix,V>;
/// Convenient alias for radix trie map of Ipv6 prefixes
pub type Ipv6RTrieMap<V> = RTrieMap<Ipv6Prefix,V>;

/// Convenient alias for LC-Trie map of Ipv4 prefixes
pub type Ipv4LCTrieMap<V> = LCTrieMap<Ipv4Prefix,V>;
/// Convenient alias for LC-Trie map of Ipv6 prefixes
pub type Ipv6LCTrieMap<V> = LCTrieMap<Ipv6Prefix,V>;



/// A radix trie map that mix both Ipv4 and Ipv6 prefixes
#[derive(Clone,Default)]
pub struct IpRTrieMap<V> {
    pub ipv4: Ipv4RTrieMap<V>,
    pub ipv6: Ipv6RTrieMap<V>,
}

/// A LC-trie map that mix both Ipv4 and Ipv6 prefixes
pub struct IpLCTrieMap<V> {
    pub ipv4: Ipv4LCTrieMap<V>,
    pub ipv6: Ipv6LCTrieMap<V>,
}

impl<V:Default> IpRTrieMap<V> {
    pub fn new() -> Self {
        Self { ipv4: Ipv4RTrieMap::new(), ipv6: Ipv6RTrieMap::new() }
    }
}

impl<V> IpRTrieMap<V> {
    pub fn with_roots(ipv4: V, ipv6: V) -> Self {
        Self { ipv4: RTrieMap::with_root(ipv4), ipv6: RTrieMap::with_root(ipv6) }
    }
}

impl<V> IpRTrieMap<V> {

    /// Returns the size of the map.
    ///
    /// Notice that it always greater or equals two since two top prefixes are
    /// always present in the map (one for Ipv4 and the other for Ipv6)
    pub fn len(&self) -> NonZeroUsize { self.ipv4.len().saturating_add(self.ipv6.len().get()) }

    /// Compress this Patricia trie in a LC-Trie.
    ///
    /// For lookup algorithms, a Patricia trie performs unit bit checking and LC-Trie
    /// performs multi bits checking. So the last one is more performant but it
    /// cannot be modified (no insertion or removal operations are provided).
    pub fn compress(self) -> IpLCTrieMap<V> { IpLCTrieMap { ipv4: self.ipv4.compress(), ipv6: self.ipv6.compress() } }

    pub fn shrink_to_fit(&mut self) { self.ipv4.shrink_to_fit(); self.ipv6.shrink_to_fit(); }

    /// Gets the value associated with an exact match of the key.
    ///
    /// To access to the longest prefix match, use [`Self::lookup`].
    ///
    /// To get a mutable access to a value, use [`Self::get_mut`].
    ///
    pub fn get(&self, ipnet: &IpNet) -> Option<&V> {
        match ipnet {
            IpNet::V4(net) => self.ipv4.get(net),
            IpNet::V6(net) => self.ipv6.get(net),
        }
    }
    /// Gets a mutable access to the value associated with an exact match of the key.
    ///
    /// To access to the longest prefix match, use [`Self::lookup_mut`].
    ///
    /// To get a mutable access to a value, use [`Self::get_mut`].
    pub fn get_mut(&mut self, ipnet: &IpNet) -> Option<&mut V> {
        match ipnet {
            IpNet::V4(net) => self.ipv4.get_mut(net),
            IpNet::V6(net) => self.ipv6.get_mut(net),
        }
    }
    /// Gets the value associated with the longest prefix match of the key.
    ///
    /// As the top prefix always matches, the lookup never fails.
    ///
    /// To access to the exact prefix match, use [`Self::get`].
    ///
    /// To get a mutable access to a value, use [`Self::lookup_mut`].
    pub fn lookup(&self, ipnet: &IpNet) -> (IpNet,&V) {
        match ipnet {
            IpNet::V4(net) => { let (&k,v) = self.ipv4.lookup(net);  (k.into(),v) },
            IpNet::V6(net) => { let (&k,v) = self.ipv6.lookup(net);  (k.into(),v) },
        }
    }
    /// Gets a mutable access to the value associated with a longest prefix match of the key.
    ///
    /// To access to the exact prefix match, use [`Self::get_mut`].
    pub fn lookup_mut(&mut self, ipnet: &IpNet) -> (IpNet,&mut V) {
        match ipnet {
            IpNet::V4(net) => { let (&k,v) = self.ipv4.lookup_mut(net);  (k.into(),v) },
            IpNet::V6(net) => { let (&k,v) = self.ipv6.lookup_mut(net);  (k.into(),v) },
        }
    }
    /// Inserts a new entry in the map.
    ///
    /// If the specified key already exists in the map, then the previous associated
    /// value is replaced by the new one and is returned.
    pub fn insert(&mut self, ipnet: IpNet, v: V) -> Option<V> {
        match ipnet {
            IpNet::V4(net) => self.ipv4.insert(net.into(), v),
            IpNet::V6(net) => self.ipv6.insert(net.into(), v),
        }
    }
    /// Removes a previously inserted prefix (exact match).
    /// # Panic
    /// Panics if trying to remove the root prefix.
    pub fn remove(&mut self, ipnet: &IpNet) -> Option<V> {
        match ipnet {
            IpNet::V4(net) => self.ipv4.remove(net),
            IpNet::V6(net) => self.ipv6.remove(net),
        }
    }
    /// Iterates over all the entries.
    ///
    /// For a mutable access of values, use [`Self::iter_mut`]
    pub fn iter(&self) -> impl Iterator<Item=(IpNet,&V)> + '_
    {
        self.ipv4.iter().map(|(k,v)| ((*k).into(), v))
            .chain(self.ipv6.iter().map(|(k,v)| ((*k).into(), v)))
    }
    /// Iterates over all the entries with a mutable access to values.
    pub fn iter_mut(&mut self) -> impl Iterator<Item=(IpNet,&mut V)> + '_
    {
        self.ipv4.iter_mut().map(|(k,v)| ((*k).into(), v))
            .chain(self.ipv6.iter_mut().map(|(k,v)| ((*k).into(), v)))
    }

    /// Gets a set of copy of all the keys in a trie set.
    pub fn prefixes(&self) -> IpRTrieSet {
        IpRTrieSet { ipv4: self.ipv4.prefixes(), ipv6: self.ipv6.prefixes() }
    }
}

impl<V> IpLCTrieMap<V> {

    /// Returns the size of the map.
    ///
    /// Notice that it always greater or equals two since two top prefixes are
    /// always present in the map (one for Ipv4 and the other for Ipv6)
    pub fn len(&self) -> NonZeroUsize { self.ipv4.len().saturating_add(self.ipv6.len().get()) }

    /// Gets the value associated with an exact match of the key.
    ///
    /// To access to the longest prefix match, use [`Self::lookup`].
    ///
    /// To get a mutable access to a value, use [`Self::get_mut`].
    ///
    pub fn get(&self, ipnet: &IpNet) -> Option<&V> {
        match ipnet {
            IpNet::V4(net) => self.ipv4.get(net),
            IpNet::V6(net) => self.ipv6.get(net),
        }
    }
    /// Gets a mutable access to the value associated with an exact match of the key.
    ///
    /// To access to the longest prefix match, use [`Self::lookup_mut`].
    ///
    /// To get a mutable access to a value, use [`Self::get_mut`].
    pub fn get_mut(&mut self, ipnet: &IpNet) -> Option<&mut V> {
        match ipnet {
            IpNet::V4(net) => self.ipv4.get_mut(net),
            IpNet::V6(net) => self.ipv6.get_mut(net),
        }
    }
    /// Gets the value associated with the longest prefix match of the key.
    ///
    /// As the top prefix always matches, the lookup never fails.
    ///
    /// To access to the exact prefix match, use [`Self::get`].
    ///
    /// To get a mutable access to a value, use [`Self::lookup_mut`].
    pub fn lookup(&self, ipnet: &IpNet) -> (IpNet,&V) {
        match ipnet {
            IpNet::V4(net) => { let (&k,v) = self.ipv4.lookup(net);  (k.into(),v) },
            IpNet::V6(net) => { let (&k,v) = self.ipv6.lookup(net);  (k.into(),v) },
        }
    }
    /// Gets a mutable access to the value associated with a longest prefix match of the key.
    ///
    /// To access to the exact prefix match, use [`Self::get_mut`].
    pub fn lookup_mut(&mut self, ipnet: &IpNet) -> (IpNet,&mut V) {
        match ipnet {
            IpNet::V4(net) => { let (&k,v) = self.ipv4.lookup_mut(net);  (k.into(),v) },
            IpNet::V6(net) => { let (&k,v) = self.ipv6.lookup_mut(net);  (k.into(),v) },
        }
    }

    /// Iterates over all the entries.
    ///
    /// For a mutable access of values, use [`Self::iter_mut`]
    pub fn iter(&self) -> impl Iterator<Item=(IpNet,&V)> + '_
    {
        self.ipv4.iter().map(|(k,v)| ((*k).into(), v))
            .chain(self.ipv6.iter().map(|(k,v)| ((*k).into(), v)))
    }
    /// Iterates over all the entries with a mutable access to values.
    pub fn iter_mut(&mut self) -> impl Iterator<Item=(IpNet,&mut V)> + '_
    {
        self.ipv4.iter_mut().map(|(k,v)| ((*k).into(), v))
            .chain(self.ipv6.iter_mut().map(|(k,v)| ((*k).into(), v)))
    }

    /// Gets a set of copy of all the keys in a trie set.
    pub fn prefixes(&self) -> IpLCTrieSet {
        IpLCTrieSet { ipv4: self.ipv4.prefixes(), ipv6: self.ipv6.prefixes() }
    }
}

