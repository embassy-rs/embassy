//! A synchronization primitive for passing the latest value to **multiple** receivers.

use core::cell::RefCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::task::{Context, Poll};

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;
use crate::waitqueue::MultiWakerRegistration;

/// The `Watch` is a single-slot signaling primitive that allows multiple receivers to concurrently await
/// changes to the value. Unlike a [`Signal`](crate::signal::Signal), `Watch` supports multiple receivers,
/// and unlike a [`PubSubChannel`](crate::pubsub::PubSubChannel), `Watch` immediately overwrites the previous
/// value when a new one is sent, without waiting for all receivers to read the previous value.
///
/// This makes `Watch` particularly useful when a single task updates a value or "state", and multiple other tasks
/// need to be notified about changes to this value asynchronously. Receivers may "lose" stale values, as they are
/// always provided with the latest value.
///
/// Typically, `Watch` instances are declared as `static`, and a [`Sender`] and [`Receiver`]
/// (or [`DynSender`] and/or [`DynReceiver`]) are obtained and passed to the relevant parts of the program.
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
/// // Obtain receivers and sender
/// let mut rcv0 = WATCH.receiver().unwrap();
/// let mut rcv1 = WATCH.dyn_receiver().unwrap();
/// let mut snd = WATCH.sender();
///
/// // No more receivers, and no update
/// assert!(WATCH.receiver().is_err());
/// assert_eq!(rcv1.try_changed(), None);
///
/// snd.send(10);
///     
/// // Receive the new value (async or try)
/// assert_eq!(rcv0.changed().await, 10);
/// assert_eq!(rcv1.try_changed(), Some(10));
///     
/// // No update
/// assert_eq!(rcv0.try_changed(), None);
/// assert_eq!(rcv1.try_changed(), None);
///
/// snd.send(20);
///
/// // Peek does not mark the value as seen
/// assert_eq!(rcv0.peek().await, 20);
/// assert_eq!(rcv0.try_changed(), Some(20));
///
/// // Get marks the value as seen
/// assert_eq!(rcv1.get().await, 20);
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
    /// Sends a new value to the `Watch`.
    fn send(&self, val: T);

    /// Clears the value of the `Watch`.
    fn clear(&self);

    /// Poll the `Watch` for the current value, **without** making it as seen.
    fn poll_peek(&self, cx: &mut Context<'_>) -> Poll<T>;

    /// Tries to peek the value of the `Watch`, **without** marking it as seen.
    fn try_peek(&self) -> Option<T>;

    /// Poll the `Watch` for the current value, making it as seen.
    fn poll_get(&self, id: &mut u64, cx: &mut Context<'_>) -> Poll<T>;

    /// Tries to get the value of the `Watch`, marking it as seen.
    fn try_get(&self, id: &mut u64) -> Option<T>;

    /// Poll the `Watch` for a changed value, marking it as seen.
    fn poll_changed(&self, id: &mut u64, cx: &mut Context<'_>) -> Poll<T>;

    /// Tries to retrieve the value of the `Watch` if it has changed, marking it as seen.
    fn try_changed(&self, id: &mut u64) -> Option<T>;

    /// Checks if the `Watch` is been initialized with a value.
    fn contains_value(&self) -> bool;

    /// Used when a receiver is dropped to decrement the receiver count.
    ///
    /// ## This method should not be called by the user.
    fn drop_receiver(&self);
}

impl<M: RawMutex, T: Clone, const N: usize> WatchBehavior<T> for Watch<M, T, N> {
    fn send(&self, val: T) {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            s.data = Some(val);
            s.current_id += 1;
            s.wakers.wake();
        })
    }

    fn clear(&self) {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            s.data = None;
        })
    }

    fn poll_peek(&self, cx: &mut Context<'_>) -> Poll<T> {
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

    fn try_peek(&self) -> Option<T> {
        self.mutex.lock(|state| state.borrow().data.clone())
    }

    fn poll_get(&self, id: &mut u64, cx: &mut Context<'_>) -> Poll<T> {
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

    fn try_get(&self, id: &mut u64) -> Option<T> {
        self.mutex.lock(|state| {
            let s = state.borrow();
            *id = s.current_id;
            state.borrow().data.clone()
        })
    }

    fn poll_changed(&self, id: &mut u64, cx: &mut Context<'_>) -> Poll<T> {
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

    fn try_changed(&self, id: &mut u64) -> Option<T> {
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

    fn contains_value(&self) -> bool {
        self.mutex.lock(|state| state.borrow().data.is_some())
    }

    fn drop_receiver(&self) {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            s.receiver_count -= 1;
        })
    }
}

impl<M: RawMutex, T: Clone, const N: usize> Watch<M, T, N> {
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

    /// Create a new [`Receiver`] for the `Watch`.
    pub fn sender(&self) -> Sender<'_, M, T, N> {
        Sender(Snd::new(self))
    }

    /// Create a new [`DynReceiver`] for the `Watch`.
    pub fn dyn_sender(&self) -> DynSender<'_, T> {
        DynSender(Snd::new(self))
    }

    /// Try to create a new [`Receiver`] for the `Watch`. If the
    /// maximum number of receivers has been reached, `None` is returned.
    pub fn receiver(&self) -> Option<Receiver<'_, M, T, N>> {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            if s.receiver_count < N {
                s.receiver_count += 1;
                Some(Receiver(Rcv::new(self, 0)))
            } else {
                None
            }
        })
    }

    /// Try to create a new [`DynReceiver`] for the `Watch`. If the
    /// maximum number of receivers has been reached, `None` is returned.
    pub fn dyn_receiver(&self) -> Option<DynReceiver<'_, T>> {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            if s.receiver_count < N {
                s.receiver_count += 1;
                Some(DynReceiver(Rcv::new(self, 0)))
            } else {
                None
            }
        })
    }
}

