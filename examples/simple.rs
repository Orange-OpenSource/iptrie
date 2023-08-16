use ipnet::Ipv4Net;
use iptrie::{DotWriter, Ipv4RTrieSet};
use iptrie::Ipv4Prefix;

fn main() {

    let prefixes = [
        "1.1.0.0/24",
        "1.1.1.0/24",
        "1.1.0.0/20",
        "1.1.2.0/24",
        "1.1.0.0/20",
        "1.1.3.0/24",
        "1.3.1.0/24",
        "1.3.0.0/20",
        "1.4.1.0/24",
        "1.5.1.0/24",
        "2.2.1.0/24",
        "2.2.2.0/24",
        "2.1.0.0/20",
        "2.3.0.0/20"
    ];

    let iter = prefixes.iter().map(|x| x.parse::<Ipv4Net>().unwrap());
    let trie = Ipv4RTrieSet::from_iter( iter.map(Ipv4Prefix::from));

    trie.generate_pdf_file(Some("simple-radixtrie.pdf")).expect("can’t generate PDF file");
    trie.compress().generate_pdf_file(Some("simple-lctrie.pdf")).expect("can’t generate PDF file");

}