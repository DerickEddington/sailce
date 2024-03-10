//! Occasionally, we wish to group [`Entry`]s based on the contents of some [`Store`].  For
//! example, a space-constrained peer might ask for the 100 newest `Entry`s when synchronising
//! data.
//!
//! We serve these use cases by combining an [`Area`] with limits to restrict the contents to the
//! `Entry`s with the greatest [`Timestamp`]s.

use {
    super::Area,
    crate::{
        Entry,
        Path,
        Store,
        StoreExt,
    },
    core::borrow::Borrow,
};


/// A grouping of [`Entry`]s that are among the newest in some [`Store`].
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct AreaOfInterest<SubspaceId, Path>
{
    /// To be included in this AreaOfInterest, an Entry must be included in the area.
    pub area:      Area<SubspaceId, Path>,
    /// To be included in this AreaOfInterest, an Entry’s timestamp must be among the max_count
    /// greatest Timestamps, unless max_count is zero.,
    pub max_count: u64,
    /// The total payload_lengths of all included Entries is at most max_size, unless max_size is
    /// zero.
    pub max_size:  u64,
}


impl<S, P> AreaOfInterest<S, P>
where
    S: Eq,
    P: Path,
{
    /// An `AreaOfInterest` `self` _includes_ an `Entry` `entry` from a `Store` `store` if
    /// - `self.area` includes `entry`,
    /// - `self.max_count` is zero, or `entry` is among the `self.max_count` newest `Entry`s of
    ///   `store`, and
    /// - `self.max_size` is zero, or the sum of the `payload_lengths` of `entry` and all newer
    ///   `Entry`s in `store` is less than or equal to `self.max_size`.
    #[inline]
    pub fn includes<N, D, Pe>(
        &self,
        entry: impl Borrow<Entry<N, S, Pe, D>>,
        store: &Store<N, impl StoreExt>,
    ) -> bool
    where
        Pe: Path,
    {
        let entry = entry.borrow();
        self.area.includes::<Entry<_, _, _, _>>(entry) && todo!()
    }
}
