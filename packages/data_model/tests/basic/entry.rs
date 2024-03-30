use sailce_data_model::{
    Entry,
    Timestamp,
};


type E = Entry<i32, i32, &'static [&'static str], [u8; 8]>;

const fn e(
    namespace_id: i32,
    subspace_id: i32,
    path: &'static [&'static str],
    timestamp: u64,
    payload_digest: [u8; 8],
    payload_length: u64,
) -> E
{
    let timestamp = Timestamp { μs_since_epoch: timestamp };
    Entry { namespace_id, subspace_id, path, timestamp, payload_digest, payload_length }
}

const E0: E = e(0, 0, &[], 0, [0; 8], 0);
const E1: E = e(1, 2, &["3"], 4, [5; 8], 6);
const E2: E = e(42, 321, &["fooba", "rzab", "oof"], 11111, [9, 8, 7, 6, 5, 4, 3, 2], 0x4000);

fn greater(e: E) -> bool
{
    E0 < e
}
fn newer(e: E) -> bool
{
    e.is_newer_than(&E0)
}
fn greater_not_newer(e: E) -> bool
{
    greater(e) && !newer(e)
}
fn greater_eq_newer(e: E) -> bool
{
    greater(e) == newer(e)
}
fn greater_and_eq_newer(e: E) -> bool
{
    greater(e) && newer(e) && greater_eq_newer(e)
}


#[test]
fn newer_than()
{
    assert!(!newer(e(0, 0, &[], 0, [0; 8], 0)));
    assert!(!newer(e(1, 2, &["3"], 0, [0; 8], 0)));
    assert!(Entry { payload_length: 7, ..E1 }.is_newer_than(&E1));
    assert!(Entry { payload_digest: [7; 8], ..E1 }.is_newer_than(&E1));
    assert!(Entry { timestamp: Timestamp { μs_since_epoch: 7 }, ..E1 }.is_newer_than(&E1));
    assert!(!Entry { path: &["7"][..], ..E1 }.is_newer_than(&E1));
    assert!(!Entry { subspace_id: 7, ..E1 }.is_newer_than(&E1));
    assert!(!Entry { namespace_id: 7, ..E1 }.is_newer_than(&E1));
}


#[test]
fn ordering()
{
    assert!(!greater(e(0, 0, &[], 0, [0; 8], 0)));
    assert!(greater(e(0, 0, &[], 0, [0; 8], 1)));
    assert!(greater(e(0, 0, &[], 0, [1; 8], 0)));
    assert!(greater(e(0, 0, &[], 1, [0; 8], 0)));
    assert!(greater(e(0, 0, &[""], 0, [0; 8], 0)));
    assert!(greater(e(0, 1, &[], 0, [0; 8], 0)));
    assert!(greater(e(1, 0, &[], 0, [0; 8], 0)));

    assert!(greater_and_eq_newer(e(0, 0, &[], 0, [0; 8], 1)));
    assert!(greater_and_eq_newer(e(0, 0, &[], 0, [0, 0, 0, 0, 0, 0, 0, 1], 0)));
    assert!(greater_and_eq_newer(e(0, 0, &[], 1, [0; 8], 0)));

    assert!(greater_and_eq_newer(e(1, 2, &["3"], 0, [0; 8], 6)));
    assert!(greater_and_eq_newer(e(1, 2, &["3"], 0, [5; 8], 0)));
    assert!(greater_and_eq_newer(e(1, 2, &["3"], 4, [0; 8], 0)));

    assert!(greater_not_newer(e(77, 66, &["5", "5"], 0, [0; 8], 0)));

    assert!(E1 < Entry { payload_length: 7, ..E1 });
    assert!(E1 < Entry { payload_digest: [7; 8], ..E1 });
    assert!(E1 < Entry { timestamp: Timestamp { μs_since_epoch: 7 }, ..E1 });
    assert!(E1 < Entry { path: &["7"][..], ..E1 });
    assert!(E1 < Entry { subspace_id: 7, ..E1 });
    assert!(E1 < Entry { namespace_id: 7, ..E1 });

    assert!(E2 > Entry { namespace_id: 17, ..E2 });
    assert!(E2 > Entry { subspace_id: 123, ..E2 });
    assert!(E2 > Entry { path: &["fooba", "rzab"][..], ..E2 });
    assert!(E2 > Entry { path: &["abcdef", "fooba", "rzab", "oof"][..], ..E2 });
    assert!(E2 > Entry { timestamp: Timestamp { μs_since_epoch: 1111 }, ..E2 });
    assert!(E2 > Entry { payload_digest: [8, 7, 6, 5, 4, 3, 2, 1], ..E2 });
    assert!(E2 > Entry { payload_length: 0x3000, ..E2 });
}


mod auth;
