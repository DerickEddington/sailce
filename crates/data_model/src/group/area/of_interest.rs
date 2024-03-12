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
        ParamsEntry,
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
    /// - `self.max_size` is zero, or the sum of the `payload_length`s of `entry` and all
    ///   [newer](Entry::is_newer_than) `Entry`s in `store` is less than or equal to
    ///   `self.max_size`.
    #[must_use]
    #[inline]
    pub fn includes<Params, Pe>(
        &self,
        entry: impl Borrow<ParamsEntry<Params, Pe>>,
        store: &Store<Params::NamespaceId, impl StoreExt<Params = Params>>,
    ) -> bool
    where
        Params: crate::Params<SubspaceId = S> + ?Sized,
        Pe: Path,
    {
        //TODO: let entry = entry.borrow(); // Causes "no field on type" bug in rust-analyzer.
        let ent = entry.borrow(); // TODO: remove

        (entry.borrow().namespace_id == store.namespace_id)
            && self.area.includes::<Entry<_, _, _, _>>(ent)
            && (self.max_count == 0 || store.newest_entries_include(self.max_count, ent))
            && (self.max_size == 0
                || store
                    .payloads_total_size_of_entry_to_newest(ent)
                    .is_some_and(|sum| sum <= self.max_size.into()))
        // Note: If summing overflowed, the sum is greater than `u64::MAX`.
    }

    /// Let `self` and `other` be `AreaOfInterest`s.  If there exists at least one [`Entry`]
    /// [included](Area::includes) in both `self.area`, and `other.area`, then we define the
    /// _(nonempty) intersection_ of `self`, and `other` as the `AreaOfInterest` whose
    /// - `area` is the [intersection](Area::intersection) of `self.area` and `other.area`, whose
    /// - `max_count` is `self.max_count` if `other.max_count` is zero, `other.max_count` if
    ///   `self.max_count` is zero, or the minimum of `self.max_count` and `other.max_count`
    ///   otherwise, and whose
    /// - `max_size` is `self.max_size` if `other.max_size` is zero, `other.max_size` if
    ///   `self.max_size` is zero, or the minimum of `self.max_size` and `other.max_size`
    ///   otherwise.
    #[must_use]
    #[inline]
    pub fn intersection(
        &self,
        other: impl Borrow<Self>,
    ) -> Self
    where
        S: Clone,
        P: Default + Clone,
    {
        fn min(
            a: u64,
            b: u64,
        ) -> u64
        {
            match (a, b) {
                (a, 0) => a,
                (0, b) => b,
                (a, b) => core::cmp::min(a, b),
            }
        }

        let other = other.borrow();
        Self {
            area:      self.area.intersection(&other.area),
            max_count: min(self.max_count, other.max_count),
            max_size:  min(self.max_size, other.max_size),
        }
    }
}
