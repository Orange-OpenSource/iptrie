
use super::*;
use ipnet::Ipv6Net;
use rand::*;
use rand::distributions::*;

#[test]
fn slot_mask()
{
    assert_eq!( 0,          u64::bitmask(0) );
    assert_eq!( 1 << 63,    u64::bitmask(1) );
    assert_eq!( u64::MAX,   u64::bitmask(64) );
}

#[test]
fn slot_root() {
    assert_eq!( u32::from(Ipv4Prefix::root().network()), 0);
    assert_eq!( Ipv4Prefix::root().len(), 0);

    assert_eq!( u128::from(Ipv6Prefix::root().network()), 0);
    assert_eq!( Ipv6Prefix::root().len(), 0);
    assert_eq!(u128::from(Ipv6Prefix120::root().network()), 0);
    assert_eq!( Ipv6Prefix::root().len(), 0);
    assert_eq!(u128::from(Ipv6Prefix56::root().network()), 0);
    assert_eq!( Ipv6Prefix::root().len(), 0);
}

#[test]
fn parse_errors() {
    assert_eq!( "1::/12".parse::<Ipv4Prefix>(), Err(IpPrefixError::AddrParseError));
    assert_eq!( "1.1.1.1/12".parse::<Ipv6Prefix56>(), Err(IpPrefixError::AddrParseError));
}


#[test]
fn prefix_ipv4_trunc()
{
    let mut rng = thread_rng();
    let len = Uniform::<u8>::from(0..=32);
    let addr = Uniform::<u32>::from(0..=u32::MAX);

    (0..10_000).for_each(|_| {
        let addr = Ipv4Addr::from(addr.sample(&mut rng));
        let len = len.sample(&mut rng);
        let ipnet = Ipv4Prefix::new(addr, len).unwrap();
        assert_eq!( ipnet.bitslot_trunc(), ipnet.bitslot() & ipnet.bitmask())
    })
}

fn prefix_ipv6_trunc<P>()
    where
        P:IpPrefix<Addr=Ipv6Addr> + TryFrom<Ipv6Net>,
        <P as TryFrom<Ipv6Net>>::Error: Debug
{
    let mut rng = thread_rng();
    let len = Uniform::<u8>::from(0..=P::MAX_LEN);
    let addr = Uniform::<u128>::from(0..=u128::MAX);

    (0..10_000).for_each(|_| {
        let addr = Ipv6Addr::from(addr.sample(&mut rng));
        let len = len.sample(&mut rng);
        let ipnet = P::try_from(Ipv6Net::new(addr, len).unwrap()).unwrap();
        assert_eq!( ipnet.bitslot_trunc(), ipnet.bitslot() & ipnet.bitmask())
    })
}

#[test] fn prefix_ipv6_128_trunc() { prefix_ipv6_trunc::<Ipv6Prefix>() }
#[test] fn prefix_ipv6_120_trunc() { prefix_ipv6_trunc::<Ipv6Prefix120>() }
#[test] fn prefix_ipv6_56_trunc() { prefix_ipv6_trunc::<Ipv6Prefix56>() }


#[test]
fn prefix_eq()
{
    let mut rng = thread_rng();
    let len = Uniform::<u8>::from(0..=128);
    let addr = Uniform::<u128>::from(0..=u128::MAX);

    (0..10_000).for_each(|_| {
        let addr = Ipv6Addr::from(addr.sample(&mut rng));
        let len = len.sample(&mut rng);
        let ipnet = Ipv6Prefix::new(addr, len).unwrap();
        let ipnet2 = Ipv6Net::new(addr, len).unwrap();
        assert_eq!( ipnet.network(), ipnet2.network());
        if ipnet.len() <= 120 {
            let ipnet120 = Ipv6Prefix120::try_from(ipnet).unwrap();
            assert!( ipnet120.covers_equally(&ipnet) );
            assert!( ipnet.covers_equally(&ipnet120) );
            if ipnet.len() <= 56 {
                let ipnet56a =  Ipv6Prefix56::try_from(ipnet).unwrap();
                let ipnet56b =  Ipv6Prefix56::try_from(ipnet120).unwrap();

                assert!( ipnet56a.covers_equally(&ipnet) );
                assert!( ipnet56a.covers_equally(&ipnet120) );
                assert!( ipnet.covers_equally(&ipnet56b) );
                assert!( ipnet120.covers_equally(&ipnet56b) );

                assert_eq!( ipnet120.bitslot_trunc(), Ipv6Prefix120::from(ipnet56a).bitslot_trunc());
                assert_eq!( ipnet120.bitslot(), Ipv6Prefix120::from(ipnet56a).bitslot());
                assert_eq!( ipnet120.len(), Ipv6Prefix120::from(ipnet56a).len());
                assert_eq!( ipnet120, Ipv6Prefix120::from(ipnet56a));
                assert_eq!( Ipv6Prefix120::from(ipnet56b), ipnet120);

                assert!( ipnet56a.covers_equally(&Ipv6Prefix120::from(ipnet56a)));
                assert!( Ipv6Prefix120::from(ipnet56b).covers_equally(&ipnet56b));
            } else {
                assert_eq!( Ipv6Prefix56::try_from(ipnet), Err(IpPrefixError::PrefixLenError))
            }
        } else {
            assert_eq!( Ipv6Prefix120::try_from(ipnet), Err(IpPrefixError::PrefixLenError))
        }
    })
}


#[test]
fn prefix_cover()
{
    let mut rng = thread_rng();
    let len = Uniform::<u8>::from(0..=120);
    let addr = Uniform::<u128>::from(0..=u128::MAX);

    (0..10_000).for_each(|_| {
        let addr = Ipv6Addr::from(addr.sample(&mut rng));
        let l1 = len.sample(&mut rng);
        let l2 = len.sample(&mut rng);

        let a1 = Ipv6Prefix::new(addr, l1).unwrap();
        let a2 = Ipv6Prefix::new(addr, l2).unwrap();
        let b1 = Ipv6Prefix120::new(addr, l1).unwrap();
        let b2 = Ipv6Prefix120::new(addr, l2).unwrap();

        assert_eq!( a1.covers(&a2), l1 <= l2 );
        assert_eq!( a1.covers(&b2), l1 <= l2 );
        assert_eq!( b1.covers(&a2), l1 <= l2 );
        assert_eq!( b1.covers(&b2), l1 <= l2 );

        assert_eq!( a2.covers(&a1), l1 >= l2 );
        assert_eq!( a2.covers(&b1), l1 >= l2 );
        assert_eq!( b2.covers(&a1), l1 >= l2 );
        assert_eq!( b2.covers(&b1), l1 >= l2 );
    })
}