/// A receiver can `.await` a change in the `Watch` value.
pub struct Snd<'a, T: Clone, W: WatchBehavior<T> + ?Sized> {
    watch: &'a W,
    _phantom: PhantomData<T>,
}

impl<'a, T: Clone, W: WatchBehavior<T> + ?Sized> Clone for Snd<'a, T, W> {
    fn clone(&self) -> Self {
        Self {
            watch: self.watch,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Clone, W: WatchBehavior<T> + ?Sized> Snd<'a, T, W> {
    /// Creates a new `Receiver` with a reference to the `Watch`.
    fn new(watch: &'a W) -> Self {
        Self {
            watch,
            _phantom: PhantomData,
        }
    }

    /// Sends a new value to the `Watch`.
    pub fn send(&self, val: T) {
        self.watch.send(val)
    }

    /// Clears the value of the `Watch`.
    /// This will cause calls to [`Rcv::get`] and [`Rcv::peek`] to be pending.
    pub fn clear(&self) {
        self.watch.clear()
    }

    /// Tries to retrieve the value of the `Watch`.
    pub fn try_peek(&self) -> Option<T> {
        self.watch.try_peek()
    }

    /// Returns true if the `Watch` contains a value.
    pub fn contains_value(&self) -> bool {
        self.watch.contains_value()
    }
}

/// A sender of a `Watch` channel.
///
/// For a simpler type definition, consider [`DynSender`] at the expense of
/// some runtime performance due to dynamic dispatch.
pub struct Sender<'a, M: RawMutex, T: Clone, const N: usize>(Snd<'a, T, Watch<M, T, N>>);

impl<'a, M: RawMutex, T: Clone, const N: usize> Clone for Sender<'a, M, T, N> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'a, M: RawMutex, T: Clone, const N: usize> Sender<'a, M, T, N> {
    /// Converts the `Sender` into a [`DynSender`].
    pub fn as_dyn(self) -> DynSender<'a, T> {
        DynSender(Snd::new(self.watch))
    }
}

impl<'a, M: RawMutex, T: Clone, const N: usize> Into<DynSender<'a, T>> for Sender<'a, M, T, N> {
    fn into(self) -> DynSender<'a, T> {
        self.as_dyn()
    }
}

impl<'a, M: RawMutex, T: Clone, const N: usize> Deref for Sender<'a, M, T, N> {
    type Target = Snd<'a, T, Watch<M, T, N>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, M: RawMutex, T: Clone, const N: usize> DerefMut for Sender<'a, M, T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A sender which holds a **dynamic** reference to a `Watch` channel.
///
/// This is an alternative to [`Sender`] with a simpler type definition,
pub struct DynSender<'a, T: Clone>(Snd<'a, T, dyn WatchBehavior<T> + 'a>);

impl<'a, T: Clone> Clone for DynSender<'a, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'a, T: Clone> Deref for DynSender<'a, T> {
    type Target = Snd<'a, T, dyn WatchBehavior<T> + 'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: Clone> DerefMut for DynSender<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
    fn new(watch: &'a W, at_id: u64) -> Self {
        Self {
            watch,
            at_id,
            _phantom: PhantomData,
        }
    }

    /// Returns the current value of the `Watch` once it is initialized, **without** marking it as seen.
    ///
    /// **Note**: Futures do nothing unless you `.await` or poll them.
    pub async fn peek(&self) -> T {
        poll_fn(|cx| self.watch.poll_peek(cx)).await
    }

