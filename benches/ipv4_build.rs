#![feature(test)]
extern crate test;

use std::iter::repeat_with;
use std::net::Ipv4Addr;
use test::Bencher;

use iptrie::*;
use  ip_network_table_deps_treebitmap::IpLookupTable;

fn random_ipv4_prefix() -> impl Iterator<Item=Ipv4Prefix>
{
    use rand::*;
    use rand::distributions::*;
    let mut rng = thread_rng();
    let prefix = Uniform::<u8>::from(8..=24);
    let addr = Uniform::<u32>::from(1..=(u32::MAX>>8));
    repeat_with(move || {
        let addr = addr.sample(&mut rng) << 8;
        Ipv4Prefix::new(addr.into(), prefix.sample(&mut rng)).unwrap()
    }).take(100_000)
}


#[bench]
fn nop_ipv4prefix_trie(bencher: &mut Bencher)
{
    let mut result = Vec::with_capacity(1_000_000);
    bencher.iter(|| result.extend(random_ipv4_prefix()));
    println!("{}", result.len());
}


#[bench]
fn build_ipv4prefix_trie(bencher: &mut Bencher)
{
    let mut trie = RTrieSet::new();
    bencher.iter(|| { trie = random_ipv4_prefix().collect(); });
    println!("{}", trie.len());
}


#[bench]
fn build_ipv4prefix_lctrie(bencher: &mut Bencher)
{
    let mut trie : LCTrieSet<_> = RTrieSet::new().compress();
    bencher.iter(|| { trie = random_ipv4_prefix().collect(); });
    println!("{}", trie.len());
}



#[bench]
fn build_ipv4net_treebit(bencher: &mut Bencher)
{
    let mut trie = IpLookupTable::new();
    bencher.iter(|| {
        trie = random_ipv4_prefix()
            .fold(IpLookupTable::<Ipv4Addr,()>::with_capacity(1_000_000),
                  |mut trie,p| { trie.insert(p.network(), p.len() as u32, ()); trie });
    });
    println!("{}", trie.len());
}

