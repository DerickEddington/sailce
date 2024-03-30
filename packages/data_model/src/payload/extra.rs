//! Additional functionality for [`Payload`]s.

#![allow(async_fn_in_trait)] // TODO: Re-evaluate.

use super::{
    errors::CopyToSliceError,
    Payload,
    SeekFrom,
};


impl<T> ExtraCore for T where T: Payload + ?Sized {}

/// Additional methods that are automatically implemented for all types that implement
/// [`Payload`].
///
/// These are always available, including with `no_std` and without our `"alloc"` or `"std"`
/// package features.  See the [`Extra`] trait for further methods that are only available with
/// our `"alloc"` package feature.
///
/// This trait should not be implemented for other types (and probably cannot ever be, due to our
/// blanket implementation).
///
/// (These aren't part of `Payload` because that would allow implementors to override these but we
/// don't want that.)
pub trait ExtraCore: Payload
{
    /// Return the current position of `self`.  Its position remains unchanged.
    ///
    /// # Errors
    /// If `self`'s implementation of [`Payload::seek`] errors.
    #[inline]
    async fn current_position(&mut self) -> Result<u64, Self::SeekError>
    {
        self.seek(SeekFrom::Current(0)).await
    }

    /// Copy a range of the bytes of a [`Payload`] into a slice.
    ///
    /// The range is `start .. (start + dest.len())`.  If `start` is `None`, the current position
    /// of `self` is used.
    ///
    /// This [`seek`](Payload::seek)s to `start` (if needed), loops [`read`](Payload::read)ing,
    /// invoking `callback` (if `Some`) on each chunk `read` (of arbitrary size `<= dest.len()`),
    /// until `dest` is completely filled, and then, if `restore` is true, `seek`s back to the
    /// original position.
    ///
    /// `callback` enables some approaches of doing some processing during the same pass over the
    /// bytes.  As an `FnMut`, it supports closures that mutate some captured state.  E.g. it can
    /// be useful for computing the [`payload_digest`](crate::Entry::payload_digest), when copying
    /// the entire payload (possibly by multiple calls to `copy_to_slice`), at the same time as
    /// our copying, which e.g. can be useful for [`Store::put`]( crate::store::async::Store::put)
    /// implementations to be more efficient.  E.g. a hasher state could be captured by a
    /// `callback` closure and updated by each call.  Or e.g. it could be useful to compute
    /// something from a known range of some structured contents, or even to mutate the copied
    /// bytes.  Note that `callback` is not `async` (does not return a `Future`) and so must not
    /// block.
    ///
    /// # Errors
    /// - If `start` or `start + dest.len()` is outside the bounds of `self`.
    /// - If any of the called `Payload` methods error.
    /// - If the implementation of `Payload` misbehaves in a detected way.
    #[inline]
    async fn copy_to_slice<C>(
        &mut self,
        start: Option<u64>,
        dest: &mut [u8],
        callback: Option<C>,
        restore: bool,
    ) -> Result<(), CopyToSliceError<Self::ReadError, Self::SeekError>>
    where
        C: FnMut(&mut [u8]),
    {
        use CopyToSliceError as Error;

        let orig_pos = self.current_position().await.map_err(Error::Seek)?;
        let payload_len = self.len().await;
        copy_to_slice_with(self, payload_len, orig_pos, start, dest, callback, restore).await
    }
}


