# rust-future-wrap

A minimal crate that lets you wrap a future to track each poll and modify the outcome.

It only contains a single trait called `WrapFuture` which adds a `wrap` function to all types that
implement `std::future::Future`. The `wrap` takes a closure as an argument which will be called whenever the future is
polled. The closure receives the wrapped `Pin<Future>` and is responsible for calling `poll` on it with the received
waker context. The closure can return `std::task::Poll<T>` of any type `T` and can therefore modify the return type of
the wrapped future (similar to `Future.map`).

## Example

This examples shows how to track and limit the time spend on polling a future.

```rust
use std::future::Future;
use std::task::Poll;
use std::time::{Duration, Instant};

use tokio::runtime::Builder;
use tokio::time::sleep;

use future_wrap::WrapFuture;

async fn some_async_fn() {
    sleep(Duration::from_secs(1)).await;
    std::thread::sleep(Duration::from_millis(3));
    sleep(Duration::from_secs(1)).await;
    std::thread::sleep(Duration::from_millis(3));
    sleep(Duration::from_secs(1)).await;
    std::thread::sleep(Duration::from_millis(3));
    sleep(Duration::from_secs(1)).await;
    std::thread::sleep(Duration::from_millis(3));
    sleep(Duration::from_secs(1)).await;
}

fn main() {
    let runtime = Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();

    runtime.block_on(async move {
        let fut = some_async_fn();

        let mut remaining_time = Duration::from_millis(10);
        fut.wrap(|fut, cx| {
            let poll_start = Instant::now();

            println!("Poll start");
            let res = fut.poll(cx);
            println!("Poll end");

            remaining_time = remaining_time.saturating_sub(poll_start.elapsed());
            if remaining_time.is_zero() {
                println!("Too much time spent on polls :(");
                Poll::Ready(None)
            } else {
                res.map(|v| Some(v))
            }
        }).await;
    });
}
```
