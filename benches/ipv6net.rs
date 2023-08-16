#![feature(test)]
extern crate test;

use std::net::Ipv6Addr;
use test::Bencher;
use ipnet::Ipv6Net;
use iptrie::*;

fn build_ipv6_samples(n: usize) -> Vec<Ipv6Net>
{
    use rand::*;
    use rand::distributions::*;
    let mut rng = thread_rng();
    let prefix = Uniform::<u8>::from(8..=50);
    let addr = Uniform::<u128>::from(1..=(u128::MAX >> 80));
    std::iter::repeat_with(|| {
        let addr = addr.sample(&mut rng) << 80;
        Ipv6Net::new(addr.into(), prefix.sample(&mut rng)).unwrap()
    }).take(n).collect()
}


fn build_ipv6_addr(n: usize) -> Vec<Ipv6Addr>
{
    use rand::*;
    use rand::distributions::*;
    let mut rng = thread_rng();
    let addr = Uniform::<u128>::from(1..=u128::MAX);
    std::iter::repeat_with(|| Ipv6Addr::from(addr.sample(&mut rng))).take(n).collect()
}

fn build_one_ipv6_trie() -> RTrieSet<Ipv6Net>
{
    let n = 10_000_000;
    build_ipv6_samples(n).into_iter().collect()
}


#[bench]
fn lookup_ipv6_trie(bencher: &mut Bencher)
{
    let trie = build_one_ipv6_trie();
    let sample = build_ipv6_addr(10_000);
    bencher.iter(|| sample.iter().for_each(|addr| {
        let _ = trie.lookup(addr);
    }))
}



#[bench]
fn lookup_ipv6_lctrie(bencher: &mut Bencher)
{
    let trie = build_one_ipv6_trie();
    let trie = trie.compress();
    let sample = build_ipv6_addr(10_000);
    bencher.iter(|| sample.iter().for_each(|addr| {
        let _ = trie.lookup(addr);
    }))
}

