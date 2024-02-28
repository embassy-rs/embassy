//! A synchronization primitive for passing the latest value to **multiple** tasks.

use core::cell::RefCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::task::{Context, Poll};

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;
use crate::waitqueue::MultiWakerRegistration;

/// A `Watch` is a single-slot signaling primitive, which can awake `N` up to separate [`Receiver`]s.
///
/// Similar to a [`Signal`](crate::signal::Signal), except `Watch` allows for multiple tasks to
/// `.await` the latest value, and all receive it.
///
/// This is similar to a [`PubSubChannel`](crate::pubsub::PubSubChannel) with a buffer size of 1, except
/// "sending" to it (calling [`Watch::write`]) will immediately overwrite the previous value instead
/// of waiting for the receivers to pop the previous value.
///
/// `Watch` is useful when a single task is responsible for updating a value or "state", which multiple other
/// tasks are interested in getting notified about changes to the latest value of. It is therefore fine for
/// [`Receiver`]s to "lose" stale values.
///
/// Anyone with a reference to the Watch can update or peek the value. Watches are generally declared
/// as `static`s and then borrowed as required to either [`Watch::peek`] the value or obtain a [`Receiver`]
/// with [`Watch::receiver`] which has async methods.
/// ```
///
/// use futures_executor::block_on;
/// use embassy_sync::watch::Watch;
/// use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
///
/// let f = async {
///
/// static WATCH: Watch<CriticalSectionRawMutex, u8, 2> = Watch::new();
///
/// // Obtain Receivers
/// let mut rcv0 = WATCH.receiver().unwrap();
/// let mut rcv1 = WATCH.receiver().unwrap();
/// assert!(WATCH.receiver().is_err());
///
/// assert_eq!(rcv1.try_changed(), None);
///
/// WATCH.write(10);
/// assert_eq!(WATCH.try_peek(), Some(10));
///
///     
/// // Receive the new value
/// assert_eq!(rcv0.changed().await, 10);
/// assert_eq!(rcv1.try_changed(), Some(10));
///     
/// // No update
/// assert_eq!(rcv0.try_changed(), None);
/// assert_eq!(rcv1.try_changed(), None);
///
/// WATCH.write(20);
///
/// // Defference `between` peek `get`.
/// assert_eq!(rcv0.peek().await, 20);
/// assert_eq!(rcv1.get().await, 20);
///
/// assert_eq!(rcv0.try_changed(), Some(20));
/// assert_eq!(rcv1.try_changed(), None);
///
/// };
/// block_on(f);
/// ```
pub struct Watch<M: RawMutex, T: Clone, const N: usize> {
    mutex: Mutex<M, RefCell<WatchState<N, T>>>,
}

struct WatchState<const N: usize, T: Clone> {
    data: Option<T>,
    current_id: u64,
    wakers: MultiWakerRegistration<N>,
    receiver_count: usize,
}

/// A trait representing the 'inner' behavior of the `Watch`.
pub trait WatchBehavior<T: Clone> {
    /// Poll the `Watch` for the current value, **without** making it as seen.
    fn inner_poll_peek(&self, cx: &mut Context<'_>) -> Poll<T>;

    /// Tries to peek the value of the `Watch`, **without** marking it as seen.
    fn inner_try_peek(&self) -> Option<T>;

    /// Poll the `Watch` for the current value, making it as seen.
    fn inner_poll_get(&self, id: &mut u64, cx: &mut Context<'_>) -> Poll<T>;

    /// Tries to get the value of the `Watch`, marking it as seen.
    fn inner_try_get(&self, id: &mut u64) -> Option<T>;

    /// Poll the `Watch` for a changed value, marking it as seen.
    fn inner_poll_changed(&self, id: &mut u64, cx: &mut Context<'_>) -> Poll<T>;

    /// Tries to retrieve the value of the `Watch` if it has changed, marking it as seen.
    fn inner_try_changed(&self, id: &mut u64) -> Option<T>;

    /// Checks if the `Watch` is been initialized with a value.
    fn inner_contains_value(&self) -> bool;
}

impl<M: RawMutex, T: Clone, const N: usize> WatchBehavior<T> for Watch<M, T, N> {
    fn inner_poll_peek(&self, cx: &mut Context<'_>) -> Poll<T> {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            match &s.data {
                Some(data) => Poll::Ready(data.clone()),
                None => {
                    s.wakers.register(cx.waker());
                    Poll::Pending
                }
            }
        })
    }

    fn inner_try_peek(&self) -> Option<T> {
        self.mutex.lock(|state| state.borrow().data.clone())
    }

