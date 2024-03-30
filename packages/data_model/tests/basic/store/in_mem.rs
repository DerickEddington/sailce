use {
    super::{
        params::{
            NamespaceName,
            Params,
            Path,
            Permission,
            User,
        },
        stored_entry::StoredEntry,
    },
    crate::{
        async_help::{
            not_yet_ready,
            Pollster,
        },
        payload::InMem as InMemPayload,
    },
    sailce_data_model::{
        path::{
            Extra as _,
            PathLimitError,
        },
        payload::{
            Extra as _,
            ToBoxedSliceError,
        },
        AuthorisedEntry,
        Params as _,
        ParamsEntry,
        Path as _,
        Payload,
        StoreAuthorisedEntry,
        StoreExt,
    },
    std::{
        borrow::Borrow,
        collections::{
            hash_map::DefaultHasher,
            BTreeMap,
            BinaryHeap,
            HashMap,
            HashSet,
        },
        hash::Hasher,
        ops::ControlFlow,
        sync::Arc,
    },
};


pub(crate) type StoredEntryHistory = BinaryHeap<StoredEntry>;
pub(crate) type StoredSubspace = BTreeMap<Path, StoredEntryHistory>;


/// The minimal amount of data needed to store `Entry`s and preserve old overwritten ones.  Even
/// though the `Store` API doesn't allow access to old overwritten ones, this type preserves them
/// just to show that it can be done and some other API could be made for accessing them.
pub(crate) struct InMem
{
    subspaces:    BTreeMap<User, StoredSubspace>,
    namespace_id: NamespaceName, // Not really needed. Just to check against for testing.
}

impl InMem
{
    pub(crate) async fn new(namespace_id: &NamespaceName) -> Self
    {
        not_yet_ready(5).await; // Just to have an async suspend point in here.
        Self { subspaces: BTreeMap::new(), namespace_id: Arc::clone(namespace_id) }
    }

    pub(crate) fn new_block_on_pollster(namespace_id: &NamespaceName) -> Self
    {
        Pollster::block_on(Self::new(namespace_id), 123_u32)
    }

    /// Iterator of each stored entry's history along with the `SubspaceId` and `Path` where it's
    /// located.
    pub(crate) fn iter_histories(
        &self
    ) -> impl Iterator<Item = (&User, &Path, &StoredEntryHistory)>
    {
        self.subspaces.iter().flat_map(|(user, subspace)| {
            subspace
                .iter()
                .map(move |(path, stored_entry_history)| (user, path, stored_entry_history))
        })
    }

    /// Iterator of each stored entry's newest value along with the `SubspaceId` and `Path` where
    /// it's located.
    pub(crate) fn iter_stored_entries(&self)
    -> impl Iterator<Item = (&User, &Path, &StoredEntry)>
    {
        self.iter_histories().flat_map(|(user, path, stored_entry_history)| {
            stored_entry_history.peek().map(|newest| (user, path, newest)).into_iter()
        })
    }
}

/// This implementation is only for exercising the API, and this uses simple approaches instead of
/// trying to be more efficient.
impl StoreExt for InMem
{
    type GetError = GetError;
    type GetPayload = InMemPayload;
    type IterAuthToken = Arc<Permission>;
    type IterPath = Path;
    type JoinError = JoinError;
    type Params = Params;
    type PutError<P> = PutError<CopyPayloadError<P>> where P: Payload + ?Sized;

    async fn get(
        &self,
        namespace_id: &NamespaceName,
        subspace_id: &User,
        path: &(impl sailce_data_model::Path + ?Sized),
    ) -> Result<Option<Self::GetPayload>, Self::GetError>
    {
        debug_assert_eq!(*namespace_id, self.namespace_id);
        not_yet_ready(4).await; // Just to have an async suspend point in here.

        if let Some((found_entry, subspace)) =
            self.subspaces.get(subspace_id).and_then(|subspace| {
                let path = Path::from_path(path);
                subspace.get(&path).and_then(|entry_history| {
                    entry_history.peek().map(|newest| (newest, subspace))
                })
            })
        {
            // All the `StoredEntry`s in the same Subspace whose `Path` is a prefix of and not
            // equal to the requested `path`, because these are what might have pruned `path`.
            let mut prefixes = subspace.iter().filter_map(|(other_path, entry_history)| {
                entry_history.peek().and_then(|newest| {
                    (other_path.is_prefix_of(path) && !other_path.eq_components(path))
                        .then_some(newest)
                })
            });
            not_yet_ready(3).await; // Just to have an async suspend point in here.

            // If a prefixing entry is newer than the found entry under its prefix, then prefix
            // pruning has deleted everything under the prefix.
            let is_pruned = prefixes.any(|prefixing_entry| prefixing_entry > found_entry);
            if is_pruned {
                Ok(None)
            }
            else if let Some(payload) = &found_entry.payload {
                let payload_clone = payload.clone();
                // We always create our internal backing InMemPayloads in a valid state with their
                // current position at their beginning, so we know we uphold the requirement that
                // the returned payload is like this when we `clone` here.
                debug_assert!(payload_clone.check_invariants());
                debug_assert_eq!(payload_clone.pos_as_u64(), 0);
                not_yet_ready(2).await; // Just to have an async suspend point in here.
                Ok(Some(payload_clone))
            }
            else {
                Err(GetError::FoundEntryMissingPayload(found_entry.to_entry(
                    namespace_id,
                    subspace_id,
                    &Path::from_path(path),
                )))
            }
        }
        else {
            Ok(None)
        }
    }

