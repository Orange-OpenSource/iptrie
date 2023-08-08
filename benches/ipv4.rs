#![feature(test)]
extern crate test;

use std::collections::HashSet;
use std::net::Ipv4Addr;
use test::Bencher;
use ipnet::Ipv4Net;
use iptrie::*;

fn build_ipv4_samples() -> HashSet<Ipv4Net>
{
    (0..u32::MAX/512)
        .into_iter()
        .map(|i| Ipv4Addr::from(i*128))
        .map(|a| {
            (16..=28).into_iter().map(move |p| Ipv4Net::new(a,p).unwrap())
        })
        .flatten()
        .collect()
}

fn build_one_ipv4_trie(sample: &HashSet<Ipv4Net>) -> Ipv4RTrieSet
{
    let n = 2_000_000;
    let mut trie = Ipv4RTrieSet::with_capacity(n);
    let mut iter = sample.iter().map(|i| *i);
    while trie.len().get() < n {
        trie.insert(iter.next().unwrap());
    }
    trie
}

//#[bench]
fn build_ipv4_trie(bencher: &mut Bencher)
{
    let sample = build_ipv4_samples();
    bencher.iter(|| build_one_ipv4_trie(&sample) )
}


//#[bench]
fn compile_ipv4_lctrie(bencher: &mut Bencher)
{
    let sample = build_ipv4_samples();
    let trie = build_one_ipv4_trie(&sample);
    bencher.iter(|| {
        let _ = trie.clone().compress();
    } )
}

#[bench]
fn lookup_ipv4_trie(bencher: &mut Bencher)
{
    let sample = build_ipv4_samples();
    let trie = build_one_ipv4_trie(&sample);
    bencher.iter(|| (0..u32::MAX).for_each(|i| {
        let addr : Ipv4Addr = i.into();
        let _ = trie.lookup(addr);
    }))
}



#[bench]
fn lookup_ipv4_lctrie(bencher: &mut Bencher)
{
    let sample = build_ipv4_samples();
    let trie = build_one_ipv4_trie(&sample);
    let lctrie = trie.compress();
    eprintln!("{}", lctrie.len());
    bencher.iter(|| (0..u32::MAX).for_each(|i| {
        let addr : Ipv4Addr = i.into();
        let _ = lctrie.lookup(addr);
    }))
}

