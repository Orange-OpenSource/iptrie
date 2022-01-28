use std::net::Ipv6Addr;
use ipnet::Ipv6Net;
use iptrie::*;

fn main() {

    let prefixes = [
        "2c0f:ffc8::",
        "2c0f:ffc8:6000::",
        "2c0f:ffc8:6001::",
        "2c0f:ffc8:6002::",
        "2c0f:ffc8:6006::",
        "2c0f:ffc8:6006::3",
        "2c0f:fff0::",
        "ff02::2",
    ];
    let mut trie = RTrieSet::with_capacity(20);

    prefixes.iter()
        .for_each(|x| {
            let p = x.parse::<Ipv6Addr>().unwrap();
            trie.insert(p);
        });

    trie.open_dot_view().expect("can’t open dot view");
    trie.compress().open_dot_view().expect("can’t open dot view");


}