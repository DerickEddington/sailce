use {
    crate::Timestamp,
    core::cmp::Ordering,
};


mod auth;
pub use auth::*;


/// The metadata for storing a [`Payload`].
///
/// The [`Ord`]ering of values of this type is based on the order of its fields, so that `Entry`s
/// are first ordered by Namespace, then by Subspace, then by `Path`, and then, for the remaining
/// fields, by the same ordering as [`is_newer_than`](Self::is_newer_than).
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct Entry<NamespaceId, SubspaceId, Path, PayloadDigest>
{
    /// Namespace to which this `Entry` belongs.
    pub namespace_id:   NamespaceId,
    /// Subspace to which this `Entry` belongs.
    pub subspace_id:    SubspaceId,
    /// Path to which this `Entry` was written.
    pub path:           Path,
    /// Claimed creation time of this `Entry`.
    pub timestamp:      Timestamp,
    /// Result of applying [`hash_payload`](crate::Params::hash_payload) to this `Entry`'s
    /// `Payload`.
    pub payload_digest: PayloadDigest,
    /// Length of this `Entry`'s `Payload` in bytes.
    pub payload_length: u64,
}

impl<N, S, P, D: Ord> Entry<N, S, P, D>
{
    /// An `Entry` `e1` is _newer_ than another `Entry` `e2` if
    /// - `e2.timestamp < e1.timestamp`, or
    /// - `e2.timestamp == e1.timestamp` and `e2.payload_digest < e1.payload_digest`, or
    /// - `e2.timestamp == e1.timestamp` and `e2.payload_digest == e1.payload_digest` and
    ///   `e2.payload_length < e1.payload_length`.
    ///
    /// (To support this comparison, [`PayloadDigest`](crate::Params::PayloadDigest) is required
    /// to be [totally ordered](Ord).)
    #[must_use]
    #[inline]
    pub fn is_newer_than(
        &self,
        other: &Self,
    ) -> bool
    {
        match self.timestamp.cmp(&other.timestamp) {
            Ordering::Equal => match self.payload_digest.cmp(&other.payload_digest) {
                Ordering::Equal => self.payload_length > other.payload_length,
                Ordering::Less => false,
                Ordering::Greater => true,
            },
            Ordering::Less => false,
            Ordering::Greater => true,
        }
    }
}


/// Same as [`Entry`] with type arguments from the given [`Params`](crate::Params).
pub type ParamsEntry<Params, Path> = Entry<
    <Params as crate::Params>::NamespaceId,
    <Params as crate::Params>::SubspaceId,
    Path,
    <Params as crate::Params>::PayloadDigest,
>;
