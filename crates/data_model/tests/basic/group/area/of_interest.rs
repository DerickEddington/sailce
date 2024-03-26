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


#[ignore]
#[allow(clippy::todo)]
#[test]
fn includes()
{
    // Need `Store::newest_entries_include` and `Store::payloads_total_size_of_entry_to_newest` to
    // be implemented
    todo!()
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
