use super::{
    Payload,
    SeekFrom,
};


impl<T> Payload for &mut T
where T: Payload + ?Sized
{
    type ReadError = <T as Payload>::ReadError;
    type SeekError = <T as Payload>::SeekError;

    #[inline]
    async fn read(
        &mut self,
        buf: &mut [u8],
    ) -> Result<usize, Self::ReadError>
    {
        <T as Payload>::read(*self, buf).await
    }

    #[inline]
    async fn seek(
        &mut self,
        pos: SeekFrom,
    ) -> Result<u64, Self::SeekError>
    {
        <T as Payload>::seek(*self, pos).await
    }

    #[inline]
    async fn len(&self) -> u64
    {
        <T as Payload>::len(*self).await
    }

    #[inline]
    async fn is_empty(&self) -> bool
    {
        <T as Payload>::is_empty(*self).await
    }
}


#[cfg(feature = "std")]
mod cursor
{
    use {
        crate::{
            payload::{
                self,
                sync,
            },
            syncify::Syncify,
            Payload,
        },
        core::{
            convert::Infallible,
            future::Future,
        },
        std::io::{
            self,
            Cursor,
            Read,
            Seek,
        },
    };


    /// This upholds the requirement that the methods won't block, because `Cursor`s are in-memory
    /// and never block.
    impl<T> Payload for Cursor<T>
    where T: AsRef<[u8]>
    {
        type ReadError = io::Error;
        type SeekError = io::Error;

        #[inline]
        async fn read(
            &mut self,
            buf: &mut [u8],
        ) -> Result<usize, Self::ReadError>
        {
            <Self as sync::Payload<()>>::read(self, buf)
        }

        #[inline]
        async fn seek(
            &mut self,
            pos: payload::SeekFrom,
        ) -> Result<u64, Self::SeekError>
        {
            <Self as sync::Payload<()>>::seek(self, pos)
        }

        #[inline]
        async fn len(&self) -> u64
        {
            <Self as sync::Payload<()>>::len(self)
        }

        #[inline]
        async fn is_empty(&self) -> bool
        {
            <Self as sync::Payload<()>>::is_empty(self)
        }
    }


    /// `Cursor`s actually don't need an executor, to reuse their methods as synchronous, but
    /// implementing this is needed to implement `sync::Payload`.
    #[allow(clippy::unreachable)]
    impl<T> Syncify<()> for Cursor<T>
    {
        type ExecutorData = Infallible;

        #[inline]
        fn get_block_on_fn<'f, F>(&self) -> impl 'f + FnOnce(F, Self::ExecutorData) -> F::Output
        where F: Future + 'f
        {
            |_, _| unreachable!()
        }

        #[inline]
        fn get_executor_data(&self) -> Self::ExecutorData
        {
            unreachable!()
        }
    }

    /// Unlike most implementations of `sync::Payload`, this provides its own implementations of
    /// the methods, instead of just using the default ones, in order to not involve an executor
    /// and to use the `Cursor`'s own already-sync methods.
    impl<T> sync::Payload<()> for Cursor<T>
    where T: AsRef<[u8]>
    {
        #[inline]
        fn read(
            &mut self,
            buf: &mut [u8],
        ) -> Result<usize, Self::ReadError>
        {
            <Self as Read>::read(self, buf)
        }

        #[inline]
        fn seek(
            &mut self,
            pos: payload::SeekFrom,
        ) -> Result<u64, Self::SeekError>
        {
            <Self as Seek>::seek(self, pos.try_into()?)
        }

        #[inline]
        fn len(&self) -> u64
        {
            // `Payload`s are defined by Willow to be at most `u64::MAX` in size, so if `usize` is
            // ever wider than 64-bit and if a `Cursor` is ever created with size greater than
            // `u64::MAX` then it conforms to Willow to ignore the part that is greater.
            self.get_ref().as_ref().len().try_into().unwrap_or(u64::MAX)
        }

        #[inline]
        fn is_empty(&self) -> bool
        {
            self.get_ref().as_ref().is_empty()
        }
    }


    impl TryFrom<payload::SeekFrom> for io::SeekFrom
    {
        type Error = io::Error;

        #[inline]
        fn try_from(value: payload::SeekFrom) -> Result<Self, Self::Error>
        {
            match value {
                payload::SeekFrom::Start(pos) => Ok(io::SeekFrom::Start(pos)),
                payload::SeekFrom::End(pos) => 0_i64
                    .checked_sub_unsigned(pos)
                    .map(io::SeekFrom::End)
                    .ok_or(io::Error::from(io::ErrorKind::InvalidInput)),
                payload::SeekFrom::Current(pos) => Ok(io::SeekFrom::Current(pos)),
            }
        }
    }

    impl TryFrom<io::SeekFrom> for payload::SeekFrom
    {
        type Error = i64;

        #[inline]
        fn try_from(value: io::SeekFrom) -> Result<Self, Self::Error>
        {
            match value {
                io::SeekFrom::Start(pos) => Ok(payload::SeekFrom::Start(pos)),
                #[allow(clippy::arithmetic_side_effects)] // FUTURE: False-positive Clippy bug.
                io::SeekFrom::End(pos) => u64::MAX
                    .checked_add_signed(pos)
                    .map(|offset| payload::SeekFrom::End(u64::MAX - offset))
                    .ok_or(pos),
                io::SeekFrom::Current(pos) => Ok(payload::SeekFrom::Current(pos)),
            }
        }
    }
}
