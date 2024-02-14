//! A synchronization primitive for passing the latest value to **multiple** tasks.
use core::cell::RefCell;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::task::{Context, Poll};

use futures_util::Future;

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;
use crate::waitqueue::MultiWakerRegistration;

/// A `MultiSignal` is a single-slot signaling primitive, which can awake `N` separate [`Receiver`]s.
///
/// Similar to a [`Signal`](crate::signal::Signal), except `MultiSignal` allows for multiple tasks to
/// `.await` the latest value, and all receive it.
///
/// This is similar to a [`PubSubChannel`](crate::pubsub::PubSubChannel) with a buffer size of 1, except
/// "sending" to it (calling [`MultiSignal::write`]) will immediately overwrite the previous value instead
/// of waiting for the receivers to pop the previous value.
///
/// `MultiSignal` is useful when a single task is responsible for updating a value or "state", which multiple other
/// tasks are interested in getting notified about changes to the latest value of. It is therefore fine for
/// [`Receiver`]s to "lose" stale values.
///
/// Anyone with a reference to the MultiSignal can update or peek the value. MultiSignals are generally declared
/// as `static`s and then borrowed as required to either [`MultiSignal::peek`] the value or obtain a [`Receiver`]
/// with [`MultiSignal::receiver`] which has async methods.
/// ```
///
/// use futures_executor::block_on;
/// use embassy_sync::multi_signal::MultiSignal;
/// use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
///
/// let f = async {
///
/// static SOME_SIGNAL: MultiSignal<CriticalSectionRawMutex, u8, 2> = MultiSignal::new(0);
///
/// // Obtain Receivers
/// let mut rcv0 = SOME_SIGNAL.receiver().unwrap();
/// let mut rcv1 = SOME_SIGNAL.receiver().unwrap();
/// assert!(SOME_SIGNAL.receiver().is_err());
///
/// SOME_SIGNAL.write(10);
///     
/// // Receive the new value
/// assert_eq!(rcv0.changed().await, 10);
/// assert_eq!(rcv1.try_changed(), Some(10));
///     
/// // No update
/// assert_eq!(rcv0.try_changed(), None);
/// assert_eq!(rcv1.try_changed(), None);
///
/// SOME_SIGNAL.write(20);
///
/// // Receive new value with predicate
/// assert_eq!(rcv0.changed_and(|x|x>&10).await, 20);
/// assert_eq!(rcv1.try_changed_and(|x|x>&30), None);
///
/// // Anyone can peek the current value
/// assert_eq!(rcv0.peek(), 20);
/// assert_eq!(rcv1.peek(), 20);
/// assert_eq!(SOME_SIGNAL.peek(), 20);
/// assert_eq!(SOME_SIGNAL.peek_and(|x|x>&30), None);
/// };
/// block_on(f);
/// ```
pub struct MultiSignal<M: RawMutex, T: Clone, const N: usize> {
    mutex: Mutex<M, RefCell<MultiSignalState<N, T>>>,
}

struct MultiSignalState<const N: usize, T: Clone> {
    data: T,
    current_id: u64,
    wakers: MultiWakerRegistration<N>,
    receiver_count: usize,
}

#[derive(Debug)]
/// An error that can occur when a `MultiSignal` returns a `Result`.
pub enum Error {
    /// The maximum number of [`Receiver`](crate::multi_signal::Receiver) has been reached.
    MaximumReceiversReached,
}

impl<'a, M: RawMutex, T: Clone, const N: usize> MultiSignal<M, T, N> {
    /// Create a new `MultiSignal` initialized with the given value.
    pub const fn new(init: T) -> Self {
        Self {
            mutex: Mutex::new(RefCell::new(MultiSignalState {
                data: init,
                current_id: 1,
                wakers: MultiWakerRegistration::new(),
                receiver_count: 0,
            })),
        }
    }

    /// Get a [`Receiver`] for the `MultiSignal`.
    pub fn receiver<'s>(&'a self) -> Result<Receiver<'a, M, T, N>, Error> {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            if s.receiver_count < N {
                s.receiver_count += 1;
                Ok(Receiver(Rcv::new(self)))
            } else {
                Err(Error::MaximumReceiversReached)
            }
        })
    }

    /// Update the value of the `MultiSignal`.
    pub fn write(&self, data: T) {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            s.data = data;
            s.current_id += 1;
            s.wakers.wake();
        })
    }

    /// Peek the current value of the `MultiSignal`.
    pub fn peek(&self) -> T {
        self.mutex.lock(|state| state.borrow().data.clone())
    }

    /// Peek the current value of the `MultiSignal` and check if it satisfies the predicate `f`.
    pub fn peek_and(&self, mut f: impl FnMut(&T) -> bool) -> Option<T> {
        self.mutex.lock(|state| {
            let s = state.borrow();
            if f(&s.data) {
                Some(s.data.clone())
            } else {
                None
            }
        })
    }

    /// Get the ID of the current value of the `MultiSignal`.
    /// This method is mostly for testing purposes.
    #[allow(dead_code)]
    fn get_id(&self) -> u64 {
        self.mutex.lock(|state| state.borrow().current_id)
    }
}

