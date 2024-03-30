use {
    crate::async_help::{
        not_yet_ready,
        Pollster,
    },
    sailce_data_model::{
        payload::{
            extra,
            sync,
            CopyToSliceError,
            SeekFrom,
        },
        syncify::Syncify,
    },
    std::{
        convert::Infallible,
        future::Future,
        io,
        num::{
            NonZeroU32,
            NonZeroU64,
        },
        sync::Arc,
    },
};


#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InMem
{
    bytes: Arc<[u8]>, // `Arc` so `clone`ing shares it efficiently.
    pos:   usize,
    len:   usize,
}

impl InMem
{
    /// Make a new in-memory [`Payload`] with contents from the given `bytes`, enforcing, by
    /// construction, the requirement of Willow that `Payload`s are at most [`u64::MAX`] bytes
    /// long.
    pub(crate) fn new(bytes: impl AsRef<[u8]>) -> Result<Self, &'static str>
    {
        let bytes: Arc<[u8]> = bytes.as_ref().into();
        let len = bytes.len();
        let len_u64: Result<u64, _> = len.try_into();
        if len_u64.is_ok() {
            Ok(Self { bytes, pos: 0, len })
        }
        else {
            Err("Input is too large!") // Could only happen if `usize` is wider than 64-bit.
        }
    }

    pub(crate) fn check_invariants(&self) -> bool
    {
        let (_, _) = (self.pos_as_u64(), self.len_as_u64());
        self.pos <= self.len && self.len == self.bytes.len()
    }

    fn usize_as_u64(val: usize) -> u64
    {
        val.try_into().expect("`usize` field is constructed to be within `u64` limit")
    }

    pub(crate) fn pos_as_u64(&self) -> u64
    {
        Self::usize_as_u64(self.pos)
    }

    pub(crate) fn len_as_u64(&self) -> u64
    {
        Self::usize_as_u64(self.len)
    }
}


#[derive(Debug, Eq, PartialEq)]
pub(crate) enum TooFar
{
    AfterEnd(u64),
    BeforeStart(u64),
}

#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
impl sailce_data_model::Payload for InMem
{
    type ReadError = Infallible;
    type SeekError = TooFar;

    async fn read(
        &mut self,
        buf: &mut [u8],
    ) -> Result<usize, Self::ReadError>
    {
        use io::Read as _;

        not_yet_ready(2).await;
        let mut avail = &self.bytes[self.pos ..];
        let consumed = avail.read(buf).expect("`<&[u8] as Read>::read` is infallible");
        not_yet_ready(1).await;
        self.pos += consumed;
        debug_assert!(self.check_invariants());
        not_yet_ready(0).await;
        Ok(consumed)
    }

    async fn seek(
        &mut self,
        from: SeekFrom,
    ) -> Result<u64, Self::SeekError>
    {
        let self_len = self.len().await; // `len()` used, just to exercise an `.await` in here.
        let self_pos = self.pos_as_u64();

        let from_start = |offset: u64| {
            if offset <= self_len { Ok(offset) } else { Err(TooFar::AfterEnd(offset - self_len)) }
        };
        let from_end = |offset| {
            self_len.checked_sub(offset).ok_or_else(|| TooFar::BeforeStart(offset - self_len))
        };
        let from_current = |offset| {
            if let Some(offset) = self_pos.checked_add_signed(offset) {
                from_start(offset)
            }
            else {
                let offset_abs = offset.unsigned_abs();
                Err(if offset.is_negative() {
                    TooFar::BeforeStart(offset_abs - self_pos)
                }
                else {
                    TooFar::AfterEnd(offset_abs - (self_len - self_pos))
                })
            }
        };
        not_yet_ready(3).await;

        let new_pos = match from {
            SeekFrom::Start(offset) => from_start(offset),
            SeekFrom::End(offset) => from_end(offset),
            SeekFrom::Current(offset) => from_current(offset),
        }?;
        self.pos = new_pos.try_into().expect("helpers keep position within `usize` limit");
        debug_assert!(self.check_invariants());
        Ok(self.pos_as_u64())
    }

    async fn len(&self) -> u64
    {
        not_yet_ready(1).await;
        self.len_as_u64()
    }
}

impl Syncify<Pollster> for InMem
{
    // Just to exercise having something non-unit that satisfies the generic type param of
    // `Pollster::block_on`.
    type ExecutorData = NonZeroU32;

