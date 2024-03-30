use core::{
    fmt::{
        self,
        Display,
        Formatter,
    },
    num::NonZeroU64,
};


macro_rules! nz {
    ($v:ident, $msg:literal) => {{
        debug_assert_ne!($v, 0, $msg);
        NonZeroU64::new($v).unwrap_or(NonZeroU64::MIN)
    }};
}

fn oob_at(at: u64) -> NonZeroU64
{
    nz!(at, "position 0 cannot be out-of-bounds")
}


/// Errors possibly returned by [`copy_to_slice`](crate::payload::ExtraCore::copy_to_slice).
#[derive(Copy, Clone, Eq, Ord, Hash, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum CopyToSliceError<ReadError, SeekError>
{
    /// One of the range arguments is out-of-bounds of the length of the `self` argument.
    OutOfBounds
    {
        /// The position that is out-of-bounds, or `None` if overflow occurred.
        at: Option<NonZeroU64>,
    },
    /// Failure of [`Payload::read`](crate::Payload::read).
    Read(ReadError),
    /// Failure of [`Payload::seek`](crate::Payload::seek).
    Seek(SeekError),
    /// The `self` argument's implementation of [`Payload`](crate::Payload) violated
    /// required behavior.
    BadImpl,
}

impl<R, S> CopyToSliceError<R, S>
{
    pub(crate) fn out_of_bounds_at(at: u64) -> Self
    {
        Self::OutOfBounds { at: Some(oob_at(at)) }
    }

    pub(crate) fn out_of_bounds_overflowed() -> Self
    {
        Self::OutOfBounds { at: None }
    }
}

impl<R, S> Display for CopyToSliceError<R, S>
{
    #[inline]
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> fmt::Result
    {
        let prefix = "`payload::ExtraCore::copy_to_slice` failed due to";
        match self {
            CopyToSliceError::OutOfBounds { at } => fmt_oob(f, prefix, *at),
            CopyToSliceError::Read(_) => fmt_r(f, prefix),
            CopyToSliceError::Seek(_) => fmt_s(f, prefix),
            CopyToSliceError::BadImpl => fmt_bi(f, prefix),
        }
    }
}


fn fmt_oob(
    f: &mut Formatter<'_>,
    prefix: &str,
    at: Option<NonZeroU64>,
) -> fmt::Result
{
    write!(f, "{prefix} out-of-bounds at ")?;
    match at {
        Some(at) => write!(f, "{at}"),
        None => write!(f, "overflowed `u64::MAX`"),
    }
}

fn fmt_r(
    f: &mut Formatter<'_>,
    prefix: &str,
) -> fmt::Result
{
    write!(f, "{prefix} `Payload::read`")
}

fn fmt_s(
    f: &mut Formatter<'_>,
    prefix: &str,
) -> fmt::Result
{
    write!(f, "{prefix} `Payload::seek`")
}

fn fmt_bi(
    f: &mut Formatter<'_>,
    prefix: &str,
) -> fmt::Result
{
    write!(f, "{prefix} bad implementation of `Payload`")
}


#[cfg(feature = "alloc")]
pub use alloc::*;

#[cfg(feature = "alloc")]
mod alloc
{
    use {
        super::{
            fmt_bi,
            fmt_oob,
            fmt_r,
            fmt_s,
            CopyToSliceError,
        },
        core::{
            fmt::Display,
            num::NonZeroU64,
        },
    };

    /// Errors possibly returned by [`to_boxed_slice`](crate::payload::Extra::to_boxed_slice).
    #[derive(Copy, Clone, Eq, Ord, Hash, PartialEq, PartialOrd, Debug)]
    #[allow(clippy::exhaustive_enums)]
    pub enum ToBoxedSliceError<ReadError, SeekError>
    {
        /// The `range` argument is out-of-bounds of the length of the `self` argument.
        OutOfBounds
        {
            /// The position that is out-of-bounds, or `None` if overflow occurred.
            at: Option<NonZeroU64>,
        },
        /// The `range` argument is too long for a single allocation to be made.
        RangeTooLong
        {
            /// How much greater than [`isize::MAX`] the `range` length is.
            by: NonZeroU64,
        },
        /// Failure of [`Payload::read`](crate::Payload::read).
        Read(ReadError),
        /// Failure of [`Payload::seek`](crate::Payload::seek).
        Seek(SeekError),
        /// The `self` argument's implementation of [`Payload`](crate::Payload) violated
        /// required behavior.
        BadImpl,
    }

