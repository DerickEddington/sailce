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
    core::{
        borrow::Borrow,
        num::NonZeroU64,
    },
};


/// A grouping of [`Entry`]s that are among the newest in some [`Store`].
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct AreaOfInterest<SubspaceId, Path>
{
    /// To be included in this `AreaOfInterest`, an `Entry` must be included in the area.
    pub area:      Area<SubspaceId, Path>,
    /// To be included in this `AreaOfInterest`, an `Entry`'s `timestamp` must be among the
    /// `max_count` greatest `Timestamp`s, unless `max_count` is `Unlimited`.
    pub max_count: Max,
    /// The total `payload_length`s of all included `Entry`s is at most `max_size`, unless
    /// `max_size` is `Unlimited`.
    pub max_size:  Max,
}


/// Determines whether [`AreaOfInterest::max_count`] or [`AreaOfInterest::max_size`] denotes a
/// limited or unlimited maximum.
///
/// The [`Ord`]ering of values of this type is based on the order of its fields, so that
/// `Unlimited` is greater than any `Limit`.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum Max
{
    /// A limited maximum.
    Limit(NonZeroU64),
    /// An unlimited maximum.
    Unlimited,
}


impl<S, P> AreaOfInterest<S, P>
where
    S: Eq,
    P: Path,
{
    /// An `AreaOfInterest` `self` _includes_ an `Entry` `entry` from a `Store` `store` if
    /// - `self.area` includes `entry`,
    /// - `self.max_count` is `Unlimited`, or `entry` is among the `self.max_count` newest
    ///   `Entry`s of `store`, and
    /// - `self.max_size` is `Unlimited`, or the sum of the `payload_length`s of `entry` and all
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

        let same_namespace = || entry.borrow().namespace_id == store.namespace_id;
        let within_area = || self.area.includes::<Entry<_, _, _, _>>(ent);
        let within_max_count = || match self.max_count {
            Max::Unlimited => true,
            Max::Limit(max_count) => {
                let max_count = u64::from(max_count);
                store.newest_entries_include(max_count, ent)
            },
        };
        let within_max_size = || match self.max_size {
            Max::Unlimited => true,
            Max::Limit(max_size) => {
                let max_size = u64::from(max_size);
                store
                    .payloads_total_size_of_entry_to_newest(ent)
                    .is_some_and(|sum| sum <= u128::from(max_size))
            },
            // Note: If summing overflowed, the sum is greater than `u64::MAX`.
        };

        same_namespace() && within_area() && within_max_count() && within_max_size()
    }

    /// Let `self` and `other` be `AreaOfInterest`s.  If there exists at least one [`Entry`]
    /// [included](Area::includes) in both `self.area`, and `other.area`, then we define the
    /// _(nonempty) intersection_ of `self`, and `other` as the `AreaOfInterest` whose
    /// - `area` is the [intersection](Area::intersection) of `self.area` and `other.area`, whose
    /// - `max_count` is `self.max_count` if `other.max_count` is `Unlimited`, `other.max_count`
    ///   if `self.max_count` is `Unlimited`, or the minimum of `self.max_count` and
    ///   `other.max_count` otherwise, and whose
    /// - `max_size` is `self.max_size` if `other.max_size` is `Unlimited`, `other.max_size` if
    ///   `self.max_size` is `Unlimited`, or the minimum of `self.max_size` and `other.max_size`
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
        let other = other.borrow();
        Self {
            area:      self.area.intersection(&other.area),
            max_count: self.max_count.min(other.max_count),
            max_size:  self.max_size.min(other.max_size),
        }
    }
}