    async fn put<P>(
        &mut self,
        namespace_id: &NamespaceName,
        auth_entry: AuthorisedEntry<
            Self::Params,
            impl sailce_data_model::Path,
            impl Borrow<Permission>,
        >,
        payload: Option<P>,
    ) -> Result<(), Self::PutError<P>>
    where
        P: Payload,
    {
        debug_assert_eq!(*namespace_id, self.namespace_id);

        let entry = auth_entry.entry();
        debug_assert_eq!(entry.namespace_id, *namespace_id); // `Store::put` must ensure this.
        debug_assert!(Params::is_authorised_write(entry, auth_entry.auth_token()));

        let subspace = self.subspaces.entry(entry.subspace_id.clone()).or_default();
        let path =
            Path::from_path_limited::<Params, _, _>(&entry.path).map_err(PutError::PathLimit)?;
        let entry_history = subspace.entry(path).or_default();
        not_yet_ready(2).await; // Just to have an async suspend point here.

        let mut stored_entry = StoredEntry::from_auth_entry(auth_entry);
        debug_assert!(stored_entry.payload.is_none());

        if let Some(payload) = payload {
            let (copied_payload, payload_digest) =
                InMemPayload::copy(payload).await.map_err(PutError::Copy)?;
            if payload_digest == stored_entry.payload_digest {
                stored_entry.payload = Some(copied_payload);
            }
            else {
                return Err(PutError::WrongDigest {
                    given:    stored_entry.payload_digest,
                    computed: payload_digest,
                });
            }
        }

        if let Some(mut existing) = entry_history.peek_mut() {
            if *existing == stored_entry {
                // We should be trying to supply the payload for the same previously-stored entry
                // that didn't supply its payload originally.
                if existing.payload.is_some() {
                    // It actually already has its payload, and we just checked that our supplying
                    // of that again has the same digest (via our impl of `==` which checks this).
                    if stored_entry.payload.is_some() {
                        // A different quality impl of `StoreExt` could also do the analogue of
                        // dropping `stored_entry.payload` and replacing it with
                        // `existing.payload`, to avoid duplicating what
                        // should be the same exact payload.  But for this
                        // test impl, we keep all duplicates in the history.
                    }
                    else {
                        // For some reason, the caller supplied `None` for `payload` for a
                        // previously-stored entry that already has its payload.  Because this
                        // test impl keeps all redundant elements, we need this `stored_entry`
                        // instance to have the same payload, in case it's used as the greatest in
                        // the history.
                        stored_entry.payload = existing.payload.clone();
                    }
                }
                else {
                    // If `payload` is `Some` and we copied it, update the previously-stored entry
                    // to now have it also.  This `clone`ing is efficient, because the `Payload`
                    // type is `InMem` which holds its bytes in an `Arc` and so we get shared
                    // ownership without copying.  If it's `None`, this doesn't change
                    // `existing.payload` because that was already `None`.
                    existing.payload = stored_entry.payload.clone();
                }
            }
            else {
                // They're not the same, so let `stored_entry` be `push`ed so that its ordering in
                // the existing history will determine whether or not it's now the newest.
            }
        }
        not_yet_ready(1).await; // Just to have an async suspend point here.

        // `push` it regardless of whether it's the newest or not and regardless of whether it's
        // equivalent to a pre-existing.  We still want to preserve the history of everything that
        // was `put`.  This can result in multiple redundant elements that represent the same
        // exact `Entry`, and it's unspecified which will be `peek`ed first but that doesn't
        // matter because they're equivalent (modulo any possible differences in their
        // `auth_token`s).
        entry_history.push(stored_entry);
        Ok(())
    }

    async fn join(
        &mut self,
        namespace_id: &NamespaceName,
        other: &Self,
        other_namespace_id: &NamespaceName,
    ) -> Result<(), Self::JoinError>
    {
        debug_assert_eq!(*namespace_id, self.namespace_id);
        debug_assert_eq!(*other_namespace_id, other.namespace_id);

        if other_namespace_id == namespace_id {
            for (user, path, history) in other.iter_histories() {
                for stored_entry in history {
                    self.put(
                        namespace_id,
                        stored_entry.to_auth_entry(namespace_id, user, path),
                        stored_entry.payload.clone(),
                    )
                    .await
                    .map_err(JoinError::Put)?;
                }
            }
            Ok(())
        }
        else {
            Err(JoinError::WrongNamespace(Arc::clone(other_namespace_id)))
        }
    }