    fn get_block_on_fn<'f, F>(&self) -> impl 'f + FnOnce(F, Self::ExecutorData) -> F::Output
    where F: Future + 'f
    {
        Pollster::block_on
    }

    fn get_executor_data(&self) -> Self::ExecutorData
    {
        NonZeroU32::MIN | 122
    }
}

impl sync::Payload<Pollster> for InMem {}


/// This exercises both the sync and the `async` methods, because the sync ones use the `async`
/// ones.
#[test]
#[allow(clippy::indexing_slicing, clippy::missing_asserts_for_indexing, clippy::too_many_lines)]
fn all_methods__sync_uses_async()
{
    use sync::Payload as _; // Use the sync methods, not the `async` ones with the same names.

    #[rustfmt::skip]
    trait Helpers {
        fn pos(&mut self) -> u64;
        fn reset(&mut self, pos: u64);
    }
    #[rustfmt::skip]
    impl Helpers for InMem {
        fn pos(&mut self) -> u64 {
            self.seek(SeekFrom::Current(0)).expect("zero offset always succeeds")
        }
        fn reset(&mut self, pos: u64) { assert_eq!(self.seek(SeekFrom::Start(pos)), Ok(pos)); }
    }

    let mut payload = InMem::new("abcdefghijklmnopqrstuvwxyz").expect("size fits `u64`");
    let space = &mut [0_u8; 64][..];
    let mut buf;

    assert!(!payload.is_empty());
    assert_eq!(payload.len(), 26);

    buf = &mut space[.. 1];
    assert_eq!(payload.read(buf), Ok(1));
    assert_eq!(buf, b"a");
    assert_eq!(payload.pos(), 1);
    buf = &mut space[.. 2];
    assert_eq!(payload.read(buf), Ok(2));
    assert_eq!(buf, b"bc");
    assert_eq!(payload.pos(), 3);
    buf = &mut space[.. 3];
    assert_eq!(payload.read(buf), Ok(3));
    assert_eq!(buf, b"def");
    assert_eq!(payload.pos(), 6);

    assert_eq!(payload.seek(SeekFrom::Start(0)), Ok(0));
    buf = &mut space[.. 7];
    assert_eq!(payload.read(buf), Ok(7));
    assert_eq!(buf, b"abcdefg");
    assert_eq!(payload.pos(), 7);

    assert_eq!(payload.seek(SeekFrom::Start(12)), Ok(12));
    buf = space;
    assert_eq!(payload.read(buf), Ok(14));
    assert_eq!(&buf[.. 14], b"mnopqrstuvwxyz");
    assert_eq!(payload.pos(), 26);

    assert_eq!(payload.seek(SeekFrom::Start(20)), Ok(20));
    buf = &mut space[32 .. 36];
    assert_eq!(payload.read(buf), Ok(4));
    assert_eq!(buf, b"uvwx");
    assert_eq!(payload.pos(), 24);

    assert_eq!(payload.seek(SeekFrom::Start(26)), Ok(26));
    buf = space;
    assert_eq!(payload.read(buf), Ok(0));
    assert_eq!(payload.pos(), 26);

    assert_eq!(payload.seek(SeekFrom::Start(27)), Err(TooFar::AfterEnd(1)));
    assert_eq!(payload.seek(SeekFrom::Start(50)), Err(TooFar::AfterEnd(24)));

    assert_eq!(payload.seek(SeekFrom::End(0)), Ok(26));
    buf = space;
    assert_eq!(payload.read(buf), Ok(0));
    assert_eq!(payload.pos(), 26);

    assert_eq!(payload.seek(SeekFrom::End(1)), Ok(25));
    buf = space;
    assert_eq!(payload.read(buf), Ok(1));
    assert_eq!(&buf[.. 1], b"z");
    assert_eq!(payload.pos(), 26);

    assert_eq!(payload.seek(SeekFrom::End(2)), Ok(24));
    buf = space;
    assert_eq!(payload.read(buf), Ok(2));
    assert_eq!(&buf[.. 2], b"yz");
    assert_eq!(payload.pos(), 26);

    assert_eq!(payload.seek(SeekFrom::End(3)), Ok(23));
    buf = space;
    assert_eq!(payload.read(buf), Ok(3));
    assert_eq!(&buf[.. 3], b"xyz");
    assert_eq!(payload.pos(), 26);

    assert_eq!(payload.seek(SeekFrom::End(26)), Ok(0));
    buf = &mut space[.. 10];
    assert_eq!(payload.read(buf), Ok(10));
    assert_eq!(buf, b"abcdefghij");
    assert_eq!(payload.pos(), 10);

    assert_eq!(payload.seek(SeekFrom::End(27)), Err(TooFar::BeforeStart(1)));
    assert_eq!(payload.seek(SeekFrom::End(60)), Err(TooFar::BeforeStart(34)));

    payload.reset(10);

    assert_eq!(payload.seek(SeekFrom::Current(0)), Ok(10));
    buf = &mut space[60 ..];
    assert_eq!(payload.read(buf), Ok(4));
    assert_eq!(buf, b"klmn");
    assert_eq!(payload.pos(), 14);

    assert_eq!(payload.seek(SeekFrom::Current(1)), Ok(15));
    buf = &mut space[.. 1];
    assert_eq!(payload.read(buf), Ok(1));
    assert_eq!(buf, b"p");
    assert_eq!(payload.pos(), 16);

    assert_eq!(payload.seek(SeekFrom::Current(2)), Ok(18));
    buf = &mut space[.. 2];
    assert_eq!(payload.read(buf), Ok(2));
    assert_eq!(buf, b"st");
    assert_eq!(payload.pos(), 20);

    assert_eq!(payload.seek(SeekFrom::Current(3)), Ok(23));
    buf = &mut space[.. 3];
    assert_eq!(payload.read(buf), Ok(3));
    assert_eq!(buf, b"xyz");
    assert_eq!(payload.pos(), 26);

    assert_eq!(payload.seek(SeekFrom::Current(-17)), Ok(9));
    buf = &mut space[.. 4];
    assert_eq!(payload.read(buf), Ok(4));
    assert_eq!(buf, b"jklm");
    assert_eq!(payload.pos(), 13);

    assert_eq!(payload.seek(SeekFrom::Current(-7)), Ok(6));
    buf = &mut space[5 .. 5];
    assert_eq!(payload.read(buf), Ok(0));
    assert_eq!(buf, b"");
    assert_eq!(payload.pos(), 6);

    assert_eq!(payload.seek(SeekFrom::Current(21)), Err(TooFar::AfterEnd(1)));
    payload.reset(10);
    assert_eq!(payload.seek(SeekFrom::Current(70)), Err(TooFar::AfterEnd(54)));
    payload.reset(5);
    assert_eq!(payload.seek(SeekFrom::Current(-20)), Err(TooFar::BeforeStart(15)));
}


