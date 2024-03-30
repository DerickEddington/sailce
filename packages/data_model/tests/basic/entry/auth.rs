use {
    crate::nz_usize,
    sailce_data_model::{
        AuthorisedEntry,
        Entry,
        Params,
        ParamsEntry,
        Path,
        Payload,
    },
    std::num::NonZeroUsize,
};

#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum Never {}


struct MockParams;

impl Params for MockParams
{
    type AuthorisationToken = bool;
    type HashPayloadError<P: Payload + ?Sized> = Never;
    type NamespaceId = u128;
    type PayloadDigest = [u8; 64];
    type SubspaceId = &'static str;

    const MAX_COMPONENT_COUNT: NonZeroUsize = nz_usize(64);
    const MAX_COMPONENT_LENGTH: NonZeroUsize = nz_usize(256);
    const MAX_PATH_LENGTH: NonZeroUsize = nz_usize(4 * 1024);

    #[allow(clippy::let_underscore_untyped)] // False-positive bug in Clippy.
    async fn hash_payload<P: Payload + ?Sized>(
        _: &mut P
    ) -> Result<Self::PayloadDigest, Self::HashPayloadError<P>>
    {
        #![allow(let_underscore_drop, clippy::unimplemented)]
        unimplemented!()
    }

    fn is_authorised_write(
        _entry: &ParamsEntry<Self, impl Path>,
        auth_token: &Self::AuthorisationToken,
    ) -> bool
    {
        *auth_token
    }
}


#[test]
fn new()
{
    let entry = Entry {
        namespace_id:   54321,
        subspace_id:    "blah",
        path:           ["foo", "bar"],
        timestamp:      0.into(),
        payload_digest: [0; 64],
        payload_length: 0,
    };

    assert!(AuthorisedEntry::<MockParams, _, _>::new(entry, false).is_none());
    let auth_entry = AuthorisedEntry::<MockParams, _, _>::new(entry, true);
    assert!(auth_entry.is_some());
    let auth_entry = auth_entry.unwrap();
    assert_eq!(auth_entry.auth_token(), &true);
    assert_eq!(auth_entry.entry(), &entry);
}
