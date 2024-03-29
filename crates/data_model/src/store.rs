//! Aspects of `Store`s.

use {
    crate::{
        AuthorisedEntry,
        ParamsEntry,
        Path,
        Payload,
    },
    core::borrow::Borrow,
};


mod errors;
pub use errors::*;


// TODO: Make a separate impl of this that uses FS, e.g. sailce/crates/fs_store/ with tests.
//
/// A _store_ is a set of [`AuthorisedEntry`]s such that
/// - all its [`Entry`](crate::Entry)s have the same [`namespace_id`](crate::Entry::namespace_id),
///   and
/// - there are no two of its `Entry`s `old` and `new` such that
///   - `old.subspace_id == new.subspace_id`, and
///   - `new.path` is a [prefix](Path::is_prefix_of) of `old.path`,
///   - and `new` is [newer](crate::Entry::is_newer_than) than `old`.
///
///  (That includes the formal definition of _prefix pruning_.)
///
/// I.e., storing a new `Entry` at the same 3-D location as another `Entry` in a Namespace will
/// logically overwrite the old one, including when the new's `Path` subsumes the old's.
///
/// This type enforces requirements that use of a `Store` must uphold, but, otherwise, it
/// delegates to a [`StoreExt`] type that provides the primary implementation.
///
/// Most of the methods of this type are provided by the [`async::Store`] trait, so see that also.
#[derive(Clone, Debug)]
pub struct Store<NamespaceId, Ext>
{
    namespace_id: NamespaceId,
    ext:          Ext,
}

impl<Params, Ext> Store<Params::NamespaceId, Ext>
where
    Params: crate::Params + ?Sized,
    Ext: StoreExt<Params = Params>,
{
    /// Make a new `Self`, that initially is empty, for the given Namespace.
    ///
    /// The given `ext` must be prepared for the same Namespace.  For some types of [`StoreExt`]
    /// this might not require anything or might be trivial, but for other types, that involve
    /// representing the Namespace in their instances' state, this might require ensuring that
    /// `ext` was created with the same `namespace_id`.  It is a logic error to give an `ext` that
    /// isn't prepared for the same Namespace, and doing so would probably cause unspecified
    /// misbehavior.
    ///
    /// Creates the Namespace if it doesn't already exist.
    #[inline]
    pub fn new(
        namespace_id: &Params::NamespaceId,
        ext: Ext,
    ) -> Self
    {
        Self { namespace_id: namespace_id.clone(), ext }
    }

    /// The [`NamespaceId`](crate::Params::NamespaceId) that `self` is for.
    #[inline]
    pub fn namespace_id(&self) -> &Params::NamespaceId
    {
        &self.namespace_id
    }

    /// Return an [`Iterator`] of all of `self`'s [`Entry`](crate::Entry)s and their
    /// [`AuthorisationToken`](crate::Params::AuthorisationToken)s.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = StoreAuthorisedEntry<Ext>> + '_
    {
        self.ext.iter(&self.namespace_id)
    }

    #[allow(clippy::todo, unused_variables)] // TODO: remove
    pub(crate) fn newest_entries_include<P: Path>(
        &self,
        max_count: u64,
        entry: impl Borrow<ParamsEntry<Params, P>>,
    ) -> bool
    {
        todo!()
    }

    /// Returns the sum of the `payload_length`s of `entry` and all [newer](Entry::is_newer_than)
    /// `Entry`s in `self`, or `None` if summing overflowed.
    #[allow(clippy::todo, unused_variables)] // TODO: remove
    pub(crate) fn payloads_total_size_of_entry_to_newest<P: Path>(
        &self,
        entry: impl Borrow<ParamsEntry<Params, P>>,
    ) -> Option<u128>
    {
        todo!()
    }
}


/// Aspects of `async`-API `Store`s.
pub mod r#async
{
    use {
        super::{
            errors::{
                JoinError,
                PutError,
            },
            StoreExt,
        },
        crate::{
            AuthorisedEntry,
            Path,
            Payload,
        },
        core::borrow::Borrow,
    };


    /// Only [`super::Store`] may implement [`Store`].
    trait Sealed {}

