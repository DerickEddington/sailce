use {
    crate::Scheme,
    core::{
        borrow::Borrow,
        cmp::Ordering,
        hash::{
            Hash,
            Hasher,
        },
        marker::PhantomData,
    },
};


/// A single encrypted [`Component`](sailce_data_model::path::Component) of an encrypted
/// [`Path`](sailce_data_model::Path).
///
/// The `Scheme` type parameter ensures that a concrete type of `EncryptedComponent` is bound to a
/// specific [`Scheme`](crate::Scheme) implementation, so that values of the type cannot be
/// mixed-up with values of other types of other `Scheme`s.
///
/// The private field prevents directly constructing values of this type, to ensure that creating
/// values is done properly by this crate's API.
#[derive(Debug)]
#[allow(clippy::partial_pub_fields, clippy::exhaustive_structs)]
pub struct EncryptedComponent<Bytes, Scheme>
where Bytes: Borrow<[u8]>
{
    /// Represents the byte-string that is the encrypted component.
    pub inner: Bytes,
    _scheme:   PhantomData<Scheme>,
}

/// Indicates that `EncryptedComponent` behaves identically to `[u8]` w.r.t. `Hash`, `Ord`, etc.
impl<B, S> Borrow<[u8]> for EncryptedComponent<B, S>
where B: Borrow<[u8]>
{
    #[inline]
    fn borrow(&self) -> &[u8]
    {
        self.inner.borrow()
    }
}

impl<B, S> EncryptedComponent<B, S>
where B: Borrow<[u8]>
{
    /// Get immutable reference to the encrypted bytes.
    #[inline]
    pub fn bytes(&self) -> &[u8]
    {
        <Self as Borrow<[u8]>>::borrow(self)
    }
}

impl<Ba, Bb, S> PartialEq<EncryptedComponent<Bb, S>> for EncryptedComponent<Ba, S>
where
    Ba: Borrow<[u8]>,
    Bb: Borrow<[u8]>,
{
    #[inline]
    fn eq(
        &self,
        other: &EncryptedComponent<Bb, S>,
    ) -> bool
    {
        self.bytes() == other.bytes()
    }
}

impl<B, S> Eq for EncryptedComponent<B, S> where B: Borrow<[u8]> {}

impl<Ba, Bb, S> PartialOrd<EncryptedComponent<Bb, S>> for EncryptedComponent<Ba, S>
where
    Ba: Borrow<[u8]>,
    Bb: Borrow<[u8]>,
{
    #[inline]
    fn partial_cmp(
        &self,
        other: &EncryptedComponent<Bb, S>,
    ) -> Option<Ordering>
    {
        Some(self.bytes().cmp(other.bytes()))
    }
}

impl<B, S> Ord for EncryptedComponent<B, S>
where B: Borrow<[u8]>
{
    #[inline]
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering
    {
        self.bytes().cmp(other.bytes())
    }
}

/// Enables `EncryptedComponent` to work with the blanket `impl`s of [`Path`](
/// sailce_data_model::Path) as the [`Component`](sailce_data_model::path::Component) type if ever
/// desired.  Also might be useful for other situations.
impl<B, S> AsRef<[u8]> for EncryptedComponent<B, S>
where B: Borrow<[u8]>
{
    #[inline]
    fn as_ref(&self) -> &[u8]
    {
        self.bytes()
    }
}

// The following can't be `derive`d, because of the `Scheme` type parameter that is necessary.

impl<B, S> Copy for EncryptedComponent<B, S> where B: Copy + Borrow<[u8]> {}

impl<B, S> Clone for EncryptedComponent<B, S>
where B: Clone + Borrow<[u8]>
{
    #[inline]
    fn clone(&self) -> Self
    {
        Self { inner: self.inner.clone(), _scheme: PhantomData }
    }
}

impl<B, S> Hash for EncryptedComponent<B, S>
where B: Hash + Borrow<[u8]>
{
    #[inline]
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    )
    {
        self.bytes().hash(state);
    }
}


impl<T> MakeEncryptedComponent for T where T: Scheme + ?Sized {}

/// Enables creating [`EncryptedComponent`]s, and helps do so properly, for all types that
/// implement [`Scheme`].
///
/// This trait should not be implemented for other types (and probably cannot ever be, due to our
/// blanket implementation).
pub trait MakeEncryptedComponent: Scheme
{
    /// Intended to be used only by implementations of [`Self::Cryptor::encrypt_component`](
    /// crate::Cryptor::encrypt_component) to create their return values, or by implementations of
    /// things like [`EncryptedPath::decrypt_components`](
    /// crate::EncryptedPath::decrypt_components) that must synthesize values of this type to pass
    /// to [`Self::Cryptor::decrypt_component`](crate::Cryptor::decrypt_component).
    #[inline]
    fn synthesize_encrypted_component<B: Borrow<[u8]>>(bytes: B) -> EncryptedComponent<B, Self>
    {
        EncryptedComponent { inner: bytes, _scheme: PhantomData }
    }
}