/// Take a given `payload_len` & `orig_pos` and assume they're correct.  This enables avoiding
/// computing them more than once.
async fn copy_to_slice_with<P, C>(
    payload: &mut P,
    payload_len: u64,
    orig_pos: u64,
    start: Option<u64>,
    dest: &mut [u8],
    mut callback: Option<C>,
    restore: bool,
) -> Result<(), CopyToSliceError<P::ReadError, P::SeekError>>
where
    P: Payload + ?Sized,
    C: FnMut(&mut [u8]),
{
    use CopyToSliceError as Error;

    let check_in_bounds =
        |x| if x <= payload_len { Ok(()) } else { Err(Error::out_of_bounds_at(x)) };
    let start = start.unwrap_or(orig_pos);
    check_in_bounds(start)?;
    let dest_len = dest.len().try_into().ok().ok_or_else(Error::out_of_bounds_overflowed)?;
    let end = start.checked_add(dest_len).ok_or_else(Error::out_of_bounds_overflowed)?;
    check_in_bounds(end)?;

    if start != orig_pos {
        let seeked_pos = payload.seek(SeekFrom::Start(start)).await.map_err(Error::Seek)?;
        if seeked_pos != start {
            return Err(Error::BadImpl);
        }
    }
    let mut buf = dest;
    // Loop on this condition, to be more robust by not relying on the `Payload` impl to behave
    // correctly when `buf.len() == 0`.
    while !buf.is_empty() {
        if let consumed @ 1 .. = payload.read(buf).await.map_err(Error::Read)? {
            if let Some(callback) = &mut callback {
                let filled = buf.get_mut(.. consumed).ok_or_else(|| Error::BadImpl)?;
                callback(filled);
            }
            let remainder = buf.get_mut(consumed ..).ok_or_else(|| Error::BadImpl)?;
            buf = remainder;
        }
        else {
            break;
        }
    }
    if !buf.is_empty() {
        return Err(Error::BadImpl); // It failed to give all of what it said was its length.
    }
    if restore {
        let restored_pos = payload.seek(SeekFrom::Start(orig_pos)).await.map_err(Error::Seek)?;
        if restored_pos != orig_pos {
            return Err(Error::BadImpl);
        }
    }
    Ok(())
}


#[cfg(feature = "alloc")]
pub use alloc::*;

#[cfg(feature = "alloc")]
mod alloc
{
    use {
        super::{
            super::errors::ToBoxedSliceError,
            copy_to_slice_with,
            ExtraCore,
            Payload,
        },
        alloc::{
            boxed::Box,
            vec,
        },
        core::ops::{
            Bound,
            RangeBounds,
        },
    };

    #[allow(clippy::as_conversions, clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    const ISIZE_MAX_AS_U64: u64 = {
        let as_u64 = isize::MAX as u64;
        let as_isize = as_u64 as isize;
        // Note: This panic can only occur at const-eval time, not at runtime.
        if as_isize == isize::MAX { as_u64 } else { panic!("need a different way") }
    };

    impl<T> Extra for T where T: Payload + ?Sized {}

    /// Additional methods that are automatically implemented for all types that implement
    /// [`Payload`].
    ///
    /// These are only available with our `"alloc"` package feature (which is enabled by default).
    ///
    /// This trait should not be implemented for other types (and probably cannot ever be, due to
    /// our blanket implementation).
    ///
    /// (These aren't part of `Payload` because that would allow implementors to override these
    /// but we don't want that.)
    pub trait Extra: ExtraCore
    {
        /// Copy a range of the bytes of a [`Payload`] into a newly-allocated boxed slice.
        ///
        /// If `range.start_bound()` is `Unbounded`, the current position of `self` is used as
        /// the start, not `0`.  I.e. `..` and `.. x` start from the current position.  To
        /// ensure the beginning is used as the start, give `0 ..` or `0 .. x`.
        ///
        /// See [`copy_to_slice`](ExtraCore::copy_to_slice) for the meaning of the other arguments
        /// and for more about the behavior.
        ///
        /// # Errors
        /// - If `range` is outside the bounds of `self`.
        /// - If `range` is longer than [`isize::MAX`] and so can't all fit in an allocation.
        /// - If any of the called `Payload` methods error.
        /// - If the implementation of `Payload` misbehaves in a detected way.
        #[inline]
        async fn to_boxed_slice<C>(
            &mut self,
            range: impl RangeBounds<u64>,
            callback: Option<C>,
            restore: bool,
        ) -> Result<Box<[u8]>, ToBoxedSliceError<Self::ReadError, Self::SeekError>>
        where
            C: FnMut(&mut [u8]),
        {
            use ToBoxedSliceError as Error;

            let orig_pos = self.current_position().await.map_err(Error::Seek)?;
            let payload_len = self.len().await;

            let checked_incr =
                |x: u64| x.checked_add(1).ok_or_else(Error::out_of_bounds_overflowed);
            let start = match range.start_bound() {
                Bound::Included(x) => *x,
                Bound::Excluded(x) => checked_incr(*x)?,
                Bound::Unbounded => orig_pos,
            };
            let end = match range.end_bound() {
                Bound::Included(x) => checked_incr(*x)?,
                Bound::Excluded(x) => *x,
                Bound::Unbounded => payload_len,
            };
            let range_len = end.saturating_sub(start); // If `start > end` then empty.
            let alloc_len =
                isize::try_from(range_len).and_then(usize::try_from).ok().ok_or_else(|| {
                    debug_assert!(range_len > ISIZE_MAX_AS_U64, "failed convert implies greater");
                    Error::range_too_long_by(range_len.saturating_sub(ISIZE_MAX_AS_U64))
                })?;

            let vec = vec![0; alloc_len]; // FUTURE: Use more-efficient uninitialized.
            debug_assert_eq!(
                vec.capacity(),
                alloc_len,
                "Want `into_boxed_slice` to not reallocate."
            );
            let mut boxed_slice = vec.into_boxed_slice();
            copy_to_slice_with(
                self,
                payload_len,
                orig_pos,
                Some(start),
                &mut boxed_slice,
                callback,
                restore,
            )
            .await?;
            Ok(boxed_slice)
        }
    }
}


