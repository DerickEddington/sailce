#![cfg_attr(unix, doc = include_str!("../README.md"))]
#![cfg_attr(windows, doc = include_str!("..\\README.md"))]
// Apply the `no_std` attribute unconditionally, to require explicit conditional `use` of
// non-`core` items.
#![no_std]
// Warn about this one but avoid annoying hits for dev-dependencies.
#![cfg_attr(test, allow(unused_crate_dependencies))]


#[cfg(feature = "alloc")]
extern crate alloc;


use {
    core::borrow::{
        Borrow,
        BorrowMut,
    },
    sailce_data_model::path::Component,
};


mod component;
pub use component::{
    EncryptedComponent,
    MakeEncryptedComponent,
};

mod crypt;

mod decrypt;

mod encrypt;
pub use encrypt::EncryptPath;

mod encrypted;
pub use encrypted::EncryptedPath;

pub mod get_dest;


/// A pair of specific algorithms for encryption-&-decryption and key derivation, for
/// [`Path`](sailce_data_model::Path) [`Component`]s
///
/// The bounds on the type help some of the uses of this trait, and this is acceptable because the
/// type should not need any state just to represent the combination of these associated types.
pub trait Scheme: Copy + Sized + 'static
{
    /// Represents the keys used with both algorithms.
    type Key: Key<Scheme = Self> + ?Sized;
    /// The specific algorithm for encryption & decryption.
    type Cryptor: Cryptor<Scheme = Self> + ?Sized;
    /// The specific algorithm for key derivation.
    type KDF: KeyDerivationFunction<Scheme = Self> + ?Sized;
}


/// A specific type of keys used with both algorithms of a [`Scheme`].
pub trait Key
{
    /// The [`Scheme`] that `Self` is for.
    type Scheme: Scheme<Key = Self>;
}


/// A specific algorithm for encryption & decryption of [`Path`](sailce_data_model::Path)
/// [`Component`]s.
pub trait Cryptor
{
    /// The [`Scheme`] that `Self` is for.
    type Scheme: Scheme<Cryptor = Self>;

    /// Encrypts a single [`Component`] with the given `key`.  Output into where `get_dest`
    /// gives as its return value.
    ///
    /// `get_dest` is called with the exact size needed for the output, and it either returns
    /// `Some` value to use as the destination of where to write the encrypted output bytes, or it
    /// returns `None` if it's unable to fulfill the needed size.
    ///
    /// Returns the wrapped representation of where the encrypted form was written.
    ///
    /// This requires `Bytes: BorrowMut<[u8]>` because we want to allow the flexibility for that
    /// type to be either a borrowed reference or an owned value, and because we want that type to
    /// behave identically to `[u8]`, so that `Hash`, `Ord`, etc. are identical to `[u8]`, for the
    /// returned `EncryptedComponent` type.
    ///
    /// # Errors
    /// If the value returned by `get_dest` is `None` or is too small for the encrypted form.
    /// This can be used to determine what size is needed, by first giving a `get_dest` that just
    /// returns `None` (or `Some` empty slice) and using the returned
    /// [`DestTooSmallError::needed`] value.
    fn encrypt_component<Bytes: BorrowMut<[u8]>>(
        key: &<Self::Scheme as Scheme>::Key,
        component: &Component<impl Borrow<[u8]>>,
        get_dest: impl FnOnce(usize) -> Option<Bytes>,
    ) -> Result<EncryptedComponent<Bytes, Self::Scheme>, DestTooSmallError>;

    /// Return the size of the destination buffer that is needed for [`Self::encrypt_component`]
    /// to succeed with the same arguments.
    #[inline]
    fn size_needed_to_encrypt_component(
        key: &<Self::Scheme as Scheme>::Key,
        component: &Component<impl Borrow<[u8]>>,
    ) -> usize
    {
        match Self::encrypt_component::<&mut [u8]>(key, component, |_| None) {
            Ok(ec) => {
                debug_assert_eq!(ec.inner.len(), 0, "bad impl");
                0
            },
            Err(DestTooSmallError { needed }) => needed,
        }
    }

    /// Decrypts a single [`EncryptedComponent`] with the given `key`.  Output into where
    /// `get_dest` gives as its return value.
    ///
    /// `get_dest` is called with the exact size needed for the output, and it either returns
    /// `Some` value to use as the destination of where to write the decrypted output bytes, or it
    /// returns `None` if it's unable to fulfill the needed size.
    ///
    /// Returns the wrapped representation of where the decrypted form was written.
    ///
    /// This requires `Bytes: BorrowMut<[u8]>` because we want to allow the flexibility for that
    /// type to be either a borrowed reference or an owned value, and because we want that type to
    /// behave identically to `[u8]`, so that `Hash`, `Ord`, etc. are identical to `[u8]`, for the
    /// returned `Component` type.
    ///
    /// # Errors
    /// If the value returned by `get_dest` is `None` or is too small for the decrypted form.
    /// This can be used to determine what size is needed, by first giving a `get_dest` that just
    /// returns `None` (or `Some` empty slice) and using the returned
    /// [`DestTooSmallError::needed`] value.
    fn decrypt_component<Bytes: BorrowMut<[u8]>>(
        key: &<Self::Scheme as Scheme>::Key,
        component: &EncryptedComponent<impl Borrow<[u8]>, Self::Scheme>,
        get_dest: impl FnOnce(usize) -> Option<Bytes>,
    ) -> Result<Component<Bytes>, DestTooSmallError>;

    /// Return the size of the destination buffer that is needed for [`Self::decrypt_component`]
    /// to succeed with the same arguments.
    #[inline]
    fn size_needed_to_decrypt_component(
        key: &<Self::Scheme as Scheme>::Key,
        component: &EncryptedComponent<impl Borrow<[u8]>, Self::Scheme>,
    ) -> usize
    {
        match Self::decrypt_component::<&mut [u8]>(key, component, |_| None) {
            Ok(c) => {
                debug_assert_eq!(c.inner.len(), 0, "bad impl");
                0
            },
            Err(DestTooSmallError { needed }) => needed,
        }
    }
}

// TODO: impl std::error::Error for this, when `any(feature = "std", feature = "anticipate",
// rust_lib_feature = "error_in_core")`
/// Error possibly returned by the methods of [`Cryptor`].
#[derive(Copy, Clone, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct DestTooSmallError
{
    /// The size that the destination buffer, as returned by the `get_dest` argument, needed to
    /// be.
    pub needed: usize,
}


/// A specific algorithm for key derivation that must be non-invertible even for known
/// [`Component`]s.
pub trait KeyDerivationFunction
{
    /// The [`Scheme`] that `Self` is for.
    type Scheme: Scheme<KDF = Self>;

    /// Create a new key for [`Component`]s that follow `component`, when those will be encrypted
    /// with `key`, in [`Path`](sailce_data_model::Path)s.
    ///
    /// The new key value is placed in `*dest`.  This enables the caller to choose where to place
    /// it.
    fn derive(
        key: &<Self::Scheme as Scheme>::Key,
        component: &Component<impl Borrow<[u8]>>,
        dest: &mut <Self::Scheme as Scheme>::Key,
    );
}
