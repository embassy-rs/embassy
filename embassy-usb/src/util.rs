use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

use embassy::blocking_mutex::raw::RawMutex;
use embassy::channel::Channel;

pub struct Pending<T> {
    _phantom: PhantomData<T>,
}

impl<T> Pending<T> {
    fn new() -> Self {
        Pending {
            _phantom: PhantomData,
        }
    }
}

impl<T> Future for Pending<T> {
    type Output = T;
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

pub async fn recv_or_wait<M: RawMutex, T, const N: usize>(ch: Option<&Channel<M, T, N>>) -> T {
    match ch {
        Some(ch) => ch.recv().await,
        None => Pending::new().await,
    }
}
