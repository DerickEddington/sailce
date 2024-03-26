//! Aspects of `Path`s.

use core::{
    borrow::Borrow,
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
    fn components(&self) -> impl ExactSizeIterator<Item = Component<'_>>;

    /// Like [`Self::components`] but gives the components as string slices from UTF-8, if
    /// possible.
    ///
    /// Note: It can be useful to `collect` the items yielded by this `Iterator` into a single
    /// `Result` that contains the first error if it occurred:
    /// ```
    /// # use {sailce_data_model::{path::StrComponent, Path as _}, core::str::Utf8Error};
    /// # let path: [&[u8; 4]; 3] = [b"good", b"bad\xFF", b"good"];
    /// let x: Result<Vec<StrComponent>, Utf8Error> = path.str_components().collect();
    /// # assert!(x.is_err());
    /// ```
    /// Similarly, it also can be useful for propagating the first error if it occurred:
    /// ```
    /// # use {sailce_data_model::{path::StrComponent, Path as _}, core::str::Utf8Error};
    /// # fn main() -> Result<(), Utf8Error> {
    /// # let path: &[&[u8]] = &[&b"good"[..], &b"alright"[..]];
    /// let x: Vec<StrComponent> = path.str_components().collect::<Result<_, _>>()?;
    /// # assert_eq!(x.iter().map(|sc| sc.as_ref()).collect::<Vec<&str>>(), ["good", "alright"]);
    /// # Ok(()) }
    /// ```
    /// But if you want to know about multiple errors and/or at which position they occurred, then
    /// don't `collect` like this.
    #[must_use]
    #[inline]
    fn str_components(&self)
    -> impl ExactSizeIterator<Item = Result<StrComponent<'_>, Utf8Error>>
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
#[derive(Copy, Clone, Eq, Ord, Hash, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct Component<'l>
{
    /// The byte-string that is the single component.
    pub bytes: &'l [u8],
}

/// Enables `Component` to work with [`from_path`](Extra::from_path).
impl<'l> From<&'l [u8]> for Component<'l>
{
    #[inline]
    fn from(bytes: &'l [u8]) -> Self
    {
        Self { bytes }
    }
}

/// Might be useful.
impl<'l> From<StrComponent<'l>> for Component<'l>
{
    #[inline]
    fn from(value: StrComponent<'l>) -> Self
    {
        Self { bytes: value.str.as_bytes() }
    }
}

/// Enables `Component` to work with the blanket `impl`s of `Path`.
impl AsRef<[u8]> for Component<'_>
{
    #[inline]
    fn as_ref(&self) -> &[u8]
    {
        self.bytes
    }
}

/// Might be useful.
impl Borrow<[u8]> for Component<'_>
{
    #[inline]
    fn borrow(&self) -> &[u8]
    {
        self.bytes
    }
}
