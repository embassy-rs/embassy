//! A synchronization primitive for passing the latest value to **multiple** receivers.

use core::cell::RefCell;
use core::future::{poll_fn, Future};
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
/// (or [`DynSender`] and/or [`DynReceiver`]) are obtained where relevant. An [`AnonReceiver`]
/// and [`DynAnonReceiver`] are also available, which do not increase the receiver count for the
/// channel, and unwrapping is therefore not required, but it is not possible to `.await` the channel.
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
/// assert!(WATCH.receiver().is_none());
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
/// // Using `get` marks the value as seen
/// assert_eq!(rcv1.get().await, 20);
/// assert_eq!(rcv1.try_changed(), None);
///
/// // But `get` also returns when unchanged
/// assert_eq!(rcv1.get().await, 20);
/// assert_eq!(rcv1.get().await, 20);
///
/// };
/// block_on(f);
/// ```
pub struct Watch<M: RawMutex, T: Clone, const N: usize> {
    mutex: Mutex<M, RefCell<WatchState<T, N>>>,
}

struct WatchState<T: Clone, const N: usize> {
    data: Option<T>,
    current_id: u64,
    wakers: MultiWakerRegistration<N>,
    receiver_count: usize,
}

trait SealedWatchBehavior<T> {
    /// Poll the `Watch` for the current value, making it as seen.
    fn poll_get(&self, id: &mut u64, cx: &mut Context<'_>) -> Poll<T>;

    /// Poll the `Watch` for the value if it matches the predicate function
    /// `f`, making it as seen.
    fn poll_get_and(&self, id: &mut u64, f: &mut dyn Fn(&T) -> bool, cx: &mut Context<'_>) -> Poll<T>;

    /// Poll the `Watch` for a changed value, marking it as seen, if an id is given.
    fn poll_changed(&self, id: &mut u64, cx: &mut Context<'_>) -> Poll<T>;

    /// Tries to retrieve the value of the `Watch` if it has changed, marking it as seen.
    fn try_changed(&self, id: &mut u64) -> Option<T>;

    /// Poll the `Watch` for a changed value that matches the predicate function
    /// `f`, marking it as seen.
    fn poll_changed_and(&self, id: &mut u64, f: &mut dyn Fn(&T) -> bool, cx: &mut Context<'_>) -> Poll<T>;

    /// Tries to retrieve the value of the `Watch` if it has changed and matches the
    /// predicate function `f`, marking it as seen.
    fn try_changed_and(&self, id: &mut u64, f: &mut dyn Fn(&T) -> bool) -> Option<T>;

    /// Used when a receiver is dropped to decrement the receiver count.
    ///
    /// ## This method should not be called by the user.
    fn drop_receiver(&self);

    /// Clears the value of the `Watch`.
    fn clear(&self);

    /// Sends a new value to the `Watch`.
    fn send(&self, val: T);

    /// Modify the value of the `Watch` using a closure. Returns `false` if the
    /// `Watch` does not already contain a value.
    fn send_modify(&self, f: &mut dyn Fn(&mut Option<T>));

    /// Modify the value of the `Watch` using a closure. Returns `false` if the
    /// `Watch` does not already contain a value.
    fn send_if_modified(&self, f: &mut dyn Fn(&mut Option<T>) -> bool);
}

/// A trait representing the 'inner' behavior of the `Watch`.
#[allow(private_bounds)]
pub trait WatchBehavior<T: Clone>: SealedWatchBehavior<T> {
    /// Tries to get the value of the `Watch`, marking it as seen, if an id is given.
    fn try_get(&self, id: Option<&mut u64>) -> Option<T>;

    /// Tries to get the value of the `Watch` if it matches the predicate function
    /// `f`, marking it as seen.
    fn try_get_and(&self, id: Option<&mut u64>, f: &mut dyn Fn(&T) -> bool) -> Option<T>;

    /// Checks if the `Watch` is been initialized with a value.
    fn contains_value(&self) -> bool;
}

impl<M: RawMutex, T: Clone, const N: usize> SealedWatchBehavior<T> for Watch<M, T, N> {
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

    fn poll_get_and(&self, id: &mut u64, f: &mut dyn Fn(&T) -> bool, cx: &mut Context<'_>) -> Poll<T> {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            match s.data {
                Some(ref data) if f(data) => {
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
                    s.data.clone()
                }
                false => None,
            }
        })
    }

    fn poll_changed_and(&self, id: &mut u64, f: &mut dyn Fn(&T) -> bool, cx: &mut Context<'_>) -> Poll<T> {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            match (&s.data, s.current_id > *id) {
                (Some(data), true) if f(data) => {
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

    fn try_changed_and(&self, id: &mut u64, f: &mut dyn Fn(&T) -> bool) -> Option<T> {
        self.mutex.lock(|state| {
            let s = state.borrow();
            match (&s.data, s.current_id > *id) {
                (Some(data), true) if f(data) => {
                    *id = s.current_id;
                    s.data.clone()
                }
                _ => None,
            }
        })
    }

    fn drop_receiver(&self) {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            s.receiver_count -= 1;
        })
    }

    fn clear(&self) {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            s.data = None;
        })
    }

    fn send(&self, val: T) {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            s.data = Some(val);
            s.current_id += 1;
            s.wakers.wake();
        })
    }

    fn send_modify(&self, f: &mut dyn Fn(&mut Option<T>)) {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            f(&mut s.data);
            s.current_id += 1;
            s.wakers.wake();
        })
    }

    fn send_if_modified(&self, f: &mut dyn Fn(&mut Option<T>) -> bool) {
        self.mutex.lock(|state| {
            let mut s = state.borrow_mut();
            if f(&mut s.data) {
                s.current_id += 1;
                s.wakers.wake();
            }
        })
    }
}