    impl<R, S> ToBoxedSliceError<R, S>
    {
        pub(crate) fn out_of_bounds_overflowed() -> Self
        {
            Self::OutOfBounds { at: None }
        }

        pub(crate) fn range_too_long_by(by: u64) -> Self
        {
            Self::RangeTooLong { by: nz!(by, "excess cannot be 0") }
        }
    }

    /// Convert all variants, instead of just wrapping `CopyToSliceError`, so that
    /// `ToBoxedSliceError` doesn't expose that it deals with `CopyToSliceError`, in case the
    /// internal implementation of `to_boxed_slice` changes in the future.
    impl<R, S> From<CopyToSliceError<R, S>> for ToBoxedSliceError<R, S>
    {
        #[inline]
        fn from(value: CopyToSliceError<R, S>) -> Self
        {
            match value {
                CopyToSliceError::OutOfBounds { at } => Self::OutOfBounds { at },
                CopyToSliceError::Read(read_error) => Self::Read(read_error),
                CopyToSliceError::Seek(seek_error) => Self::Seek(seek_error),
                CopyToSliceError::BadImpl => Self::BadImpl,
            }
        }
    }

    impl<R, S> Display for ToBoxedSliceError<R, S>
    {
        #[inline]
        fn fmt(
            &self,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result
        {
            let prefix = "`payload::Extra::to_boxed_slice` failed due to";
            match self {
                ToBoxedSliceError::OutOfBounds { at } => fmt_oob(f, prefix, *at),
                ToBoxedSliceError::RangeTooLong { by } =>
                    write!(f, "{prefix} range-too-long by {by}"),
                ToBoxedSliceError::Read(_) => fmt_r(f, prefix),
                ToBoxedSliceError::Seek(_) => fmt_s(f, prefix),
                ToBoxedSliceError::BadImpl => fmt_bi(f, prefix),
            }
        }
    }
}


#[cfg(any(feature = "std", feature = "anticipate", rust_lib_feature = "error_in_core"))]
mod standard_error
{
    use super::{
        CopyToSliceError,
        ToBoxedSliceError,
    };

    cfg_if::cfg_if! { if #[cfg(any(feature = "anticipate", rust_lib_feature = "error_in_core"))]
    {
        use core::error::Error;
    }
    else if #[cfg(feature = "std")]
    {
        use std::error::Error;
    } }


    impl<R, S> Error for CopyToSliceError<R, S>
    where
        R: Error + 'static,
        S: Error + 'static,
    {
        #[inline]
        fn source(&self) -> Option<&(dyn Error + 'static)>
        {
            match self {
                CopyToSliceError::Read(read_error) => Some(read_error),
                CopyToSliceError::Seek(seek_error) => Some(seek_error),
                CopyToSliceError::OutOfBounds { .. } | CopyToSliceError::BadImpl => None,
            }
        }
    }

    #[cfg(feature = "alloc")]
    impl<R, S> Error for ToBoxedSliceError<R, S>
    where
        R: Error + 'static,
        S: Error + 'static,
    {
        #[inline]
        fn source(&self) -> Option<&(dyn Error + 'static)>
        {
            match self {
                ToBoxedSliceError::Read(read_error) => Some(read_error),
                ToBoxedSliceError::Seek(seek_error) => Some(seek_error),
                ToBoxedSliceError::OutOfBounds { .. }
                | ToBoxedSliceError::RangeTooLong { .. }
                | ToBoxedSliceError::BadImpl => None,
            }
        }
    }
}