impl io::Read for InMem
{
    fn read(
        &mut self,
        buf: &mut [u8],
    ) -> io::Result<usize>
    {
        #![allow(clippy::unwrap_in_result)]

        Ok(<Self as sync::Payload<Pollster>>::read(self, buf)
            .expect("`InMem::read` is infallible"))
    }
}

#[test]
fn std_Read()
{
    use io::Read as _;

    fn new(bytes: &[u8]) -> InMem
    {
        InMem::new(bytes).expect("size fits `u64`")
    }

    let payload1 = &mut new(b"A BC DEF GHIJ ");
    let payload2 = &mut new(b"KLMNO PQRSTU VWXYZ");
    let mut s = String::new();

    assert_eq!(payload1.take(5).chain(payload2.take(10)).read_to_string(&mut s).ok(), Some(15));
    assert_eq!(s, "A BC KLMNO PQRS");
    assert_eq!(payload1.pos, 5);
    assert_eq!(payload2.pos, 10);
}


impl extra::sync::ExtraCore<Pollster> for InMem {}
#[cfg(feature = "alloc")]
impl extra::sync::Extra<Pollster> for InMem {}

const NONE_CALLBACK: Option<fn(&mut [u8])> = None;
const UNREACHABLE_CALLBACK: Option<fn(&mut [u8])> = Some(unreachable_callback);

fn unreachable_callback(_chunk: &mut [u8])
{
    #![allow(clippy::unreachable)]
    unreachable!()
}

