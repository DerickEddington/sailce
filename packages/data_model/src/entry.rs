use {
    crate::{
        path::Extra as _,
        Path,
        Timestamp,
    },
    core::cmp::Ordering,
};


mod auth;
pub use auth::*;


/// The metadata for storing a [`Payload`](crate::Payload).
///
/// The [`Ord`]ering of values of this type is based on the order of its fields, so that `Entry`s
/// are first ordered by Namespace, then by Subspace, then by `Path`, and then, for the remaining
/// fields, by the same ordering as [`cmp_newer_than`](Self::cmp_newer_than).
#[derive(Copy, Clone, Hash, Debug)]
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
        self.cmp_newer_than(other) == Ordering::Greater
    }

    /// Like [`Self::is_newer_than`] but returns the [`Ordering`].
    #[must_use]
    #[inline]
    pub fn cmp_newer_than(
        &self,
        other: &Self,
    ) -> Ordering
    {
        match self.timestamp.cmp(&other.timestamp) {
            Ordering::Equal => match self.payload_digest.cmp(&other.payload_digest) {
                Ordering::Equal => self.payload_length.cmp(&other.payload_length),
                unequal @ (Ordering::Less | Ordering::Greater) => unequal,
            },
            unequal @ (Ordering::Less | Ordering::Greater) => unequal,
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


// The following are not `derive`d, so that their `Rhs`s can be generic and so that their `path`s
// can be compared by `Component`s.

impl<Na, Sa, Pa, Da, Nb, Sb, Pb, Db> PartialEq<Entry<Nb, Sb, Pb, Db>> for Entry<Na, Sa, Pa, Da>
where
    Na: PartialEq<Nb>,
    Sa: PartialEq<Sb>,
    Pa: Path,
    Pb: Path,
    Da: PartialEq<Db>,
{
    #[inline]
    fn eq(
        &self,
        other: &Entry<Nb, Sb, Pb, Db>,
    ) -> bool
    {
        self.namespace_id == other.namespace_id
            && self.subspace_id == other.subspace_id
            && self.path.eq_components(&other.path)
            && self.timestamp == other.timestamp
            && self.payload_digest == other.payload_digest
            && self.payload_length == other.payload_length
    }
}

impl<N, S, P, D> Eq for Entry<N, S, P, D>
where
    N: Eq,
    S: Eq,
    P: Path,
    D: Eq,
{
}

impl<Na, Sa, Pa, Da, Nb, Sb, Pb, Db> PartialOrd<Entry<Nb, Sb, Pb, Db>> for Entry<Na, Sa, Pa, Da>
where
    Na: PartialOrd<Nb>,
    Sa: PartialOrd<Sb>,
    Pa: Path,
    Pb: Path,
    Da: PartialOrd<Db>,
{
    #[inline]
    fn partial_cmp(
        &self,
        other: &Entry<Nb, Sb, Pb, Db>,
    ) -> Option<Ordering>
    {
        match self.namespace_id.partial_cmp(&other.namespace_id) {
            Some(Ordering::Equal) => match self.subspace_id.partial_cmp(&other.subspace_id) {
                Some(Ordering::Equal) => match self.path.cmp_components(&other.path) {
                    Ordering::Equal => match self.timestamp.partial_cmp(&other.timestamp) {
                        Some(Ordering::Equal) =>
                            match self.payload_digest.partial_cmp(&other.payload_digest) {
                                Some(Ordering::Equal) =>
                                    self.payload_length.partial_cmp(&other.payload_length),
                                unequal => unequal,
                            },
                        unequal => unequal,
                    },
                    unequal @ (Ordering::Less | Ordering::Greater) => Some(unequal),
                },
                unequal => unequal,
            },
            unequal => unequal,
        }
    }
}

impl<N, S, P, D> Ord for Entry<N, S, P, D>
where
    N: Ord,
    S: Ord,
    P: Path,
    D: Ord,
{
    #[inline]
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering
    {
        match self.namespace_id.cmp(&other.namespace_id) {
            Ordering::Equal => match self.subspace_id.cmp(&other.subspace_id) {
                Ordering::Equal => match self.path.cmp_components(&other.path) {
                    Ordering::Equal => match self.timestamp.cmp(&other.timestamp) {
                        Ordering::Equal => match self.payload_digest.cmp(&other.payload_digest) {
                            Ordering::Equal => self.payload_length.cmp(&other.payload_length),
                            unequal @ (Ordering::Less | Ordering::Greater) => unequal,
                        },
                        unequal @ (Ordering::Less | Ordering::Greater) => unequal,
                    },
                    unequal @ (Ordering::Less | Ordering::Greater) => unequal,
                },
                unequal @ (Ordering::Less | Ordering::Greater) => unequal,
            },
            unequal @ (Ordering::Less | Ordering::Greater) => unequal,
        }
    }
}