    /// Tries to peek the current value of the `Watch` without waiting, and **without** marking it as seen.
    pub fn try_peek(&self) -> Option<T> {
        self.watch.try_peek()
    }

    /// Returns the current value of the `Watch` once it is initialized, marking it as seen.
    ///
    /// **Note**: Futures do nothing unless you `.await` or poll them.
    pub async fn get(&mut self) -> T {
        poll_fn(|cx| self.watch.poll_get(&mut self.at_id, cx)).await
    }

    /// Tries to get the current value of the `Watch` without waiting, marking it as seen.
    pub fn try_get(&mut self) -> Option<T> {
        self.watch.try_get(&mut self.at_id)
    }

    /// Waits for the `Watch` to change and returns the new value, marking it as seen.
    ///
    /// **Note**: Futures do nothing unless you `.await` or poll them.
    pub async fn changed(&mut self) -> T {
        poll_fn(|cx| self.watch.poll_changed(&mut self.at_id, cx)).await
    }

    /// Tries to get the new value of the watch without waiting, marking it as seen.
    pub fn try_changed(&mut self) -> Option<T> {
        self.watch.try_changed(&mut self.at_id)
    }

    /// Checks if the `Watch` contains a value. If this returns true,
    /// then awaiting [`Rcv::get`] and [`Rcv::peek`] will return immediately.
    pub fn contains_value(&self) -> bool {
        self.watch.contains_value()
    }
}

impl<'a, T: Clone, W: WatchBehavior<T> + ?Sized> Drop for Rcv<'a, T, W> {
    fn drop(&mut self) {
        self.watch.drop_receiver();
    }
}

/// A receiver of a `Watch` channel.
pub struct Receiver<'a, M: RawMutex, T: Clone, const N: usize>(Rcv<'a, T, Watch<M, T, N>>);

impl<'a, M: RawMutex, T: Clone, const N: usize> Receiver<'a, M, T, N> {
    /// Converts the `Receiver` into a [`DynReceiver`].
    pub fn as_dyn(self) -> DynReceiver<'a, T> {
        // We need to increment the receiver count since the original
        // receiver is being dropped, which decrements the count.
        self.watch.mutex.lock(|state| {
            state.borrow_mut().receiver_count += 1;
        });
        DynReceiver(Rcv::new(self.0.watch, self.at_id))
    }
}

impl<'a, M: RawMutex, T: Clone, const N: usize> Into<DynReceiver<'a, T>> for Receiver<'a, M, T, N> {
    fn into(self) -> DynReceiver<'a, T> {
        self.as_dyn()
    }
}

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

