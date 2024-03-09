/*!
Willow lets authors place Entries in Namespaces, and within each Namespace, Entries are arranged
according to three orthogonal dimensions: [`SubspaceId`](crate::Params::SubspaceId),
[`Path`](crate::Path), and [`Timestamp`](crate::Timestamp).  This suggests a powerful way of
thinking about Willow: a Namespace is a collection of points (Entries) in a three-dimensional
space.  Or more accurately, a Namespace is a mapping from points in this three-dimensional space
to hashes and sizes of [`Payload`](crate::Payload)s.

This viewpoint enables us to meaningfully group Entries together.  An application might want to
access all chess games that a certain author played in the past week.  This kind of query
corresponds to a box (a [rectangular cuboid](https://en.wikipedia.org/wiki/Rectangular_cuboid) to
use precise terminology) in the three-dimensional Willow space.

In this module, we provide a basis for grouping Entries based on these three dimensions.  This
isn't necessary for defining and understanding the core data model, but this is commonly used by
things that use or extend Willow.

See: <https://willowprotocol.org/specs/grouping-entries/index.html#grouping_entries>
 */

pub mod range;
pub use range::{
    Range,
    ThreeDimRange,
};
