//! Aspects of `Payload`s.
//!
//! The `async` `fn`s of this API are the only interface for their respective functionality, to
//! avoid otherwise also providing sync ones that do the same.  This `async` API can still be used
//! from sync code that wishes to block (instead of `.await` suspending), by using an executor
//! that does such (e.g. [`futures::executor::block_on`](
//! https://docs.rs/futures/latest/futures/executor/fn.block_on.html), or [`pollster`](
//! https://docs.rs/pollster/latest/pollster/), etc.).
//!
//! **TODO**: The API of this module is definitely unstable and will probably need to be changed,
//! since we have very little experience with it.

use {
    crate::anticipated_or_like::Error,
    cfg_if::cfg_if,
    core::fmt::{
        self,
        Display,
        Formatter,
    },
};


cfg_if! { if #[cfg(feature = "anticipate")]
{
    pub struct DefaultIsEmptyError;

    impl Error for DefaultIsEmptyError {}
}
else
{
    /// [`Payload::is_empty`] failed for any reason.
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    #[allow(clippy::exhaustive_structs)]
    pub struct IsEmptyError;

    impl Display for IsEmptyError
    {
        #[inline]
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
        {
            f.write_str("`Payload::is_empty()` failed")
        }
    }

    impl Error for IsEmptyError {}
} }

#[cfg(all(not(feature = "anticipate"), rust_lang_feature = "associated_type_defaults"))]
/// If the `associated_type_defaults` feature becomes stable in a future Rust version and we're
/// not wanting to use our experimental "anticipate" package-feature, automatically preserve
/// compatibility without breaking the SemVer of our API.
type DefaultIsEmptyError = IsEmptyError;


/// An arbitrary sequence of bytes.  I.e. a single logical byte-string.  At most [`u64::MAX`]
/// bytes.
///
/// Applications read and write `Payload`s from and to Subspaces, addressing via hierarchical
/// [`Path`](crate::Path)s.
///
/// (This API is somewhat like [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html)
/// and [`std::io::Seek`](https://doc.rust-lang.org/std/io/trait.Seek.html), but differs in some
/// significant ways, such as: being `async`, erroring when attempting to seek beyond the end, and
/// having multiple separate generic error types.)
#[allow(async_fn_in_trait)] // TODO: Re-evaluate.
pub trait Payload
{
    /// Error(s) possibly returned by [`read`](Self::read).
    type ReadError: Error;
    /// Error(s) possibly returned by [`seek`](Self::seek).
    type SeekError: Error;

    // Have this, if our package-feature "anticipate" is activated.  Or, automatically have this,
    // if the `associated_type_defaults` feature becomes stable in a future Rust version.
    #[cfg(any(feature = "anticipate", rust_lang_feature = "associated_type_defaults"))]
    /// Error(s) possibly returned by [`is_empty`](Self::is_empty).
    type IsEmptyError: Error = DefaultIsEmptyError;

    /// Pull some bytes from this `Payload` into the specified buffer, returning how many bytes
    /// were read.
    ///
    /// This function does guarantee that it won't block waiting for data.
    ///
    /// If the return value of this method is [`Ok(n)`], then implementations must guarantee that
    /// `0 <= n <= buf.len()`.  A nonzero `n` value indicates that the buffer `buf` has been
    /// filled in with `n` bytes of data from this `Payload`.  If `n` is `0`, then it can indicate
    /// one of two scenarios:
    /// 1. This reader has reached its "end of file" and will no longer be able to produce bytes.
    /// 2. The buffer specified was 0 bytes in length.
    ///
    /// It is not an error if the returned value `n` is smaller than the buffer size, even when
    /// the reader is not at the end of the `Payload` yet.  This may happen for example because
    /// fewer bytes are actually available right now.
    ///
    /// No guarantees are provided about the contents of `buf` when this function is called, so
    /// implementations cannot rely on any property of the contents of `buf` being true.  It is
    /// recommended that implementations only write data to `buf` instead of reading its contents.
    ///
    /// # Errors
    /// If this function encounters any form of I/O or other error, an error variant will be
    /// returned.  If an error is returned then it must be guaranteed that no bytes were read.
    async fn read(
        &mut self,
        buf: &mut [u8],
    ) -> Result<usize, Self::ReadError>;

    // TODO: More methods like `std::io::Read` and/or `AsyncRead` (of Tokio or async_std) that are
    // provided with default impls.

    /// Seek to an offset, in bytes, in this `Payload`.
    ///
    /// If the seek operation completed successfully, this method returns the new position from
    /// the start of the `Payload`.  That position can be used later with [`SeekFrom::Start`].
    ///
    /// # Errors
    /// Seeking can fail, depending on the implementation.
    /// It's always an error to seek beyond the end of a `Payload`, and it's always an error to
    /// seek before byte 0.
    async fn seek(
        &mut self,
        pos: SeekFrom,
    ) -> Result<u64, Self::SeekError>;

