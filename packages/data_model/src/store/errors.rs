use core::fmt::{
    self,
    Display,
    Formatter,
};


/// Errors possibly returned by [`Store::put`](crate::store::async::Store::put).
#[derive(Copy, Clone, Eq, Ord, Hash, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum PutError<E>
{
    /// The `auth_entry` argument is not for the same Namespace.
    DifferentNamespace,
    /// Failure of [`StoreExt::put`](crate::StoreExt::put)
    Put(E),
}

impl<E> Display for PutError<E>
{
    #[inline]
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> fmt::Result
    {
        write!(f, "`Store::put()` failed due to {}", match self {
            PutError::DifferentNamespace => "different namespace",
            PutError::Put(_) => "`StoreExt::put()`",
        })
    }
}


/// Errors possibly returned by [`Store::join`](crate::store::async::Store::join).
#[derive(Copy, Clone, Eq, Ord, Hash, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum JoinError<E>
{
    /// The `other` argument is not for the same Namespace.
    DifferentNamespace,
    /// Failure of [`StoreExt::join`](crate::StoreExt::join)
    Join(E),
}

impl<E> Display for JoinError<E>
{
    #[inline]
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> fmt::Result
    {
        write!(f, "`Store::join()` failed due to {}", match self {
            JoinError::DifferentNamespace => "different namespace",
            JoinError::Join(_) => "`StoreExt::join()`",
        })
    }
}


#[cfg(any(feature = "std", feature = "anticipate", rust_lib_feature = "error_in_core"))]
mod standard_error
{
    use super::{
        JoinError,
        PutError,
    };

    cfg_if::cfg_if! { if #[cfg(any(feature = "anticipate", rust_lib_feature = "error_in_core"))]
    {
        use core::error::Error;
    }
    else if #[cfg(feature = "std")]
    {
        use std::error::Error;
    } }

    impl<E> Error for PutError<E>
    where E: Error + 'static
    {
        #[inline]
        fn source(&self) -> Option<&(dyn Error + 'static)>
        {
            match self {
                PutError::DifferentNamespace => None,
                PutError::Put(put_error) => Some(put_error),
            }
        }
    }

    impl<E> Error for JoinError<E>
    where E: Error + 'static
    {
        #[inline]
        fn source(&self) -> Option<&(dyn Error + 'static)>
        {
            match self {
                JoinError::DifferentNamespace => None,
                JoinError::Join(join_error) => Some(join_error),
            }
        }
    }
}
