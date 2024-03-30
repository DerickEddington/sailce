use {
    crate::nz_usize,
    sailce_data_model::{
        group::Range,
        payload::{
            CopyToSliceError,
            ExtraCore as _,
            SeekFrom,
        },
        ParamsEntry,
        Path as _,
        Payload,
        Timestamp,
    },
    std::{
        collections::{
            hash_map::{
                DefaultHasher,
                RandomState,
            },
            HashSet,
        },
        hash::{
            BuildHasher,
            Hasher,
        },
        num::NonZeroUsize,
        sync::Arc,
        time::Instant,
    },
};


pub(crate) type UserName = Arc<str>;
pub(crate) type NamespaceName = Arc<str>;
pub(crate) type PathComponent = Arc<[u8]>;
pub(crate) type Path = Arc<[PathComponent]>;


#[derive(Clone, Eq, Ord, Hash, PartialEq, PartialOrd, Debug)]
pub(crate) struct User
{
    pub(crate) name: UserName,
    /// Disambiguates users with the same name.
    pub(crate) id:   u128,
}

impl User
{
    pub(crate) fn new(name: impl AsRef<str>) -> Self
    {
        /// Abuse `RandomState`'s randomness to generate random numbers.
        fn rand_id(name: &str) -> u128
        {
            let rand_state = RandomState::new();
            let upper_half = u128::from(rand_state.hash_one(name));
            let lower_half = u128::from(rand_state.hash_one(Instant::now()));
            (upper_half << 64) | lower_half
        }

        let name: UserName = name.as_ref().into();
        let id = rand_id(&name);
        Self { name, id }
    }
}


/// Only adequate for this test module.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Permission
{
    pub(crate) user:       User,
    pub(crate) namespaces: HashSet<NamespaceName>,
    pub(crate) subspaces:  HashSet<User>,
    pub(crate) paths:      HashSet<Path>,
    pub(crate) times:      HashSet<Range<Timestamp>>,
}


/// Represents our choice of the Willow parameters for this test module.
pub(crate) struct Params;

impl sailce_data_model::Params for Params
{
    type AuthorisationToken = Permission;
    type HashPayloadError<P> =
        HashPayloadError<P::SeekError, P::ReadError> where P: Payload + ?Sized;
    type NamespaceId = NamespaceName;
    type PayloadDigest = u64;
    type SubspaceId = User;

    const MAX_COMPONENT_COUNT: NonZeroUsize = nz_usize(128);
    const MAX_COMPONENT_LENGTH: NonZeroUsize = nz_usize(512);
    const MAX_PATH_LENGTH: NonZeroUsize = nz_usize(8 * 1024);

    /// Just for this testing module, use `std`'s hashing ability.  (This doesn't follow the
    /// recommendation to use a secure hash function.)
    async fn hash_payload<P>(
        payload: &mut P
    ) -> Result<Self::PayloadDigest, Self::HashPayloadError<P>>
    where P: Payload + ?Sized
    {
        use HashPayloadError as Error;
        const BUF_SIZE: u16 = 4 * 1024; // As `u16` enables `into` and avoiding `try_into`.

        let payload_len = payload.len().await;
        let _: u64 = payload.seek(SeekFrom::Start(0)).await.map_err(Error::Seek)?;
        let mut hasher = DefaultHasher::default();
        let mut buf = vec![0; BUF_SIZE.into()];
        let mut remaining = payload_len;

        while remaining >= 1 {
            // Must only give it a slice length that is within its bounds.
            let ask_len = BUF_SIZE.min(remaining.try_into().unwrap_or(BUF_SIZE));
            let buf = if let Some(buf) = buf.get_mut(.. ask_len.into()) { buf } else { &mut buf };
            debug_assert_eq!(buf.len(), ask_len.into());
            debug_assert!(u64::from(ask_len) <= remaining);

            payload
                .copy_to_slice(
                    None, // Use the current position of `payload`, as it advances.
                    buf,
                    Some(|chunk: &mut [u8]| hasher.write(chunk)),
                    false, // Advance the position of `payload`.
                )
                .await
                .map_err(Error::CopyToSlice)?;

            remaining = remaining.saturating_sub(ask_len.into());
        }
        Ok(hasher.finish())
    }

    fn is_authorised_write(
        entry: &ParamsEntry<Self, impl sailce_data_model::Path>,
        auth_token: &Self::AuthorisationToken,
    ) -> bool
    {
        auth_token.namespaces.contains(&entry.namespace_id)
            && (auth_token.user == entry.subspace_id
                || auth_token.subspaces.contains(&entry.subspace_id))
            && auth_token.paths.iter().any(|path| path.is_prefix_of(&entry.path))
            && auth_token.times.iter().any(|time_range| time_range.includes(entry.timestamp))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum HashPayloadError<SeekError, ReadError>
{
    Seek(SeekError),
    CopyToSlice(CopyToSliceError<ReadError, SeekError>),
}
