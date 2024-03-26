use sailce_data_model::{
    group::{
        Range,
        ThreeDimRange,
    },
    Entry,
    Timestamp,
};


fn D3<S, P>(
    sr: impl Into<Range<S>>,
    pr: impl Into<Range<P>>,
    tr: impl Into<Range<Timestamp>>,
) -> ThreeDimRange<S, P>
{
    ThreeDimRange { subspaces: sr.into(), paths: pr.into(), times: tr.into() }
}


const fn E<S, P>(
    subspace_id: S,
    path: P,
    timestamp: u64,
) -> Entry<u128, S, P, ()>
{
    Entry {
        namespace_id: 0,
        subspace_id,
        path,
        timestamp: Timestamp { μs_since_epoch: timestamp },
        payload_digest: (),
        payload_length: 0,
    }
}


#[test]
fn ordering()
{
    // Empty ranges.
    let subspaces = Range::from(0_u16 .. 0);
    let paths = Range::from(&[0_i8; 0][..] .. &[][..]);
    let times = Range::from(Timestamp { μs_since_epoch: 0 } .. Timestamp { μs_since_epoch: 0 });
    let empty = ThreeDimRange { subspaces, paths, times };

    assert_eq!(empty, empty);

    assert!(empty < ThreeDimRange { times: (0 .. 1).into(), ..empty });
    assert!(empty < ThreeDimRange { times: (1 .. 0).into(), ..empty });
    assert!(empty < ThreeDimRange { paths: (&[][..] .. &[1][..]).into(), ..empty });
    assert!(empty < ThreeDimRange { paths: (&[1][..] .. &[][..]).into(), ..empty });
    assert!(empty < ThreeDimRange { subspaces: (0 .. 1).into(), ..empty });
    assert!(empty < ThreeDimRange { subspaces: (1 .. 0).into(), ..empty });

    assert!(empty < ThreeDimRange { times: (0 ..).into(), ..empty });
    assert!(empty < ThreeDimRange { paths: (&[][..] ..).into(), ..empty });
    assert!(empty < ThreeDimRange { subspaces: (0 ..).into(), ..empty });
}


#[test]
fn includes()
{
    macro_rules! case {
                    ([$($r:expr),*] : { $($e:expr),* }) => {
                        assert!(D3($($r),*).includes(E($($e),*)));
                    };
                    ([$($r:expr),*] ! { $($e:expr),* }) => {
                        assert!(!D3($($r),*).includes(E($($e),*)));
                    };
                }

    case!([(0..0), (""..""),  (0..0)] ! {0,"",0});
    case!([(0..0), (""..""),  (0..1)] ! {0,"",0});
    case!([(0..0), ("".."a"), (0..0)] ! {0,"",0});
    case!([(0..1), (""..""),  (0..0)] ! {0,"",0});
    case!([(0..1), ("".."a"), (0..1)] : {0,"",0});
    case!([('m'..), (["foo"]..), (123..)] ! {'a',["bar"],42});
    case!([('m'..), (["foo"]..), (123..)] : {'o',["zab"],456});
}


#[test]
fn is_empty()
{
    let empty = ThreeDimRange {
        subspaces: (0 .. 0).into(),
        paths:     ("" .. "").into(),
        times:     (0 .. 0).into(),
    };
    // Non-empty ranges.
    let subspaces = Range::<i32>::from(0 .. 100);
    let paths = Range::from("a" .. "bb");
    let times = Range::<Timestamp>::from(0 .. 100);

    assert!(empty.is_empty());
    assert!(ThreeDimRange { times, ..empty }.is_empty());
    assert!(ThreeDimRange { paths, ..empty }.is_empty());
    assert!(ThreeDimRange { paths, times, ..empty }.is_empty());
    assert!(ThreeDimRange { subspaces, ..empty }.is_empty());
    assert!(ThreeDimRange { subspaces, paths, ..empty }.is_empty());
    assert!(ThreeDimRange { subspaces, times, ..empty }.is_empty());
    assert!(!ThreeDimRange { subspaces, paths, times }.is_empty());
}


#[test]
fn intersection()
{
    macro_rules! case {
                    ([$($r1:expr),*] & [$($r2:expr),*] == [$($r3:expr),*]) => {
                        assert_eq!(D3($($r1),*).intersection(D3($($r2),*)), D3($($r3),*));
                    };
                }

    case!(
        [(0 .. 0), ("" .. ""), (0 .. 0)] & [(0 .. 0), ("" .. ""), (0 .. 0)]
            == [(0 .. 0), ("" .. ""), (0 .. 0)]
    );
    case!(
        [(0 .. 1), ("" .. ""), (1 .. 2)] & [(1 .. 0), ("a" .. "b"), (3 .. 4)]
            == [(1 .. 0), ("a" .. ""), (3 .. 2)]
    );
    case!(
        [('c' .. 'f'), (["az"] .. ["za"]), (12 .. 34)]
            & [('a' .. 'h'), (["b"] .. ["x"]), (23 .. 45)]
            == [('c' .. 'f'), (["b"] .. ["x"]), (23 .. 34)]
    );
    case!(
        [('c' ..), (["az"] .. ["za"]), (12 ..)] & [('a' .. 'h'), (["b"] ..), (23 .. 45)]
            == [('c' .. 'h'), (["b"] .. ["za"]), (23 .. 45)]
    );
}
