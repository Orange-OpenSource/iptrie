#![feature(test)]
extern crate test;

use std::iter::repeat_with;
use std::net::Ipv6Addr;
use test::Bencher;

use iptrie::*;
use  ip_network_table_deps_treebitmap::IpLookupTable;

fn random_ipv6_prefix() -> impl Iterator<Item=Ipv6Prefix>
{
    use rand::*;
    use rand::distributions::*;
    let mut rng = thread_rng();
    let prefix = Uniform::<u8>::from(8..=50);
    let addr = Uniform::<u128>::from(1..=(u128::MAX >> 80));
    repeat_with( move || {
        let addr = addr.sample(&mut rng) << 80;
        Ipv6Prefix::new(addr.into(), prefix.sample(&mut rng)).unwrap()
    }).take(100_000)
}


#[bench]
fn nop_ipv6prefix_trie(bencher: &mut Bencher)
{
    let mut result = Vec::with_capacity(1_000_000);
    bencher.iter(|| result.extend(random_ipv6_prefix()));
    println!("{}", result.len());
}


#[bench]
fn build_ipv6prefix_trie(bencher: &mut Bencher)
{
    let mut trie = RTrieSet::new();
    bencher.iter(|| { trie = random_ipv6_prefix().collect(); });
    println!("{}", trie.len());
}


#[bench]
fn build_ipv6prefix_lctrie(bencher: &mut Bencher)
{
    let mut trie : LCTrieSet<_> = RTrieSet::new().compress();
    bencher.iter(|| { trie = random_ipv6_prefix().collect(); });
    println!("{}", trie.len());
}



#[bench]
fn build_ipv6net_treebit(bencher: &mut Bencher)
{
    let mut trie = IpLookupTable::new();
    bencher.iter(|| {
        trie = random_ipv6_prefix()
            .fold(IpLookupTable::<Ipv6Addr,()>::with_capacity(1_000_000),
                  |mut trie,p| { trie.insert(p.network(), p.len() as u32, ()); trie });
    });
    println!("{}", trie.len());
}

