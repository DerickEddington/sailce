//! `Area`s are an alternative to [`ThreeDimRange`](crate::group::ThreeDimRange)s that can be used
//! even when encrypting [`Path`]s and [`SubspaceId`](crate::Params::SubspaceId)s.

use {
    crate::{
        group::Range,
        EmptyPath,
        Entry,
        Path,
        Timestamp,
    },
    core::borrow::Borrow,
};


pub mod of_interest;
pub use of_interest::AreaOfInterest;


/// A grouping of [`Entry`]s.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct Area<SubspaceId, Path>
{
    /// To be included in this `Area`, an `Entry`'s [`subspace_id`](Entry::subspace_id) must be
    /// equal to this, when [`Id`](Subspace::Id).  Or, when [`Any`](Subspace::Any), an `Entry`'s
    /// `subspace_id` can be anything.
    pub subspace: Subspace<SubspaceId>,
    /// To be included in this `Area`, an `Entry`'s `path` must be
    /// [prefixed](Path::is_prefix_of) by this.
    pub path:     Path,
    /// To be included in this `Area`, an `Entry`'s `timestamp` must be
    /// [included](Range::includes) in this.
    pub times:    Range<Timestamp>,
}


/// Determines whether [`Area::subspace`] denotes a single Subspace or all Subspaces.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum Subspace<SubspaceId>
{
    /// A single particular Subspace.
    Id(SubspaceId),
    /// All Subspaces.
    Any,
}

impl<S> Subspace<S>
where S: Eq
{
    fn includes(
        &self,
        other: &Subspace<impl Borrow<S>>,
    ) -> bool
    {
        match (self, other) {
            (Subspace::Any, _) => true,
            (Subspace::Id(self_id), Subspace::Id(other_id)) => self_id == other_id.borrow(),
            (Subspace::Id(_), Subspace::Any) => false,
        }
    }
}


impl<S, P> Area<S, P>
where
    S: Eq,
    P: Path,
{
    /// Creates an empty `Area`.
    #[must_use]
    #[inline]
    pub fn empty() -> Self
    where P: Default
    {
        Self {
            subspace: Subspace::Any,
            path:     P::default(),
            times:    Range::empty(), // This makes it be empty.
        }
    }

    /// The _full area_ is the `Area` whose `subspace` is `Any`, whose `path` is the empty `Path`,
    /// and whose `times` is the open `Range<Timestamp>` with `start` `0`.  It includes all
    /// [`Entry`]s.
    #[must_use]
    #[inline]
    pub fn full() -> Self
    where P: EmptyPath
    {
        Self { subspace: Subspace::Any, path: P::empty(), times: Range::default() }
    }

    /// The _subspace area_ of the `SubspaceId` `sub` is the `Area` whose `subspace` is `sub`,
    /// whose `path` is the empty `Path`, and whose `times` is the open `Range<Timestamp>` with
    /// `start` `0`.  It includes exactly the [`Entry`]s with `subspace_id` `sub`.
    #[must_use]
    #[inline]
    pub fn subspace(sub: S) -> Self
    where P: EmptyPath
    {
        Self { subspace: Subspace::Id(sub), path: P::empty(), times: Range::default() }
    }

    /// An `Area` `a` _includes_ an [`Entry`] `e` if
    /// - `a.subspace == Subspace::Any` or `a.subspace == e.subspace_id`,
    /// - `a.path` [prefixes](Path::is_prefix_of) `e.path`, and
    /// - `a.times` includes `e.timestamp`.
    ///
    /// An `Area` _includes_ another `Area` if the first `Area` includes all `Entry`s that the
    /// second `Area` includes.  In particular, every `Area` includes itself.
    #[must_use]
    #[inline]
    pub fn includes<V>(
        &self,
        value: impl Borrow<V>,
    ) -> bool
    where
        V: Includable<S>,
    {
        value.borrow().included_in(self)
    }

    /// An `Area` is _empty_ if it includes no [`Entry`]s.  This is the case if and only if its
    /// `times` is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool
    {
        self.times.is_empty()
    }

    /// If two `Area`s overlap, the overlap is again an `Area`.  Let `self` and `other` be
    /// `Area`s.  If there exists at least one [`Entry`] [included](Self::includes) in both `self`
    /// and `other`, then we define the _(nonempty) intersection_ of `self` and `other` as the
    /// `Area` whose
    /// - `subspace` is `a1.subspace` if `a1.subspace` is not `Any`, or `a2.subspace` otherwise,
    ///   whose
    /// - `path` is the longer of `self.path` and `other.path` (one is a prefix of the other,
    ///   otherwise the intersection would be empty), and whose
    /// - `times` is the intersection of `self.times` and `other.times`.
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

        let Some(subspace) = (match (&self.subspace, &other.subspace) {
            (Subspace::Any, Subspace::Any) => Some(Subspace::Any),
            (Subspace::Any, Subspace::Id(other_id)) => Some(Subspace::Id(other_id)),
            (Subspace::Id(self_id), Subspace::Any) => Some(Subspace::Id(self_id)),
            (Subspace::Id(self_id), Subspace::Id(other_id)) =>
                (self_id == other_id).then_some(Subspace::Id(self_id)),
        })
        else {
            return Self::empty();
        };

        let Some(path) = (if self.path.is_prefix_of(&other.path) {
            Some(&other.path)
        }
        else if other.path.is_prefix_of(&self.path) {
            Some(&self.path)
        }
        else {
            None
        })
        else {
            return Self::empty();
        };

        #[allow(clippy::needless_borrows_for_generic_args)]
        let times = self.times.intersection(&other.times);

        if times.is_empty() {
            Self::empty()
        }
        else {
            // Might as well wait to clone until certain they're needed.
            let subspace = match subspace {
                Subspace::Id(id) => Subspace::Id(id.clone()),
                Subspace::Any => Subspace::Any,
            };
            let path = path.clone();
            Self { subspace, path, times }
        }
    }
}


use sealed::Includable;
mod sealed
{
    use {
        super::{
            Area,
            Entry,
            Subspace,
        },
        crate::Path,
    };

    pub trait Includable<S>
    {
        fn included_in<Pa>(
            &self,
            area: &Area<S, Pa>,
        ) -> bool
        where
            Pa: Path;
    }

    impl<N, S, P, D> Includable<S> for Entry<N, S, P, D>
    where
        S: Eq,
        P: Path,
    {
        fn included_in<Pa>(
            &self,
            area: &Area<S, Pa>,
        ) -> bool
        where
            Pa: Path,
        {
            #![allow(clippy::needless_borrows_for_generic_args)]

            area.subspace.includes(&Subspace::Id(&self.subspace_id))
                && area.path.is_prefix_of(&self.path)
                && area.times.includes(&self.timestamp)
        }
    }

    impl<S, P> Includable<S> for Area<S, P>
    where
        S: Eq,
        P: Path,
    {
        fn included_in<Pa>(
            &self,
            other: &Area<S, Pa>,
        ) -> bool
        where
            Pa: Path,
        {
            other.subspace.includes(&self.subspace)
                && other.path.is_prefix_of(&self.path)
                && other.times.includes_range(&self.times)
        }
    }
}
