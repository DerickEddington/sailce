use {
    super::{
        errors::PathLimitError,
        Path,
    },
    core::cmp::Ordering,
};


impl<T> Extra for T where T: Path + ?Sized {}

/// Additional methods that are automatically implemented for all types that implement [`Path`].
///
/// This trait should not be implemented for other types (and probably cannot ever be, due to our
/// blanket implementation).
///
/// (These aren't part of `Path` because that would allow implementors to override these but we
/// don't want that.)
pub trait Extra: Path
{
    /// Create an instance of `Self` from another type of [`Path`].
    ///
    /// If `Self` can be created from an `Iterator` of an `Item` type that can be created from the
    /// `&[u8]` bytes of `Path` [`Component`](super::Component)s.
    #[inline]
    fn from_path<'l, Po, C>(other: &'l Po) -> Self
    where
        Self: FromIterator<C>,
        Po: Path + ?Sized,
        &'l [u8]: Into<C>,
    {
        other.components().map(|c| c.inner.into()).collect()
    }

    /// Like [`Self::from_path`] but uses fallible conversions that might fail.
    ///
    /// # Errors
    /// If converting bytes into the desired component type fails.
    #[inline]
    fn try_from_path<'l, Po, C, E>(other: &'l Po) -> Result<Self, E>
    where
        Self: FromIterator<C>,
        Po: Path + ?Sized,
        &'l [u8]: TryInto<C, Error = E>,
    {
        other.components().map(|c| c.inner.try_into()).collect()
    }

    /// Like [`Self::from_path`] but enforce the limits of a [`Params`](crate::Params).
    ///
    /// # Errors
    /// If the limits are exceeded by `other`.
    #[inline]
    fn from_path_limited<'l, Params, Po, C>(other: &'l Po) -> Result<Self, PathLimitError>
    where
        Params: crate::Params + ?Sized,
        Self: FromIterator<C>,
        Po: Path + ?Sized,
        &'l [u8]: Into<C>,
    {
        let mut total_sz = 0_usize;

        other
            .components()
            .enumerate()
            .map(|(index, c)| {
                let c_sz = c.bytes().len();
                let mut total_overflowed = false;
                match total_sz.checked_add(c_sz) {
                    Some(t) => total_sz = t,
                    None => total_overflowed = true,
                }
                let within_max_component_length = c_sz <= Params::MAX_COMPONENT_LENGTH.into();
                let within_max_component_count = index <= Params::MAX_COMPONENT_COUNT.into();
                let within_max_path_length =
                    total_sz <= Params::MAX_PATH_LENGTH.into() && !total_overflowed;

                if within_max_component_length
                    && within_max_component_count
                    && within_max_path_length
                {
                    Ok(c.inner.into())
                }
                else {
                    Err(PathLimitError {
                        index,
                        within_max_component_length,
                        within_max_component_count,
                        within_max_path_length,
                    })
                }
            })
            .collect()
    }

    /// Return whether or not `self` and `other` are equal by their `Component`s.
    #[inline]
    fn eq_components<Po>(
        &self,
        other: &Po,
    ) -> bool
    where
        Po: Path + ?Sized,
    {
        self.components().eq(other.components())
    }

    /// Return how `self` and `other` compare lexicographically by their `Component`s.
    #[inline]
    fn cmp_components<Po>(
        &self,
        other: &Po,
    ) -> Ordering
    where
        Po: Path + ?Sized,
    {
        self.components().cmp(other.components())
    }
}