    /// The primary methods of [`crate::Store`].  As a trait, to enable easy disambiguation
    /// between [`sync::Store`](super::sync::Store) methods - so only one of these traits can be
    /// in-scope.
    ///
    /// The `async` methods are the primary interface for their respective functionality.  This
    /// `async` API can still be used from sync code that wishes to block (instead of `.await`
    /// suspending), by using the [`sync::Store`](super::sync::Store) trait that extends this.
    /// The `async` API allows the `impl StoreExt for Ext` flexibility in the representation and
    /// operations (e.g. for slow I/O or for calling [`Payload`] methods which are `async`.).
    #[allow(private_bounds, async_fn_in_trait)] // TODO: Re-evaluate `async_fn_in_trait`.
    pub trait Store<Params, Ext>: Sealed
    where
        Params: crate::Params + ?Sized,
        Ext: StoreExt<Params = Params>,
    {
        /// Retrieve the [`Payload`] of an [`Entry`](crate::Entry).
        ///
        /// Returns `None` if there is no such `Entry` stored in this `Store`'s Namespace,
        /// including when an old `Entry` has been overwritten even if overwritten
        /// `Entry`s are still persisted somehow.
        ///
        /// When `Ok(Some(payload))` is returned, the current seek position of `payload` is `0` so
        /// that `read`s start from the beginning.
        ///
        /// # Errors
        /// If retrieval fails for any reason.
        async fn get(
            &self,
            subspace_id: &Params::SubspaceId,
            path: &(impl Path + ?Sized),
        ) -> Result<Option<Ext::GetPayload>, Ext::GetError>;

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
        async fn put<P: Payload>(
            &mut self,
            auth_entry: AuthorisedEntry<
                Params,
                impl Path,
                impl Borrow<Params::AuthorisationToken>,
            >,
            payload: Option<P>,
        ) -> Result<(), PutError<Ext::PutError<P>>>;

        /// TODO ... maybe copy-paste the def of this op from the Willow Data Model webpage ...
        /// note because this gets the `Entry`s of another `Store`, those `Entry`s have
        /// digests that were already verified and have `timestamp`s that are already as
        /// desired ...
        ///
        /// # Errors
        /// If joining fails for any reason.
        async fn join(
            &mut self,
            other: &Self,
        ) -> Result<(), JoinError<Ext::JoinError>>;
    }


    /// The only type that may implement our trait.
    impl<Params, Ext> Sealed for super::Store<Params::NamespaceId, Ext>
    where
        Params: crate::Params + ?Sized,
        Ext: StoreExt<Params = Params>,
    {
    }

    /// The actual implementation of [`crate::Store`]'s primary methods.
    impl<Params, Ext> Store<Params, Ext> for super::Store<Params::NamespaceId, Ext>
    where
        Params: crate::Params + ?Sized,
        Ext: StoreExt<Params = Params>,
    {
        #[inline]
        async fn get(
            &self,
            subspace_id: &Params::SubspaceId,
            path: &(impl Path + ?Sized),
        ) -> Result<Option<Ext::GetPayload>, Ext::GetError>
        {
            self.ext.get(&self.namespace_id, subspace_id, path).await
        }

        #[inline]
        async fn put<P: Payload>(
            &mut self,
            auth_entry: AuthorisedEntry<
                Params,
                impl Path,
                impl Borrow<Params::AuthorisationToken>,
            >,
            payload: Option<P>,
        ) -> Result<(), PutError<Ext::PutError<P>>>
        {
            if self.namespace_id == auth_entry.entry().namespace_id {
                self.ext.put(&self.namespace_id, auth_entry, payload).await.map_err(PutError::Put)
            }
            else {
                Err(PutError::DifferentNamespace)
            }
        }

        #[inline]
        async fn join(
            &mut self,
            other: &Self,
        ) -> Result<(), JoinError<Ext::JoinError>>
        {
            if self.namespace_id == other.namespace_id {
                self.ext
                    .join(&self.namespace_id, &other.ext, &other.namespace_id)
                    .await
                    .map_err(JoinError::Join)
            }
            else {
                Err(JoinError::DifferentNamespace)
            }
        }
    }
}


/// See [`Store`].
///
/// The `namespace_id` parameter of each method enables knowing the instance's Namespace, for
/// implementations of `StoreExt` that don't store it (e.g. because they don't otherwise need it,
/// and the wrapping `Store` instance already handles it).
#[allow(async_fn_in_trait, clippy::missing_errors_doc)] // TODO: Re-evaluate `async_fn_in_trait`.
pub trait StoreExt: Sized
{
    /// Our specific parameterisation of the Willow Data Model.
    type Params: crate::Params + ?Sized;
    /// Success possibly returned by [`get`](Self::get).
    type GetPayload: Payload;
    /// Error(s) possibly returned by [`get`](Self::get).
    type GetError;
    /// Error(s) possibly returned by [`put`](Self::put).
    type PutError<P: Payload + ?Sized>;
    /// Error(s) possibly returned by [`join`](Self::join).
    type JoinError;
    /// Part of what is yielded by the type returned by [`Self::iter`].
    type IterPath: Path;
    /// Part of what is yielded by the type returned by [`Self::iter`].
    type IterAuthToken: Borrow<<Self::Params as crate::Params>::AuthorisationToken>;