impl<M: RawMutex, T: Clone, const N: usize> WatchBehavior<T> for Watch<M, T, N> {
    fn try_get(&self, id: Option<&mut u64>) -> Option<T> {
        self.mutex.lock(|state| {
            let s = state.borrow();
            if let Some(id) = id {
                *id = s.current_id;
            }
            s.data.clone()
        })
    }

    fn try_get_and(&self, id: Option<&mut u64>, f: &mut dyn Fn(&T) -> bool) -> Option<T> {
        self.mutex.lock(|state| {
            let s = state.borrow();
            match s.data {
                Some(ref data) if f(data) => {
                    if let Some(id) = id {
                        *id = s.current_id;
                    }
                    Some(data.clone())
                }
                _ => None,
            }
        })
    }

    fn contains_value(&self) -> bool {
        self.mutex.lock(|state| state.borrow().data.is_some())
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

    /// Create a new `Watch` channel with default data.
    pub const fn new_with(data: T) -> Self {
        Self {
            mutex: Mutex::new(RefCell::new(WatchState {
                data: Some(data),
                current_id: 0,
                wakers: MultiWakerRegistration::new(),
                receiver_count: 0,
            })),
        }
    }

    /// Create a new [`Sender`] for the `Watch`.
    pub fn sender(&self) -> Sender<'_, M, T, N> {
        Sender(Snd::new(self))
    }

    /// Create a new [`DynSender`] for the `Watch`.
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

