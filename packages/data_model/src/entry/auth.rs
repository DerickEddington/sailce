use {
    crate::ParamsEntry,
    core::{
        borrow::Borrow,
        cmp::Ordering,
        fmt::{
            self,
            Debug,
            Formatter,
        },
        hash::{
            Hash,
            Hasher,
        },
    },
};


/// A `PossiblyAuthorisedEntry` is a pair of an [`Entry`](crate::Entry) and an
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
pub struct AuthorisedEntry<Params, Path, AuthToken>
where
    Params: crate::Params + ?Sized,
    AuthToken: Borrow<Params::AuthorisationToken>,
{
    entry:      ParamsEntry<Params, Path>,
    auth_token: AuthToken,
}


impl<Params, Path, AuthToken> AuthorisedEntry<Params, Path, AuthToken>
where
    Params: crate::Params + ?Sized,
    Path: crate::Path,
    AuthToken: Borrow<Params::AuthorisationToken>,
{
    /// Ensure by construction that every `AuthorisedEntry` upholds the requirement that its
    /// [`AuthorisationToken`](crate::Params::AuthorisationToken) authorises its
    /// [`Entry`](crate::Entry) according to the given [`Params::is_authorised_write`](
    /// crate::Params::is_authorised_write).
    ///
    /// If this requirement is not upheld, then `None` is returned.
    #[inline]
    pub fn new(
        entry: ParamsEntry<Params, Path>,
        auth_token: AuthToken,
    ) -> Option<Self>
    {
        Params::is_authorised_write(&entry, auth_token.borrow())
            .then_some(Self { entry, auth_token })
    }

    /// The [`Entry`](crate::Entry) that was authorised.
    #[inline]
    pub fn entry(&self) -> &ParamsEntry<Params, Path>
    {
        &self.entry
    }

    /// The [`AuthorisationToken`](crate::Params::AuthorisationToken) that authorises the `Entry`.
    #[inline]
    pub fn auth_token(&self) -> &Params::AuthorisationToken
    {
        self.auth_token.borrow()
    }

    /// Decompose `self` into its [`Entry`](crate::Entry) and [`AuthorisationToken`](
    /// crate::Params::AuthorisationToken).
    ///
    /// This cannot lead to violations of the `is_authorised_write` requirement, because anything
    /// which requires that must use the `AuthorisedEntry` type and not the components separately.
    #[inline]
    pub fn into_parts(self) -> (ParamsEntry<Params, Path>, AuthToken)
    {
        (self.entry, self.auth_token)
    }
}


// The following can't be `derive`d, because of the `Params` type parameter that is necessary.

impl<Params, Path, AuthToken> Debug for AuthorisedEntry<Params, Path, AuthToken>
where
    Params: crate::Params + ?Sized,
    AuthToken: Borrow<Params::AuthorisationToken> + Debug,
    ParamsEntry<Params, Path>: Debug,
{
    #[inline]
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> fmt::Result
    {
        f.debug_struct("AuthorisedEntry")
            .field("entry", &self.entry)
            .field("auth_token", &self.auth_token)
            .finish()
    }
}

impl<Params, Path, AuthToken> Clone for AuthorisedEntry<Params, Path, AuthToken>
where
    Params: crate::Params + ?Sized,
    AuthToken: Borrow<Params::AuthorisationToken> + Clone,
    ParamsEntry<Params, Path>: Clone,
{
    #[inline]
    fn clone(&self) -> Self
    {
        Self { entry: self.entry.clone(), auth_token: self.auth_token.clone() }
    }
}

impl<Params, Path, AuthToken> Copy for AuthorisedEntry<Params, Path, AuthToken>
where
    Params: crate::Params + ?Sized,
    AuthToken: Borrow<Params::AuthorisationToken> + Copy,
    ParamsEntry<Params, Path>: Copy,
{
}

impl<Params, Path, AuthToken> PartialEq for AuthorisedEntry<Params, Path, AuthToken>
where
    Params: crate::Params + ?Sized,
    AuthToken: Borrow<Params::AuthorisationToken>,
    ParamsEntry<Params, Path>: PartialEq,
    Params::AuthorisationToken: PartialEq,
{
    #[inline]
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        self.entry == other.entry && self.auth_token.borrow() == other.auth_token.borrow()
    }
}

impl<Params, Path, AuthToken> Eq for AuthorisedEntry<Params, Path, AuthToken>
where
    Params: crate::Params + ?Sized,
    AuthToken: Borrow<Params::AuthorisationToken>,
    ParamsEntry<Params, Path>: Eq,
    Params::AuthorisationToken: Eq,
{
}

impl<Params, Path, AuthToken> Ord for AuthorisedEntry<Params, Path, AuthToken>
where
    Params: crate::Params + ?Sized,
    AuthToken: Borrow<Params::AuthorisationToken>,
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
            Ordering::Equal => self.auth_token.borrow().cmp(other.auth_token.borrow()),
            Ordering::Greater => Ordering::Greater,
        }
    }
}

impl<Params, Path, AuthToken> PartialOrd for AuthorisedEntry<Params, Path, AuthToken>
where
    Params: crate::Params + ?Sized,
    AuthToken: Borrow<Params::AuthorisationToken>,
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

impl<Params, Path, AuthToken> Hash for AuthorisedEntry<Params, Path, AuthToken>
where
    Params: crate::Params + ?Sized,
    AuthToken: Borrow<Params::AuthorisationToken>,
    ParamsEntry<Params, Path>: Hash,
    Params::AuthorisationToken: Hash,
{
    #[inline]
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    )
    {
        self.entry.hash(state);
        self.auth_token.borrow().hash(state);
    }
}