    fn inner_poll_get(&self, id: &mut u64, cx: &mut Context<'_>) -> Poll<T> {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            match &s.data {
                Some(data) => {
                    *id = s.current_id;
                    Poll::Ready(data.clone())
                }
                None => {
                    s.wakers.register(cx.waker());
                    Poll::Pending
                }
            }
        })
    }

    fn inner_try_get(&self, id: &mut u64) -> Option<T> {
        self.mutex.lock(|state| {
            let s = state.borrow();
            *id = s.current_id;
            state.borrow().data.clone()
        })
    }

    fn inner_poll_changed(&self, id: &mut u64, cx: &mut Context<'_>) -> Poll<T> {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            match (&s.data, s.current_id > *id) {
                (Some(data), true) => {
                    *id = s.current_id;
                    Poll::Ready(data.clone())
                }
                _ => {
                    s.wakers.register(cx.waker());
                    Poll::Pending
                }
            }
        })
    }

    fn inner_try_changed(&self, id: &mut u64) -> Option<T> {
        self.mutex.lock(|state| {
            let s = state.borrow();
            match s.current_id > *id {
                true => {
                    *id = s.current_id;
                    state.borrow().data.clone()
                }
                false => None,
            }
        })
    }

    fn inner_contains_value(&self) -> bool {
        self.mutex.lock(|state| state.borrow().data.is_some())
    }
}

#[derive(Debug)]
/// An error that can occur when a `Watch` returns a `Result::Err(_)`.
pub enum Error {
    /// The maximum number of [`Receiver`](crate::watch::Receiver)/[`DynReceiver`](crate::watch::DynReceiver) has been reached.
    MaximumReceiversReached,
}

impl<'a, M: RawMutex, T: Clone, const N: usize> Watch<M, T, N> {
    /// Create a new `Watch` channel.
    pub const fn new() -> Self {
        Self {
            mutex: Mutex::new(RefCell::new(WatchState {
                data: None,
                current_id: 0,
                wakers: MultiWakerRegistration::new(),
                receiver_count: 0,
            })),
        }
    }

    /// Write a new value to the `Watch`.
    pub fn write(&self, val: T) {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            s.data = Some(val);
            s.current_id += 1;
            s.wakers.wake();
        })
    }

    /// Create a new [`Receiver`] for the `Watch`.
    pub fn receiver(&self) -> Result<Receiver<'_, M, T, N>, Error> {
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

    /// Create a new [`DynReceiver`] for the `Watch`.
    pub fn dyn_receiver(&self) -> Result<DynReceiver<'_, T>, Error> {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            if s.receiver_count < N {
                s.receiver_count += 1;
                Ok(DynReceiver(Rcv::new(self)))
            } else {
                Err(Error::MaximumReceiversReached)
            }
        })
    }

    /// Tries to retrieve the value of the `Watch`.
    pub fn try_peek(&self) -> Option<T> {
        self.inner_try_peek()
    }

    /// Returns true if the `Watch` contains a value.
    pub fn contains_value(&self) -> bool {
        self.inner_contains_value()
    }

    /// Clears the value of the `Watch`. This will cause calls to [`Rcv::get`] and [`Rcv::peek`] to be pending.
    pub fn clear(&self) {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            s.data = None;
        })
    }
}

/// A receiver can `.await` a change in the `Watch` value.
pub struct Rcv<'a, T: Clone, W: WatchBehavior<T> + ?Sized> {
    watch: &'a W,
    at_id: u64,
    _phantom: PhantomData<T>,
}

impl<'a, T: Clone, W: WatchBehavior<T> + ?Sized> Rcv<'a, T, W> {
    /// Creates a new `Receiver` with a reference to the `Watch`.
    fn new(watch: &'a W) -> Self {
        Self {
            watch,
            at_id: 0,
            _phantom: PhantomData,
        }
    }

    /// Returns the current value of the `Watch` if it is initialized, **without** marking it as seen.
    pub async fn peek(&self) -> T {
        poll_fn(|cx| self.watch.inner_poll_peek(cx)).await
    }

    /// Tries to peek the current value of the `Watch` without waiting, and **without** marking it as seen.
    pub fn try_peek(&self) -> Option<T> {
        self.watch.inner_try_peek()
    }

    /// Returns the current value of the `Watch` if it is initialized, marking it as seen.
    pub async fn get(&mut self) -> T {
        poll_fn(|cx| self.watch.inner_poll_get(&mut self.at_id, cx)).await
    }

    /// Tries to get the current value of the `Watch` without waiting, marking it as seen.
    pub fn try_get(&mut self) -> Option<T> {
        self.watch.inner_try_get(&mut self.at_id)
    }

    /// Waits for the `Watch` to change and returns the new value, marking it as seen.
    pub async fn changed(&mut self) -> T {
        poll_fn(|cx| self.watch.inner_poll_changed(&mut self.at_id, cx)).await
    }

    /// Tries to get the new value of the watch without waiting, marking it as seen.
    pub fn try_changed(&mut self) -> Option<T> {
        self.watch.inner_try_changed(&mut self.at_id)
    }

    /// Checks if the `Watch` contains a value. If this returns true,
    /// then awaiting [`Rcv::get`] and [`Rcv::peek`] will return immediately.
    pub fn contains_value(&self) -> bool {
        self.watch.inner_contains_value()
    }
}

/// A receiver of a `Watch` channel.
pub struct Receiver<'a, M: RawMutex, T: Clone, const N: usize>(Rcv<'a, T, Watch<M, T, N>>);

