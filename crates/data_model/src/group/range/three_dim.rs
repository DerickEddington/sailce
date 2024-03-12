//! When we combine [`Range`]s of all three dimensions, we can delimit boxes in Willow space.

use {
    super::{
        End,
        Least,
        Range,
    },
    crate::{
        EmptyPath,
        Entry,
        Timestamp,
    },
    core::borrow::Borrow,
};


/// A three-dimensional range that includes every [`Entry`] included in all three of its
/// [`Range`]s.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct ThreeDimRange<SubspaceId, Path>
{
    /// The range of [`Subspace`](crate::Params::SubspaceId)s.
    pub subspaces: Range<SubspaceId>,
    /// The range of [`Path`](crate::Path)s.
    pub paths:     Range<Path>,
    /// The range of [`Timestamp`]s.
    pub times:     Range<Timestamp>,
}


impl<SubspaceId, Path> ThreeDimRange<SubspaceId, Path>
where
    SubspaceId: Ord,
    Path: Ord,
{
    /// Creates an empty 3-D range.
    #[must_use]
    #[inline]
    pub fn empty() -> Self
    where
        SubspaceId: Default,
        Path: Default,
    {
        Self {
            subspaces: Range::<SubspaceId>::empty(),
            paths:     Range::<Path>::empty(),
            times:     Range::<Timestamp>::empty(),
        }
    }

    /// A 3-D range includes every [`Entry`] whose `subspace_id`, `path`, and `timestamp` are all
    /// [included](Range::includes) in their respective `Range`.
    #[must_use]
    #[inline]
    pub fn includes<NamespaceId, PayloadDigest>(
        &self,
        entry: impl Borrow<Entry<NamespaceId, SubspaceId, Path, PayloadDigest>>,
    ) -> bool
    {
        let entry = entry.borrow();
        self.subspaces.includes(&entry.subspace_id) && self.paths.includes(&entry.path) && {
            #[allow(clippy::needless_borrows_for_generic_args)]
            self.times.includes(&entry.timestamp)
        }
    }

    /// A 3-D range is _empty_ if any one of its dimensions includes no values.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool
    {
        self.subspaces.is_empty() || self.paths.is_empty() || self.times.is_empty()
    }

    /// The intersection of `self` and `other` is the `Self` whose ranges are the
    /// [`intersection`](Range::intersection)s of the corresponding ranges of `self` and `other`.
    ///
    /// (This is analogous to [`Range::intersection`], but was not part of the Willow documents
    /// (as of 2024-03), but this would seem to be appropriate.)
    #[must_use]
    #[inline]
    pub fn intersection(
        &self,
        other: impl Borrow<Self>,
    ) -> Self
    where
        SubspaceId: Clone,
        Path: Clone,
    {
        let other = other.borrow();
        Self {
            subspaces: self.subspaces.intersection(&other.subspaces),
            paths:     self.paths.intersection(&other.paths),
            times:     {
                #[allow(clippy::needless_borrows_for_generic_args)]
                self.times.intersection(&other.times)
            },
        }
    }
}


/// We define `default_3d_range(default_subspace)` to denote the [`ThreeDimRange`] with the
/// following members:
/// - `subspaces` is the open `Range<SubspaceId>` with `start` `default_subspace`,
/// - `paths` is the open `Range<Path>` whose `start` is the empty [`Path`](crate::Path), and
/// - `times` is the open `Range<Timestamp>` with `start` `0`.
///
/// This is the three-dimensional range that includes the entire space of **all** [`Entry`]s in a
/// Namespace.
impl<SubspaceId, Path> Default for ThreeDimRange<SubspaceId, Path>
where
    SubspaceId: Least,
    Path: EmptyPath,
{
    #[inline]
    fn default() -> Self
    {
        Self {
            subspaces: Range { start: SubspaceId::least(), end: End::Open },
            paths:     Range { start: Path::empty(), end: End::Open },
            times:     Range { start: Timestamp { μs_since_epoch: 0 }, end: End::Open },
        }
    }
}
