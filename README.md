# iptrie

[![Crates.io](https://img.shields.io/crates/v/iptrie?style=flat)](https://crates.io/crates/iptrie)
[![Crates.io](https://img.shields.io/crates/d/iptrie?style=flat)](https://crates.io/crates/iptrie)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat)](https://crates.io/crates/iptrie)
[![Docs](https://img.shields.io/docsrs/iptrie)](https://docs.rs/iptrie)

This crate implements tries dedicated to IP addresses and prefixes lookup.

It provides sets and maps for Ipv4, Ipv6 and both mixed.

Each structure exists in two versions:
* a first one based on Patricia trie which can be viewed as a standard map or set with a lookup operation for finding the longest prefix match
* a compressed one based one Level-Compressed trie (LC-Trie), optimized for lookup operation (longest prefix match) but which can’t be modified


## Example

```rust
fn main()
{
    let prefixes = [
        "1.1.0.0/24",
        "1.1.1.0/24",
        "1.1.0.0/20",
        "1.2.2.0/24"
    ];

    let iter = prefixes.iter().map(|x| x.parse::<Ipv4Prefix>().unwrap());

    // a set based on Patricia trie
    let trie = Ipv4RTrieSet::from_iter(iter);

    // lpm lookup for Ipv4 address
    assert_eq!(trie.lookup(&"1.1.1.2".parse::<Ipv4Addr>().unwrap()).to_string(), "1.1.1.0/24");
    assert_eq!(trie.lookup(&"1.1.2.2".parse::<Ipv4Addr>().unwrap()).to_string(), "1.1.0.0/20");

    // lpm lookup for Ipv4 prefix also works
    assert_eq!(trie.lookup(&"1.1.0.0/25".parse::<Ipv4Prefix>().unwrap()).to_string(), "1.1.0.0/24");
    assert_eq!(trie.lookup(&"1.1.0.0/21".parse::<Ipv4Prefix>().unwrap()).to_string(), "1.1.0.0/20");


    // now, compute the set based on LC-trie
    let lctrie = trie.compress();

    // lpm lookup for Ipv4 address
    assert_eq!(lctrie.lookup(&"1.1.1.2".parse::<Ipv4Addr>().unwrap()).to_string(), "1.1.1.0/24");
    assert_eq!(lctrie.lookup(&"1.1.2.2".parse::<Ipv4Addr>().unwrap()).to_string(), "1.1.0.0/20");

    // lpm lookup for Ipv4 prefix also works
    assert_eq!(lctrie.lookup(&"1.1.0.0/25".parse::<Ipv4Prefix>().unwrap()).to_string(), "1.1.0.0/24");
    assert_eq!(lctrie.lookup(&"1.1.0.0/21".parse::<Ipv4Prefix>().unwrap()).to_string(), "1.1.0.0/20");
}
```


# Performances

For this crate, we want the highest performance for lookup despite the insertion operation.
We made comparison with the crate [ip_network_table-deps-treebitmap](https://crates.io/crates/ip_network_table-deps-treebitmap)
identified by `IpLookupTable` in the next sections.

## Lookup algorithms

### Randomly generated prefixes

We generated one million of random prefixes for Ipv4 and Ipv6 in order to feed
the lookup table. Then, we checked the lookup procedure with randomly generated
Ip addresses.

|                              | Ipv4 lookup | Ipv6 lookup |
|------------------------------|:-----------:|:-----------:|
| IpLookupTable                |    50 ns    |   165 ns    |
| Patricia trie _(this crate)_ |   125 ns    |   700 ns    |
| LC-Trie _(this crate)_       |    80 ns    |   320 ns    |

The lookup table based on tree bitmap is the best choice.

### BGP prefixes

But the internet has an internal structure that is not random. So, we use
a real BGP table with more than 1M Ipv4 prefixes and more than 175k Ipv6 prefixes.
Then, we checked the lookup procedure with randomly generated
Ip addresses.

|                              | Ipv4 lookup | Ipv6 lookup |
|------------------------------|:-----------:|:-----------:|
| IpLookupTable                |    61 ns    |    50 ns    |
| Patricia trie _(this crate)_ |   130 ns    |    42 ns    |
| LC-Trie _(this crate)_       |    47 ns    |    24 ns    |

This time, the lookup based on LC-Trie has the best performances.

## Building the tries

### Time consuming

We built lookup tables from 100k of random prefixes for Ipv4
and the same for Ipv6, and then we built lookup tables from 100k 
out of a real BGP table for Ipv4 and the same for Ipv6.

The BGP table is read and inserted in prefix order (i.e. highest prefix in first) which
is the best case for building a prefix trie.

|                              | Ipv4 random | Ipv6 random | Ipv4 BGP | Ipv6 BGP |
|------------------------------|:-----------:|:-----------:|:--------:|:--------:|
| IpLookupTable                |    21 ms    |    58 ms    |  24 ms   |  95 ms   |
| Patricia trie _(this crate)_ |    13 ms    |    16 ms    |  12 ms   |   8 ms   |
| LC-Trie _(this crate)_       |    15 ms    |    21 ms    |  17 ms   |  14 ms   |


Note that building a LC-Trie consists of building a Patricia Trie
then compressing it.

### Memory

We use a real BGP table with more than 1M Ipv4 prefixes 
and more than 175k Ipv6 prefixes.

|                              | Ipv4 BGP | Ipv6 BGP |
|------------------------------|:--------:|:--------:|
| IpLookupTable                |  5.7 M   |  2.2 M   |
| Patricia trie _(this crate)_ |  28.8 M  |  9.1 M   |
| LC-Trie _(this crate)_       |  19.4 M  |  7.6 M   |


The `IpLookupTable` is definitively the good choice for applications
with limited memory.

_(I guess I will work on this for next releases)_