#[test]
fn copy_to_slice()
{
    use {
        extra::sync::ExtraCore as _,
        sync::Payload as _,
    };

    let mut p0 = InMem::new([]).unwrap();
    assert_eq!(p0.current_position(), Ok(0));
    assert_eq!(p0.copy_to_slice(None, &mut [], NONE_CALLBACK, false), Ok(()));
    assert_eq!(p0.current_position(), Ok(0));
    assert_eq!(
        p0.copy_to_slice(None, &mut [0], NONE_CALLBACK, false),
        Err(CopyToSliceError::OutOfBounds { at: Some(NonZeroU64::new(1).unwrap()) })
    );
    assert_eq!(p0.current_position(), Ok(0));

    let mut p1 = InMem::new("a").unwrap();
    assert_eq!(p1.current_position(), Ok(0));
    let mut buf1 = [0];
    assert_eq!(p1.copy_to_slice(None, &mut [], NONE_CALLBACK, false), Ok(()));
    assert_eq!(p1.current_position(), Ok(0));
    assert_eq!(p1.copy_to_slice(None, &mut buf1, NONE_CALLBACK, false), Ok(()));
    assert_eq!(&buf1, b"a");
    assert_eq!(p1.current_position(), Ok(1));
    assert_eq!(
        p1.copy_to_slice(None, &mut buf1, NONE_CALLBACK, false),
        Err(CopyToSliceError::OutOfBounds { at: Some(NonZeroU64::new(2).unwrap()) })
    );
    assert_eq!(p1.current_position(), Ok(1));

    let mut p8 = InMem::new(b"bcdefghi").unwrap();
    assert_eq!(p8.current_position(), Ok(0));
    let mut buf8 = [0; 8];
    assert_eq!(p8.copy_to_slice(Some(3), &mut [], NONE_CALLBACK, true), Ok(()));
    assert_eq!(p8.current_position(), Ok(0));
    assert_eq!(p8.copy_to_slice(Some(2), &mut buf8[.. 4], NONE_CALLBACK, true), Ok(()));
    assert_eq!(p8.current_position(), Ok(0));
    assert_eq!(&buf8, "defg\0\0\0\0".as_bytes());
    assert_eq!(
        p8.copy_to_slice(Some(5), &mut buf8[4 .. 6], Some(<[u8]>::make_ascii_uppercase), false),
        Ok(())
    );
    assert_eq!(p8.current_position(), Ok(7));
    assert_eq!(&buf8, "defgGH\0\0".as_bytes());
    assert_eq!(p8.copy_to_slice(None, &mut buf8[7 ..], NONE_CALLBACK, false), Ok(()));
    assert_eq!(p8.current_position(), Ok(8));
    assert_eq!(&buf8, "defgGH\0i".as_bytes());
    assert_eq!(p8.copy_to_slice(None, &mut buf8[8 ..], NONE_CALLBACK, false), Ok(()));
    assert_eq!(p8.current_position(), Ok(8));
    assert_eq!(p8.seek(SeekFrom::End(5)), Ok(3));
    assert_eq!(
        p8.copy_to_slice(Some(8), &mut buf8, UNREACHABLE_CALLBACK, true),
        Err(CopyToSliceError::OutOfBounds { at: Some(NonZeroU64::new(16).unwrap()) })
    );
    assert_eq!(p8.current_position(), Ok(3));
    assert_eq!(&buf8, "defgGH\0i".as_bytes()); // Unchanged.
}


