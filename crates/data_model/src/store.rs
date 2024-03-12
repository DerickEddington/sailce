//! Aspects of `Store`s.

use {
    crate::{
        ParamsAuthorisedEntry,
        ParamsEntry,
        Path,
        Payload,
    },
    core::borrow::Borrow,
};


// TODO: Make a separate impl of this that uses FS, e.g. sailce/crates/fs_store/ with tests.
//
/// A _store_ is a set of [`AuthorisedEntry`](crate::AuthorisedEntry)s such that
/// - all its [`Entry`](crate::Entry)s have the same [`namespace_id`](crate::Entry::namespace_id),
///   and
/// - there are no two of its `Entry`s `old` and `new` such that
///   - `old.subspace_id == new.subspace_id`, and
///   - `new.path` is a [prefix](Path::is_prefix_of) of `old.path`,
///   - and `new` is newer than `old`.
///
///  (That includes the formal definition of _prefix pruning_.)
///
/// I.e., storing a new `Entry` at the same 3-D location as another `Entry` in a Namespace will
/// logically overwrite the old one, including when the new's `Path` subsumes the old's.
///
/// This type enforces requirements that use of a `Store` must uphold, but, otherwise, it
/// delegates to a [`StoreExt`] type that provides the primary implementation.
#[derive(Debug)]
#[allow(clippy::partial_pub_fields)]
pub struct Store<NamespaceId, Ext>
{
    /// Which Namespace this `Store` is for.
    pub namespace_id: NamespaceId,
    ext:              Ext,
}

impl<Params, Ext> Store<Params::NamespaceId, Ext>
where
    Params: crate::Params + ?Sized,
    Ext: StoreExt<Params = Params>,
{
    /// Make a new `Self`, that initially is empty, for the given Namespace.
    ///
    /// Creates the Namespace if it doesn't already exist.
    ///
    /// # Errors
    /// If creating the new `Self` fails for any reason.
    #[inline]
    pub fn new(
        namespace_id: &Params::NamespaceId,
        args: Ext::NewArgs,
    ) -> Result<Self, Ext::NewError>
    {
        Ok(Self {
            namespace_id: namespace_id.clone(),
            ext:          Ext::new(namespace_id, args)?,
        })
    }

    /// Make a new `Self`, that initially has only a single `Entry`, for the Namespace given by
    /// `auth_entry.entry().namespace_id`.
    ///
    /// Creates the Namespace if it doesn't already exist.
    ///
    /// # Errors
    /// If creating the new `Self` fails for any reason.
    #[inline]
    pub fn singleton(
        auth_entry: &ParamsAuthorisedEntry<Params, impl Path>,
        payload: Option<impl Payload>,
        args: Ext::NewArgs,
    ) -> Result<Self, SingletonError<Ext>>
    {
        let mut new =
            Self::new(&auth_entry.entry().namespace_id, args).map_err(SingletonError::New)?;
        new.ext.put(auth_entry, payload).map_err(SingletonError::Put)?;
        Ok(new)
    }

    /// Retrieve the [`Payload`] of an [`Entry`].
    ///
    /// Returns an [`Iterator`] of chunks that represents a single logical byte-string.  This
    /// allows the `impl`ementor flexibility in the representation (e.g. to retrieve chunks lazily
    /// and not hold them all in-memory at once).  The sizes of and boundaries between chunks are
    /// arbitrary and might be inconsistent across calls for the same `Entry`.
    ///
    /// Returns `None` if there is no such [`Entry`] stored in this `Store`'s Namespace, including
    /// when an old [`Entry`] has been overwritten even if overwritten [`Entry`]s are still
    /// persisted somehow.
    ///
    /// # Errors
    /// If retrieval fails for any reason.
    #[inline]
    pub fn get(
        &self,
        subspace_id: &Params::SubspaceId,
        path: &impl Path,
    ) -> Result<Option<Ext::GetPayload>, Ext::GetError>
    {
        self.ext.get(subspace_id, path)
    }

    /// TODO ... unique `&mut self` enables approaches that exclude concurrent access; but
    /// different approaches that support concurrency can involve multiple clones of `Self` so
    /// that each can access concurrently (because each `self` can be borrowed `&mut`
    /// independently)
    ///
    /// If `payload` is `None`, store `auth_entry` without its payload.  The same `Entry` can
    /// later be `put` again with `payload` being `Some`.
    ///
    /// # Errors
    /// If putting fails for any reason.
    #[inline]
    pub fn put(
        &mut self,
        auth_entry: &ParamsAuthorisedEntry<Params, impl Path>,
        payload: Option<impl Payload>,
    ) -> Result<(), PutError<Ext>>
    {
        if self.namespace_id == auth_entry.entry().namespace_id {
            self.ext.put(auth_entry, payload).map_err(PutError::Ext)
        }
        else {
            Err(PutError::DifferentNamespace)
        }
    }

    /// TODO ... maybe copy-paste the def of this op from the Willow Data Model webpage ...  note
    /// because this gets the `Entry`s of another `Store`, those `Entry`s have digests that were
    /// already verified and have `timestamp`s that are already as desired ...
    ///
    /// # Errors
    /// If joining fails for any reason.
    #[inline]
    pub fn join(
        &mut self,
        other: &Self,
    ) -> Result<(), JoinError<Ext>>
    {
        if self.namespace_id == other.namespace_id {
            self.ext.join(&other.ext).map_err(JoinError::Ext)
        }
        else {
            Err(JoinError::DifferentNamespace)
        }
    }

    /// Return an [`Iterator`] of all of `self`'s [`Entry`](crate::Entry)s and their
    /// [`AuthorisationToken`](crate::Params::AuthorisationToken)s.
    #[inline]
    pub fn iter(&self) -> Ext::Iter
    {
        self.ext.iter()
    }

    #[allow(clippy::todo, unused_variables)] // TODO: remove
    pub(crate) fn newest_entries_include<P>(
        &self,
        max_count: u64,
        entry: impl Borrow<ParamsEntry<Params, P>>,
    ) -> bool
    where
        P: Path,
    {
        todo!()
    }

    /// Returns the sum of the `payload_length`s of `entry` and all [newer](Entry::is_newer_than)
    /// `Entry`s in `self`, or `None` if summing overflowed.
    #[allow(clippy::todo, unused_variables)] // TODO: remove
    pub(crate) fn payloads_total_size_of_entry_to_newest<P>(
        &self,
        entry: impl Borrow<ParamsEntry<Params, P>>,
    ) -> Option<u128>
    where
        P: Path,
    {
        todo!()
    }
}


