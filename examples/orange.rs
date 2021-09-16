use iptrie::*;
use std::net::Ipv4Addr;

fn main() {

    let prefixes = [
        "1.1.0.0/24",
        "1.1.1.0/24",
        "1.1.2.0/24",
        "1.1.3.0/24",
        "1.1.0.0/20",
        "1.2.1.0/24",
        "2.1.1.0/24",
    ];
    let mut trie = IpPrefixMap::with_root(20);

    prefixes.iter()
        .map(|x| x.parse::<IpPrefixLtd::<Ipv4>>().unwrap())
        .enumerate()
        .for_each(|(i,p)| { trie.insert(p, i*100+7);});

    let addr = [
        "1.1.1.1".parse::<Ipv4Addr>().unwrap(),
        "1.1.1.13".parse::<Ipv4Addr>().unwrap(),
        "1.1.1.3".parse::<Ipv4Addr>().unwrap(),
        "1.1.2.3".parse::<Ipv4Addr>().unwrap(),
        "1.1.3.3".parse::<Ipv4Addr>().unwrap(),
        "1.2.2.2".parse::<Ipv4Addr>().unwrap(),
        "1.2.1.2".parse::<Ipv4Addr>().unwrap(),
    ];
    addr.iter()
        .for_each(|a| {
            let (k,v) = trie.lookup(a);
            println!("{} -> ({},{})", a, k, v);
        });

    trie.open_dot_view().expect("can’t open dot view");
    println!();

    let trie = trie.compile();
    addr.iter()
        .for_each(|a| {
            let (k,v) = trie.lookup(a);
            println!("{} -> ({},{})", a, k, v);
        });

    trie.open_dot_view().expect("can’t open dot view");

    /*
        let mut trie  = www.iter()
            .map(|s| FQDN::from_str_without_trailing_dot(s))
            .collect::<Result<FqdnTrieSet<_>,_>>()
            .unwrap();

        //trie.open_dot_view().expect("can’t open dot view");

        www.iter()
            .step_by(2)
            .map(|s| FQDN::from_str_without_trailing_dot(s).unwrap())
            .for_each(|s| {trie.remove(&s);});

        //trie.open_dot_view().expect("can’t open dot view");
    */

}