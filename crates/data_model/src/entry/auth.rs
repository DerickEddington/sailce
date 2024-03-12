use {
    crate::ParamsEntry,
    core::{
        cmp::Ordering,
        fmt::{
            self,
            Debug,
            Formatter,
        },
    },
};


/// A `PossiblyAuthorisedEntry` is a pair of an [`Entry`] and an
/// [`AuthorisationToken`](crate::Params::AuthorisationToken).  An `AuthorisedEntry` is a
/// `PossiblyAuthorisedEntry` for which
/// [`Params::is_authorised_write`](crate::Params::is_authorised_write) returns `true`.
///
/// The `Params` type parameter ensures that a concrete type of `AuthorisedEntry` is bound to a
/// specific `is_authorised_write` implementation along with the other items of a
/// [`Params`](crate::Params), so that values of the type can only be constructed when the
/// specific implementation allows.
///
/// The fields are private, to prevent constructing arbitrary values that might not uphold the
/// requirement.
pub struct AuthorisedEntry<Params, Path>
where Params: crate::Params + ?Sized
{
    entry:      ParamsEntry<Params, Path>,
    auth_token: Params::AuthorisationToken,
}


impl<Params, Path> AuthorisedEntry<Params, Path>
where
    Params: crate::Params + ?Sized,
    Path: crate::Path,
{
    /// Ensure by construction that every `AuthorisedEntry` upholds the requirement that its
    /// [`AuthorisationToken`](crate::Params::AuthorisationToken) authorises its [`Entry`]
    /// according to the given [`Params::is_authorised_write`].
    ///
    /// If this requirement is not upheld, then `None` is returned.
    #[inline]
    pub fn new(
        entry: ParamsEntry<Params, Path>,
        auth_token: Params::AuthorisationToken,
    ) -> Option<Self>
    {
        Params::is_authorised_write(&entry, &auth_token).then_some(Self { entry, auth_token })
    }

    /// The [`Entry`] that was authorised.
    #[inline]
    pub fn entry(&self) -> &ParamsEntry<Params, Path>
    {
        &self.entry
    }

    /// The [`AuthorisationToken`](crate::Params::AuthorisationToken) that authorises the `Entry`.
    #[inline]
    pub fn auth_token(&self) -> &Params::AuthorisationToken
    {
        &self.auth_token
    }
}


// The following can't be `derive`d, because of the `Params` type parameter that is necessary.

impl<Params, Path> Debug for AuthorisedEntry<Params, Path>
where
    Params: crate::Params + ?Sized,
    ParamsEntry<Params, Path>: Debug,
    Params::AuthorisationToken: Debug,
{
    #[inline]
    fn fmt(
        &self,
        fmt: &mut Formatter<'_>,
    ) -> fmt::Result
    {
        fmt.debug_struct("AuthorisedEntry")
            .field("entry", &self.entry)
            .field("auth_token", &self.auth_token)
            .finish()
    }
}

impl<Params, Path> Clone for AuthorisedEntry<Params, Path>
where
    Params: crate::Params + ?Sized,
    ParamsEntry<Params, Path>: Clone,
    Params::AuthorisationToken: Clone,
{
    #[inline]
    fn clone(&self) -> Self
    {
        Self { entry: self.entry.clone(), auth_token: self.auth_token.clone() }
    }
}

impl<Params, Path> Copy for AuthorisedEntry<Params, Path>
where
    Params: crate::Params + ?Sized,
    ParamsEntry<Params, Path>: Copy,
    Params::AuthorisationToken: Copy,
{
}

impl<Params, Path> PartialEq for AuthorisedEntry<Params, Path>
where
    Params: crate::Params + ?Sized,
    ParamsEntry<Params, Path>: PartialEq,
    Params::AuthorisationToken: PartialEq,
{
    #[inline]
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        self.entry == other.entry && self.auth_token == other.auth_token
    }
}

impl<Params, Path> Eq for AuthorisedEntry<Params, Path>
where
    Params: crate::Params + ?Sized,
    ParamsEntry<Params, Path>: Eq,
    Params::AuthorisationToken: Eq,
{
}

impl<Params, Path> Ord for AuthorisedEntry<Params, Path>
where
    Params: crate::Params + ?Sized,
    ParamsEntry<Params, Path>: Ord,
    Params::AuthorisationToken: Ord,
{
    #[inline]
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering
    {
        match self.entry.cmp(&other.entry) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.auth_token.cmp(&other.auth_token),
            Ordering::Greater => Ordering::Greater,
        }
    }
}

impl<Params, Path> PartialOrd for AuthorisedEntry<Params, Path>
where
    Params: crate::Params + ?Sized,
    ParamsEntry<Params, Path>: Ord,
    Params::AuthorisationToken: Ord,
{
    #[inline]
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering>
    {
        Some(self.cmp(other))
    }
}
