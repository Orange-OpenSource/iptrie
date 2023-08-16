use std::net::Ipv6Addr;
use ipnet::Ipv6Net;

use rand::*;
use rand::distributions::*;
use crate::*;


#[test]
fn ipv6_tries() {

    let mut rng = thread_rng();

    let samples = {
        let prefix = Uniform::<u8>::from(8..=50);
        let addr = Uniform::<u128>::from(1..=u128::MAX);
        std::iter::repeat_with(|| {
            Ipv6Net::new(addr.sample(&mut rng).into(), prefix.sample(&mut rng)).unwrap()
        }).take(1_000_000).collect::<Vec<_>>()
    };

    let t1: RTrieSet<Ipv6Prefix> = samples.iter().map(|i| Ipv6Prefix::from(*i)).collect();
    let t2: RTrieSet<Ipv6Prefix56> = samples.iter().map(|i| Ipv6Prefix56::try_from(*i).unwrap()).collect();
    let t3: RTrieSet<Ipv6Prefix120> = samples.iter().map(|i| Ipv6Prefix120::try_from(*i).unwrap()).collect();
    let t4: RTrieSet<Ipv6Net> = RTrieSet::from_iter(samples);

    let addr = Uniform::<u128>::from(1..=u128::MAX);
    std::iter::repeat_with(|| Ipv6Addr::from(addr.sample(&mut rng)))
        .take(1_000_000)
        .for_each(|ip| {
            let p1 = t1.lookup(&ip);
            let p2 = t2.lookup(&ip);
            let p3 = t3.lookup(&ip);
            let p4 = t4.lookup(&ip);
            assert!( p1.covers_equally(p2) );
            assert!( p2.covers_equally(p3) );
            assert!( p3.covers_equally(p4) );
        });
}