#![cfg_attr(unix, doc = include_str!("../README.md"))]
#![cfg_attr(windows, doc = include_str!("..\\README.md"))]
//!
//! Note: Many of the doc-comments of this crate are copied from the specification of Willow's
//! [Data Model](https://willowprotocol.org/specs/data-model/index.html).

// Apply the `no_std` attribute unconditionally, to require explicit conditional `use` of
// non-`core` items.
#![no_std]

use core::{
    future::Future,
    num::NonZeroUsize,
};


mod entry;
pub use entry::*;

pub mod group;

pub mod path;
pub use path::{
    EmptyPath,
    Path,
};

pub mod store;
pub use store::{
    Store,
    StoreExt,
};


/// Willow is a higher-order protocol: you supply specific choices for its parameters, and you get
/// a concrete protocol that you can then use.
///
/// If different systems instantiate Willow with different parameters, they will not be
/// interoperable, even though both systems use Willow.
pub trait Params
{
    /// Identifies Namespaces.
    type NamespaceId: Clone + Eq;
    /// Identifies Subspaces.
    type SubspaceId: Clone + Eq;
    /// Content-addresses the data that Willow stores.
    type PayloadDigest: Ord;
    /// Proves write permission.
    type AuthorisationToken;

    /// Limits the length of each `Component` of a [`Path`].
    const MAX_COMPONENT_LENGTH: NonZeroUsize;
    /// Limits the amount of `Component`s per [`Path`].
    const MAX_COMPONENT_COUNT: NonZeroUsize;
    /// Limits the total length of all `Component`s of a [`Path`].
    const MAX_PATH_LENGTH: NonZeroUsize;

    /// Computes the [`PayloadDigest`](Self::PayloadDigest) of a byte-string (of length at most
    /// [`u64::MAX`]).
    ///
    /// Since this function provides the only way in which Willow tracks `Payload`s, you probably
    /// want to use a [secure hash function](https://en.wikipedia.org/wiki/Secure_hash_function).
    #[allow(async_fn_in_trait)] // TODO: re-evaluate
    #[must_use]
    async fn hash_payload(payload: impl Payload) -> Self::PayloadDigest;

    /// Indicates whether the given `auth_token` proves write permission for the given `entry`.
    // TODO: Should this be `async`? To support impls that might take a while, block on I/O, etc?
    #[must_use]
    fn is_authorised_write(
        entry: &ParamsEntry<Self, impl Path>,
        auth_token: &Self::AuthorisationToken,
    ) -> bool;
}


// TODO: Reconsider if this should be some kind of seekable interface, instead of an interator, to
// allow uses more like a traditional file that want to seek without getting contents from the
// start.
//
/// An arbitrary sequence of bytes.  I.e. a single logical byte-string.  At most [`u64::MAX`]
/// bytes.
///
/// Applications read and write `Payload`s from and to Subspaces, addressing via hierarchical
/// `Path`s.
///
/// (If `core::async_iter::AsyncIterator` becomes stabilized, it might be better to change the
/// super-trait bound to that.)
pub trait Payload: IntoIterator<Item = Self::FutureChunk>
{
    /// The item type given by the iterator is a [`Future`] of a chunk of bytes.
    ///
    /// This allows a variety of `impl`ementations (e.g. chunks which involve blocking to
    /// retrieve).
    type FutureChunk: Future<Output = Self::Chunk>;

    /// A single chunk of bytes.
    ///
    /// All the chunks together in sequence represent the single logical byte-string.  The sizes
    /// of and boundaries between chunks are arbitrary and might be inconsistent across the same
    /// type or multiple uses of the same instance.
    ///
    /// This allows a variety of `impl`ementations (e.g. multiple chunks which aren't all
    /// in-memory at once).
    type Chunk: AsRef<[u8]>;
}


/// A time in microseconds since the [Unix epoch](https://en.wikipedia.org/wiki/Unix_epoch).
#[derive(Default, Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct Timestamp
{
    /// The microseconds since the Unix epoch.
    pub μs_since_epoch: u64,
}

impl From<u64> for Timestamp
{
    #[inline]
    fn from(value: u64) -> Self
    {
        Self { μs_since_epoch: value }
    }
}