#[cfg(feature = "alloc")]
#[test]
fn to_boxed_slice()
{
    use {
        extra::sync::{
            Extra as _,
            ExtraCore as _,
        },
        sailce_data_model::payload::ToBoxedSliceError,
        std::{
            mem::size_of,
            ops::Bound,
        },
        sync::Payload as _,
    };

    let mut p0 = InMem::new([]).unwrap();

    if size_of::<usize>() <= size_of::<u64>() {
        assert_eq!(
            p0.to_boxed_slice(0 .. u64::MAX, NONE_CALLBACK, false),
            Err(ToBoxedSliceError::RangeTooLong {
                by: NonZeroU64::new(u64::MAX - u64::try_from(isize::MAX).unwrap()).unwrap(),
            })
        );
        assert_eq!(p0.current_position(), Ok(0));
    }
    assert_eq!(p0.to_boxed_slice(0 .., UNREACHABLE_CALLBACK, false), Ok([].into()));
    assert_eq!(p0.current_position(), Ok(0));
    assert_eq!(p0.to_boxed_slice(.., UNREACHABLE_CALLBACK, false), Ok([].into()));
    assert_eq!(p0.current_position(), Ok(0));
    assert_eq!(
        p0.to_boxed_slice(0 .. 1, UNREACHABLE_CALLBACK, false),
        Err(ToBoxedSliceError::OutOfBounds { at: Some(NonZeroU64::new(1).unwrap()) })
    );
    assert_eq!(p0.current_position(), Ok(0));
    assert_eq!(
        p0.to_boxed_slice(2 .. 3, UNREACHABLE_CALLBACK, true),
        Err(ToBoxedSliceError::OutOfBounds { at: Some(NonZeroU64::new(2).unwrap()) })
    );
    assert_eq!(p0.current_position(), Ok(0));

    let mut p1 = InMem::new("0123456789").unwrap();
    assert_eq!(p1.seek(SeekFrom::Start(2)), Ok(2));

    if size_of::<usize>() <= size_of::<u64>() {
        assert_eq!(
            p1.to_boxed_slice(
                .. u64::try_from(isize::MAX).unwrap() + 3,
                UNREACHABLE_CALLBACK,
                false
            ),
            Err(ToBoxedSliceError::RangeTooLong { by: NonZeroU64::new(1).unwrap() })
        );
        assert_eq!(p1.current_position(), Ok(2));
    }
    assert_eq!(p1.to_boxed_slice(0 .., NONE_CALLBACK, true), Ok((*b"0123456789").into()));
    assert_eq!(p1.current_position(), Ok(2));
    assert_eq!(
        p1.to_boxed_slice(
            ..= 7,
            Some(|chunk: &mut [u8]| for byte in chunk {
                *byte = u8::MAX - *byte;
            }),
            false
        ),
        Ok([205, 204, 203, 202, 201, 200].into())
    );
    assert_eq!(p1.current_position(), Ok(8));
    assert_eq!(
        p1.to_boxed_slice(..= 10, UNREACHABLE_CALLBACK, false),
        Err(ToBoxedSliceError::OutOfBounds { at: Some(NonZeroU64::new(11).unwrap()) })
    );
    assert_eq!(p1.current_position(), Ok(8));
    assert_eq!(
        p1.to_boxed_slice(..= u64::MAX, NONE_CALLBACK, false),
        Err(ToBoxedSliceError::OutOfBounds { at: None })
    );
    assert_eq!(p1.current_position(), Ok(8));
    assert_eq!(
        p1.to_boxed_slice((Bound::Excluded(u64::MAX), Bound::Unbounded), NONE_CALLBACK, false),
        Err(ToBoxedSliceError::OutOfBounds { at: None })
    );
    assert_eq!(p1.current_position(), Ok(8));
    assert_eq!(p1.to_boxed_slice(9 .. 9, UNREACHABLE_CALLBACK, false), Ok([].into()));
    assert_eq!(p1.current_position(), Ok(9));
}


#[cfg(feature = "std")]
#[test]
#[allow(unstable_name_collisions)]
fn cursor()
{
    use {
        sailce_data_model::payload::{
            sync::Payload as _,
            SeekFrom,
        },
        std::io::Cursor,
    };

    let buf = &mut [0_u8; 16];

    let mut c0 = Cursor::new([0_u8; 0]);
    assert_eq!(c0.read(&mut []).ok(), Some(0));
    assert_eq!(c0.len(), 0);
    assert_eq!(c0.seek(SeekFrom::Start(0)).ok(), Some(0));
    assert_eq!(c0.seek(SeekFrom::Current(0)).ok(), Some(0));
    assert_eq!(c0.seek(SeekFrom::End(0)).ok(), Some(0));
    assert!(c0.is_empty());

    let mut c1 = Cursor::new("foo bar zab");
    assert_eq!(c1.read(&mut buf[.. 4]).ok(), Some(4));
    assert_eq!(&buf[.. 4], b"foo ");
    assert_eq!(c1.len(), 11);
    assert_eq!(c1.seek(SeekFrom::Current(0)).ok(), Some(4));
    assert_eq!(c1.seek(SeekFrom::Start(123)).ok(), Some(123));
    assert_eq!(c1.read(buf).ok(), Some(0));
    assert_eq!(c1.seek(SeekFrom::End(3)).ok(), Some(8));
    assert_eq!(c1.read(&mut buf[4 .. 7]).ok(), Some(3));
    assert!(!c1.is_empty());

    assert_eq!(buf, b"foo zab\0\0\0\0\0\0\0\0\0");
}