    /// Try to create a new [`AnonReceiver`] for the `Watch`.
    pub fn anon_receiver(&self) -> AnonReceiver<'_, M, T, N> {
        AnonReceiver(AnonRcv::new(self, 0))
    }

    /// Try to create a new [`DynAnonReceiver`] for the `Watch`.
    pub fn dyn_anon_receiver(&self) -> DynAnonReceiver<'_, T> {
        DynAnonReceiver(AnonRcv::new(self, 0))
    }

    /// Returns the message ID of the latest message sent to the `Watch`.
    ///
    /// This counter is monotonic, and is incremented every time a new message is sent.
    pub fn get_msg_id(&self) -> u64 {
        self.mutex.lock(|state| state.borrow().current_id)
    }

    /// Tries to get the value of the `Watch`.
    pub fn try_get(&self) -> Option<T> {
        WatchBehavior::try_get(self, None)
    }

    /// Tries to get the value of the `Watch` if it matches the predicate function `f`.
    pub fn try_get_and<F>(&self, mut f: F) -> Option<T>
    where
        F: Fn(&T) -> bool,
    {
        WatchBehavior::try_get_and(self, None, &mut f)
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
    /// This will cause calls to [`Rcv::get`] to be pending.
    pub fn clear(&self) {
        self.watch.clear()
    }

    /// Tries to retrieve the value of the `Watch`.
    pub fn try_get(&self) -> Option<T> {
        self.watch.try_get(None)
    }

    /// Tries to peek the current value of the `Watch` if it matches the predicate
    /// function `f`.
    pub fn try_get_and<F>(&self, mut f: F) -> Option<T>
    where
        F: Fn(&T) -> bool,
    {
        self.watch.try_get_and(None, &mut f)
    }

    /// Returns true if the `Watch` contains a value.
    pub fn contains_value(&self) -> bool {
        self.watch.contains_value()
    }

    /// Modify the value of the `Watch` using a closure.
    pub fn send_modify<F>(&self, mut f: F)
    where
        F: Fn(&mut Option<T>),
    {
        self.watch.send_modify(&mut f)
    }

    /// Modify the value of the `Watch` using a closure. The closure must return
    /// `true` if the value was modified, which notifies all receivers.
    pub fn send_if_modified<F>(&self, mut f: F)
    where
        F: Fn(&mut Option<T>) -> bool,
    {
        self.watch.send_if_modified(&mut f)
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

    /// Returns the current value of the `Watch` once it is initialized, marking it as seen.
    ///
    /// **Note**: Futures do nothing unless you `.await` or poll them.
    pub fn get(&mut self) -> impl Future<Output = T> + '_ {
        poll_fn(|cx| self.watch.poll_get(&mut self.at_id, cx))
    }

    /// Tries to get the current value of the `Watch` without waiting, marking it as seen.
    pub fn try_get(&mut self) -> Option<T> {
        self.watch.try_get(Some(&mut self.at_id))
    }

    /// Returns the value of the `Watch` if it matches the predicate function `f`,
    /// or waits for it to match, marking it as seen.
    ///
    /// **Note**: Futures do nothing unless you `.await` or poll them.
    pub async fn get_and<F>(&mut self, mut f: F) -> T
    where
        F: Fn(&T) -> bool,
    {
        poll_fn(|cx| self.watch.poll_get_and(&mut self.at_id, &mut f, cx)).await
    }

    /// Tries to get the current value of the `Watch` if it matches the predicate
    /// function `f` without waiting, marking it as seen.
    pub fn try_get_and<F>(&mut self, mut f: F) -> Option<T>
    where
        F: Fn(&T) -> bool,
    {
        self.watch.try_get_and(Some(&mut self.at_id), &mut f)
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

    /// Waits for the `Watch` to change to a value which satisfies the predicate
    /// function `f` and returns the new value, marking it as seen.
    ///
    /// **Note**: Futures do nothing unless you `.await` or poll them.
    pub async fn changed_and<F>(&mut self, mut f: F) -> T
    where
        F: Fn(&T) -> bool,
    {
        poll_fn(|cx| self.watch.poll_changed_and(&mut self.at_id, &mut f, cx)).await
    }

    /// Tries to get the new value of the watch which satisfies the predicate
    /// function `f` and returns the new value without waiting, marking it as seen.
    pub fn try_changed_and<F>(&mut self, mut f: F) -> Option<T>
    where
        F: Fn(&T) -> bool,
    {
        self.watch.try_changed_and(&mut self.at_id, &mut f)
    }

    /// Checks if the `Watch` contains a value. If this returns true,
    /// then awaiting [`Rcv::get`] will return immediately.
    pub fn contains_value(&self) -> bool {
        self.watch.contains_value()
    }
}

impl<'a, T: Clone, W: WatchBehavior<T> + ?Sized> Drop for Rcv<'a, T, W> {
    fn drop(&mut self) {
        self.watch.drop_receiver();
    }
}

/// A anonymous receiver can NOT `.await` a change in the `Watch` value.
pub struct AnonRcv<'a, T: Clone, W: WatchBehavior<T> + ?Sized> {
    watch: &'a W,
    at_id: u64,
    _phantom: PhantomData<T>,
}

impl<'a, T: Clone, W: WatchBehavior<T> + ?Sized> AnonRcv<'a, T, W> {
    /// Creates a new `Receiver` with a reference to the `Watch`.
    fn new(watch: &'a W, at_id: u64) -> Self {
        Self {
            watch,
            at_id,
            _phantom: PhantomData,
        }
    }

