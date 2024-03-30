#![cfg(feature = "alloc")]

use {
    crate::{
        async_help::Pollster,
        payload::InMem as InMemPayload,
    },
    sailce_data_model::{
        path::Extra as _,
        store::{
            sync,
            PutError,
        },
        syncify::Syncify,
        AuthorisedEntry,
        EmptyPath as _,
        Entry,
        Params as _,
        Store,
        Timestamp,
    },
    std::{
        future::Future,
        sync::Arc,
        time::SystemTime,
    },
};


mod params;
pub(crate) use params::{
    NamespaceName,
    Params,
    Path,
    Permission,
    User,
};

mod stored_entry;

mod in_mem;
pub(crate) use in_mem::InMem;


impl Syncify<Pollster> for Store<<Params as sailce_data_model::Params>::NamespaceId, InMem>
{
    type ExecutorData = ();

    fn get_block_on_fn<'f, F>(&self) -> impl 'f + FnOnce(F, Self::ExecutorData) -> F::Output
    where F: Future + 'f
    {
        fn adapt<F: Future>(
            fut: F,
            (): (),
        ) -> F::Output
        {
            Pollster::block_on(fut, 123_u32)
        }

        adapt
    }

    fn get_executor_data(&self) -> Self::ExecutorData {}
}

impl sync::Store<Pollster, Params, InMem>
    for Store<<Params as sailce_data_model::Params>::NamespaceId, InMem>
{
}


pub(crate) fn current_timestamp() -> Timestamp
{
    let duration = SystemTime::UNIX_EPOCH
        .elapsed()
        .expect("time drift usually won't occur during these tests");
    Timestamp {
        μs_since_epoch: duration
            .as_micros()
            .try_into()
            .expect("584.5K years until this will fail"),
    }
}

pub(crate) fn payload_and_digest(bytes: impl AsRef<[u8]>) -> (InMemPayload, u64)
{
    let mut payload = InMemPayload::new(bytes).expect("size fits");
    let digest = pollster::block_on(Params::hash_payload(&mut payload)).expect("should work");
    (payload, digest)
}


