use iptrie::*;
use iptrie::graphviz::DotWriter;

fn main() {

    let prefixes = [
        "2c0f:ffc8::/49",
        "2c0f:ffc8:6000::/46",
        "2c0f:fff0::/32",
        "ff02::2/128",
    ];
    let mut trie = IpPrefixSet::with_capacity(20);

    prefixes.iter()
        .for_each(|x| {
            let p = x.parse::<IpWholePrefix<Ipv6>>().unwrap();
            trie.insert(p);
        });

    trie.open_dot_view().expect("can’t open dot view");
    let trie = IpPrefixLCSet::new(trie);

}