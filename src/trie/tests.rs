use ipnet::Ipv6Net;
use std::net::Ipv6Addr;

use crate::*;
use rand::distributions::*;
use rand::*;

#[test]
fn ipv6_tries() {
    let mut rng = thread_rng();

    let samples = {
        let prefix = Uniform::<u8>::from(8..=50);
        let addr = Uniform::<u128>::from(1..=u128::MAX);
        std::iter::repeat_with(|| {
            Ipv6Net::new(addr.sample(&mut rng).into(), prefix.sample(&mut rng)).unwrap()
        })
        .take(100_000)
        .collect::<Vec<_>>()
    };

    let t1: RTrieSet<Ipv6Prefix> = samples.iter().map(|i| Ipv6Prefix::from(*i)).collect();
    let t2: RTrieSet<Ipv6NetPrefix> = samples
        .iter()
        .map(|i| Ipv6NetPrefix::try_from(*i).unwrap())
        .collect();
    let t3: RTrieSet<Ipv6Net> = RTrieSet::from_iter(samples);

    let addr = Uniform::<u128>::from(((u64::MAX as u128) << 64)..=u128::MAX);
    std::iter::repeat_with(|| Ipv6Addr::from(addr.sample(&mut rng)))
        .take(100_000)
        .for_each(|ip| {
            let p1 = t1.lookup(&ip);
            let p2 = t2.lookup(&ip);
            let p3 = t3.lookup(&ip);
            assert!(p1.covers_equally(p2));
            assert!(p2.covers_equally(p3));
        });
}

#[test]
fn remove_reindexes_moved_escape_leaf() {
    let mut trie = Ipv4RTrieMap::new();
    let leaf_to_keep = "80.0.0.0/4".parse::<Ipv4Prefix>().unwrap();
    let unrelated_leaf = "128.0.0.0/3".parse::<Ipv4Prefix>().unwrap();
    let leaf_to_remove = "96.0.0.0/4".parse::<Ipv4Prefix>().unwrap();
    let moved_escape_leaf = "0.0.0.0/1".parse::<Ipv4Prefix>().unwrap();

    // The insertion order is important: removing `leaf_to_remove` moves the final
    // leaf into its slot. That final leaf is also inherited as an escape leaf by a
    // descendant branch, so all propagated escape references must be reindexed.
    trie.insert(leaf_to_keep, 1);
    trie.insert(unrelated_leaf, 2);
    trie.insert(leaf_to_remove, 3);
    trie.insert(moved_escape_leaf, 4);

    assert_eq!(trie.remove(&leaf_to_remove), Some(3));
    assert_eq!(trie.get(&leaf_to_remove), None);
    assert_eq!(trie.get(&leaf_to_keep), Some(&1));
    assert_eq!(trie.get(&unrelated_leaf), Some(&2));
    assert_eq!(trie.get(&moved_escape_leaf), Some(&4));
}
