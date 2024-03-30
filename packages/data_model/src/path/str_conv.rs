use {
    crate::path::Component,
    core::{
        borrow::Borrow,
        ops::{
            Deref,
            DerefMut,
        },
        str::{
            from_utf8,
            Utf8Error,
        },
    },
};


/// A single component of a `Path` as a `str` string slice.
#[derive(Copy, Clone, Eq, Ord, Hash, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct StrComponent<'l>
{
    /// The string slice that is the single component.
    pub str: &'l str,
}

/// Enables `StrComponent` to work with [`try_from_path`](crate::path::Extra::try_from_path).
impl<'l> TryFrom<&'l [u8]> for StrComponent<'l>
{
    type Error = Utf8Error;

    #[inline]
    fn try_from(value: &'l [u8]) -> Result<Self, Self::Error>
    {
        Ok(Self { str: from_utf8(value)? })
    }
}

/// Might be useful.
impl<'l> TryFrom<Component<'l>> for StrComponent<'l>
{
    type Error = Utf8Error;

    #[inline]
    fn try_from(value: Component<'l>) -> Result<Self, Self::Error>
    {
        value.bytes.try_into()
    }
}

/// Enables `StrComponent` to work with the blanket `impl`s of `Path`.
impl AsRef<[u8]> for StrComponent<'_>
{
    #[inline]
    fn as_ref(&self) -> &[u8]
    {
        self.str.as_ref()
    }
}

/// Might be useful.
impl AsRef<str> for StrComponent<'_>
{
    #[inline]
    fn as_ref(&self) -> &str
    {
        self.str
    }
}

/// Might be useful.
impl Borrow<str> for StrComponent<'_>
{
    #[inline]
    fn borrow(&self) -> &str
    {
        self.str
    }
}

// (`StrComponent` must not `impl Borrow<[u8]>` because of the requirements of that trait.)


/// Provides [`TryFrom`] conversion from UTF-8 bytes for any type that represents a string.
/// Useful with [`try_from_path`](crate::path::Extra::try_from_path) as the component type of a
/// [`Path`](crate::Path) type.
///
/// This represents a string and so [`Deref`]s and [`Borrow`]s as such and so can directly be
/// used as such.  This is why the type itself has the required bound on these traits.
///
/// The field is public, because this might be useful for accessing the inner value and/or for
/// creating instances directly.
#[derive(Copy, Clone, Eq, Ord, Hash, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct Strish<T>(pub T)
where T: Deref<Target = str> + Borrow<str> + ?Sized;


/// Enables `Strish` to work with [`try_from_path`](crate::path::Extra::try_from_path).
impl<'l, T> TryFrom<&'l [u8]> for Strish<T>
where
    &'l str: Into<T>,
    T: Deref<Target = str> + Borrow<str>,
{
    type Error = Utf8Error;

    #[inline]
    fn try_from(value: &'l [u8]) -> Result<Self, Self::Error>
    {
        from_utf8(value).map(|s| Self(s.into()))
    }
}


/// Enables `Strish` to be directly used as a string.
impl<T> Deref for Strish<T>
where T: Deref<Target = str> + Borrow<str> + ?Sized
{
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target
    {
        Deref::deref(&self.0)
    }
}

/// Enables `Strish` to be directly used as a string.
impl<T> DerefMut for Strish<T>
where T: DerefMut<Target = str> + Borrow<str> + ?Sized
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        DerefMut::deref_mut(&mut self.0)
    }
}


/// Enables `Strish` to work with the blanket `impl`s of `Path`.
impl<T> AsRef<[u8]> for Strish<T>
where T: AsRef<[u8]> + Deref<Target = str> + Borrow<str> + ?Sized
{
    #[inline]
    fn as_ref(&self) -> &[u8]
    {
        self.0.as_ref()
    }
}

/// Might be useful.
impl<T> AsRef<str> for Strish<T>
where T: Deref<Target = str> + Borrow<str> + ?Sized
{
    #[inline]
    fn as_ref(&self) -> &str
    {
        &self.0
    }
}

/// Might be useful.
impl<T> Borrow<str> for Strish<T>
where T: Borrow<str> + Deref<Target = str> + ?Sized
{
    #[inline]
    fn borrow(&self) -> &str
    {
        self.0.borrow()
    }
}

// (`Strish` must not `impl Borrow<[u8]>` because of the requirements of that trait.)
