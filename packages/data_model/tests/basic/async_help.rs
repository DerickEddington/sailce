use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
    thread::{
        self,
        sleep,
    },
    time::Duration,
};


/// Simulates an `async` operation, to simulate `.await` points.
pub(crate) async fn not_yet_ready(tries: u16)
{
    NotYetReady { tries, sleep_ms: 1 }.await;
}

struct NotYetReady
{
    tries:    u16,
    sleep_ms: u16,
}

impl Future for NotYetReady
{
    type Output = ();

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output>
    {
        if self.tries >= 1 {
            self.tries = self.tries.saturating_sub(1);
            let waker = cx.waker().clone();
            let sleep_dur = Duration::from_millis(self.sleep_ms.into());
            let _join_handle = thread::spawn(move || {
                sleep(sleep_dur);
                waker.wake();
            });
            Poll::Pending
        }
        else {
            Poll::Ready(())
        }
    }
}


/// Represents using the [`pollster`] executor to block the thread waiting on a `Future`.
pub(crate) struct Pollster;

impl Pollster
{
    pub(crate) fn block_on<F: Future>(
        fut: F,
        data: impl Into<u32>, // Just to exercise having something non-unit and generic.
    ) -> F::Output
    {
        assert_eq!(data.into(), 123);
        pollster::block_on(fut)
    }
}
