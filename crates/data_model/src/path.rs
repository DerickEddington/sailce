//! Aspects of Paths.


/// A Path is a sequence of at most [`MAX_COMPONENT_COUNT`](crate::Params::MAX_COMPONENT_COUNT)
/// many byte-strings, each of at most
/// [`MAX_COMPONENT_LENGTH`](crate::Params::MAX_COMPONENT_LENGTH) bytes, and whose total number of
/// bytes is at most [`MAX_PATH_LENGTH`](crate::Params::MAX_PATH_LENGTH). The byte-strings that
/// make up a Path are called its Components.
pub trait Path
{
    /// The Components of `self`.
    ///
    /// `Iterator` enables `impl`ementations to avoid needing heap allocation, while
    /// `ExactSizeIterator` enables knowing the amount of Components upfront.
    #[must_use]
    fn components(&self) -> impl ExactSizeIterator<Item = Component<'_>>;

    /// A Path `s` is a prefix of a Path `t` if the first Components of `t` are exactly the
    /// Components of `s`.
    ///
    /// # Example
    /// `["a"]` is a prefix of `["a"]` and of `["a", "b"]`, but not of `["ab"]`.
    #[must_use]
    #[inline]
    fn is_prefix_of(
        &self,
        other: &impl Path,
    ) -> bool
    {
        let self_comps = self.components();
        let other_comps = other.components();
        let self_len = self_comps.len();
        self_len <= other_comps.len() && self_comps.eq(other_comps.take(self_len))
    }
}


/// Single Component of a Path.
#[derive(Eq, Ord, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct Component<'l>
{
    /// The byte-string that is the single Component.
    pub bytes: &'l [u8],
}

impl AsRef<[u8]> for Component<'_>
{
    #[inline]
    fn as_ref(&self) -> &[u8]
    {
        self.bytes
    }
}


/// Automatically implemented for any type that can be an iterator of any item type that can give
/// a byte-string.
///
/// This makes various standard (and 3rd-party) types usable as Paths, e.g.: `[&[u8]; N]`,
/// `&[&str]`, `Vec<String>`, etc.  This enables flexibility, for different situations, in what is
/// used to represent Paths, e.g.: `[b"a", b"b"]`, `&["a", "b"][..]`, `vec!["a".to_owned(),
/// "b".to_owned()]`, etc.
impl<T, C> Path for T
where
    T: ?Sized,
    C: AsRef<[u8]> + ?Sized,
    for<'l> &'l Self: IntoIterator<Item = &'l C>,
    for<'l> <&'l Self as IntoIterator>::IntoIter: ExactSizeIterator,
{
    #[inline]
    fn components(&self) -> impl ExactSizeIterator<Item = Component<'_>>
    {
        self.into_iter().map(|bytes| Component { bytes: bytes.as_ref() })
    }
}


/// A [`Path`] that can represent the empty Path, i.e. that has no Components.
///
/// This is needed because some types that otherwise can `impl`ement [`Path`] cannot represent the
/// empty Path, e.g. `[C; N]` where `N >= 1`.
pub trait EmptyPath: Path
{
    /// The empty Path, that has no Components, as represented by the `Self` type.
    ///
    /// This must uphold: `Self::empty().components().len() == 0`.
    #[must_use]
    fn empty() -> Self;
}

/// Automatically implemented for any type that can be constructed by [`FromIterator`].
///
/// This means it can be constructed from an empty sequence, as required to represent the empty
/// Path.  This excludes types, e.g. `[C; N]` where `N >= 1`, that cannot represent the empty
/// Path.
impl<'c, T> EmptyPath for T
where T: Path + FromIterator<Component<'c>> + ?Sized
{
    #[inline]
    fn empty() -> Self
    {
        Self::from_iter([])
    }
}

impl<C> EmptyPath for [C; 0]
where C: AsRef<[u8]>
{
    #[inline]
    fn empty() -> Self
    {
        []
    }
}

// TODO: impl for more types as appropriate.