/// A receiver which holds a **dynamic** reference to a `Watch` channel.
///
/// This is an alternative to [`Receiver`] with a simpler type definition, at the expense of
/// some runtime performance due to dynamic dispatch.
pub struct DynReceiver<'a, T: Clone>(Rcv<'a, T, dyn WatchBehavior<T> + 'a>);

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

    use super::Watch;
    use crate::blocking_mutex::raw::CriticalSectionRawMutex;

    #[test]
    fn multiple_sends() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 1> = Watch::new();

            // Obtain receiver and sender
            let mut rcv = WATCH.receiver().unwrap();
            let snd = WATCH.sender();

            // Not initialized
            assert_eq!(rcv.try_changed(), None);

            // Receive the new value
            snd.send(10);
            assert_eq!(rcv.changed().await, 10);

            // Receive another value
            snd.send(20);
            assert_eq!(rcv.try_changed(), Some(20));

            // No update
            assert_eq!(rcv.try_changed(), None);
        };
        block_on(f);
    }

    #[test]
    fn receive_after_create() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 1> = Watch::new();

            // Obtain sender and send value
            let snd = WATCH.sender();
            snd.send(10);

            // Obtain receiver and receive value
            let mut rcv = WATCH.receiver().unwrap();
            assert_eq!(rcv.try_changed(), Some(10));
        };
        block_on(f);
    }

    #[test]
    fn max_receivers_drop() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 2> = Watch::new();

            // Try to create 3 receivers (only 2 can exist at once)
            let rcv0 = WATCH.receiver();
            let rcv1 = WATCH.receiver();
            let rcv2 = WATCH.receiver();

            // Ensure the first two are successful and the third is not
            assert!(rcv0.is_some());
            assert!(rcv1.is_some());
            assert!(rcv2.is_none());

            // Drop the first receiver
            drop(rcv0);

            // Create another receiver and ensure it is successful
            let rcv3 = WATCH.receiver();
            assert!(rcv3.is_some());
        };
        block_on(f);
    }

    #[test]
    fn multiple_receivers() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 2> = Watch::new();

            // Obtain receivers and sender
            let mut rcv0 = WATCH.receiver().unwrap();
            let mut rcv1 = WATCH.receiver().unwrap();
            let snd = WATCH.sender();

            // No update for both
            assert_eq!(rcv0.try_changed(), None);
            assert_eq!(rcv1.try_changed(), None);

            // Send a new value
            snd.send(0);

            // Both receivers receive the new value
            assert_eq!(rcv0.try_changed(), Some(0));
            assert_eq!(rcv1.try_changed(), Some(0));
        };
        block_on(f);
    }

    #[test]
    fn clone_senders() {
        let f = async {
            // Obtain different ways to send
            static WATCH: Watch<CriticalSectionRawMutex, u8, 1> = Watch::new();
            let snd0 = WATCH.sender();
            let snd1 = snd0.clone();

            // Obtain Receiver
            let mut rcv = WATCH.receiver().unwrap().as_dyn();

            // Send a value from first sender
            snd0.send(10);
            assert_eq!(rcv.try_changed(), Some(10));

            // Send a value from second sender
            snd1.send(20);
            assert_eq!(rcv.try_changed(), Some(20));
        };
        block_on(f);
    }

    #[test]
    fn peek_get_changed() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 2> = Watch::new();

            // Obtain receiver and sender
            let mut rcv = WATCH.receiver().unwrap();
            let snd = WATCH.sender();

            // Send a value
            snd.send(10);

            // Ensure peek does not mark as seen
            assert_eq!(rcv.peek().await, 10);
            assert_eq!(rcv.try_changed(), Some(10));
            assert_eq!(rcv.try_changed(), None);
            assert_eq!(rcv.try_peek(), Some(10));

            // Send a value
            snd.send(20);

            // Ensure get does mark as seen
            assert_eq!(rcv.get().await, 20);
            assert_eq!(rcv.try_changed(), None);
            assert_eq!(rcv.try_get(), Some(20));
        };
        block_on(f);
    }

    #[test]
    fn use_dynamics() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 2> = Watch::new();

            // Obtain receiver and sender
            let mut dyn_rcv = WATCH.dyn_receiver().unwrap();
            let dyn_snd = WATCH.dyn_sender();

            // Send a value
            dyn_snd.send(10);

            // Ensure the dynamic receiver receives the value
            assert_eq!(dyn_rcv.try_changed(), Some(10));
            assert_eq!(dyn_rcv.try_changed(), None);
        };
        block_on(f);
    }

    #[test]
    fn convert_to_dyn() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 2> = Watch::new();

            // Obtain receiver and sender
            let rcv = WATCH.receiver().unwrap();
            let snd = WATCH.sender();

            // Convert to dynamic
            let mut dyn_rcv = rcv.as_dyn();
            let dyn_snd = snd.as_dyn();

            // Send a value
            dyn_snd.send(10);

            // Ensure the dynamic receiver receives the value
            assert_eq!(dyn_rcv.try_changed(), Some(10));
            assert_eq!(dyn_rcv.try_changed(), None);
        };
        block_on(f);
    }

    #[test]
    fn dynamic_receiver_count() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 2> = Watch::new();

            // Obtain receiver and sender
            let rcv0 = WATCH.receiver();
            let rcv1 = WATCH.receiver();
            let rcv2 = WATCH.receiver();

            // Ensure the first two are successful and the third is not
            assert!(rcv0.is_some());
            assert!(rcv1.is_some());
            assert!(rcv2.is_none());

            // Convert to dynamic
            let dyn_rcv0 = rcv0.unwrap().as_dyn();

            // Drop the (now dynamic) receiver
            drop(dyn_rcv0);

            // Create another receiver and ensure it is successful
            let rcv3 = WATCH.receiver();
            let rcv4 = WATCH.receiver();
            assert!(rcv3.is_some());
            assert!(rcv4.is_none());
        };
        block_on(f);
    }

    #[test]
    fn contains_value() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 2> = Watch::new();

            // Obtain receiver and sender
            let rcv = WATCH.receiver().unwrap();
            let snd = WATCH.sender();

            // check if the watch contains a value
            assert_eq!(rcv.contains_value(), false);
            assert_eq!(snd.contains_value(), false);

            // Send a value
            snd.send(10);

            // check if the watch contains a value
            assert_eq!(rcv.contains_value(), true);
            assert_eq!(snd.contains_value(), true);
        };
        block_on(f);
    }
}