/// Aspects of synchronous-API `Payload`s.
pub mod sync
{
    use {
        super::super::errors::CopyToSliceError,
        crate::syncify::Syncify,
    };


    /// Like [`crate::payload::ExtraCore`] but all methods are synchronous (i.e. not `async`) and
    /// might block callers.
    #[allow(clippy::missing_errors_doc)]
    pub trait ExtraCore<Executor>: super::ExtraCore + Syncify<Executor>
    where Executor: ?Sized
    {
        /// Like [`crate::payload::ExtraCore::current_position`] but synchronous.  Might block.
        #[inline]
        fn current_position(&mut self) -> Result<u64, Self::SeekError>
        {
            let (block_on, data) = get_block_on_and_data!(self);
            block_on(super::ExtraCore::current_position(self), data)
        }

        /// Like [`crate::payload::ExtraCore::copy_to_slice`] but synchronous.  Might block.
        #[inline]
        fn copy_to_slice<C>(
            &mut self,
            start: Option<u64>,
            dest: &mut [u8],
            callback: Option<C>,
            restore: bool,
        ) -> Result<(), CopyToSliceError<Self::ReadError, Self::SeekError>>
        where
            C: FnMut(&mut [u8]),
        {
            let (block_on, data) = get_block_on_and_data!(self);
            block_on(super::ExtraCore::copy_to_slice(self, start, dest, callback, restore), data)
        }
    }


    #[cfg(feature = "alloc")]
    pub use alloc::*;

    #[cfg(feature = "alloc")]
    mod alloc
    {
        use {
            super::super::super::errors::ToBoxedSliceError,
            alloc::boxed::Box,
            core::ops::RangeBounds,
        };

        /// Like [`crate::payload::Extra`] but all methods are synchronous (i.e. not `async`) and
        /// might block callers.
        #[allow(clippy::missing_errors_doc)]
        pub trait Extra<Executor>: super::super::Extra + super::ExtraCore<Executor>
        where Executor: ?Sized
        {
            /// Like [`crate::payload::Extra::to_boxed_slice`] but synchronous.  Might block.
            #[inline]
            #[allow(clippy::type_complexity)]
            fn to_boxed_slice<C>(
                &mut self,
                range: impl RangeBounds<u64>,
                callback: Option<C>,
                restore: bool,
            ) -> Result<Box<[u8]>, ToBoxedSliceError<Self::ReadError, Self::SeekError>>
            where
                C: FnMut(&mut [u8]),
            {
                let (block_on, data) = get_block_on_and_data!(self);
                block_on(
                    super::super::Extra::to_boxed_slice(self, range, callback, restore),
                    data,
                )
            }
        }
    }
}
