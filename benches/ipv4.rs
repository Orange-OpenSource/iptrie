#![feature(test)]
extern crate test;

use std::iter;
use std::net::Ipv4Addr;
use test::Bencher;
use ipnet::Ipv4Net;
use iptrie::*;

fn build_one_ipv4_trie() -> RTrieSet<Ipv4Net>
{
    (0..100_000).into_iter()
        .fold(RTrieSet::with_capacity(200_000),
              |mut trie, i| {
                  trie.insert(Ipv4Net::new(((i*1000)+250).into(), 25).unwrap());
                  trie.insert(Ipv4Net::new(((i*1000)+500).into(), 20).unwrap());
                  trie.insert(Ipv4Net::new((i*1000).into(), 18).unwrap());
                  trie
              })
}


#[bench]
fn build_ipv4_trie(bencher: &mut Bencher)
{
    bencher.iter(|| build_one_ipv4_trie() )
}

#[bench]
fn lookup_ipv4_trie(bencher: &mut Bencher)
{
    let trie = build_one_ipv4_trie();
    bencher.iter(|| (0..u32::MAX).for_each(|i| {
        let addr : Ipv4Addr = i.into();
        let _ = trie.lookup(&addr);
    }))
}

#[bench]
fn compile_ipv4_lctrie(bencher: &mut Bencher)
{
    let trie = build_one_ipv4_trie();
    bencher.iter(|| {
        let _ = trie.clone().compress();
    } )
}


#[bench]
fn lookup_ipv4_lctrie(bencher: &mut Bencher)
{
    let trie = build_one_ipv4_trie();
    let lctrie = trie.clone().compress();

    bencher.iter(|| (0..1_000_000_000).for_each(|i| {
        let addr : Ipv4Addr = (i * 4).into();
        let _ = lctrie.lookup(&addr);
    }))
}
