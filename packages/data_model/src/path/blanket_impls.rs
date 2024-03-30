use super::{
    Component,
    EmptyPath,
    Path,
};


// FUTURE: If "trait specialization" is ever stabilized in Rust, it'd be good if there could also
// be blanket impls of `Path` `where C: AsRef<str>` that give a different impl of
// `str_components`, that overrides the default provided one, that is more efficient by not
// needing to do `from_utf8` because the `Self` type already contains the needed `str` values.


impl<T> Path for &T
where T: Path + ?Sized
{
    #[inline]
    fn components(&self) -> impl ExactSizeIterator<Item = Component<'_>>
    {
        <T as Path>::components(*self)
    }
}


macro_rules! path_iter_impl {
    ($t:ty) => {
        impl<C> Path for $t
        where C: AsRef<[u8]>
        {
            #[inline]
            fn components(&self) -> impl ExactSizeIterator<Item = Component<'_>>
            {
                self.iter().map(|bytes| Component { bytes: bytes.as_ref() })
            }
        }
    };
}


path_iter_impl! { [C] }

impl<C> EmptyPath for &[C]
where C: AsRef<[u8]>
{
    #[inline]
    fn empty() -> Self
    {
        &[][..]
    }
}


macro_rules! emptypath_impl {
    ($t:ty) => {
        impl<C> EmptyPath for $t
        where C: AsRef<[u8]>
        {
            #[inline]
            fn empty() -> Self
            {
                Self::from([])
            }
        }
    };
}


impl<const N: usize, C> Path for [C; N]
where C: AsRef<[u8]>
{
    #[inline]
    fn components(&self) -> impl ExactSizeIterator<Item = Component<'_>>
    {
        <[C] as Path>::components(&self[..])
    }
}

emptypath_impl! { [C; 0] }

impl<C> EmptyPath for &[C; 0]
where C: AsRef<[u8]>
{
    #[inline]
    fn empty() -> Self
    {
        &[]
    }
}


#[cfg(feature = "alloc")]
mod alloc
{
    use {
        super::{
            Component,
            Path,
        },
        crate::EmptyPath,
        alloc::{
            borrow::{
                Cow,
                ToOwned,
            },
            boxed::Box,
            collections::{
                LinkedList,
                VecDeque,
            },
            rc::Rc,
            sync::Arc,
            vec::Vec,
        },
    };


    // TODO: See about supporting the unstable `A: Allocator` type param of collections, via
    // `cfg(rust_lib_feature = "allocator_api")`


    macro_rules! path_slice_impl {
        ($t:ty) => {
            impl<C> Path for $t
            where C: AsRef<[u8]>
            {
                #[inline]
                fn components(&self) -> impl ExactSizeIterator<Item = Component<'_>>
                {
                    <[C] as Path>::components(&self[..])
                }
            }
        };
    }
    macro_rules! path_slice_impls {
        ($($t:ty)*) => { $(
            path_slice_impl! { $t }
            emptypath_impl! { $t }
        )* };
    }
    macro_rules! path_iter_impls {
        ($($t:ty)*) => { $(
            path_iter_impl! { $t }
            emptypath_impl! { $t }
        )* };
    }


    path_slice_impls! { Box<[C]> Rc<[C]> Arc<[C]> Vec<C> }
    path_iter_impls! { VecDeque<C> LinkedList<C> }


    impl<C> Path for Cow<'_, [C]>
    where
        C: AsRef<[u8]>,
        [C]: ToOwned,
    {
        #[inline]
        fn components(&self) -> impl ExactSizeIterator<Item = Component<'_>>
        {
            self.iter().map(|bytes| Component { bytes: bytes.as_ref() })
        }
    }

    impl<C> EmptyPath for Cow<'_, [C]>
    where
        C: AsRef<[u8]>,
        [C]: ToOwned,
    {
        #[inline]
        fn empty() -> Self
        {
            Cow::Borrowed(&[][..])
        }
    }
}
