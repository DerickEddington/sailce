use sailce_data_model::{
    group::{
        area::Subspace::{
            self,
            Any,
            Id,
        },
        Area,
        Range,
    },
    Entry,
    Timestamp,
};


fn A<'p, const N: usize>(
    subspace: Subspace<i32>,
    path: &'p [&'static str; N],
    times: impl Into<Range<Timestamp>>,
) -> Area<i32, &'p [&'static str]>
{
    Area { subspace, path: &path[..], times: times.into() }
}


mod subspace
{
    use sailce_data_model::group::area::Subspace::{
        Any,
        Id,
    };

    #[test]
    fn ordering()
    {
        assert_eq!(Id('λ'), Id('λ'));
        assert_ne!(Id(1), Id(2));
        assert_ne!(Id(""), Any);
        assert_eq!(Any::<u16>, Any);
        assert!(Id(0) < Id(1));
        assert!(Id(u128::MAX) < Any);
    }
}


#[test]
fn ordering()
{
    let empty = A(Any, &[], 0 .. 0);

    assert_eq!(empty, empty);
    assert_ne!(A(Id(0), &[], 0 .. 0), empty);
    assert_ne!(A(Any, &[""], 0 .. 0), empty);
    assert_ne!(A(Any, &[], 0 .. 1), empty);
    assert!(A(Id(0), &[], 0 .. 1) < A(Id(0), &[], 0 .. 2));
    assert!(A(Id(0), &[], 0 ..) < A(Id(0), &["foo"], 0 ..));
    assert!(A(Id(-42), &[], 0 ..) < A(Id(42), &[], 0 ..));
    assert!(A(Id(i32::MAX), &["foo", "bar"], 123 .. 456) < A(Any, &[], 0 .. 1));
}


#[test]
fn includes()
{
    let empty: Area<i32, [[u8; 0]; 0]> = Area::empty();
    let full: Area<i32, &[&str]> = Area::full();
    let e = Entry {
        namespace_id:   "widgets",
        subspace_id:    123,
        path:           &["ab", "c"][..],
        timestamp:      9876.into(),
        payload_digest: [42_u8; 16],
        payload_length: 5678,
    };

    assert!(!empty.includes(e));
    assert!(full.includes::<Entry<_, _, _, _>>(&e));
    assert!(Area::<_, [&[u8]; 0]>::subspace(123).includes(e));
    assert!(!Area::<i32, &[[u8; 1]; 0]>::subspace(456).includes(e));
    assert!(A(Id(123), &["ab"], 9000 .. 10000).includes(e));
    assert!(!A(Id(123), &["ab"], 9 .. 10).includes(e));
    assert!(!A(Any, &["ab", "c", ""], 9000 .. 10000).includes(e));
    assert!(A(Any, &["ab", "c"], 9876 .. 9877).includes(e));

    let a = A(Id(99), &["z", "yx", "wvu"], 789 ..);

    assert!(!empty.includes::<Area<_, _>>(&a));
    assert!(full.includes(a));
    assert!(Area::<_, &[&[u8]]>::subspace(99).includes(a));
    assert!(!Area::<_, [&str; 0]>::subspace(100).includes(a));
    assert!(a.includes(a));
    assert!(A(Id(99), &["z", "yx", "wvu"], 5 ..).includes(a));
    assert!(!A(Any, &[], 0 .. u64::MAX).includes(a));
    assert!(A(Any, &["z"], 0 ..).includes(a));
    assert!(!A(Id(99), &["z", "a"], 0 ..).includes(a));
    assert!(A(Id(99), &["z"], 0 ..).includes(a));
    assert!(!A(Id(1), &[], 0 ..).includes(a));
    assert!(!a.includes(A(Any, &["z", "yx", "wvu"], 1000 .. 2000)));
    assert!(a.includes(A(Id(99), &["z", "yx", "wvu", "tsrq"], 1000 .. 2000)));

    let sub = Area::<_, &[&str]>::subspace(3);

    assert!(empty.includes(empty));
    assert!(!empty.includes(full));
    assert!(!empty.includes(sub));
    assert!(full.includes(full));
    assert!(full.includes(empty));
    assert!(full.includes(sub));
    assert!(sub.includes(sub));
    assert!(sub.includes(empty));
    assert!(!sub.includes(full));

    let a2 = A(Any, &[], 1 .. 2);
    assert!(a2.includes(a2));

    #[cfg(feature = "alloc")]
    #[allow(clippy::shadow_unrelated)]
    {
        use std::collections::LinkedList;

        let full: Area<i32, LinkedList<String>> = Area::full();

        assert!(full.includes(e));
    }
}


#[test]
fn is_empty()
{
    assert!(Area::<&[char], &[&str]>::empty().is_empty());
    assert!(!Area::<bool, [[u8; 1]; 0]>::full().is_empty());
    assert!(!Area::<&str, &[&[u8]]>::subspace("bob").is_empty());
    assert!(A(Any, &["a"], 8 .. 8).is_empty());
    assert!(!A(Id(0), &["a"], 7 .. 8).is_empty());
}


#[test]
fn intersection()
{
    let empty = Area::empty();
    let full = Area::full();
    let sub = Area::subspace(3);
    let a1 = A(Any, &["bb"], 22 .. 33);
    let a2 = A(Id(4), &["bb", "ccc"], 27 ..);
    let a3 = A(Id(4), &[], 1 ..);

    assert_eq!(empty.intersection(empty), empty);

    assert_eq!(empty.intersection(full), empty);
    assert_eq!(full.intersection(empty), empty);
    assert_eq!(full.intersection(full), full);

    assert_eq!(empty.intersection(sub), empty);
    assert_eq!(sub.intersection(empty), empty);
    assert_eq!(full.intersection(sub), sub);
    assert_eq!(sub.intersection(full), sub);
    assert_eq!(sub.intersection(sub), sub);

    assert_eq!(empty.intersection(a1), empty);
    assert_eq!(a1.intersection(empty), empty);
    assert_eq!(full.intersection(a1), a1);
    assert_eq!(a1.intersection(full), a1);
    {
        let i = A(Id(3), &["bb"], 22 .. 33);
        assert_eq!(sub.intersection(a1), i);
        assert_eq!(a1.intersection(sub), i);
    }
    assert_eq!(a1.intersection(a1), a1);

    assert_eq!(empty.intersection(a2), empty);
    assert_eq!(a2.intersection(empty), empty);
    assert_eq!(full.intersection(a2), a2);
    assert_eq!(a2.intersection(full), a2);
    assert_eq!(sub.intersection(a2), empty);
    assert_eq!(a2.intersection(sub), empty);
    assert_eq!(a2.intersection(a2), a2);
    {
        let i = A(Id(4), &["bb", "ccc"], 27 .. 33);
        assert_eq!(a1.intersection(a2), i);
        assert_eq!(a2.intersection(a1), i);
    }

    assert_eq!(empty.intersection(a3), empty);
    assert_eq!(a3.intersection(empty), empty);
    assert_eq!(full.intersection(a3), a3);
    assert_eq!(a3.intersection(full), a3);
    assert_eq!(sub.intersection(a3), empty);
    assert_eq!(a3.intersection(sub), empty);
    assert_eq!(a3.intersection(a3), a3);
    {
        let i = A(Id(4), &["bb"], 22 .. 33);
        assert_eq!(a1.intersection(a3), i);
        assert_eq!(a3.intersection(a1), i);
    }
    {
        let i = A(Id(4), &["bb", "ccc"], 27 ..);
        assert_eq!(a2.intersection(a3), i);
        assert_eq!(a3.intersection(a2), i);
    }
}


mod of_interest;