    /// See [`Store::get`](async::Store::get).
    async fn get(
        &self,
        namespace_id: &<Self::Params as crate::Params>::NamespaceId,
        subspace_id: &<Self::Params as crate::Params>::SubspaceId,
        path: &(impl Path + ?Sized),
    ) -> Result<Option<Self::GetPayload>, Self::GetError>;

    /// See [`Store::put`](async::Store::put).
    async fn put<P: Payload>(
        &mut self,
        namespace_id: &<Self::Params as crate::Params>::NamespaceId,
        auth_entry: AuthorisedEntry<
            Self::Params,
            impl Path,
            impl Borrow<<Self::Params as crate::Params>::AuthorisationToken>,
        >,
        payload: Option<P>,
    ) -> Result<(), Self::PutError<P>>;

    /// See [`Store::join`](async::Store::join).
    async fn join(
        &mut self,
        namespace_id: &<Self::Params as crate::Params>::NamespaceId,
        other: &Self,
        other_namespace_id: &<Self::Params as crate::Params>::NamespaceId,
    ) -> Result<(), Self::JoinError>;

    /// See [`Store::iter`].
    ///
    /// It seems reasonable for this to not be `async`, because creating an instance of an
    /// `Iterator` type can be done without blocking, and because the operations on
    /// `Iterator`s aren't `async`.
    fn iter(
        &self,
        namespace_id: &<Self::Params as crate::Params>::NamespaceId,
    ) -> impl Iterator<Item = StoreAuthorisedEntry<Self>>;
}


/// Same as [`AuthorisedEntry`] with type arguments from the given [`StoreExt`].
pub type StoreAuthorisedEntry<Ext> = AuthorisedEntry<
    <Ext as StoreExt>::Params,
    <Ext as StoreExt>::IterPath,
    <Ext as StoreExt>::IterAuthToken,
>;


#[cfg(test)]
#[allow(clippy::todo)]
mod tests
{
    #[ignore]
    #[test]
    fn newest_entries_include()
    {
        todo!()
    }

    #[ignore]
    #[test]
    fn payloads_total_size_of_entry_to_newest()
    {
        todo!()
    }
}


/// Aspects of synchronous-API `Store`s.
pub mod sync
{
    use {
        super::{
            r#async,
            errors::{
                JoinError,
                PutError,
            },
            StoreExt,
        },
        crate::{
            syncify::Syncify,
            AuthorisedEntry,
            Path,
            Payload,
        },
        core::borrow::Borrow,
    };


    /// Like [`async::Store`] but all methods are synchronous (i.e. not `async`) and might block
    /// callers.
    #[allow(clippy::missing_errors_doc)]
    pub trait Store<Executor, Params, Ext>:
        r#async::Store<Params, Ext> + Syncify<Executor>
    where
        Executor: ?Sized,
        Params: crate::Params + ?Sized,
        Ext: StoreExt<Params = Params>,
    {
        /// Like [`async::Store::get`] but synchronous.  Might block.
        #[inline]
        fn get(
            &self,
            subspace_id: &Params::SubspaceId,
            path: &(impl Path + ?Sized),
        ) -> Result<Option<Ext::GetPayload>, Ext::GetError>
        {
            let (block_on, data) = get_block_on_and_data!(self);
            block_on(r#async::Store::get(self, subspace_id, path), data)
        }

        /// Like [`async::Store::put`] but synchronous.  Might block.
        #[inline]
        fn put<P: Payload>(
            &mut self,
            auth_entry: AuthorisedEntry<
                Params,
                impl Path,
                impl Borrow<Params::AuthorisationToken>,
            >,
            payload: Option<P>,
        ) -> Result<(), PutError<Ext::PutError<P>>>
        {
            let (block_on, data) = get_block_on_and_data!(self);
            block_on(r#async::Store::put(self, auth_entry, payload), data)
        }

        /// Like [`async::Store::join`] but synchronous.  Might block.
        #[inline]
        fn join(
            &mut self,
            other: &Self,
        ) -> Result<(), JoinError<Ext::JoinError>>
        {
            let (block_on, data) = get_block_on_and_data!(self);
            block_on(r#async::Store::join(self, other), data)
        }

        // TODO?: newest_entries_include & payloads_total_size_of_entry_to_newest ?
    }
}