/// A receiver is able to `.await` a changed `MultiSignal` value.
pub struct Rcv<'a, M: RawMutex, T: Clone, const N: usize> {
    multi_sig: &'a MultiSignal<M, T, N>,
    at_id: u64,
}

impl<'s, 'a, M: RawMutex, T: Clone, const N: usize> Rcv<'a, M, T, N> {
    /// Create a new `Receiver` with a reference the given `MultiSignal`.
    fn new(multi_sig: &'a MultiSignal<M, T, N>) -> Self {
        Self { multi_sig, at_id: 0 }
    }

    /// Wait for a change to the value of the corresponding `MultiSignal`.
    pub async fn changed(&mut self) -> T {
        ReceiverWaitFuture { subscriber: self }.await
    }

    /// Wait for a change to the value of the corresponding `MultiSignal` which matches the predicate `f`.
    pub async fn changed_and<F>(&mut self, f: F) -> T
    where
        F: FnMut(&T) -> bool,
    {
        ReceiverPredFuture {
            subscriber: self,
            predicate: f,
        }
        .await
    }

    /// Try to get a changed value of the corresponding `MultiSignal`.
    pub fn try_changed(&mut self) -> Option<T> {
        self.multi_sig.mutex.lock(|state| {
            let s = state.borrow();
            match s.current_id > self.at_id {
                true => {
                    self.at_id = s.current_id;
                    Some(s.data.clone())
                }
                false => None,
            }
        })
    }

    /// Try to get a changed value of the corresponding `MultiSignal` which matches the predicate `f`.
    pub fn try_changed_and<F>(&mut self, mut f: F) -> Option<T>
    where
        F: FnMut(&T) -> bool,
    {
        self.multi_sig.mutex.lock(|state| {
            let s = state.borrow();
            match s.current_id > self.at_id && f(&s.data) {
                true => {
                    self.at_id = s.current_id;
                    Some(s.data.clone())
                }
                false => None,
            }
        })
    }

    /// Peek the current value of the corresponding `MultiSignal`.
    pub fn peek(&self) -> T {
        self.multi_sig.peek()
    }

    /// Peek the current value of the corresponding `MultiSignal` and check if it satisfies the predicate `f`.
    pub fn peek_and<F>(&self, f: F) -> Option<T>
    where
        F: FnMut(&T) -> bool,
    {
        self.multi_sig.peek_and(f)
    }

    /// Check if the value of the corresponding `MultiSignal` has changed.
    pub fn has_changed(&mut self) -> bool {
        self.multi_sig
            .mutex
            .lock(|state| state.borrow().current_id > self.at_id)
    }
}

/// A `Receiver` is able to `.await` a change to the corresponding [`MultiSignal`] value.
pub struct Receiver<'a, M: RawMutex, T: Clone, const N: usize>(Rcv<'a, M, T, N>);

impl<'s, 'a, M: RawMutex, T: Clone, const N: usize> Deref for Receiver<'a, M, T, N> {
    type Target = Rcv<'a, M, T, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'s, 'a, M: RawMutex, T: Clone, const N: usize> DerefMut for Receiver<'a, M, T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Future for the `Receiver` wait action
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ReceiverWaitFuture<'s, 'a, M: RawMutex, T: Clone, const N: usize> {
    subscriber: &'s mut Rcv<'a, M, T, N>,
}

impl<'s, 'a, M: RawMutex, T: Clone, const N: usize> Unpin for ReceiverWaitFuture<'s, 'a, M, T, N> {}
impl<'s, 'a, M: RawMutex, T: Clone, const N: usize> Future for ReceiverWaitFuture<'s, 'a, M, T, N> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_with_context(Some(cx))
    }
}

