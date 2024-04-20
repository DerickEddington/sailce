// TODO: move all this into encrypt.rs?

use {
    crate::Scheme,
    core::marker::PhantomData,
    sailce_data_model::Path,
};


/// An encrypted [`Path`](sailce_data_model::Path).
///
/// The `Scheme` type parameter ensures that a concrete type of `EncryptedPath` is bound to a
/// specific [`Scheme`](crate::Scheme) implementation, so that values of the type cannot be
/// mixed-up with values of other types of other `Scheme`s.
///
/// The private field prevents directly constructing values of this type, to ensure that creating
/// values is done properly by this crate's API.
#[derive(Debug)]
#[allow(clippy::partial_pub_fields, clippy::exhaustive_structs)]
pub struct EncryptedPath<Path, Scheme>
{
    /// The encrypted form of the path.  Intended to be used where it's needed to supply the
    /// encrypted form as a plain [`Path`] to other parts that transparently handle `Path`s
    /// without awareness of whether they're encrypted.
    pub path: Path,
    _scheme:  PhantomData<Scheme>,
}

impl<P: Path, S: Scheme> EncryptedPath<P, S>
{
    pub(crate) fn new(path: P) -> Self
    {
        Self { path, _scheme: PhantomData }
    }
}
