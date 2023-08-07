use ipnet::Ipv4Net;
use iptrie::{DotWriter, Ipv4RTrieSet};

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
    let mut trie = Ipv4RTrieSet::new();

    prefixes.iter()
        .for_each(|x| {
            let mut x = x.split('|');
            let p = x.next().unwrap().parse::<Ipv4Net>().unwrap();
            trie.insert(p);
        });

    trie.generate_pdf_file(Some("simple-radixtrie.pdf")).expect("can’t generate PDF file");
    trie.compress().generate_pdf_file(Some("simple-lctrie.pdf")).expect("can’t generate PDF file");
}