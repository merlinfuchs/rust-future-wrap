use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use pin_project::pin_project;

#[pin_project]
pub struct WrappedFuture<Fut, F, O>
    where
        Fut: Future,
        F: FnMut(Pin<&mut Fut>, &mut Context) -> Poll<O>,
{
    #[pin]
    inner: Fut,
    f: F,
}

impl<Fut, F, O> Future for WrappedFuture<Fut, F, O>
    where
        Fut: Future,
        F: FnMut(Pin<&mut Fut>, &mut Context) -> Poll<O>,
{
    type Output = O;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();

        (this.f)(this.inner, cx)
    }
}

/// Adds a `wrap` function to all types that implement `Future`
/// Lets you track each poll, interrupt the execution of the Future and change the return type
/// ```
/// use future_wrap::WrapFuture;
/// use std::future::Future;
/// use std::task::Poll;
/// use std::time::{Duration, Instant};
///
/// let fut = some_async_fn();
///
/// let mut remaining_time = Duration::from_millis(10);
/// fut.wrap(|fut, cx| {
///     let poll_start = Instant::now();
///
///     println!("Poll start");
///     let res = fut.poll(cx).map(|v| Some(v));
///     println!("Poll end");
///
///     remaining_time = remaining_time.saturating_sub(poll_start.elapsed());
///     if remaining_time.is_zero() {
///         println!("Too much time spent on polls :(");
///         Poll::Ready(None)
///     } else {
///         res.map(|v| Some(v))
///     }
/// }).await;
/// ```
pub trait WrapFuture<O>: Sized + Future {
    fn wrap<F>(self, f: F) -> WrappedFuture<Self, F, O>
        where
            F: FnMut(Pin<&mut Self>, &mut Context) -> Poll<O>,
    {
        WrappedFuture {
            inner: self,
            f,
        }
    }
}

impl<F: Future, O> WrapFuture<O> for F {}