impl<'s, 'a, M: RawMutex, T: Clone, const N: usize> ReceiverWaitFuture<'s, 'a, M, T, N> {
    /// Poll the `MultiSignal` with an optional context.
    fn get_with_context(&mut self, cx: Option<&mut Context>) -> Poll<T> {
        self.subscriber.multi_sig.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            match s.current_id > self.subscriber.at_id {
                true => {
                    self.subscriber.at_id = s.current_id;
                    Poll::Ready(s.data.clone())
                }
                _ => {
                    if let Some(cx) = cx {
                        s.wakers.register(cx.waker());
                    }
                    Poll::Pending
                }
            }
        })
    }
}

/// Future for the `Receiver` wait action, with the ability to filter the value with a predicate.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ReceiverPredFuture<'s, 'a, M: RawMutex, T: Clone, F: FnMut(&'a T) -> bool, const N: usize> {
    subscriber: &'s mut Rcv<'a, M, T, N>,
    predicate: F,
}

impl<'s, 'a, M: RawMutex, T: Clone, F: FnMut(&T) -> bool, const N: usize> Unpin
    for ReceiverPredFuture<'s, 'a, M, T, F, N>
{
}
impl<'s, 'a, M: RawMutex, T: Clone, F: FnMut(&T) -> bool, const N: usize> Future
    for ReceiverPredFuture<'s, 'a, M, T, F, N>
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_with_context_pred(Some(cx))
    }
}

