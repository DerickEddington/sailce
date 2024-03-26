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


// TODO: Similarly for `sync::Payload`


// TODO?: `impl Payload for std::io::Cursor` when `#[cfg(feature = "std")]`, and also
// `sync::Payload<()>` with all methods (i.e. don't have the default provided methods) without an
// executor?
