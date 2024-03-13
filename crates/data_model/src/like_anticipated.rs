use {
    cfg_if::cfg_if,
    core::fmt::{
        Debug,
        Display,
    },
};


/// Like [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html), but
/// available when `no_std`.
pub trait Error: Debug + Display
{
    /// The lower-level source of this error, if any.
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)>
    {
        None
    }
}


cfg_if! { if #[cfg(feature = "std")]
{
    extern crate std;

    impl<T> Error for T
    where T: std::error::Error + 'static
    {
        #[inline]
        fn source(&self) -> Option<&(dyn Error + 'static)>
        {
            std::error::Error::source(self).and_then(|std_e| {
                let o: Option<&T> = std_e.downcast_ref();
                debug_assert!(o.is_some(), "impossible");
                o.map(|v| {
                    let our_e: &dyn Error = v;
                    our_e
                })
            })
        }
    }
} }
