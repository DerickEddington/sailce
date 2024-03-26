//! Aspects of `Payload`s.
//!
//! **TODO**: The API of this module is definitely unstable and will probably need to be changed,
//! since we have very little experience with it.


mod errors;
pub use errors::*;

pub mod extra;
pub use extra::*;

mod impls;


/// An arbitrary sequence of bytes.  I.e. a single logical byte-string.  At most [`u64::MAX`]
/// bytes.
///
/// Applications read and write `Payload`s from and to Subspaces, addressing via hierarchical
/// [`Path`](crate::Path)s.
///
/// The contents of `Payload`s by themselves are immutable because they're identified by the
/// digest (usually cryptographic hash) of their content (i.e. content addressing) and so this
/// must not change.  It's up to the implementing type to provide the means of creating it with
/// content.  If reusing the backing storage or memory for mutation is desired, the type should
/// provide conversion into some other type that doesn't implement `Payload`.  As such, this trait
/// intentionally does not provide methods for mutating the contents, and its methods take `&mut
/// self` only to support mutating of the seek position or other non-content (e.g. caching) state.
///
/// The `async` methods of this API are the primary interface for their respective functionality.
/// This `async` API can still be used from sync code that wishes to block (instead of `.await`
/// suspending), by using the [`sync::Payload`] trait that extends this.
///
/// The seeking-&-pulling-at-current-position API allows the implementor flexibility in the
/// representation (e.g. to retrieve chunks lazily and not hold them all in-memory at once).
///
/// (This API is somewhat like [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html)
/// and [`std::io::Seek`](https://doc.rust-lang.org/std/io/trait.Seek.html), but differs in some
/// significant ways, such as: being `async`, erroring when attempting to seek beyond the end, and
/// having multiple separate generic error types.)
#[allow(async_fn_in_trait)] // TODO: Re-evaluate.
pub trait Payload
{
    /// Error(s) possibly returned by [`read`](Self::read).
    type ReadError;
    /// Error(s) possibly returned by [`seek`](Self::seek).
    type SeekError;

    /// Pull some bytes from this `Payload` into the specified buffer, returning how many bytes
    /// were read.
    ///
    /// This function guarantees that it won't block waiting for data.
    ///
    /// If the return value of this method is `Ok(n)`, then implementations must guarantee that
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

    /// Returns the length of this `Payload` (in bytes).
    async fn len(&self) -> u64;

    /// Returns `true` if this `Payload` has a length of 0.
    ///
    /// This is equivalent to `self.len().await == 0` but might allow some implementations to be
    /// more efficient.
    #[inline]
    async fn is_empty(&self) -> bool
    {
        self.len().await == 0
    }
}


/// The possible ways to seek within a [`Payload`].
///
/// (This differs from [`std::io::SeekFrom`](https://doc.rust-lang.org/std/io/enum.SeekFrom.html),
/// because Payloads in Willow are fixed-size.  Also, this needs to be provided when the `std`
/// library isn't available, because this crate supports `no_std`.)
#[derive(Copy, Clone, Eq, Ord, Hash, PartialEq, PartialOrd, Debug)]
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


/// Aspects of synchronous-API `Payload`s.
pub mod sync
{
    use {
        super::SeekFrom,
        crate::syncify::Syncify,
    };

    /// Like [`crate::Payload`] but all methods are synchronous (i.e. not `async`) and might block
    /// callers.
    #[allow(clippy::missing_errors_doc)]
    pub trait Payload<Executor>: super::Payload + Syncify<Executor>
    where Executor: ?Sized
    {
        /// Like [`crate::Payload::read`] but synchronous.  Might block.
        #[inline]
        fn read(
            &mut self,
            buf: &mut [u8],
        ) -> Result<usize, Self::ReadError>
        {
            let (block_on, data) = get_block_on_and_data!(self);
            block_on(super::Payload::read(self, buf), data)
        }

        /// Like [`crate::Payload::seek`] but synchronous.  Might block.
        #[inline]
        fn seek(
            &mut self,
            pos: SeekFrom,
        ) -> Result<u64, Self::SeekError>
        {
            let (block_on, data) = get_block_on_and_data!(self);
            block_on(super::Payload::seek(self, pos), data)
        }

        /// Like [`crate::Payload::len`] but synchronous.  Might block.
        #[inline]
        fn len(&self) -> u64
        {
            let (block_on, data) = get_block_on_and_data!(self);
            block_on(super::Payload::len(self), data)
        }

        /// Like [`crate::Payload::is_empty`] but synchronous.  Might block.
        #[inline]
        fn is_empty(&self) -> bool
        {
            let (block_on, data) = get_block_on_and_data!(self);
            block_on(super::Payload::is_empty(self), data)
        }
    }
}
