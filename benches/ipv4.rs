#![feature(test)]
extern crate test;

use std::iter;
use std::net::Ipv4Addr;
use test::Bencher;
use iptrie::*;

fn build_one_ipv4_trie() -> IpWholePrefixSet<Ipv4>
{
    (0..100_000).into_iter()
        .fold(IpWholePrefixSet::with_capacity(200_000),
              |mut trie, i| {
                  trie.insert(IpWholePrefix::<Ipv4>::new(((i*1000)+250).into(), 25.try_into().unwrap()));
                  trie.insert(IpWholePrefix::new(((i*1000)+500).into(), 20.try_into().unwrap()));
                  trie.insert(IpWholePrefix::new((i*1000).into(), 18.try_into().unwrap()));
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
        let _ = IpWholePrefixLCSet::new(trie.clone());
    } )
}


#[bench]
fn lookup_ipv4_lctrie(bencher: &mut Bencher)
{
    let trie = build_one_ipv4_trie();
    let lctrie = IpWholePrefixLCSet::new(trie);

    bencher.iter(|| (0..1_000_000_000).for_each(|i| {
        let addr : Ipv4Addr = (i * 4).into();
        let _ = lctrie.lookup(&addr);
    }))
}
