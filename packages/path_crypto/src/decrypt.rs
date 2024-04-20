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
        MakeEncryptedComponent as _,
        Scheme,
    },
    core::borrow::{
        Borrow,
        BorrowMut,
    },
    sailce_data_model::{
        path::Component,
        Path,
    },
};


impl<P, S> EncryptedPath<P, S>
where
    P: Path,
    S: Scheme,
{
    /// The inverse of [`EncryptPath::encrypt_components`](
    /// crate::EncryptPath::encrypt_components).  The keys are automatically derived the same as
    /// for `EncryptPath::encrypt_components` (not as some kind of inverse).
    ///
    /// **Note**: The same concern applies as noted by [`EncryptPath::encrypt_components`](
    /// crate::EncryptPath::encrypt_components).
    #[inline]
    pub fn decrypt_components<'l, B>(
        &'l self,
        key_0: &'l S::Key,
        get_dest: impl FnMut(usize) -> Option<B> + 'l,
    ) -> impl ExactSizeIterator<Item = Result<Component<B>, DestTooSmallError>> + 'l
    where
        S::Key: Default,
        B: BorrowMut<[u8]>,
    {
        self.decrypt_components_with_key_space(
            key_0,
            [S::Key::default(), S::Key::default()],
            get_dest,
        )
    }

    /// Like [`Self::decrypt_components`] but the automatically derived `key_i`, including the
    /// extra `key_N`, are also assigned to each dereferenced item of `keys_dest`, like
    /// [`EncryptPath::derive_keys`](crate::EncryptPath::derive_keys), where `i >= 1` (i.e. an
    /// assignment for `key_0` is not done).
    ///
    /// As such, this method doesn't internally hold `Key` values and so that concern of
    /// `Self::decrypt_components` doesn't apply, and the `Key` type doesn't need to implement
    /// `Default`.
    ///
    /// If `keys_dest.into_iter().len() < self.components().len()`, only that lesser amount of
    /// components are yielded by the returned iterator.  If `keys_dest` has more elements than
    /// `self` has, the extra are ignored.
    #[inline]
    pub fn decrypt_components_and_save_keys<'l, I, B>(
        &'l self,
        key_0: &'l S::Key,
        keys_dest: impl IntoIterator<IntoIter = I>,
        mut get_dest: impl FnMut(usize) -> Option<B> + 'l,
    ) -> impl ExactSizeIterator<Item = Result<Component<B>, DestTooSmallError>> + 'l
    where
        I: ExactSizeIterator<Item = &'l mut S::Key> + 'l,
        B: BorrowMut<[u8]>,
    {
        crypt_components_and_save_keys::<S, _, _>(
            &self.path,
            key_0,
            keys_dest,
            move |key_i, component_i| {
                S::Cryptor::decrypt_component(
                    key_i,
                    &synthesize_encrypted_component(*component_i),
                    &mut get_dest,
                )
            },
        )
    }

    /// Like [`Self::decrypt_components`] but the `key_i` for each `Component` are provided by
    /// `keys`.
    ///
    /// The output values of [`EncryptPath::derive_keys`](crate::EncryptPath::derive_keys) (or
    /// equivalent), and, first, the `key_0` which was given to that, should be used as the
    /// items of `keys`.  If the given `keys` aren't a proper derivation sequence matching
    /// `self.path.components()` for `S as Scheme`, the decrypted results will not be usable
    /// by typical other things that expect such.
    ///
    /// If `keys` yields less items than `self` has (i.e. `self.components().len()`), only that
    /// lesser amount of components are yielded by the returned iterator.  If `keys` yields more
    /// items than self has, the extra are ignored.
    #[inline]
    pub fn decrypt_components_with_keys<'k, 'r, I, B>(
        &'r self,
        keys: impl IntoIterator<IntoIter = I>,
        mut get_dest: impl FnMut(usize) -> Option<B> + 'r,
    ) -> impl ExactSizeIterator<Item = Result<Component<B>, DestTooSmallError>> + 'r
    where
        I: ExactSizeIterator<Item = &'k S::Key> + 'r,
        B: BorrowMut<[u8]>,
    {
        crypt_components_with_keys::<S, _, _>(&self.path, keys, move |key_i, component_i| {
            S::Cryptor::decrypt_component(
                key_i,
                &synthesize_encrypted_component(*component_i),
                &mut get_dest,
            )
        })
    }

    /// Like [`Self::decrypt_components`] but with temporary space for the derived keys provided
    /// by `key_space`.  The values of `key_space` afterwards should not be relied on.
    #[inline]
    pub fn decrypt_components_with_key_space<'l, B: BorrowMut<[u8]>>(
        &'l self,
        key_0: &'l S::Key,
        key_space: [impl BorrowMut<S::Key> + 'l; 2],
        mut get_dest: impl FnMut(usize) -> Option<B> + 'l,
    ) -> impl ExactSizeIterator<Item = Result<Component<B>, DestTooSmallError>> + 'l
    {
        crypt_components_with_key_space::<S, _>(
            &self.path,
            key_0,
            key_space,
            move |key_i, component_i| {
                S::Cryptor::decrypt_component(
                    key_i,
                    &synthesize_encrypted_component(*component_i),
                    &mut get_dest,
                )
            },
        )
    }

    // TODO: decrypt() et al that create new Path objects
}


fn synthesize_encrypted_component<B: Borrow<[u8]>, S: Scheme>(
    component: Component<B>
) -> EncryptedComponent<B, S>
{
    let bytes_of_encrypted = component.inner;
    S::synthesize_encrypted_component(bytes_of_encrypted)
}
