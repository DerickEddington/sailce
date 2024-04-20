// TODO: Should have ones that give Box<[u8]>.  Maybe these should replace the Vec ones, i.e. not
// have Vec ones?  They can interconvert without copying nor reallocating, when capacity==len,
// right?

//! Premade helpers for making `get_dest` closures that return "destination" buffers for writing
//! outputs, as used with [`Cryptor::encrypt_component`](crate::Cryptor::encrypt_component) or
//! [`Cryptor::decrypt_component`](crate::Cryptor::decrypt_component), and also used with
//! [`EncryptPath`](crate::EncryptPath) or [`EncryptedPath`](crate::EncryptedPath) methods.
//!
//! These are sometimes useful for situations and so are provided by this crate, but these are not
//! the only possibilities.  You may provide your own `get_dest` closures instead that do it
//! differently.
//!
//! A `get_dest(needed_size)` closure call returns `Some(space)` when it provides exactly
//! `needed_size` writable bytes at location `space`, or it returns `None` when it has exhausted
//! its supply or when a `needed_size` is not within its limits.  After a `needed_size` was
//! rejected, a different (typically, smaller) value might still succeed.


/// Make and return a `get_dest` closure that provides space as consecutive sub-slices from a
/// single backing `slice` until exhausted.
#[inline]
pub fn from_slice<'l>(slice: &'l mut [u8]) -> impl FnMut(usize) -> Option<&'l mut [u8]>
{
    from_slice_with_limits(slice, None, None)
}

/// Like [`from_slice`] but the total count of sub-slices given must be within `limit_count` and
/// each `needed_size` must be within `limit_each` or else the closure returns `None`.  If any of
/// the limits is `None` then it's unlimited in that regard (but will still return `None` when
/// exhausted).
///
/// (This doesn't need to take a `limit_total` because limiting the total size of all
/// `needed_size` should be done by choosing the size of `slice`.)
#[inline]
pub fn from_slice_with_limits<'l>(
    slice: &'l mut [u8],
    mut limit_count: Option<usize>,
    limit_each: Option<usize>,
) -> impl FnMut(usize) -> Option<&'l mut [u8]>
{
    let mut slice = Some(slice);

    move |needed_size| {
        check_limit_count(&mut limit_count)?;
        check_limit_each(needed_size, limit_each)?;
        // `take`ing `slice` enables returning `space` with lifetime `'l` (otherwise it'd be
        // restricted to the lifetime of the call of this closure but that wouldn't work).
        slice.take().and_then(|slice_here| {
            if needed_size <= slice_here.len() {
                let (space, remainder) = slice_here.split_at_mut(needed_size);
                slice = Some(remainder);
                Some(space)
            }
            else {
                slice = Some(slice_here); // Might be usable by a different call for smaller.
                None
            }
        })
    }
}


fn check_limit_each(
    needed_size: usize,
    limit_each: Option<usize>,
) -> Option<()>
{
    match limit_each {
        Some(limit_each) => (needed_size <= limit_each).then_some(()),
        None => Some(()),
    }
}

fn check_limit_count(limit_count: &mut Option<usize>) -> Option<()>
{
    match limit_count {
        Some(limit_count) => limit_count.checked_sub(1).map(|decr| {
            *limit_count = decr;
        }),
        None => Some(()),
    }
}


#[cfg(feature = "alloc")]
pub use alloc::*;

#[cfg(feature = "alloc")]
mod alloc
{
    use {
        super::{
            check_limit_count,
            check_limit_each,
        },
        alloc::vec::Vec,
    };


    fn check_limit_total(
        needed_size: usize,
        limit_total: &mut Option<usize>,
    ) -> Option<()>
    {
        match limit_total {
            Some(limit_total) => limit_total.checked_sub(needed_size).map(|decr| {
                *limit_total = decr;
            }),
            None => Some(()),
        }
    }


