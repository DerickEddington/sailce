use crate::{
    Entry,
    ParamsEntry,
    Path,
};


/// A `PossiblyAuthorisedEntry` is a pair of an [`Entry`] and an
/// [`AuthorisationToken`](crate::Params::AuthorisationToken).  An `AuthorisedEntry` is a
/// `PossiblyAuthorisedEntry` for which
/// [`is_authorised_write`](crate::Params::is_authorised_write) returns `true`.
///
/// The fields are private, to prevent constructing arbitrary values that might not uphold the
/// requirement.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct AuthorisedEntry<Entry, AuthorisationToken>
{
    entry:      Entry,
    auth_token: AuthorisationToken,
}


impl<N, S, P, D, A> AuthorisedEntry<Entry<N, S, P, D>, A>
where P: Path
{
    /// Ensure by construction that every `AuthorisedEntry` upholds the requirement that its
    /// [`AuthorisationToken`](crate::Params::AuthorisationToken) authorises its [`Entry`]
    /// according to the given [`crate::Params::is_authorised_write`].
    ///
    /// If this requirement is not upheld, then `None` is returned.
    #[inline]
    pub fn new<Params>(
        entry: Entry<N, S, P, D>,
        auth_token: A,
    ) -> Option<Self>
    where
        Params: crate::Params<
                NamespaceId = N,
                SubspaceId = S,
                PayloadDigest = D,
                AuthorisationToken = A,
            > + ?Sized,
    {
        Params::is_authorised_write(&entry, &auth_token).then_some(Self { entry, auth_token })
    }

    /// The [`Entry`] that was authorised.
    #[inline]
    pub fn entry(&self) -> &Entry<N, S, P, D>
    {
        &self.entry
    }

    /// The [`AuthorisationToken`](crate::Params::AuthorisationToken) that authorises the Entry.
    #[inline]
    pub fn auth_token(&self) -> &A
    {
        &self.auth_token
    }
}


/// Same as [`AuthorisedEntry`] with type arguments from the given [`Params`](crate::Params).
pub type ParamsAuthorisedEntry<Params, Path> =
    AuthorisedEntry<ParamsEntry<Params, Path>, <Params as crate::Params>::AuthorisationToken>;
