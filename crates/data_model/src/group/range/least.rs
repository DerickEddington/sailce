use {
    super::{
        Range,
        ThreeDimRange,
    },
    crate::Timestamp,
};


/// A type that `impl`ements this has a _least_ value that is less than all other values of it.
///
/// This is needed because [`Default`] does not guarantee that its value is the least (e.g. it
/// gives `0` for `i32`).  (If the standard `core` library ever gains an equivalent trait, that
/// should be used instead, and, as a transition phase, this could be `impl`emented for everything
/// which `impl`ements that.)
pub trait Least
{
    /// The value that is less than all other values of `Self`.
    fn least() -> Self;
}


macro_rules! impl_Least {
    ($ty:ty = $val:expr) => {
        impl $crate::group::range::Least for $ty
        {
            #[inline]
            fn least() -> Self
            {
                $val
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! impl_Least_zero {
    ($($ty:ty)*) => { $(impl_Least! { $ty = 0 })* }
}

#[allow(unused_macros)]
macro_rules! impl_Least_empty {
    ($($ty:ty)*) => { $(impl_Least! { $ty = <$ty>::from_iter([]) })* }
}

macro_rules! impl_Least_MIN {
    ($($ty:ty)*) => { $(impl_Least! { $ty = <$ty>::MIN })* }
}

macro_rules! impl_Least_str {
    ($($ty:ty)*) => { $(impl_Least! { $ty = "".into() })* }
}


impl_Least_MIN! { u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize }

impl_Least_str! { &str }

impl_Least! { char = '\0' }

impl<const N: usize, L: Least + Copy> Least for [L; N]
{
    #[inline]
    fn least() -> Self
    {
        [L::least(); N]
    }
}

impl Least for Timestamp
{
    #[inline]
    fn least() -> Self
    {
        Self { μs_since_epoch: 0_u64 }
    }
}

impl<T: Least> Least for Range<T>
{
    #[inline]
    fn least() -> Self
    {
        (T::least() .. T::least()).into()
    }
}

impl<S: Least, P: Least> Least for ThreeDimRange<S, P>
{
    #[inline]
    fn least() -> Self
    {
        Self { subspaces: Range::least(), paths: Range::least(), times: Range::least() }
    }
}


#[cfg(feature = "alloc")]
mod alloc
{
    extern crate alloc;
    use alloc::string::String;

    impl_Least_str! { String }
}


// TODO: impl for more types as appropriate.