    async fn newest_includes_within_total_size<P>(
        &self,
        namespace_id: &<Self::Params as sailce_data_model::Params>::NamespaceId,
        max_count: Option<u64>,
        entry: impl Borrow<ParamsEntry<Self::Params, P>>,
        max_size: Option<u64>,
    ) -> bool
    where
        P: sailce_data_model::Path,
    {
        debug_assert_eq!(*namespace_id, self.namespace_id);
        let entry = entry.borrow();
        debug_assert_eq!(entry.namespace_id, *namespace_id); // The `Store` method ensures this.

        // (This inefficient approach is just for testing.)
        let mut entries = self.iter(namespace_id).map(|auth_entry| auth_entry.into_parts().0);

        if max_count.is_none() && max_size.is_none() {
            // We don't use `Self::get` because that might error if missing payload but we want to
            // find such entry.
            return entries.any(|e| *entry == e);
        }

        let mut newest = entries.collect::<Vec<_>>();
        newest.sort_unstable_by(|a, b| b.cmp_newer_than(a));
        if let Some(max_count) = max_count {
            if let Ok(max_count) = max_count.try_into() {
                newest.truncate(max_count);
            }
        }

        if let Some(max_size) = max_size {
            if let ControlFlow::Break(Some(total_size)) =
                newest.iter().try_fold(Some(entry.payload_length), |total_size, e| {
                    if entry == e {
                        // Found it among the newest. Return success with the total.
                        ControlFlow::Break(total_size)
                    }
                    else if let Some(total_size) =
                        total_size.and_then(|x: u64| x.checked_add(e.payload_length))
                    {
                        // Keep searching and summing.
                        ControlFlow::Continue(Some(total_size))
                    }
                    else {
                        // Summing overflowed. Return failure.
                        ControlFlow::Break(None)
                    }
                })
            {
                // Found `entry` amoung the `max_count` newest.  Is it within `max_size` total
                // size of newer?
                total_size <= max_size
            }
            else {
                // Either: Not found; or, summing overflowed and `max_size <= u64::MAX <
                // total_size`.
                false
            }
        }
        else {
            debug_assert!(max_count.is_some());
            newest.iter().any(|e| entry == e)
        }
    }

    fn iter(
        &self,
        namespace_id: &NamespaceName,
    ) -> impl Iterator<Item = StoreAuthorisedEntry<Self>>
    {
        debug_assert_eq!(*namespace_id, self.namespace_id);

        self.iter_stored_entries()
            // Must filter-out those that have been prefix-pruned.  (This inefficient approach is
            // just for testing.)
            .scan(
                HashMap::<&User, HashSet<(&Path, &StoredEntry)>>::new(),
                |seen, item @ (user, path, stored_entry)| {
                    let seen_sub = seen.entry(user).or_default();
                    let mut prefixes = seen_sub.iter().filter_map(
                        |&(seen_path, seen_entry): &(&Path, &StoredEntry)| {
                            (seen_path.is_prefix_of(path)
                                && !seen_path.eq_components(path))
                            .then_some(seen_entry)
                        },
                    );
                    // This relies on our `iter_stored_entries` yielding them in lexicographic
                    // order of their paths (because they're held in a `BTreeMap`) which
                    // guarantees that prefixes were seen before everything they prefix.
                    let is_pruned =
                        prefixes.any(|prefixing_entry| prefixing_entry > stored_entry);
                    let added = seen_sub.insert((path, stored_entry));
                    debug_assert!(added);
                    Some((is_pruned, item))
                },
            )
            .filter_map(|(is_pruned, item)| (!is_pruned).then_some(item))
            .map(|(user, path, stored_entry)| {
                stored_entry.to_auth_entry(namespace_id, user, path)
            })
    }
}


#[derive(Debug, Eq, PartialEq)]
pub(crate) enum GetError
{
    FoundEntryMissingPayload(ParamsEntry<Params, Path>),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum PutError<CopyPayloadError>
{
    PathLimit(PathLimitError),
    Copy(CopyPayloadError),
    WrongDigest
    {
        given:    u64,
        computed: u64,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum JoinError
{
    WrongNamespace(NamespaceName),
    Put(PutError<CopyPayloadError<InMemPayload>>),
}


#[allow(clippy::multiple_inherent_impl)]
impl InMemPayload
{
    pub(crate) async fn copy<P>(mut other: P) -> Result<(Self, u64), CopyPayloadError<P>>
    where P: Payload
    {
        // Compute its hash while we copy it.
        let mut hasher = DefaultHasher::default();
        let bytes = other
            .to_boxed_slice(
                0 ..,
                Some(|chunk: &mut [u8]| hasher.write(chunk)),
                true, // So callers can depend on this.
            )
            .await
            .map_err(CopyPayloadError::ToBoxedSlice)?;

        Ok((Self::new(bytes).map_err(CopyPayloadError::New)?, hasher.finish()))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CopyPayloadError<P: Payload + ?Sized>
{
    ToBoxedSlice(ToBoxedSliceError<P::ReadError, P::SeekError>),
    New(#[allow(dead_code)] &'static str),
}
