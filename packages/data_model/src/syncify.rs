//! Helpers for adapting `async` functions into synchronous equivalents.

#![macro_use] // Make this module's macro visible to all following modules.

use core::future::Future;


/// Helps a sub-trait be an adaptor trait that has synchronous (i.e. not `async`) wrappers of the
/// `async` functions of another trait.
///
/// I.e. this helps when creating a counterpart trait to a primary trait when it's desired to
/// adapt `async` functions into synchronous equivalents that will block callers when the
/// underlying primary `async` functions `.await` suspend.
///
/// Having the `Executor` type parameter enables this trait to be implemented multiple times, with
/// a different type argument for each, for a single type that wants to use this trait with
/// multiple executors.
///
/// Examples of executors which can be used with this include:
/// [`futures::executor::block_on`](
/// https://docs.rs/futures/latest/futures/executor/fn.block_on.html), [`pollster`](
/// https://docs.rs/pollster/latest/pollster/), etc.
pub trait Syncify<Executor>
where Executor: ?Sized
{
    /// Passed to each call of a function returned by [`Self::get_block_on_fn`].  If unneeded,
    /// should be `()`.
    type ExecutorData;

    /// Return the function that will run a `Future` to completion on `Executor`.
    ///
    /// This enables the possibility of returning dynamically different values for some
    /// implementations, or, for other implementations that return a constant, calls to this
    /// should become optimized away.
    ///
    /// (The `'f` lifetime parameter is needed to make the return type's lifetime be independent
    /// of the lifetime of the `&self` borrow.  This means that the return value cannot borrow
    /// from `&self`, because this is required for our use of the return value with a `Future`
    /// that must be able to borrow from `&mut self` exclusively.)
    fn get_block_on_fn<'f, F>(&self) -> impl 'f + FnOnce(F, Self::ExecutorData) -> F::Output
    where F: Future + 'f;

    /// Return the data that is to be passed to a call of the function returned by
    /// [`Self::get_block_on_fn`].
    fn get_executor_data(&self) -> Self::ExecutorData;
}


macro_rules! get_block_on_and_data {
    ($self:ident) => {
        ($self.get_block_on_fn(), $self.get_executor_data())
    };
}
