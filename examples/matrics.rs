use std::net::Ipv6Addr;
use ipnet::Ipv6Net;
use ip_trie::*;

fn main() {

  /*  let prefixes = [
        "2c0f:ffc8::",
        "2c0f:ffc8:6000::",
        "2c0f:ffc8:6001::",
        "2c0f:ffc8:6002::",
        "2c0f:ffc8:6003::",
        "2c0f:ffc8:6004::",
        "2c0f:ffc8:6005::",
        "2c0f:ffc8:6006::",
        "2c0f:ffc8:6007::",
        "2c0f:ffc8:6008::",
        "2c0f:ffc8:6009::",
        "2c0f:ffc8:6010::",
        "2c0f:ffc8:6011::",
        "2c0f:ffc8:6012::",
        "2c0f:ffc8:6013::",
        "2c0f:ffc8:6014::",
        "2c0f:ffc8:6015::",
        "2c0f:fff0::",
        "ff02::2",
    ];*/

    let prefixes = [
        "::0",
        "::1",
        "::2",
        "::5",
        "::7",
        "::8",
        "::9",
        "::a",
        "::c",
        "::d",
        "::e",
        "::f",
    ];
    let mut trie = Ipv6RTrieSet::with_capacity(20);

    prefixes.iter()
        .for_each(|x| {
            let p = x.parse::<Ipv6Addr>().unwrap();
            trie.insert(p);
        });

    trie.open_dot_view().expect("can’t open dot view");
    trie.compress().open_dot_view().expect("can’t open dot view");


}