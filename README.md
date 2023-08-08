# iptrie

[![Crates.io](https://img.shields.io/crates/v/iptrie?style=flat)](https://crates.io/crates/iptrie)
[![Crates.io](https://img.shields.io/crates/d/iptrie?style=flat)](https://crates.io/crates/iptrie)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat)](https://crates.io/crates/iptrie)
[![Docs](https://img.shields.io/docsrs/iptrie)](https://docs.rs/iptrie)

This crate implements tries dedicated to IP addresses and prefixes lookup.

It provides sets and maps for Ipv4, Ipv6 and both mixed.

Each structure exists in two versions:
* a modifiable one based on Patricia trie
* a compressed and very efficient one based one Level-Compressed trie (LC-Trie)

## Performances

The following results are based on a set of Ipv4 prefixes and the lookup is performed
from Ipv4 addresses (the whole spectrum is used).

Performances are very dependant from the initial set of prefixes but comparison
between Patricia trie and LC-Trie are significant which is faster from 3 to 5 times
(tests done from sets of 100k to 10M of randomly chosen Ipv4 prefixes).


## Example

```
 let prefixes = [
        "1.1.0.0/24",
        "1.1.1.0/24",
        "1.1.0.0/20",
        "1.1.2.0/24"
    ];

let iter = prefixes.iter().map(|x| x.parse::<Ipv4Net>().unwrap());

// a set based on Patricia trie
let trie = Ipv4RTrieSet::from_iter(iter);

// a set based on LC-trie
let lctrie =  trie.compress();
```