    /// Tries to get the current value of the `Watch` without waiting, marking it as seen.
    pub fn try_get(&mut self) -> Option<T> {
        self.watch.try_get(Some(&mut self.at_id))
    }

    /// Tries to get the current value of the `Watch` if it matches the predicate
    /// function `f` without waiting, marking it as seen.
    pub fn try_get_and<F>(&mut self, mut f: F) -> Option<T>
    where
        F: Fn(&T) -> bool,
    {
        self.watch.try_get_and(Some(&mut self.at_id), &mut f)
    }

    /// Tries to get the new value of the watch without waiting, marking it as seen.
    pub fn try_changed(&mut self) -> Option<T> {
        self.watch.try_changed(&mut self.at_id)
    }

    /// Tries to get the new value of the watch which satisfies the predicate
    /// function `f` and returns the new value without waiting, marking it as seen.
    pub fn try_changed_and<F>(&mut self, mut f: F) -> Option<T>
    where
        F: Fn(&T) -> bool,
    {
        self.watch.try_changed_and(&mut self.at_id, &mut f)
    }

    /// Checks if the `Watch` contains a value. If this returns true,
    /// then awaiting [`Rcv::get`] will return immediately.
    pub fn contains_value(&self) -> bool {
        self.watch.contains_value()
    }
}

/// A receiver of a `Watch` channel.
pub struct Receiver<'a, M: RawMutex, T: Clone, const N: usize>(Rcv<'a, T, Watch<M, T, N>>);

impl<'a, M: RawMutex, T: Clone, const N: usize> Receiver<'a, M, T, N> {
    /// Converts the `Receiver` into a [`DynReceiver`].
    pub fn as_dyn(self) -> DynReceiver<'a, T> {
        let rcv = DynReceiver(Rcv::new(self.0.watch, self.at_id));
        core::mem::forget(self); // Ensures the destructor is not called
        rcv
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

/// A receiver of a `Watch` channel that cannot `.await` values.
pub struct AnonReceiver<'a, M: RawMutex, T: Clone, const N: usize>(AnonRcv<'a, T, Watch<M, T, N>>);

impl<'a, M: RawMutex, T: Clone, const N: usize> AnonReceiver<'a, M, T, N> {
    /// Converts the `Receiver` into a [`DynReceiver`].
    pub fn as_dyn(self) -> DynAnonReceiver<'a, T> {
        let rcv = DynAnonReceiver(AnonRcv::new(self.0.watch, self.at_id));
        core::mem::forget(self); // Ensures the destructor is not called
        rcv
    }
}

impl<'a, M: RawMutex, T: Clone, const N: usize> Into<DynAnonReceiver<'a, T>> for AnonReceiver<'a, M, T, N> {
    fn into(self) -> DynAnonReceiver<'a, T> {
        self.as_dyn()
    }
}

impl<'a, M: RawMutex, T: Clone, const N: usize> Deref for AnonReceiver<'a, M, T, N> {
    type Target = AnonRcv<'a, T, Watch<M, T, N>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, M: RawMutex, T: Clone, const N: usize> DerefMut for AnonReceiver<'a, M, T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A receiver that cannot `.await` value, which holds a **dynamic** reference to a `Watch` channel.
///
/// This is an alternative to [`AnonReceiver`] with a simpler type definition, at the expense of
/// some runtime performance due to dynamic dispatch.
pub struct DynAnonReceiver<'a, T: Clone>(AnonRcv<'a, T, dyn WatchBehavior<T> + 'a>);

impl<'a, T: Clone> Deref for DynAnonReceiver<'a, T> {
    type Target = AnonRcv<'a, T, dyn WatchBehavior<T> + 'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: Clone> DerefMut for DynAnonReceiver<'a, T> {
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
    fn all_try_get() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 1> = Watch::new();

            // Obtain receiver and sender
            let mut rcv = WATCH.receiver().unwrap();
            let snd = WATCH.sender();

            // Not initialized
            assert_eq!(WATCH.try_get(), None);
            assert_eq!(rcv.try_get(), None);
            assert_eq!(snd.try_get(), None);

            // Receive the new value
            snd.send(10);
            assert_eq!(WATCH.try_get(), Some(10));
            assert_eq!(rcv.try_get(), Some(10));
            assert_eq!(snd.try_get(), Some(10));

