use {
    crate::{
        KeyDerivationFunction as _,
        Scheme,
    },
    core::borrow::BorrowMut,
    sailce_data_model::{
        path::Component,
        Path,
    },
};


pub(crate) fn crypt_components_and_save_keys<'l, S, I, R>(
    path: &'l (impl Path + ?Sized),
    key_0: &'l S::Key,
    keys_dest: impl IntoIterator<IntoIter = I>,
    mut crypt_component: impl (FnMut(&S::Key, &Component<&[u8]>) -> R) + 'l,
) -> impl ExactSizeIterator<Item = R> + 'l
where
    S: Scheme,
    I: ExactSizeIterator<Item = &'l mut S::Key> + 'l,
{
    let mut key_i = key_0;

    path.components().zip(keys_dest).map(move |(component_i, key_i_plus_1): (_, &'l mut _)| {
        let result = crypt_component(key_i, &component_i);
        S::KDF::derive(key_i, &component_i, key_i_plus_1);
        key_i = key_i_plus_1;
        result
    })
}


pub(crate) fn crypt_components_with_keys<'k, 'r, S, I, R>(
    path: &'r (impl Path + ?Sized),
    keys: impl IntoIterator<IntoIter = I>,
    mut crypt_component: impl (FnMut(&S::Key, &Component<&[u8]>) -> R) + 'r,
) -> impl ExactSizeIterator<Item = R> + 'r
where
    S: Scheme,
    I: ExactSizeIterator<Item = &'k S::Key> + 'r,
{
    /* For some strange reason, having this slightly-different equivalent impl:
         path.components().zip(keys).map(move |(component_i, key_i)| { ... })
       would cause this error:
         hidden type
         `core::iter::Map<
              core::iter::Zip<
                  impl core::iter::ExactSizeIterator
                     + core::iter::Iterator<Item = Component<&'r [u8]>>,
                  I>,
              {closure}>`
         captures the lifetime `'k`

       But that same form of impl does work for EncryptPath::encrypt_components_with_keys.
       If this is a bug in rustc that gets fixed in the future, then change to that form.
    */
    let do_each = move |(component_i, key_i): (_, &_)| crypt_component(key_i, &component_i);

    path.components().zip(keys).map(do_each)
}


pub(crate) fn crypt_components_with_key_space<'l, S: Scheme, R>(
    path: &'l (impl Path + ?Sized),
    key_0: &'l S::Key,
    key_space: [impl BorrowMut<S::Key> + 'l; 2],
    mut crypt_component: impl (FnMut(&S::Key, &Component<&[u8]>) -> R) + 'l,
) -> impl ExactSizeIterator<Item = R> + 'l
{
    let mut key_0 = Some(key_0);
    let [mut key_a, mut key_b] = key_space;
    let mut alt = false;

    path.components().map(move |component_i| {
        let [key_a, key_b] = [key_a.borrow_mut(), key_b.borrow_mut()];
        // Use `key_0` the first time, but `key_a` or `key_b` thereafter.
        let (key_i, key_i_plus_1): (&_, &mut _) = match key_0.take() {
            Some(key_0) => {
                debug_assert!(!alt, "was initialized to `false`");
                (key_0, key_a)
            },
            None =>
                if alt {
                    (key_a, key_b)
                }
                else {
                    (key_b, key_a)
                },
        };
        let result = crypt_component(key_i, &component_i);
        // Reuse the space of the unused alternate `key_i_plus_1` for the next derived key and use
        // that as `key_i` next time, and reuse the space of now-unused `key_i` as `key_i_plus_1`
        // next time.  Two `Key` values are needed for the space, because `derive` needs to borrow
        // two keys simultaneously with one as `mut`, and so a single `Key` value wouldn't work.
        S::KDF::derive(key_i, &component_i, key_i_plus_1);
        alt = !alt;
        result
    })
}
