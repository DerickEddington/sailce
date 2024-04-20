//! Aspects of `Path`s.

use core::{
    borrow::Borrow,
    cmp::Ordering,
    str::Utf8Error,
};


mod blanket_impls;

mod errors;
pub use errors::*;

mod extra;
pub use extra::*;

mod str_conv;
pub use str_conv::{
    StrComponent,
    Strish,
};


/// A `Path` is a sequence of at most [`MAX_COMPONENT_COUNT`](crate::Params::MAX_COMPONENT_COUNT)
/// many byte-strings, each of at most
/// [`MAX_COMPONENT_LENGTH`](crate::Params::MAX_COMPONENT_LENGTH) bytes, and whose total number of
/// bytes is at most [`MAX_PATH_LENGTH`](crate::Params::MAX_PATH_LENGTH). The byte-strings that
/// make up a `Path` are called its [`Component`]s.
pub trait Path
{
    /// The `Component`s of `self`.
    ///
    /// `Iterator` enables implementations to avoid needing heap allocation, while
    /// `ExactSizeIterator` enables knowing the amount of `Component`s upfront.
    #[must_use]
    fn components(&self) -> impl ExactSizeIterator<Item = Component<&[u8]>>;

    /// Like [`Self::components`] but gives the components as string slices from UTF-8, if
    /// possible.
    ///
    /// Note: It can be useful to `collect` the items yielded by this `Iterator` into a single
    /// `Result` that contains the first error if it occurred:
    /// ```
    /// # use {sailce_data_model::{path::StrComponent, Path as _}, core::str::Utf8Error};
    /// # let path: [&[u8; 4]; 3] = [b"good", b"bad\xFF", b"good"];
    /// let x: Result<Vec<StrComponent<&str>>, Utf8Error> = path.str_components().collect();
    /// # assert!(x.is_err());
    /// ```
    /// Similarly, it also can be useful for propagating the first error if it occurred:
    /// ```
    /// # use {sailce_data_model::{path::StrComponent, Path as _}, core::str::Utf8Error};
    /// # fn main() -> Result<(), Utf8Error> {
    /// # let path: &[&[u8]] = &[&b"good"[..], &b"alright"[..]];
    /// let x: Vec<StrComponent<&str>> = path.str_components().collect::<Result<_, _>>()?;
    /// # assert_eq!(x.iter().map(|sc| sc.as_ref()).collect::<Vec<&str>>(), ["good", "alright"]);
    /// # Ok(()) }
    /// ```
    /// But if you want to know about multiple errors and/or at which position they occurred, then
    /// don't `collect` like this.
    #[must_use]
    #[inline]
    fn str_components(
        &self
    ) -> impl ExactSizeIterator<Item = Result<StrComponent<&str>, Utf8Error>>
    {
        self.components().map(StrComponent::try_from)
    }

    /// A `Path` `s` is a prefix of a `Path` `t` if the first `Component`s of `t` are exactly the
    /// `Component`s of `s`.
    ///
    /// # Example
    /// `["a"]` is a prefix of `["a"]` and of `["a", "b"]`, but not of `["ab"]`.
    #[must_use]
    #[inline]
    fn is_prefix_of(
        &self,
        other: &(impl Path + ?Sized),
    ) -> bool
    {
        let self_comps = self.components();
        let other_comps = other.components();
        let self_len = self_comps.len();
        self_len <= other_comps.len() && self_comps.eq(other_comps.take(self_len))
    }
}


/// A [`Path`] type that can construct the empty `Path`, i.e. that has no `Component`s.
///
/// This is needed because some types that otherwise can implement `Path` cannot represent and/or
/// cannot construct the empty `Path`, e.g. `[C; N]` where `N >= 1`.
pub trait EmptyPath: Path
{
    /// The empty [`Path`], that has no `Component`s, as represented by the `Self` type.
    ///
    /// This must uphold: `Self::empty().components().len() == 0`.
    #[must_use]
    fn empty() -> Self;
}


/// A single component of a `Path`.
#[derive(Copy, Clone, Hash, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct Component<Bytes>
where Bytes: Borrow<[u8]>
{
    /// Represents the byte-string that is the single component.
    pub inner: Bytes,
}

/// Indicates that `Component` behaves identically to `[u8]` w.r.t. `Hash`, `Ord`, etc.
impl<B> Borrow<[u8]> for Component<B>
where B: Borrow<[u8]>
{
    #[inline]
    fn borrow(&self) -> &[u8]
    {
        self.inner.borrow()
    }
}

impl<B> Component<B>
where B: Borrow<[u8]>
{
    /// Get immutable reference to the bytes.
    #[inline]
    pub fn bytes(&self) -> &[u8]
    {
        <Self as Borrow<[u8]>>::borrow(self)
    }
}

impl<Ba, Bb> PartialEq<Component<Bb>> for Component<Ba>
where
    Ba: Borrow<[u8]>,
    Bb: Borrow<[u8]>,
{
    #[inline]
    fn eq(
        &self,
        other: &Component<Bb>,
    ) -> bool
    {
        self.bytes() == other.bytes()
    }
}

impl<B> Eq for Component<B> where B: Borrow<[u8]> {}

impl<Ba, Bb> PartialOrd<Component<Bb>> for Component<Ba>
where
    Ba: Borrow<[u8]>,
    Bb: Borrow<[u8]>,
{
    #[inline]
    fn partial_cmp(
        &self,
        other: &Component<Bb>,
    ) -> Option<Ordering>
    {
        Some(self.bytes().cmp(other.bytes()))
    }
}

impl<B> Ord for Component<B>
where B: Borrow<[u8]>
{
    #[inline]
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering
    {
        self.bytes().cmp(other.bytes())
    }
}

// The following conversions are not the only possibilities.  These are provided by this crate
// because they seem like they'll be useful.  If others are also somewhat commonly useful, more
// should be added here.

/// Enables `Component` to work with [`from_path`](Extra::from_path).
impl<'l> From<&'l [u8]> for Component<&'l [u8]>
{
    #[inline]
    fn from(bytes: &'l [u8]) -> Self
    {
        Self { inner: bytes }
    }
}

/// Might be useful.
///
/// (Should be replaced by `impl<S, B> From<StrComponent<S>> for Component<B> where S: Into<B>`,
/// if the standard library ever adds `impl From<&str> for &[u8]`.)
impl<'l> From<StrComponent<&'l str>> for Component<&'l [u8]>
{
    #[inline]
    fn from(value: StrComponent<&'l str>) -> Self
    {
        Self { inner: value.inner.as_bytes() }
    }
}

/// Enables `Component` to work with the blanket `impl`s of `Path`.
impl<B> AsRef<[u8]> for Component<B>
where B: Borrow<[u8]>
{
    #[inline]
    fn as_ref(&self) -> &[u8]
    {
        self.bytes()
    }
}