    /// Make and return a `get_dest` closure that provides space as newly-allocated [`Vec`]s that
    /// are given as separately owned.
    #[inline]
    pub fn from_vec() -> impl FnMut(usize) -> Option<Vec<u8>>
    {
        from_vec_with_limits(None, None, None)
    }

    /// Like [`from_vec`] but the total count of `Vec`s given must be within `limit_count` and
    /// each `needed_size` must be within `limit_each` and the total size of all `needed_size`
    /// must be within `limit_total` or else the closure returns `None`.  If any of the limits is
    /// `None` then it's unlimited in that regard.
    #[inline]
    pub fn from_vec_with_limits(
        mut limit_count: Option<usize>,
        limit_each: Option<usize>,
        mut limit_total: Option<usize>,
    ) -> impl FnMut(usize) -> Option<Vec<u8>>
    {
        move |needed_size| {
            check_limit_count(&mut limit_count)?;
            check_limit_each(needed_size, limit_each)?;
            check_limit_total(needed_size, &mut limit_total)?;
            let mut v = Vec::new();
            v.try_reserve_exact(needed_size).ok()?;
            // (It's hoped that the standard library will have a more-efficient implementation of
            // `Vec::resize(_, 0)` in the future.  This approach is how I want it.)
            v.resize(needed_size, 0);
            Some(v)
        }
    }


    /// Make and return a `get_dest` closure that provides space as newly-allocated [`Vec`]s that
    /// are given as borrows from a single collection of `Vec`s given by `vecs`.
    ///
    /// This enables the use of them to continue to hold (e.g. to return) borrows of their
    /// contents.  This also enables keeping them grouped together and sequenced regardless of
    /// however they get used.
    ///
    /// The desired limit of available new `Vec`s is given by `reserve`, and the closure will
    /// return `None` after that is reached.  This is always necessary because `vecs` cannot be
    /// resized after its elements are borrowed.  Any preexisting elements of `vecs` are ignored
    /// and the new `Vec`s are added to it as new elements.  `vecs` may be given as initially
    /// empty.
    #[inline]
    pub fn from_vecs<'l>(
        vecs: &'l mut Vec<Vec<u8>>,
        reserve: usize,
    ) -> impl FnMut(usize) -> Option<&'l mut Vec<u8>>
    {
        from_vecs_with_limits(vecs, reserve, None, None)
    }

    /// Like [`from_vecs`] but each `needed_size` must be within `limit_each` and the total size
    /// of all `needed_size` must be within `limit_total` or else the closure returns `None`.  If
    /// any of the limits is `None` then it's unlimited in that regard (but will still return
    /// `None` when exhausted).
    ///
    /// (This doesn't need to take a `limit_count` because limiting the total count of `Vec`s
    /// given should be done by choosing the value of `reserve`.)
    #[inline]
    pub fn from_vecs_with_limits<'l>(
        vecs: &'l mut Vec<Vec<u8>>,
        reserve: usize,
        limit_each: Option<usize>,
        mut limit_total: Option<usize>,
    ) -> impl FnMut(usize) -> Option<&'l mut Vec<u8>>
    {
        let cur = vecs.len();
        let mut vecs = if vecs.try_reserve_exact(reserve).is_ok() {
            vecs.resize(cur.saturating_add(reserve), Vec::new());
            vecs.get_mut(cur ..)
        }
        else {
            None
        };

        move |needed_size| {
            check_limit_each(needed_size, limit_each)?;
            check_limit_total(needed_size, &mut limit_total)?;
            // `take`ing `vecs` enables returning `space` with lifetime `'l` (otherwise it'd be
            // restricted to the lifetime of the call of this closure but that wouldn't work).
            vecs.take().and_then(|vecs_here| {
                vecs_here.split_first_mut().and_then(|(space, remainder)| {
                    vecs = Some(remainder);
                    debug_assert!(space.is_empty(), "was created as empty");
                    space.try_reserve_exact(needed_size).ok()?;
                    space.resize(needed_size, 0); // (See above hope about `resize`.)
                    Some(space)
                })
            })
        }
    }
}
