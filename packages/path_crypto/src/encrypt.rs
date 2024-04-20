use {
    crate::{
        crypt::{
            crypt_components_and_save_keys,
            crypt_components_with_key_space,
            crypt_components_with_keys,
        },
        Cryptor as _,
        DestTooSmallError,
        EncryptedComponent,
        EncryptedPath,
        KeyDerivationFunction as _,
        Scheme,
    },
    core::borrow::BorrowMut,
    sailce_data_model::Path,
};


impl<T> EncryptPath for T where T: Path + ?Sized {}

/// Methods to encrypt [`Path`]s, for all types that implement `Path`.
///
/// Most of these methods deal with `Key` values only by reference, because this requires the
/// caller to choose the security implications of where to place keys in memory and this avoids
/// these methods needing to move/copy/clone sensitive key material around in other memory
/// locations (e.g. the stack where it might later be exposed); and because this avoids needing
/// dynamic heap allocations, which supports `no_std` usage.
///
/// However, a couple of the methods do internally place `Key` values on the stack, in closures,
/// and/or in iterators, to provide methods that can be more convenient.  Those should only be
/// used when the security of the key material is not negatively affected by this, e.g. when the
/// `Key` type only points to key material that is placed somewhere else securely outside of the
/// `Key` values and so moving/copying/cloning the values only moves/copies the pointers and not
/// key material.
///
/// This trait should not be implemented for other types (and probably cannot ever be, due to our
/// blanket implementation).
///
/// These methods are sometimes useful for situations and so are provided by this crate, but these
/// are not the only possibilities.  You may instead use [`Cryptor`](crate::Cryptor) and
/// [`KeyDerivationFunction`](crate::KeyDerivationFunction) types directly and do it differently.
pub trait EncryptPath: Path
{
    /// Assign, to each dereferenced item of `keys_dest`, the respective key from the sequence
    /// `key_1, ..., key_N`, where `N` is `self.components().len()` and `L` is
    /// `keys_dest.into_iter().len()`, stopping early at `key_L` if `L < N`.
    ///
    /// The key-derivation sequence is described by [`Self::encrypt_components`].
    ///
    /// It's unnecessary to assign `key_0`, because it's already known.  Even though there isn't a
    /// `component_N` (due to zero-based indexing, `component_N-1` is the last), this does assign
    /// `key_N` (if `L >= N`), because it's sometimes needed when crypting further components
    /// prefixed by those of `self`.  If you don't want `key_N` then give `L = N - 1`, and,
    /// similarly, if you don't want `key_i, ..., key_N` then give `L = i - 1`.
    ///
    /// Return the amount of elements assigned, i.e. `min(N, L)`.
    ///
    /// Placing the created keys in each dereferenced item enables the caller to choose where to
    /// place them.
    #[inline]
    #[allow(single_use_lifetimes)] // Silence false positive.
    fn derive_keys<'k, S: Scheme>(
        &self,
        key_0: &S::Key,
        keys_dest: impl IntoIterator<Item = &'k mut S::Key>,
    ) -> usize
    {
        self.derive_keys_and_size_needed::<S>(key_0, keys_dest, false).0
    }

    /// Like [`Self::derive_keys`] but also optionally returns the size of a single byte-buffer
    /// needed to hold `min(N, L)` amount of `self`'s components as encrypted, i.e. the minimum
    /// size needed for a backing slice given to make a `get_dest` closure, such as via
    /// [`get_dest::from_slice`](crate::get_dest::from_slice), given to
    /// [`Self::encrypt_components`] (et al) to succeed with the same key-derivation sequence.
    ///
    /// It sometimes makes sense to compute this size along with deriving the keys, because
    /// computing the size requires each key of the sequence.
    #[inline]
    #[allow(single_use_lifetimes)] // Silence false positive.
    fn derive_keys_and_size_needed<'k, S: Scheme>(
        &self,
        key_0: &S::Key,
        keys_dest: impl IntoIterator<Item = &'k mut S::Key>,
        want_size_needed: bool,
    ) -> (usize, Option<usize>)
    {
        let (mut i, mut key_i, mut size) = (0_usize, key_0, want_size_needed.then_some(0_usize));

        for (component_i, key_i_plus_1) in self.components().zip(keys_dest) {
            if let Some(size) = &mut size {
                *size = size.saturating_add(S::Cryptor::size_needed_to_encrypt_component(
                    key_i,
                    &component_i,
                ));
            }

            S::KDF::derive(key_i, &component_i, key_i_plus_1);
            key_i = key_i_plus_1;

            match i.checked_add(1) {
                Some(i_plus_1) => i = i_plus_1,
                None => break, // Stop at `usize::MAX`, if they're somehow longer than that.
            }
        }
        (i, size)
    }

    /// Given an initial key `key_0`, encrypt a `Path` as follows:
    /// - Encrypting the empty `Path` yields the empty `Path` again.
    /// - Encrypt a `Path` with a single `Component` `component_0` as `encrypt_component(key_0,
    ///   component_0)`.
    /// - For any other (longer) `Path`, denote its final `Component` as `component_n`, the `Path`
    ///   formed by its prior `Component`s as `path_p`, the final component of `path_p` as
    ///   `component_p`, and denote the key that is used to encrypt `component_p` as `key_p`. Then
    ///   encrypt the `Path` by first encrypting `path_p`, and then appending
    ///   `encrypt_component(kdf(key_p, component_p), component_n)`.
    ///
    /// `get_dest` provides the buffers needed for the encrypted output values.  The returned
    /// iterator yields the wrapped representation of the buffers filled with the encrypted
    /// outputs.  If `get_dest` provides a buffer that is too small for an encrypted output, the
    /// iterator will yield `Err` for that component but will still return
    /// `self.components().len()` amount of items, and later components might be `Ok`.  The `B`
    /// type of these buffers can be either a borrowed reference or an owned value, which allows
    /// flexibility.
    ///
    /// The needed sequence of keys is automatically derived.  This can be more efficient than
    /// calling [`Self::derive_keys`] and using those results for calling
    /// [`Self::encrypt_components_with_keys`], because that would involve two passes over
    /// `self.components()` whereas this involves only one.
    ///
    /// This requires that the `Key` type implements `Default`, and also that it's `Sized`, just
    /// to initialize internal temporary space to "zero" values, and these "zero" values are
    /// ignored and so the `default` can give arbitrary values (that should not be real keys).
    ///
    /// **Note**: This method internally places the derived `Key` values in the returned iterator
    /// which usually is placed on the stack and maybe moved around arbitrarily, and so this
    /// should only be used when the security of the key material is not negatively affected by
    /// this (e.g. when the `Key` type only points to key material that is placed somewhere else
    /// securely outside of the `Key` values).
    #[inline]
    fn encrypt_components<'l, S, B>(
        &'l self,
        key_0: &'l S::Key,
        get_dest: impl FnMut(usize) -> Option<B> + 'l,
    ) -> impl ExactSizeIterator<Item = Result<EncryptedComponent<B, S>, DestTooSmallError>> + 'l
    where
        S: Scheme,
        S::Key: Default,
        B: BorrowMut<[u8]>,
    {
        self.encrypt_components_with_key_space(
            key_0,
            [S::Key::default(), S::Key::default()],
            get_dest,
        )
    }

    /// Like [`Self::encrypt_components`] but the automatically derived `key_i`, including the
    /// extra `key_N`, are also assigned to each dereferenced item of `keys_dest`, like
    /// [`Self::derive_keys`], where `i >= 1` (i.e. an assignment for `key_0` is not done).
    ///
    /// As such, this method doesn't internally hold `Key` values and so that concern of
    /// `Self::encrypt_components` doesn't apply, and the `Key` type doesn't need to implement
    /// `Default`.
    ///
    /// If `keys_dest.into_iter().len() < self.components().len()`, only that lesser amount of
    /// components are yielded by the returned iterator.  If `keys_dest` has more elements than
    /// `self` has, the extra are ignored.
    #[inline]
    fn encrypt_components_and_save_keys<'l, S, I, B>(
        &'l self,
        key_0: &'l S::Key,
        keys_dest: impl IntoIterator<IntoIter = I>,
        mut get_dest: impl FnMut(usize) -> Option<B> + 'l,
    ) -> impl ExactSizeIterator<Item = Result<EncryptedComponent<B, S>, DestTooSmallError>> + 'l
    where
        S: Scheme,
        I: ExactSizeIterator<Item = &'l mut S::Key> + 'l,
        B: BorrowMut<[u8]>,
    {
        crypt_components_and_save_keys::<S, _, _>(
            self,
            key_0,
            keys_dest,
            move |key_i, component_i| {
                S::Cryptor::encrypt_component(key_i, component_i, &mut get_dest)
            },
        )
    }

    /// Like [`Self::encrypt_components`] but the `key_i` for each `Component` are provided by
    /// `keys`.
    ///
    /// The output values of [`Self::derive_keys`] (or equivalent), and, first, the `key_0` which
    /// was given to that, should be used as the items of `keys`.  If the given `keys` aren't a
    /// proper derivation sequence for `self.components()` for `S as Scheme`, the encrypted
    /// results will not be usable by typical other things that expect such.
    ///
    /// If `keys` yields less items than `self` has (i.e. `self.components().len()`), only that
    /// lesser amount of components are yielded by the returned iterator.  If `keys` yields more
    /// items than self has, the extra are ignored.
    #[inline]
    fn encrypt_components_with_keys<'k, 'r, S, I, B>(
        &'r self,
        keys: impl IntoIterator<IntoIter = I>,
        mut get_dest: impl FnMut(usize) -> Option<B> + 'r,
    ) -> impl ExactSizeIterator<Item = Result<EncryptedComponent<B, S>, DestTooSmallError>> + 'r
    where
        S: Scheme,
        I: ExactSizeIterator<Item = &'k S::Key> + 'r,
        B: BorrowMut<[u8]>,
    {
        crypt_components_with_keys::<S, _, _>(self, keys, move |key_i, component_i| {
            S::Cryptor::encrypt_component(key_i, component_i, &mut get_dest)
        })
    }

    /// Like [`Self::encrypt_components`] but with temporary space for the derived keys provided
    /// by `key_space`.  The values of `key_space` afterwards should not be relied on.
    #[inline]
    fn encrypt_components_with_key_space<'l, S: Scheme, B: BorrowMut<[u8]>>(
        &'l self,
        key_0: &'l S::Key,
        key_space: [impl BorrowMut<S::Key> + 'l; 2],
        mut get_dest: impl FnMut(usize) -> Option<B> + 'l,
    ) -> impl ExactSizeIterator<Item = Result<EncryptedComponent<B, S>, DestTooSmallError>> + 'l
    {
        crypt_components_with_key_space::<S, _>(
            self,
            key_0,
            key_space,
            move |key_i, component_i| {
                S::Cryptor::encrypt_component(key_i, component_i, &mut get_dest)
            },
        )
    }

    /// Like [`Self::encrypt_components`] but a new [`Path`] is created from the encrypted output
    /// values.
    ///
    /// **Note**: The same concern applies as noted by [`Self::encrypt_components`].
    ///
    /// # Errors
    /// If encrypting any `Component` can't fit in the buffer returned for it by `get_dest`.
    #[inline]
    fn encrypt<S, B, P>(
        &self,
        key_0: &S::Key,
        get_dest: impl FnMut(usize) -> Option<B>,
    ) -> Result<EncryptedPath<P, S>, DestTooSmallError>
    where
        S: Scheme,
        S::Key: Default,
        B: BorrowMut<[u8]>,
        P: Path + FromIterator<B>,
    {
        Ok(EncryptedPath::new(
            self.encrypt_components::<S, _>(key_0, get_dest)
                .map(|result| result.map(|encrypted_component| encrypted_component.inner))
                .collect::<Result<P, _>>()?,
        ))
    }

    /// Like [`Self::encrypt_components_and_save_keys`] but a new [`Path`] is created from the
    /// encrypted output values.
    ///
    /// # Errors
    /// If encrypting any `Component` can't fit in the buffer returned for it by `get_dest`.
    #[inline]
    fn encrypt_and_save_keys<'l, S, I, B, P>(
        &'l self,
        key_0: &'l S::Key,
        keys_dest: impl IntoIterator<IntoIter = I>,
        get_dest: impl FnMut(usize) -> Option<B> + 'l,
    ) -> Result<EncryptedPath<P, S>, DestTooSmallError>
    where
        S: Scheme,
        I: ExactSizeIterator<Item = &'l mut S::Key> + 'l,
        B: BorrowMut<[u8]>,
        P: Path + FromIterator<B>,
    {
        Ok(EncryptedPath::new(
            self.encrypt_components_and_save_keys::<S, _, _>(key_0, keys_dest, get_dest)
                .map(|result| result.map(|encrypted_component| encrypted_component.inner))
                .collect::<Result<P, _>>()?,
        ))
    }

    /// Like [`Self::encrypt_components_with_keys`] but a new [`Path`] is created from the
    /// encrypted output values.
    ///
    /// # Errors
    /// If encrypting any `Component` can't fit in the buffer returned for it by `get_dest`.
    #[inline]
    fn encrypt_with_keys<'k, S, I, B, P>(
        &self,
        keys: impl IntoIterator<IntoIter = I>,
        get_dest: impl FnMut(usize) -> Option<B>,
    ) -> Result<EncryptedPath<P, S>, DestTooSmallError>
    where
        S: Scheme,
        I: ExactSizeIterator<Item = &'k S::Key>,
        B: BorrowMut<[u8]>,
        P: Path + FromIterator<B>,
    {
        Ok(EncryptedPath::new(
            self.encrypt_components_with_keys::<S, _, _>(keys, get_dest)
                .map(|result| result.map(|encrypted_component| encrypted_component.inner))
                .collect::<Result<P, _>>()?,
        ))
    }

    /// Like [`Self::encrypt_components_with_key_space`] but a new [`Path`] is created from the
    /// encrypted output values.
    ///
    /// # Errors
    /// If encrypting any `Component` can't fit in the buffer returned for it by `get_dest`.
    #[inline]
    fn encrypt_with_key_space<S, B, P>(
        &self,
        key_0: &S::Key,
        key_space: [impl BorrowMut<S::Key>; 2],
        get_dest: impl FnMut(usize) -> Option<B>,
    ) -> Result<EncryptedPath<P, S>, DestTooSmallError>
    where
        S: Scheme,
        B: BorrowMut<[u8]>,
        P: Path + FromIterator<B>,
    {
        Ok(EncryptedPath::new(
            self.encrypt_components_with_key_space::<S, _>(key_0, key_space, get_dest)
                .map(|result| result.map(|encrypted_component| encrypted_component.inner))
                .collect::<Result<P, _>>()?,
        ))
    }
}
