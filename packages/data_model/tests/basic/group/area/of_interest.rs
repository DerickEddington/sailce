use sailce_data_model::group::{
    area::{
        of_interest::Max,
        Subspace,
    },
    Area,
    AreaOfInterest,
};


mod max
{
    use {
        sailce_data_model::group::area::of_interest::Max,
        std::num::NonZeroU64,
    };

    #[test]
    fn ordering()
    {
        use Max::{
            Limit,
            Unlimited,
        };

        let one = Limit(1.try_into().unwrap());
        let big = Limit(NonZeroU64::MAX);

        assert_eq!(one, one);
        assert_ne!(one, big);
        assert_ne!(big, Unlimited);
        assert_eq!(Unlimited, Unlimited);
        assert!(one < big);
        assert!(big < Unlimited);
    }
}


#[cfg(feature = "alloc")]
#[test]
#[allow(clippy::too_many_lines)]
fn includes()
{
    use {
        crate::{
            payload::InMem as InMemPayload,
            store::{
                payload_and_digest,
                InMem as InMemStore,
                Path,
                Permission,
                User,
            },
        },
        sailce_data_model::{
            payload::sync::Payload as _,
            store::{
                sync::Store as _,
                Store,
            },
            AuthorisedEntry,
            EmptyPath as _,
            Entry,
        },
        std::{
            num::NonZeroU64,
            sync::Arc,
        },
    };

    let past = 1_000_000_000_000_000; // 2001-09-09T01:46:40+00:00

    let namespace_id = &"stuff".into();
    let user = User::new("somebody");
    let mut store = Store::new(namespace_id, InMemStore::new_block_on_pollster(namespace_id));

    let aoi1 = AreaOfInterest {
        area:      Area {
            subspace: Subspace::Id(user.clone()),
            path:     &[b"alpha"][..],
            times:    (past ..).into(),
        },
        max_count: Max::Limit(NonZeroU64::new(100).unwrap()),
        max_size:  Max::Limit(NonZeroU64::new(1000).unwrap()),
    };
    let (empty_payload, empty_payload_digest) = payload_and_digest([]);
    let entry1 = Entry {
        namespace_id:   Arc::clone(namespace_id),
        subspace_id:    user.clone(),
        path:           ["alpha", "beta"],
        timestamp:      (past + 123_456_789).into(),
        payload_digest: empty_payload_digest,
        payload_length: 0,
    };
    let auth_token_1 = Permission {
        user:       user.clone(),
        namespaces: [Arc::clone(namespace_id)].into(),
        subspaces:  [].into(), // Not needed with `user == subspace_id`.
        paths:      [Path::empty()].into(),
        times:      [(0 ..).into()].into(),
    };
    let ae1 =
        AuthorisedEntry::new(entry1.clone(), auth_token_1.clone()).expect("auth should succeed");

    assert!(!pollster::block_on(aoi1.includes(&entry1, &store)));
    assert_eq!(store.put(ae1, None::<InMemPayload>), Ok(()));
    assert!(pollster::block_on(aoi1.includes(&entry1, &store)));

    let area2 = Area::<_, [[u8; 0]; 0]> {
        subspace: Subspace::Any,
        path:     [],
        times:    (0 ..).into(),
    };
    let (payload2, payload2_digest) = payload_and_digest("0123456789");
    let entry2 = Entry {
        namespace_id:   Arc::clone(namespace_id),
        subspace_id:    user.clone(),
        path:           ["gamma"],
        timestamp:      (past + 987_654).into(),
        payload_digest: payload2_digest,
        payload_length: payload2.len(),
    };
    let ae2 =
        AuthorisedEntry::new(entry2.clone(), auth_token_1.clone()).expect("auth should succeed");
    assert_eq!(store.put(ae2, Some(payload2.clone())), Ok(()));

    let aoi2 = AreaOfInterest {
        area:      area2.clone(),
        max_count: Max::Limit(NonZeroU64::new(1).unwrap()),
        max_size:  Max::Unlimited,
    };
    assert!(!pollster::block_on(aoi2.includes(&entry2, &store)));

    let aoi3 = AreaOfInterest {
        area:      area2.clone(),
        max_count: Max::Unlimited,
        max_size:  Max::Limit(NonZeroU64::new(9).unwrap()),
    };
    assert!(!pollster::block_on(aoi3.includes(&entry2, &store)));

    let aoi4 = AreaOfInterest {
        area:      area2.clone(),
        max_count: Max::Limit(NonZeroU64::new(2).unwrap()),
        max_size:  Max::Limit(NonZeroU64::new(10).unwrap()),
    };
    assert!(pollster::block_on(aoi4.includes(&entry2, &store)));

    let mut entry3 = entry2.clone();
    entry3.path = ["elsewhere"];
    entry3.timestamp = (past + 42).into();
    let ae3 =
        AuthorisedEntry::new(entry3.clone(), auth_token_1.clone()).expect("auth should succeed");
    assert_eq!(store.put(ae3, Some(payload2.clone())), Ok(()));

    let mut entry4 = entry1.clone();
    entry4.path = ["over", "yonder"];
    let ae4 =
        AuthorisedEntry::new(entry4.clone(), auth_token_1.clone()).expect("auth should succeed");
    assert_eq!(store.put(ae4, Some(empty_payload.clone())), Ok(()));

    let aoi5 = AreaOfInterest {
        area:      area2.clone(),
        max_count: Max::Limit(NonZeroU64::new(2).unwrap()),
        max_size:  Max::Unlimited,
    };
    assert!(!pollster::block_on(aoi5.includes(&entry3, &store)));

    let aoi6 = AreaOfInterest {
        area:      area2.clone(),
        max_count: Max::Unlimited,
        max_size:  Max::Limit(NonZeroU64::new(10).unwrap()),
    };
    assert!(!pollster::block_on(aoi6.includes(&entry3, &store)));
}


#[test]
fn intersection()
{
    let aoi1 = AreaOfInterest {
        area:      Area {
            subspace: Subspace::Id('z'),
            path:     &[b"1"][..],
            times:    (321 ..).into(),
        },
        max_count: Max::Limit(100.try_into().unwrap()),
        max_size:  Max::Unlimited,
    };
    let aoi2 = AreaOfInterest {
        area:      Area {
            subspace: Subspace::Any::<char>,
            path:     &[b"1", b"2"][..],
            times:    (123 .. 456).into(),
        },
        max_count: Max::Unlimited,
        max_size:  Max::Limit(1_000_000.try_into().unwrap()),
    };

    assert_eq!(aoi1.intersection(aoi1), aoi1);
    assert_eq!(aoi2.intersection(aoi2), aoi2);
    {
        let i = AreaOfInterest {
            area:      Area {
                subspace: Subspace::Id('z'),
                path:     &[b"1", b"2"][..],
                times:    (321 .. 456).into(),
            },
            max_count: Max::Limit(100.try_into().unwrap()),
            max_size:  Max::Limit(1_000_000.try_into().unwrap()),
        };

        assert_eq!(aoi1.intersection(aoi2), i);
        assert_eq!(aoi2.intersection(aoi1), i);
    }
}
