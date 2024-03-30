#![cfg_attr(unix, doc = include_str!("../README.md"))]
#![cfg_attr(windows, doc = include_str!("..\\README.md"))]
//!
//! Note: Many of the doc-comments of this crate are copied from the specification of Willow's
//! [Data Model](https://willowprotocol.org/specs/data-model/index.html).

// Apply the `no_std` attribute unconditionally, to require explicit conditional `use` of
// non-`core` items.
#![no_std]
// Our workspace ensures that unstable Rust features are denied by default.  But allow if our
// package-feature "anticipate" is activated.
#![cfg_attr(feature = "anticipate", allow(unstable_features))]
// When our package-feature "anticipate" is activated, cause breaking changes to our API that use
// Rust-features that our crate anticipates adopting in a future version if they become stable.
// While unstable, they must be enabled here; or if some become stable, they will already be
// enabled.
#![cfg_attr(
    all(feature = "anticipate", not(rust_lib_feature = "error_in_core")),
    feature(error_in_core)
)]
// Warn about this one but avoid annoying hits for dev-dependencies.
#![cfg_attr(test, allow(unused_crate_dependencies))]


#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use {
    cfg_if as _, // Suppress `unused_crate_dependencies` lint, when package-features alter use.
    core::num::NonZeroUsize,
};


mod entry;
pub use entry::*;

// The items in this are not re-exported, because the Grouping aspects are defined by the Willow
// Specification as separate from the Core Data Model.  This crate provides them, because they're
// closely related.
pub mod group;

pub mod path;
pub use path::{
    EmptyPath,
    Path,
};

pub mod syncify; // Must be before all modules that use its macro.

pub mod payload;
pub use payload::Payload;

pub mod store;
pub use store::{
    Store,
    StoreAuthorisedEntry,
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
    /// Error(s) possibly returned by [`hash_payload`](Self::hash_payload).
    type HashPayloadError<P: Payload + ?Sized>;

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
    async fn hash_payload<P: Payload + ?Sized>(
        payload: &mut P
    ) -> Result<Self::PayloadDigest, Self::HashPayloadError<P>>;

    /// Indicates whether the given `auth_token` proves write permission for the given `entry`.
    // TODO: Should this be `async`? To support impls that might take a while, block on I/O, etc?
    #[must_use]
    fn is_authorised_write(
        entry: &ParamsEntry<Self, impl Path>,
        auth_token: &Self::AuthorisationToken,
    ) -> bool;
}


/// A time in microseconds since the [Unix epoch](https://en.wikipedia.org/wiki/Unix_epoch).
#[derive(Default, Copy, Clone, Eq, Ord, Hash, PartialEq, PartialOrd, Debug)]
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


// TODO?: Should there be a `sync::Params` that uses `Syncify`, to have sync adaptors of its
// `async` methods?
