#![feature(test)]
extern crate test;

use std::iter::repeat_with;
use std::net::Ipv4Addr;
use test::Bencher;

use ipnet::*;
use iptrie::*;
use  ip_network_table_deps_treebitmap::IpLookupTable;

fn random_ipv4net() -> impl Iterator<Item=Ipv4Net>
{
    use rand::*;
    use rand::distributions::*;
    let mut rng = thread_rng();
    let prefix = Uniform::<u8>::from(8..=24);
    let addr = Uniform::<u32>::from(1..=(u32::MAX>>8));
    repeat_with(move || {
        let addr = addr.sample(&mut rng) << 8;
        Ipv4Net::new(addr.into(), prefix.sample(&mut rng)).unwrap()
    }).take(1_000_000)
}


fn random_ipv4addr() -> impl Iterator<Item=Ipv4Addr>
{
    use rand::*;
    use rand::distributions::*;
    let mut rng = thread_rng();
    let addr = Uniform::<u32>::from(1..=u32::MAX);
    repeat_with(move || Ipv4Addr::from(addr.sample(&mut rng)))
}


#[bench]
fn nop_ipv4prefix_trie(bencher: &mut Bencher)
{
    let mut sample = random_ipv4addr();
    let mut result = Vec::with_capacity(1_000);
    bencher.iter(|| result.push(sample.next().unwrap()));
    println!("{}", result.len());
}


#[bench]
fn lookup_ipv4prefix_trie(bencher: &mut Bencher)
{
    let trie: Ipv4RTrieSet = random_ipv4net().map(Ipv4Prefix::from).collect();
    let mut sample = random_ipv4addr();
    let mut result = Vec::with_capacity(1_000);
    bencher.iter(|| result.push(trie.lookup(&sample.next().unwrap())) );
    println!("{}", result.len());
}

#[bench]
fn lookup_ipv4net_trie(bencher: &mut Bencher)
{
    let trie: RTrieSet<Ipv4Net> = random_ipv4net().collect();
    let mut sample = random_ipv4addr();
    let mut result = Vec::with_capacity(1_000);
    bencher.iter(|| result.push(trie.lookup(&sample.next().unwrap())) );
    println!("{}", result.len());
}

#[bench]
fn lookup_ipv4prefix_lctrie(bencher: &mut Bencher)
{
    let trie: Ipv4RTrieSet = random_ipv4net().map(Ipv4Prefix::from).collect();
    let trie = trie.compress();
    let mut sample = random_ipv4addr();
    let mut result = Vec::with_capacity(1_000);
    bencher.iter(|| result.push(trie.lookup(&sample.next().unwrap())) );
    println!("{}", result.len());
}

#[bench]
fn lookup_ipv4net_lctrie(bencher: &mut Bencher)
{
    let trie: RTrieSet<Ipv4Net> = random_ipv4net().collect();
    let trie = trie.compress();
    let mut sample = random_ipv4addr();
    let mut result = Vec::with_capacity(1_000);
    bencher.iter(|| result.push(trie.lookup(&sample.next().unwrap())) );
    println!("{}", result.len());
}

#[bench]
fn lookup_ipv4net_treebit(bencher: &mut Bencher)
{
    let trie = random_ipv4net()
        .fold(IpLookupTable::<Ipv4Addr,()>::with_capacity(1_000_000),
              |mut trie,p| { trie.insert(p.network(), p.len() as u32, ()); trie });
    let mut sample = random_ipv4addr();
    let mut result = Vec::with_capacity(1_000);
    bencher.iter(|| result.push(trie.longest_match(sample.next().unwrap())) );
    println!("{}", result.len());
}