impl<'s, 'a, M: RawMutex, T: Clone, F: FnMut(&T) -> bool, const N: usize> ReceiverPredFuture<'s, 'a, M, T, F, N> {
    /// Poll the `MultiSignal` with an optional context.
    fn get_with_context_pred(&mut self, cx: Option<&mut Context>) -> Poll<T> {
        self.subscriber.multi_sig.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            match s.current_id > self.subscriber.at_id {
                true if (self.predicate)(&s.data) => {
                    self.subscriber.at_id = s.current_id;
                    Poll::Ready(s.data.clone())
                }
                _ => {
                    if let Some(cx) = cx {
                        s.wakers.register(cx.waker());
                    }
                    Poll::Pending
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use futures_executor::block_on;

    use super::*;
    use crate::blocking_mutex::raw::CriticalSectionRawMutex;

    #[test]
    fn multiple_writes() {
        let f = async {
            static SOME_SIGNAL: MultiSignal<CriticalSectionRawMutex, u8, 2> = MultiSignal::new(0);

            // Obtain Receivers
            let mut rcv0 = SOME_SIGNAL.receiver().unwrap();
            let mut rcv1 = SOME_SIGNAL.receiver().unwrap();

            SOME_SIGNAL.write(10);

            // Receive the new value
            assert_eq!(rcv0.changed().await, 10);
            assert_eq!(rcv1.changed().await, 10);

            // No update
            assert_eq!(rcv0.try_changed(), None);
            assert_eq!(rcv1.try_changed(), None);

            SOME_SIGNAL.write(20);

            assert_eq!(rcv0.changed().await, 20);
            assert_eq!(rcv1.changed().await, 20);
        };
        block_on(f);
    }

    #[test]
    fn max_receivers() {
        let f = async {
            static SOME_SIGNAL: MultiSignal<CriticalSectionRawMutex, u8, 2> = MultiSignal::new(0);

            // Obtain Receivers
            let _ = SOME_SIGNAL.receiver().unwrap();
            let _ = SOME_SIGNAL.receiver().unwrap();
            assert!(SOME_SIGNAL.receiver().is_err());
        };
        block_on(f);
    }

    // Really weird edge case, but it's possible to have a receiver that never gets a value.
    #[test]
    fn receive_initial() {
        let f = async {
            static SOME_SIGNAL: MultiSignal<CriticalSectionRawMutex, u8, 2> = MultiSignal::new(0);

            // Obtain Receivers
            let mut rcv0 = SOME_SIGNAL.receiver().unwrap();
            let mut rcv1 = SOME_SIGNAL.receiver().unwrap();

            assert_eq!(rcv0.try_changed(), Some(0));
            assert_eq!(rcv1.try_changed(), Some(0));

            assert_eq!(rcv0.try_changed(), None);
            assert_eq!(rcv1.try_changed(), None);
        };
        block_on(f);
    }

    #[test]
    fn count_ids() {
        let f = async {
            static SOME_SIGNAL: MultiSignal<CriticalSectionRawMutex, u8, 2> = MultiSignal::new(0);

            // Obtain Receivers
            let mut rcv0 = SOME_SIGNAL.receiver().unwrap();
            let mut rcv1 = SOME_SIGNAL.receiver().unwrap();

            SOME_SIGNAL.write(10);

            assert_eq!(rcv0.changed().await, 10);
            assert_eq!(rcv1.changed().await, 10);

            assert_eq!(rcv0.try_changed(), None);
            assert_eq!(rcv1.try_changed(), None);

            SOME_SIGNAL.write(20);
            SOME_SIGNAL.write(20);
            SOME_SIGNAL.write(20);

            assert_eq!(rcv0.changed().await, 20);
            assert_eq!(rcv1.changed().await, 20);

            assert_eq!(rcv0.try_changed(), None);
            assert_eq!(rcv1.try_changed(), None);

            assert_eq!(SOME_SIGNAL.get_id(), 5);
        };
        block_on(f);
    }

    #[test]
    fn peek_still_await() {
        let f = async {
            static SOME_SIGNAL: MultiSignal<CriticalSectionRawMutex, u8, 2> = MultiSignal::new(0);

            // Obtain Receivers
            let mut rcv0 = SOME_SIGNAL.receiver().unwrap();
            let mut rcv1 = SOME_SIGNAL.receiver().unwrap();

            SOME_SIGNAL.write(10);

            assert_eq!(rcv0.peek(), 10);
            assert_eq!(rcv1.peek(), 10);

            assert_eq!(rcv0.changed().await, 10);
            assert_eq!(rcv1.changed().await, 10);
        };
        block_on(f);
    }

    #[test]
    fn predicate() {
        let f = async {
            static SOME_SIGNAL: MultiSignal<CriticalSectionRawMutex, u8, 2> = MultiSignal::new(0);

            // Obtain Receivers
            let mut rcv0 = SOME_SIGNAL.receiver().unwrap();
            let mut rcv1 = SOME_SIGNAL.receiver().unwrap();

            SOME_SIGNAL.write(20);

            assert_eq!(rcv0.changed_and(|x| x > &10).await, 20);
            assert_eq!(rcv1.try_changed_and(|x| x > &30), None);
        };
        block_on(f);
    }

    #[test]
    fn mutable_predicate() {
        let f = async {
            static SOME_SIGNAL: MultiSignal<CriticalSectionRawMutex, u8, 2> = MultiSignal::new(0);

            // Obtain Receivers
            let mut rcv = SOME_SIGNAL.receiver().unwrap();

            SOME_SIGNAL.write(10);

            let mut largest = 0;
            let mut predicate = |x: &u8| {
                if *x > largest {
                    largest = *x;
                }
                true
            };

            assert_eq!(rcv.changed_and(&mut predicate).await, 10);

            SOME_SIGNAL.write(20);

            assert_eq!(rcv.changed_and(&mut predicate).await, 20);

            SOME_SIGNAL.write(5);

            assert_eq!(rcv.changed_and(&mut predicate).await, 5);

            assert_eq!(largest, 20)
        };
        block_on(f);
    }

    #[test]
    fn peek_and() {
        let f = async {
            static SOME_SIGNAL: MultiSignal<CriticalSectionRawMutex, u8, 2> = MultiSignal::new(0);

            // Obtain Receivers
            let mut rcv0 = SOME_SIGNAL.receiver().unwrap();
            let mut rcv1 = SOME_SIGNAL.receiver().unwrap();

            SOME_SIGNAL.write(20);

            assert_eq!(rcv0.peek_and(|x| x > &10), Some(20));
            assert_eq!(rcv1.peek_and(|x| x > &30), None);

            assert_eq!(rcv0.changed().await, 20);
            assert_eq!(rcv1.changed().await, 20);
        };
        block_on(f);
    }

    #[test]
    fn peek_with_static() {
        let f = async {
            static SOME_SIGNAL: MultiSignal<CriticalSectionRawMutex, u8, 2> = MultiSignal::new(0);

            // Obtain Receivers
            let rcv0 = SOME_SIGNAL.receiver().unwrap();
            let rcv1 = SOME_SIGNAL.receiver().unwrap();

            SOME_SIGNAL.write(20);

            assert_eq!(rcv0.peek(), 20);
            assert_eq!(rcv1.peek(), 20);
            assert_eq!(SOME_SIGNAL.peek(), 20);
            assert_eq!(SOME_SIGNAL.peek_and(|x| x > &30), None);
        };
        block_on(f);
    }
}