            assert_eq!(WATCH.try_get_and(|x| x > &5), Some(10));
            assert_eq!(rcv.try_get_and(|x| x > &5), Some(10));
            assert_eq!(snd.try_get_and(|x| x > &5), Some(10));

            assert_eq!(WATCH.try_get_and(|x| x < &5), None);
            assert_eq!(rcv.try_get_and(|x| x < &5), None);
            assert_eq!(snd.try_get_and(|x| x < &5), None);
        };
        block_on(f);
    }

    #[test]
    fn once_lock_like() {
        let f = async {
            static CONFIG0: u8 = 10;
            static CONFIG1: u8 = 20;

            static WATCH: Watch<CriticalSectionRawMutex, &'static u8, 1> = Watch::new();

            // Obtain receiver and sender
            let mut rcv = WATCH.receiver().unwrap();
            let snd = WATCH.sender();

            // Not initialized
            assert_eq!(rcv.try_changed(), None);

            // Receive the new value
            snd.send(&CONFIG0);
            let rcv0 = rcv.changed().await;
            assert_eq!(rcv0, &10);

            // Receive another value
            snd.send(&CONFIG1);
            let rcv1 = rcv.try_changed();
            assert_eq!(rcv1, Some(&20));

            // No update
            assert_eq!(rcv.try_changed(), None);

            // Ensure similarity with original static
            assert_eq!(rcv0, &CONFIG0);
            assert_eq!(rcv1, Some(&CONFIG1));
        };
        block_on(f);
    }

    #[test]
    fn sender_modify() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 1> = Watch::new();

            // Obtain receiver and sender
            let mut rcv = WATCH.receiver().unwrap();
            let snd = WATCH.sender();

            // Receive the new value
            snd.send(10);
            assert_eq!(rcv.try_changed(), Some(10));

            // Modify the value inplace
            snd.send_modify(|opt| {
                if let Some(inner) = opt {
                    *inner += 5;
                }
            });

            // Get the modified value
            assert_eq!(rcv.try_changed(), Some(15));
            assert_eq!(rcv.try_changed(), None);
        };
        block_on(f);
    }

    #[test]
    fn predicate_fn() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 1> = Watch::new();

            // Obtain receiver and sender
            let mut rcv = WATCH.receiver().unwrap();
            let snd = WATCH.sender();

            snd.send(15);
            assert_eq!(rcv.try_get_and(|x| x > &5), Some(15));
            assert_eq!(rcv.try_get_and(|x| x < &5), None);
            assert!(rcv.try_changed().is_none());

            snd.send(20);
            assert_eq!(rcv.try_changed_and(|x| x > &5), Some(20));
            assert_eq!(rcv.try_changed_and(|x| x > &5), None);

            snd.send(25);
            assert_eq!(rcv.try_changed_and(|x| x < &5), None);
            assert_eq!(rcv.try_changed(), Some(25));

            snd.send(30);
            assert_eq!(rcv.changed_and(|x| x > &5).await, 30);
            assert_eq!(rcv.get_and(|x| x > &5).await, 30);
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
            let mut rcv1 = WATCH.anon_receiver();
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
    fn use_dynamics() {
        let f = async {
            static WATCH: Watch<CriticalSectionRawMutex, u8, 2> = Watch::new();

            // Obtain receiver and sender
            let mut anon_rcv = WATCH.dyn_anon_receiver();
            let mut dyn_rcv = WATCH.dyn_receiver().unwrap();
            let dyn_snd = WATCH.dyn_sender();

            // Send a value
            dyn_snd.send(10);

            // Ensure the dynamic receiver receives the value
            assert_eq!(anon_rcv.try_changed(), Some(10));
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
            let anon_rcv = WATCH.anon_receiver();
            let rcv = WATCH.receiver().unwrap();
            let snd = WATCH.sender();

            // Convert to dynamic
            let mut dyn_anon_rcv = anon_rcv.as_dyn();
            let mut dyn_rcv = rcv.as_dyn();
            let dyn_snd = snd.as_dyn();

            // Send a value
            dyn_snd.send(10);

            // Ensure the dynamic receiver receives the value
            assert_eq!(dyn_anon_rcv.try_changed(), Some(10));
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