/// Errors possibly returned by [`Store::singleton`].
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum SingletonError<Ext: StoreExt>
{
    /// Failure of [`StoreExt::new`]
    New(Ext::NewError),
    /// Failure of [`StoreExt::put`]
    Put(Ext::PutError),
}

/// Errors possibly returned by [`Store::put`].
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum PutError<Ext: StoreExt>
{
    /// The `auth_entry` argument is not for the same Namespace.
    DifferentNamespace,
    /// Failure of [`StoreExt::put`]
    Ext(Ext::PutError),
}

/// Errors possibly returned by [`Store::join`].
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum JoinError<Ext: StoreExt>
{
    /// The `other` argument is not for the same Namespace.
    DifferentNamespace,
    /// Failure of [`StoreExt::join`]
    Ext(Ext::JoinError),
}


impl<Params, Ext> IntoIterator for &Store<Params::NamespaceId, Ext>
where
    Params: crate::Params + ?Sized,
    Ext: StoreExt<Params = Params>,
{
    type IntoIter = Ext::Iter;
    type Item = <Ext::Iter as Iterator>::Item;

    #[inline]
    fn into_iter(self) -> Self::IntoIter
    {
        self.iter()
    }
}


/// See [`Store`].
#[allow(clippy::missing_errors_doc)]
pub trait StoreExt: Sized
{
    /// Our specific parameterisation of the Willow Data Model.
    type Params: crate::Params + ?Sized;
    /// Arguments for [`new`](Self::new).
    type NewArgs;
    /// Error(s) possibly returned by [`new`](Self::new).
    type NewError;
    /// Success possibly returned by [`get`](Self::get).
    type GetPayload: Payload;
    /// Error(s) possibly returned by [`get`](Self::get).
    type GetError;
    /// Error(s) possibly returned by [`put`](Self::put).
    type PutError;
    /// Error(s) possibly returned by [`join`](Self::join).
    type JoinError;
    /// Returned by [`iter`](Self::iter).
    type Iter: Iterator<Item = ParamsAuthorisedEntry<Self::Params, Self::IterPath>>;
    /// Part of what is yielded by [`Self::Iter`].
    type IterPath: Path;

    /// See [`Store::new`].
    fn new(
        namespace_id: &<Self::Params as crate::Params>::NamespaceId,
        args: Self::NewArgs,
    ) -> Result<Self, Self::NewError>;

    /// See [`Store::get`].
    fn get(
        &self,
        subspace_id: &<Self::Params as crate::Params>::SubspaceId,
        path: &impl Path,
    ) -> Result<Option<Self::GetPayload>, Self::GetError>;

    /// See [`Store::put`].
    fn put(
        &mut self,
        auth_entry: &ParamsAuthorisedEntry<Self::Params, impl Path>,
        payload: Option<impl Payload>,
    ) -> Result<(), Self::PutError>;

    /// See [`Store::join`].
    fn join(
        &mut self,
        other: &Self,
    ) -> Result<(), Self::JoinError>;

    /// See [`Store::iter`].
    fn iter(&self) -> Self::Iter;
}
