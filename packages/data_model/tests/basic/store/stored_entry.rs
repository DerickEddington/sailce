use {
    super::params::{
        NamespaceName,
        Params,
        Path,
        Permission,
        User,
    },
    crate::payload::InMem as InMemPayload,
    sailce_data_model::{
        AuthorisedEntry,
        EmptyPath as _,
        Entry,
        ParamsEntry,
        Path as _,
        Timestamp,
    },
    std::{
        borrow::Borrow,
        cmp::Ordering,
        hash::{
            Hash,
            Hasher,
        },
        sync::Arc,
    },
};


/// The minimal amount of data needed to store an `Entry`.
pub(crate) struct StoredEntry
{
    pub(crate) timestamp:      Timestamp,
    pub(crate) payload_digest: u64,
    pub(crate) payload_length: u64,
    pub(crate) payload:        Option<InMemPayload>,
    pub(crate) auth_token:     Arc<Permission>,
}

impl Ord for StoredEntry
{
    /// Compare by the same ordering as `Entry::is_newer_than`, so that `BinaryHeap<Self>` has the
    /// newest as the greatest.  `to_dummy_entry` always uses the same dummy values for the other
    /// fields of `Entry`, and so those don't affect the comparison.  The `payload` & `auth_token`
    /// fields of `Self` are ignored, because they're not relevant to comparison of `Entry`s, and
    /// we want `Self` to be a representation of `Entry`.
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering
    {
        self.to_dummy_entry().cmp(&other.to_dummy_entry())
    }
}

impl PartialOrd for StoredEntry
{
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering>
    {
        Some(self.cmp(other))
    }
}

impl PartialEq for StoredEntry
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for StoredEntry {}

impl Hash for StoredEntry
{
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    )
    {
        // Must be consistent with our `cmp` & `eq`.
        self.to_dummy_entry().hash(state);
    }
}

impl StoredEntry
{
    pub(crate) fn from_auth_entry(
        auth_entry: AuthorisedEntry<
            Params,
            impl sailce_data_model::Path,
            impl Borrow<Permission>,
        >
    ) -> Self
    {
        let (entry, auth_token) = auth_entry.into_parts();

        Self {
            timestamp:      entry.timestamp,
            payload_digest: entry.payload_digest,
            payload_length: entry.payload_length,
            payload:        None,
            auth_token:     Arc::new(auth_token.borrow().clone()),
        }
    }

    pub(crate) fn to_dummy_entry(&self) -> ParamsEntry<Params, Path>
    {
        // Dummy values.
        let namespace_id = "".into();
        let subspace_id = User { name: "".into(), id: 0 };
        let path = Path::empty();

        self.to_entry(&namespace_id, &subspace_id, &path)
    }

    pub(crate) fn to_entry(
        &self,
        namespace_id: &NamespaceName,
        subspace_id: &User,
        path: &Path,
    ) -> ParamsEntry<Params, Path>
    {
        Entry {
            namespace_id:   Arc::clone(namespace_id),
            subspace_id:    subspace_id.clone(),
            path:           Arc::clone(path),
            timestamp:      self.timestamp,
            payload_digest: self.payload_digest,
            payload_length: self.payload_length,
        }
    }

    pub(crate) fn to_auth_entry(
        &self,
        namespace_id: &NamespaceName,
        subspace_id: &User,
        path: &Path,
    ) -> AuthorisedEntry<Params, Path, Arc<Permission>>
    {
        // These are redundant with `Params::is_authorised_write` (via `AuthorisedEntry::new`)
        // already checking these conditions next, but having these here shows what would be good
        // to do if `AuthorisedEntry::new` weren't used and instead the original
        // `AuthorisedEntry`s were stored.
        debug_assert!(self.auth_token.namespaces.contains(namespace_id));
        debug_assert!(
            *subspace_id == self.auth_token.user
                || self.auth_token.subspaces.contains(subspace_id)
        );
        debug_assert!(self.auth_token.paths.iter().any(|p| p.is_prefix_of(path)));

        AuthorisedEntry::new(
            self.to_entry(namespace_id, subspace_id, path),
            Arc::clone(&self.auth_token),
        )
        .expect("the `Entry` was already authorized by the `auth_token`")

        // Note: A different implementation that forbids potential panics should instead store the
        // `AuthorisedEntry`s as taken in by `StoreExt::put`, to avoid needing to do something
        // like our `expect` above, so that it can instead simply return a clone of the
        // `AuthorisedEntry` it stored.  TODO: But how to deserialize to that type which has
        // private fields?  TODO: Maybe the `sailce_data_model` crate will need a package-feature
        // to have a `Deserialize` trait impl'ed for that type?
    }
}
