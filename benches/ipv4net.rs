#![feature(test)]
extern crate test;

use std::net::Ipv4Addr;
use test::Bencher;
use ipnet::Ipv4Net;
use iptrie::*;

fn build_ipv4_samples(n: usize) -> Vec<Ipv4Net>
{
    use rand::*;
    use rand::distributions::*;
    let mut rng = thread_rng();
    let prefix = Uniform::<u8>::from(8..=24);
    let addr = Uniform::<u32>::from(1..=(u32::MAX>>8));
    std::iter::repeat_with(|| {
        let addr = addr.sample(&mut rng) << 8;
        Ipv4Net::new(addr.into(), prefix.sample(&mut rng)).unwrap()
    }).take(n).collect()
}


fn build_ipv4_addr(n: usize) -> Vec<Ipv4Addr>
{
    use rand::*;
    use rand::distributions::*;
    let mut rng = thread_rng();
    let addr = Uniform::<u32>::from(1..=u32::MAX);
    std::iter::repeat_with(|| Ipv4Addr::from(addr.sample(&mut rng))).take(n).collect()
}

fn build_one_ipv4_trie() -> RTrieSet<Ipv4Net>
{
    let n = 1_000_000;
    build_ipv4_samples(n).into_iter().collect()
}


#[bench]
fn lookup_ipv4_trie(bencher: &mut Bencher)
{
    let trie = build_one_ipv4_trie();
    let sample = build_ipv4_addr(10_000);
    bencher.iter(|| sample.iter().for_each(|addr| {
        let _ = trie.lookup(addr);
    }))
}



#[bench]
fn lookup_ipv4_lctrie(bencher: &mut Bencher)
{
    let trie = build_one_ipv4_trie();
    let trie = trie.compress();
    let sample = build_ipv4_addr(10_000);
    bencher.iter(|| sample.iter().for_each(|addr| {
        let _ = trie.lookup(addr);
    }))
}