    /// Rewind to the beginning of the `Payload`.
    ///
    /// This is a convenience method, equivalent to `self.seek(SeekFrom::Start(0)).await`.
    #[inline]
    async fn rewind(&mut self) -> Result<(), Self::SeekError>
    {
        let _: u64 = self.seek(SeekFrom::Start(0)).await?;
        Ok(())
    }

    /// Returns the length of this `Payload` (in bytes).
    ///
    /// This method is implemented using up to three seek operations.  If this method returns
    /// successfully, the seek position is unchanged (i.e. the position before calling this method
    /// is the same as afterwards).  However, if this method returns an error, the seek position
    /// is unspecified.
    ///
    /// If you need to obtain the length of _many_ `Payload`s and you don't care about the seek
    /// position afterwards, you can reduce the number of seek operations by simply calling
    /// `seek(SeekFrom::End(0)).await` and using its return value (it is also the `Payload`
    /// length).
    #[inline]
    async fn len(&mut self) -> Result<u64, Self::SeekError>
    {
        let old_pos = self.position().await?;
        let len = self.seek(SeekFrom::End(0)).await?;

        // Avoid seeking a third time when we were already at the end of the `Payload`.  The
        // branch is usually way cheaper than a seek operation.
        if old_pos != len {
            let _: u64 = self.seek(SeekFrom::Start(old_pos)).await?;
        }

        Ok(len)
    }

    cfg_if! { if #[cfg(not(any(feature = "anticipate",
                               rust_lang_feature = "associated_type_defaults")))]
    {
        /// Returns `true` if this `Payload` has a length of 0.
        ///
        /// This is equivalent to `self.len().await? == 0` but might allow some implementations to
        /// be more efficient.
        ///
        /// # Errors
        /// If the implementation encounters any error, only `Err` of the unit-like type
        /// `IsEmptyError` is returned.  For the default provided implementation, this would be
        /// caused by some `Self::SeekError`; but for other implementations, this could be caused
        /// by any arbitrary reason, or might be impossible (i.e. infallible).  (If
        /// "associated-type defaults" ever becomes stabilized, maybe the error type of this
        /// should instead be changed to some `Self::IsEmptyError` that has default `type
        /// IsEmptyError = DefaultIsEmptyError` which might enable implementations to provide
        /// better error info when desired by choosing a different type for `Self::IsEmptyError`.)
        #[inline]
        async fn is_empty(&mut self) -> Result<bool, IsEmptyError>
        {
            Ok(self.len().await.or(Err(IsEmptyError))? == 0)
        }
    }
    else
    {
        /// Returns `true` if this `Payload` has a length of 0.
        ///
        /// This is equivalent to `self.len().await? == 0` but might allow some implementations to
        /// be more efficient.
        ///
        /// # Errors
        /// If the implementation encounters any error.  For the default provided implementation,
        /// this would be caused by some `Self::SeekError`; but for other implementations, this
        /// could be caused by any arbitrary reason, or might be impossible (i.e. infallible).
        #[inline]
        async fn is_empty(&mut self) -> Result<bool, Self::IsEmptyError>
        {
            Ok(self.len().await? == 0)
        }
    } }

    /// Returns the current seek position from the start of the `Payload`.
    ///
    /// This is equivalent to `self.seek(SeekFrom::Current(0)).await`.
    /// ```
    #[inline]
    async fn position(&mut self) -> Result<u64, Self::SeekError>
    {
        self.seek(SeekFrom::Current(0)).await
    }

    /// Seeks relative to the current position.
    ///
    /// This is equivalent to `self.seek(SeekFrom::Current(offset)).await` but doesn't return the
    /// new position which might allow some implementations to perform more efficient seeks.
    #[inline]
    async fn seek_relative(
        &mut self,
        offset: i64,
    ) -> Result<(), Self::SeekError>
    {
        let _: u64 = self.seek(SeekFrom::Current(offset)).await?;
        Ok(())
    }
}


/// The possible ways to seek within a [`Payload`].
///
/// (This differs from [`std::io::SeekFrom`](https://doc.rust-lang.org/std/io/enum.SeekFrom.html),
/// because Payloads in Willow are fixed-size.  Also, this needs to be provided when the `std`
/// library isn't available, because this crate supports `no_std`.)
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum SeekFrom
{
    /// Sets the offset to the provided number of bytes.
    ///
    /// It's an error to seek beyond the end of a `Payload`.
    Start(u64),

    /// Sets the offset to the size of the `Payload` **minus** the specified number of bytes.
    ///
    /// This ensures this can't seek beyond the end of a `Payload`.
    ///
    /// It's an error to seek before byte 0.
    End(u64),

    /// Sets the offset to the current position plus the specified number of bytes.
    ///
    /// It's an error to seek beyond the end of a `Payload`, and it's an error to seek before
    /// byte 0.
    Current(i64),
}
