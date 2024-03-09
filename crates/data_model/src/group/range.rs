//! Ranges are ways of grouping Entries.  They can express groupings such as "last week's
//! Entries".

use {
    crate::Timestamp,
    core::{
        borrow::Borrow,
        cmp::{
            max,
            min,
        },
    },
};


mod three_dim;
pub use three_dim::ThreeDimRange;

mod least;
pub use least::Least;


/// Determines whether a [`Range`] is _closed_ or _open_.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum End<T>
{
    /// A _closed range_ consists of a _start value_ and an _end value_.
    Closed(T),
    /// An _open range_ consists only of a _start value_.
    Open,
}


/// A _range_ is a simple one-dimensional way of grouping Entries, and is either a _closed range_
/// or an _open range_.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct Range<T>
{
    /// A value must be greater than or equal to this to be included in the range.
    pub start: T,
    /// If [`Open`](End::Open), the range is an open range.  Otherwise, a value must be strictly
    /// less than this to be included in the range.
    pub end:   End<T>,
}


impl<T> From<core::ops::Range<T>> for Range<T>
{
    #[inline]
    fn from(value: core::ops::Range<T>) -> Self
    {
        Self { start: value.start, end: End::Closed(value.end) }
    }
}

impl<T> From<core::ops::RangeFrom<T>> for Range<T>
{
    #[inline]
    fn from(value: core::ops::RangeFrom<T>) -> Self
    {
        Self { start: value.start, end: End::Open }
    }
}

impl From<core::ops::Range<u64>> for Range<Timestamp>
{
    #[inline]
    fn from(value: core::ops::Range<u64>) -> Self
    {
        Self { start: value.start.into(), end: End::Closed(value.end.into()) }
    }
}

impl From<core::ops::RangeFrom<u64>> for Range<Timestamp>
{
    #[inline]
    fn from(value: core::ops::RangeFrom<u64>) -> Self
    {
        Self { start: value.start.into(), end: End::Open }
    }
}


impl<T> Range<T>
where T: Ord
{
    /// Creates an empty range.
    #[must_use]
    #[inline]
    pub fn empty() -> Self
    where T: Default
    {
        Self { start: T::default(), end: End::Closed(T::default()) }
    }

    /// A range _includes_ all values greater than or equal to its `start` value and strictly less
    /// than its `end` value (if it is has one).
    #[must_use]
    #[inline]
    pub fn includes(
        &self,
        value: impl Borrow<T>,
    ) -> bool
    {
        let value = value.borrow();
        self.start <= *value
            && match &self.end {
                End::Closed(end) => *value < *end,
                End::Open => true,
            }
    }

    /// A range is _empty_ if it includes no values.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool
    {
        match &self.end {
            End::Closed(end) => *end <= self.start,
            End::Open => false,
        }
    }

    /// The intersection of `self` and `other` is the `Self` whose `start` value is the greater of
    /// the `start` values of `self` and `other`, and whose `end` value is the lesser of the `end`
    /// values of `self` and `other` (if both are closed ranges), the one `end` value among `self`
    /// and `other` (if exactly one of them is a closed range), or no `end` value (if both `self`
    /// and `other` are open ranges).
    #[must_use]
    #[inline]
    pub fn intersection(
        &self,
        other: impl Borrow<Self>,
    ) -> Self
    where
        T: Clone,
    {
        let other = other.borrow();
        let start = max(&self.start, &other.start).clone();
        let end = match (&self.end, &other.end) {
            (End::Closed(self_end), End::Closed(other_end)) =>
                End::Closed(min(self_end, other_end).clone()),
            (End::Closed(self_end), End::Open) => End::Closed(self_end.clone()),
            (End::Open, End::Closed(other_end)) => End::Closed(other_end.clone()),
            (End::Open, End::Open) => End::Open,
        };
        Self { start, end }
    }
}


/// This is the range that includes the entirety of **all** the values of type `T`.
///
/// (This is analogous to [`ThreeDimRange::default`], but was not part of the Willow documents (as
/// of 2024-03), but this would seem to be appropriate.)
impl<T> Default for Range<T>
where T: Least
{
    #[inline]
    fn default() -> Self
    {
        Self { start: T::least(), end: End::Open }
    }
}
