#![feature(test)]
extern crate test;

use std::iter::repeat_with;
use std::net::Ipv6Addr;
use test::Bencher;

use ipnet::*;
use iptrie::*;
use iptrie::set::RTrieSet;
use  ip_network_table_deps_treebitmap::IpLookupTable;

fn random_ipv6net() -> impl Iterator<Item=Ipv6Net>
{
    use rand::*;
    use rand::distributions::*;
    let mut rng = thread_rng();
    let prefix = Uniform::<u8>::from(8..=50);
    let addr = Uniform::<u128>::from(1..=(u128::MAX >> 80));
    repeat_with( move || {
        let addr = addr.sample(&mut rng) << 80;
        Ipv6Net::new(addr.into(), prefix.sample(&mut rng)).unwrap()
    }).take(10_000_000)
}


fn random_ipv6addr() -> impl Iterator<Item=Ipv6Addr>
{
    use rand::*;
    use rand::distributions::*;
    let mut rng = thread_rng();
    let addr = Uniform::<u128>::from(1..=u128::MAX);
    repeat_with(move || Ipv6Addr::from(addr.sample(&mut rng)))
}



#[bench]
fn nop_ipv6prefix_trie(bencher: &mut Bencher)
{
    let mut sample = random_ipv6addr();
    let mut result = Vec::with_capacity(1_000);
    bencher.iter(|| result.push(sample.next().unwrap()) );
    println!("{}", result.len());
}


#[bench]
fn lookup_ipv6prefix_trie(bencher: &mut Bencher)
{
    let trie: Ipv6RTrieSet = random_ipv6net().map(Ipv6Prefix::from).collect();
    let mut sample = random_ipv6addr();
    let mut result = Vec::with_capacity(1_000);
    bencher.iter(|| result.push(trie.lookup(&sample.next().unwrap())) );
    println!("{}", result.len());
}

#[bench]
fn lookup_ipv6netprefix_trie(bencher: &mut Bencher)
{
    let trie: RTrieSet<_> = random_ipv6net()
        .map(Ipv6NetPrefix::try_from)
        .collect::<Result<_,_>>()
        .unwrap();
    let mut sample = random_ipv6addr();
    let mut result = Vec::with_capacity(1_000);
    bencher.iter(|| result.push(trie.lookup(&sample.next().unwrap())) );
    println!("{}", result.len());
}

#[bench]
fn lookup_ipv6net_trie(bencher: &mut Bencher)
{
    let trie: RTrieSet<Ipv6Net> = random_ipv6net().collect();
    let mut sample = random_ipv6addr();
    let mut result = Vec::with_capacity(1_000);
    bencher.iter(|| result.push(trie.lookup(&sample.next().unwrap())) );
    println!("{}", result.len());
}

#[bench]
fn lookup_ipv6net_treebit(bencher: &mut Bencher)
{
    let trie = random_ipv6net()
        .fold(IpLookupTable::<Ipv6Addr,()>::with_capacity(1_000_000),
              |mut trie,p| { trie.insert(p.network(), p.len() as u32, ()); trie });
    let mut sample = random_ipv6addr();
    let mut result = Vec::with_capacity(1_000);
    bencher.iter(|| result.push(trie.longest_match(sample.next().unwrap())) );
    println!("{}", result.len());
}



#[bench]
fn lookup_ipv6prefix_lctrie(bencher: &mut Bencher)
{
    let trie: Ipv6RTrieSet = random_ipv6net().map(Ipv6Prefix::from).collect();
    let trie = trie.compress();
    let mut sample = random_ipv6addr();
    let mut result = Vec::with_capacity(1_000);
    bencher.iter(|| result.push(trie.lookup(&sample.next().unwrap())) );
    println!("{}", result.len());
}

#[bench]
fn lookup_ipv6netprefix_lctrie(bencher: &mut Bencher)
{
    let trie: RTrieSet<_> = random_ipv6net()
        .map(Ipv6NetPrefix::try_from)
        .collect::<Result<_,_>>()
        .unwrap();
    let trie = trie.compress();
    let mut sample = random_ipv6addr();
    let mut result = Vec::with_capacity(1_000);
    bencher.iter(|| result.push(trie.lookup(&sample.next().unwrap())) );
    println!("{}", result.len());
}

#[bench]
fn lookup_ipv6net_lctrie(bencher: &mut Bencher)
{
    let trie: RTrieSet<Ipv6Net> = random_ipv6net().collect();
    let trie = trie.compress();
    let mut sample = random_ipv6addr();
    let mut result = Vec::with_capacity(1_000);
    bencher.iter(|| result.push(trie.lookup(&sample.next().unwrap())) );
    println!("{}", result.len());
}