/// A receiver which holds a **reference** to a `Watch` channel.
///
/// This is an alternative to [`Receiver`] with a simpler type definition, at the expense of
/// some runtime performance due to dynamic dispatch.
pub struct DynReceiver<'a, T: Clone>(Rcv<'a, T, dyn WatchBehavior<T> + 'a>);

impl<'a, M: RawMutex, T: Clone, const N: usize> Deref for Receiver<'a, M, T, N> {
    type Target = Rcv<'a, T, Watch<M, T, N>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, M: RawMutex, T: Clone, const N: usize> DerefMut for Receiver<'a, M, T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: Clone> Deref for DynReceiver<'a, T> {
    type Target = Rcv<'a, T, dyn WatchBehavior<T> + 'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: Clone> DerefMut for DynReceiver<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
            static WATCH: Watch<CriticalSectionRawMutex, u8, 2> = Watch::new();

            // Obtain Receivers
            let mut rcv0 = WATCH.receiver().unwrap();
            let mut rcv1 = WATCH.dyn_receiver().unwrap();

            WATCH.write(10);

            // Receive the new value
            assert_eq!(rcv0.changed().await, 10);
            assert_eq!(rcv1.changed().await, 10);

            // No update
            assert_eq!(rcv0.try_changed(), None);
            assert_eq!(rcv1.try_changed(), None);

            WATCH.write(20);

            assert_eq!(rcv0.changed().await, 20);
            assert_eq!(rcv1.changed().await, 20);
        };
        block_on(f);
    }

    #[test]
    fn max_receivers() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 2> = Watch::new();

            // Obtain Receivers
            let _ = WATCH.receiver().unwrap();
            let _ = WATCH.receiver().unwrap();
            assert!(WATCH.receiver().is_err());
        };
        block_on(f);
    }

    #[test]
    fn receive_initial() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 2> = Watch::new();

            // Obtain Receivers
            let mut rcv0 = WATCH.receiver().unwrap();
            let mut rcv1 = WATCH.receiver().unwrap();

            assert_eq!(rcv0.contains_value(), false);

            assert_eq!(rcv0.try_changed(), None);
            assert_eq!(rcv1.try_changed(), None);

            WATCH.write(0);

            assert_eq!(rcv0.contains_value(), true);

            assert_eq!(rcv0.try_changed(), Some(0));
            assert_eq!(rcv1.try_changed(), Some(0));
        };
        block_on(f);
    }

    #[test]
    fn peek_get_changed() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 2> = Watch::new();

            // Obtain Receivers
            let mut rcv0 = WATCH.receiver().unwrap();

            WATCH.write(10);

            // Ensure peek does not mark as seen
            assert_eq!(rcv0.peek().await, 10);
            assert_eq!(rcv0.try_changed(), Some(10));
            assert_eq!(rcv0.try_changed(), None);
            assert_eq!(rcv0.peek().await, 10);

            WATCH.write(20);

            // Ensure get does mark as seen
            assert_eq!(rcv0.get().await, 20);
            assert_eq!(rcv0.try_changed(), None);
            assert_eq!(rcv0.try_get(), Some(20));
        };
        block_on(f);
    }

    #[test]
    fn count_ids() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 2> = Watch::new();

            // Obtain Receivers
            let mut rcv0 = WATCH.receiver().unwrap();
            let mut rcv1 = WATCH.receiver().unwrap();

            let get_id = || WATCH.mutex.lock(|state| state.borrow().current_id);

            WATCH.write(10);

            assert_eq!(rcv0.changed().await, 10);
            assert_eq!(rcv1.changed().await, 10);

            assert_eq!(rcv0.try_changed(), None);
            assert_eq!(rcv1.try_changed(), None);

            WATCH.write(20);
            WATCH.write(20);
            WATCH.write(20);

            assert_eq!(rcv0.changed().await, 20);
            assert_eq!(rcv1.changed().await, 20);

            assert_eq!(rcv0.try_changed(), None);
            assert_eq!(rcv1.try_changed(), None);

            assert_eq!(get_id(), 4);
        };
        block_on(f);
    }

    #[test]
    fn peek_still_await() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 2> = Watch::new();

            // Obtain Receivers
            let mut rcv0 = WATCH.receiver().unwrap();
            let mut rcv1 = WATCH.receiver().unwrap();

            WATCH.write(10);

            assert_eq!(rcv0.peek().await, 10);
            assert_eq!(rcv1.try_peek(), Some(10));

            assert_eq!(rcv0.changed().await, 10);
            assert_eq!(rcv1.changed().await, 10);
        };
        block_on(f);
    }

    #[test]
    fn peek_with_static() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 2> = Watch::new();

            // Obtain Receivers
            let rcv0 = WATCH.receiver().unwrap();
            let rcv1 = WATCH.receiver().unwrap();

            WATCH.write(20);

            assert_eq!(rcv0.peek().await, 20);
            assert_eq!(rcv1.peek().await, 20);
            assert_eq!(WATCH.try_peek(), Some(20));
        };
        block_on(f);
    }
}
