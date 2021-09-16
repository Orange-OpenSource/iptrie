use std::{env, io};
use iptrie::*;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::ptr::null_mut;

static EMPTY_PREFIX: &str = "<empty prefix>";

type Ipv4Prefix = IpWholePrefix<Ipv4>;
type Ipv6Prefix = IpWholePrefix<Ipv6>;

fn main() {
    let file = env::args().skip(1).next().expect("needs a LPM file");

    let fd = unsafe {
        let file = CString::new(file.clone()).unwrap();
        libc::open(file.as_c_str().as_ptr(), libc::O_RDONLY)
    };
    if fd == -1 {
        panic!("FATAL: can't open LPM file {}", file);
    }
    let length = {
        let mut sb= libc::stat {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_ino: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_atime: 0,
            st_atime_nsec: 0,
            st_mtime: 0,
            st_mtime_nsec: 0,
            st_ctime: 0,
            st_ctime_nsec: 0,
            st_birthtime: 0,
            st_birthtime_nsec: 0,
            st_size: 0,
            st_blocks: 0,
            st_blksize: 0,
            st_flags: 0,
            st_gen: 0,
            st_lspare: 0,
            st_qspare: [0;2]
        };
        unsafe {
            if libc::fstat(fd, &mut sb as * mut libc::stat) == -1 {
                panic!("FATAL: can't access to LPM file {}", file);
            }
        }
        sb.st_size as usize
    };

    let lpmfile = unsafe {
        libc::mmap(null_mut(), length, libc::PROT_READ, libc::MAP_PRIVATE, fd, 0) as *const c_char
    };
    if lpmfile.is_null() {
        panic!("FATAL: can't load LPM file {}", file);
    }
    let lpmfile = unsafe { std::str::from_utf8(CStr::from_ptr(lpmfile).to_bytes()) }.unwrap();


    let mut map4 = IpPrefixMap::<Ipv4,Ipv4Prefix,&str>::with_root_and_capacity(EMPTY_PREFIX, 2000000);
    let mut map6 = IpPrefixMap::<Ipv6,Ipv6Prefix,&str>::with_root_and_capacity(EMPTY_PREFIX, 2000000);

    lpmfile.split('\n').into_iter()
        //.take(10000)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))// skip empty and comment lines
        .map(|s| (s, s.split_ascii_whitespace().into_iter().next().expect("bad formatted line")))
        .for_each(| (line, prefix) | {
            if let Ok(pfx) = prefix.parse::<Ipv4Prefix>() {
                map4.insert(pfx, line);
            } else if let Ok(pfx) = prefix.parse::<Ipv6Prefix>() {
                map6.insert(pfx, line);
            } else {
                println!("WARN: skip bad formatted line: {}", line);
            }
        });
    //map6.generate_graphviz_file("ipv6".into());
    //maq’p6.open_dot_view();
    let map4 = map4.compile();
    let map6 = map6.compile();
    println!("ready ....");
    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) =>  break,
            Ok(_) => {
                let input = &input[..(input.len()-1)];

                if let Ok(addr) = input.parse::<Ipv4Addr>() {
                    println!("{}", map4.lookup(&addr).1);
                } else if let Ok(addr) = input.parse::<Ipv6Addr>() {
                    println!("{}", map6.lookup(&addr).1);
                } else {
                    println!("WARN: can’t parse '{}' (not an IP address)", input);
                }
            }
            Err(error) => panic!("{}", error)
        }
    }
}