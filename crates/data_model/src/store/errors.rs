use {
    super::StoreExt,
    crate::anticipated_or_like::Error,
    core::fmt::{
        self,
        Debug,
        Display,
        Formatter,
    },
};


/// Errors possibly returned by [`Store::singleton`].
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum SingletonError<Ext: StoreExt>
{
    /// Failure of [`StoreExt::new`]
    New(Ext::NewError),
    /// Failure of [`StoreExt::put`]
    Put(Ext::PutError),
}

impl<Ext> Display for SingletonError<Ext>
where Ext: StoreExt
{
    #[inline]
    fn fmt(
        &self,
        fmt: &mut Formatter<'_>,
    ) -> fmt::Result
    {
        write!(fmt, "`Store::singleton()` failed due to {}", match self {
            SingletonError::New(_) => "`StoreExt::new()`",
            SingletonError::Put(_) => "`StoreExt::put()`",
        })
    }
}

impl<Ext> Error for SingletonError<Ext>
where
    Self: Display + Debug,
    Ext: StoreExt,
    Ext::NewError: Error + 'static,
    Ext::PutError: Error + 'static,
{
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)>
    {
        Some(match self {
            SingletonError::New(new_error) => new_error,
            SingletonError::Put(put_error) => put_error,
        })
    }
}


/// Errors possibly returned by [`Store::put`].
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum PutError<Ext: StoreExt>
{
    /// The `auth_entry` argument is not for the same Namespace.
    DifferentNamespace,
    /// Failure of [`StoreExt::put`]
    Ext(Ext::PutError),
}

impl<Ext> Display for PutError<Ext>
where Ext: StoreExt
{
    #[inline]
    fn fmt(
        &self,
        fmt: &mut Formatter<'_>,
    ) -> fmt::Result
    {
        write!(fmt, "`Store::put()` failed due to {}", match self {
            PutError::DifferentNamespace => "different namespace",
            PutError::Ext(_) => "`StoreExt::put()`",
        })
    }
}

impl<Ext> Error for PutError<Ext>
where
    Self: Display + Debug,
    Ext: StoreExt,
    Ext::PutError: Error + 'static,
{
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)>
    {
        match self {
            PutError::DifferentNamespace => None,
            PutError::Ext(put_error) => Some(put_error),
        }
    }
}


/// Errors possibly returned by [`Store::join`].
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum JoinError<Ext: StoreExt>
{
    /// The `other` argument is not for the same Namespace.
    DifferentNamespace,
    /// Failure of [`StoreExt::join`]
    Ext(Ext::JoinError),
}

impl<Ext> Display for JoinError<Ext>
where Ext: StoreExt
{
    #[inline]
    fn fmt(
        &self,
        fmt: &mut Formatter<'_>,
    ) -> fmt::Result
    {
        write!(fmt, "`Store::join()` failed due to {}", match self {
            JoinError::DifferentNamespace => "different namespace",
            JoinError::Ext(_) => "`StoreExt::join()`",
        })
    }
}

impl<Ext> Error for JoinError<Ext>
where
    Self: Display + Debug,
    Ext: StoreExt,
    Ext::JoinError: Error + 'static,
{
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)>
    {
        match self {
            JoinError::DifferentNamespace => None,
            JoinError::Ext(join_error) => Some(join_error),
        }
    }
}