/// This exercises both the sync and the `async` methods, because the sync ones use the `async`
/// ones.
#[test]
#[allow(clippy::too_many_lines)]
fn most_methods__sync_uses_async()
{
    use sailce_data_model::{
        payload::sync::Payload as _,
        store::sync::Store as _,
    };

    fn all_entries(
        store: &Store<NamespaceName, InMem>
    ) -> Vec<(User, Path, <Params as sailce_data_model::Params>::PayloadDigest, u64)>
    {
        let mut entries = store
            .iter()
            .map(|auth_entry| {
                let entry = auth_entry.into_parts().0;
                (entry.subspace_id, entry.path, entry.payload_digest, entry.payload_length)
            })
            .collect::<Vec<_>>();
        entries.sort();
        entries
    }

    let ns1 = "namespace-1".into();
    let user1 = User::new("uno");
    let mut store = Store::new(&ns1, InMem::new_block_on_pollster(&ns1));
    assert_eq!(store.namespace_id(), &ns1);

    let (empty_payload, empty_payload_digest) = payload_and_digest([]);
    let (another_payload, another_payload_digest) = payload_and_digest("foo bar");
    let ae1 = AuthorisedEntry::new(
        Entry {
            namespace_id:   Arc::clone(&ns1),
            subspace_id:    user1.clone(),
            path:           &["some", "where"][..],
            timestamp:      current_timestamp(),
            payload_digest: empty_payload_digest,
            payload_length: 0,
        },
        Permission {
            user:       user1.clone(),
            namespaces: [Arc::clone(&ns1)].into(),
            subspaces:  [].into(), // Not needed with `user == subspace_id`.
            paths:      [Path::empty()].into(),
            times:      [(0 ..).into()].into(),
        },
    )
    .expect("auth should succeed");
    let ae2 = {
        let (mut entry, auth_token) = ae1.clone().into_parts();
        entry = Entry {
            timestamp: (entry.timestamp.μs_since_epoch + 1).into(),
            payload_digest: another_payload_digest,
            payload_length: another_payload.len(),
            ..entry
        };
        AuthorisedEntry::new(entry, auth_token).expect("auth should succeed")
    };
    let ae3 = {
        let (e1, auth_token) = ae1.clone().into_parts();
        let entry = Entry {
            timestamp:      (e1.timestamp.μs_since_epoch + 2).into(),
            path:           vec![b"some"], // Different type is fine.
            namespace_id:   e1.namespace_id,
            subspace_id:    e1.subspace_id,
            payload_digest: e1.payload_digest,
            payload_length: e1.payload_length,
        };
        AuthorisedEntry::new(entry, auth_token).expect("auth should succeed")
    };

    // Attempt to get non-existent.
    assert_eq!(store.get(&User::new("nobody"), &["nope"]), Ok(None));
    // Put without payload yet.
    assert_eq!(store.put(ae1.clone(), None::<InMemPayload>), Ok(()));
    // Attempt to get what was put without payload.
    assert!(matches!(
        store.get(&user1, &["some", "where"]),
        Err(in_mem::GetError::FoundEntryMissingPayload(_))
    ));
    // Supply payload for what was already put.
    assert_eq!(store.put(ae1.clone(), Some(empty_payload.clone())), Ok(()));
    // Now its payload can be gotten.
    {
        let mut got = store.get(&user1, &["some", "where"]).unwrap().unwrap();
        assert_eq!(got.len(), 0);
        assert_eq!(got.read(&mut [0; 16]), Ok(0));
    }
    // Weird redundant put of the same entry but without payload.
    assert_eq!(store.put(ae1.clone(), None::<InMemPayload>), Ok(()));
    // Its payload stays the same.
    {
        let mut got = store.get(&user1, &["some", "where"]).unwrap().unwrap();
        assert_eq!(got.len(), 0);
        assert_eq!(got.read(&mut [0; 16]), Ok(0));
    }
    // Redundant put of the same entry with same payload.
    assert_eq!(store.put(ae1.clone(), Some(empty_payload.clone())), Ok(()));
    // Its payload stays the same.
    {
        let mut got = store.get(&user1, &["some", "where"]).unwrap().unwrap();
        assert_eq!(got.len(), 0);
        assert_eq!(got.read(&mut [0; 16]), Ok(0));
    }
    // Weird redundant put of the same entry but with different payload with mismatched digest.
    assert_eq!(
        store.put(ae1.clone(), Some(another_payload.clone())),
        Err(PutError::Put(in_mem::PutError::WrongDigest {
            given:    empty_payload_digest,
            computed: another_payload_digest,
        }))
    );
    // Replace at same path.
    assert_eq!(store.put(ae2.clone(), Some(another_payload.clone())), Ok(()));
    // Different new contents.
    {
        let mut got = store.get(&user1, &["some", "where"]).unwrap().unwrap();
        assert_eq!(got.len(), 7);
        let mut buf = [0; 16];
        assert_eq!(got.read(&mut buf), Ok(7));
        assert_eq!(buf[.. 7], b"foo bar"[..]);
    }
    // Non-existent at parent path.
    assert_eq!(store.get(&user1, &ae3.entry().path), Ok(None));
    // Prefix pruning at parent path.
    assert_eq!(store.put(ae3.clone(), Some(empty_payload.clone())), Ok(()));
    // Now parent-path payload can be gotten.
    {
        let mut got = store.get(&user1, &[&b"some"][..]).unwrap().unwrap();
        assert_eq!(got.len(), 0);
        assert_eq!(got.read(&mut [0; 16]), Ok(0));
    }
    // Now the deeper path is non-existent, because prefix pruning deleted it.
    assert_eq!(store.get(&user1, &["some", "where"]), Ok(None));

    // Which entries should be present now.
    assert_eq!(all_entries(&store), [(
        user1.clone(),
        Path::from_path(&["some"]),
        empty_payload_digest,
        0
    )]);

    let mut store2 = Store::new(&ns1, InMem::new_block_on_pollster(&ns1));
    let ae4 = {
        let (mut entry, auth_token) = ae2.clone().into_parts();
        entry = Entry {
            // timestamp is less-than ae3's that prefix-pruned.
            path: &["some", "where", "under", "a", "cloud"][..],
            ..entry
        };
        AuthorisedEntry::new(entry, auth_token).expect("auth should succeed")
    };
    let ae5 = {
        let (mut entry, auth_token) = ae2.clone().into_parts();
        entry = Entry {
            // timestamp is greater-than ae3's that prefix-pruned.
            timestamp: (ae3.entry().timestamp.μs_since_epoch + 1).into(),
            path: &["some", "where", "over", "a", "rainbow"][..],
            ..entry
        };
        AuthorisedEntry::new(entry, auth_token).expect("auth should succeed")
    };
    // Populate `store2` for following `join` tests.
    debug_assert!(ae4.entry().timestamp < ae3.entry().timestamp);
    assert_eq!(store2.put(ae4.clone(), Some(another_payload.clone())), Ok(()));
    assert!(matches!(store2.get(&user1, &["some", "where", "under", "a", "cloud"]), Ok(Some(_))));
    debug_assert!(ae5.entry().timestamp > ae3.entry().timestamp);
    assert_eq!(store2.put(ae5.clone(), Some(another_payload.clone())), Ok(()));
    assert!(matches!(
        store2.get(&user1, &["some", "where", "over", "a", "rainbow"]),
        Ok(Some(_))
    ));

    // Which entries should be present in `store2` now.
    assert_eq!(all_entries(&store2), [
        (
            user1.clone(),
            Path::from_path(&["some", "where", "over", "a", "rainbow"]),
            another_payload_digest,
            7
        ),
        (
            user1.clone(),
            Path::from_path(&["some", "where", "under", "a", "cloud"]),
            another_payload_digest,
            7
        )
    ]);

    // Join the two `Store`s into the first.
    assert_eq!(store.join(&store2), Ok(()));
    // Prefix pruning affects joined entry also, so that it's deleted.
    assert_eq!(store.get(&user1, &["some", "where", "under", "a", "cloud"]), Ok(None));
    // Joined entry which is newer than the prefix-pruned is present.
    {
        let mut got =
            store.get(&user1, &["some", "where", "over", "a", "rainbow"]).unwrap().unwrap();
        assert_eq!(got.len(), 7);
        let mut buf = [0; 16];
        assert_eq!(got.read(&mut buf), Ok(7));
        assert_eq!(buf[.. 7], b"foo bar"[..]);
    }

    // Which entries should be present in `store` now - the join.
    assert_eq!(all_entries(&store), [
        (user1.clone(), Path::from_path(&["some"]), empty_payload_digest, 0),
        (
            user1.clone(),
            Path::from_path(&["some", "where", "over", "a", "rainbow"]),
            another_payload_digest,
            7
        )
    ]);

    // Which entries should be present in `store2` now - unchanged.
    assert_eq!(all_entries(&store2), [
        (
            user1.clone(),
            Path::from_path(&["some", "where", "over", "a", "rainbow"]),
            another_payload_digest,
            7
        ),
        (
            user1.clone(),
            Path::from_path(&["some", "where", "under", "a", "cloud"]),
            another_payload_digest,
            7
        )
    ]);
}
