//! A zero-copy queue for sending values between asynchronous tasks.
//!
//! It can be used concurrently by a producer (sender) and a
//! consumer (receiver), i.e. it is an  "SPSC channel".
//!
//! This queue takes a Mutex type so that various
//! targets can be attained. For example, a ThreadModeMutex can be used
//! for single-core Cortex-M targets where messages are only passed
//! between tasks running in thread mode. Similarly, a CriticalSectionMutex
//! can also be used for single-core targets where messages are to be
//! passed from exception mode e.g. out of an interrupt handler.
//!
//! This module provides a bounded channel that has a limit on the number of
//! messages that it can store, and if this limit is reached, trying to send
//! another message will result in an error being returned.

use core::cell::{RefCell, UnsafeCell};
use core::future::{poll_fn, Future};
use core::marker::PhantomData;
use core::task::{Context, Poll};

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;
use crate::waitqueue::WakerRegistration;

struct ChannelInner<M: RawMutex, T> {
    state: Mutex<M, RefCell<State<T>>>,
}

impl<M: RawMutex, T> ChannelInner<M, T> {
    /// Initialize a new [`Channel`].
    ///
    /// The provided buffer will be used and reused by the channel's logic, and thus dictates the
    /// channel's capacity.
    fn new(len: usize, buf: BufferPtr<T>) -> Self {
        assert!(len != 0);

        Self {
            state: Mutex::new(RefCell::new(State {
                capacity: len,
                buf,
                front: 0,
                back: 0,
                full: false,
                send_waker: WakerRegistration::new(),
                receive_waker: WakerRegistration::new(),
                have_sender: false,
                have_receiver: false,
            })),
        }
    }

    /// Creates a [`Sender`] and [`Receiver`] from an existing channel.
    ///
    /// Further Senders and Receivers can be created through [`Sender::borrow`] and
    /// [`Receiver::borrow`] respectively.
    pub fn split(&mut self) -> (Sender<'_, M, T>, Receiver<'_, M, T>) {
        let mut s = self.state.get_mut().borrow_mut();
        // We can unconditionally add a sender/receiver since
        // split() takes a mut reference, so there must be no
        // existing Sender or Receiver.
        s.add_sender();
        s.add_receiver();
        drop(s);
        (Sender { channel: self }, Receiver { channel: self })
    }

    /// Create a [`Receiver`] from an existing channel.
    ///
    /// Only one `Receiver` may be borrowed.
    pub fn receiver(&self) -> Option<Receiver<'_, M, T>> {
        self.state
            .lock(|s| s.borrow_mut().add_receiver())
            .then(|| Receiver { channel: self })
    }

    /// Create a [`Sender`] from an existing channel.
    ///
    /// Only one `Sender` may be borrowed.
    pub fn sender(&self) -> Option<Sender<'_, M, T>> {
        self.state
            .lock(|s| s.borrow_mut().add_sender())
            .then(|| Sender { channel: self })
    }

    /// Clears all elements in the channel.
    pub fn clear(&mut self) {
        self.state.get_mut().borrow_mut().clear();
    }

    /// Returns the number of elements currently in the channel.
    pub fn len(&self) -> usize {
        self.state.lock(|s| s.borrow().len())
    }

    /// Returns whether the channel is empty.
    pub fn is_empty(&self) -> bool {
        self.state.lock(|s| s.borrow().is_empty())
    }

    /// Returns whether the channel is full.
    pub fn is_full(&self) -> bool {
        self.state.lock(|s| s.borrow().is_full())
    }
}

#[repr(transparent)]
struct BufferPtr<T>(*mut T);

impl<T> BufferPtr<T> {
    unsafe fn add(&self, count: usize) -> *mut T {
        self.0.add(count)
    }
}

unsafe impl<T> Send for BufferPtr<T> {}
unsafe impl<T> Sync for BufferPtr<T> {}

/// A bounded zero-copy channel for communicating between asynchronous tasks
/// with backpressure. Uses a borrowed buffer.
///
/// The channel will buffer up to the provided number of messages.  Once the
/// buffer is full, attempts to `send` new messages will wait until a message is
/// received from the channel.
///
/// All data sent will become available in the same order as it was sent.
///
/// The channel requires a buffer of recyclable elements.  Writing to the channel is done through
/// an `&mut T`.
pub struct Channel<'a, M: RawMutex, T> {
    channel: ChannelInner<M, T>,
    phantom: PhantomData<&'a mut T>,
}

impl<'a, M: RawMutex, T> Channel<'a, M, T> {
    /// Initialize a new [`Channel`].
    ///
    /// The provided buffer will be used and reused by the channel's logic, and thus dictates the
    /// channel's capacity.
    pub fn new(buf: &'a mut [T]) -> Self {
        Self {
            channel: ChannelInner::new(buf.len(), BufferPtr(buf.as_mut_ptr())),
            phantom: PhantomData,
        }
    }

    /// Creates a [`Sender`] and [`Receiver`] from an existing channel.
    ///
    /// Further Senders and Receivers can be created through [`Sender::borrow`] and
    /// [`Receiver::borrow`] respectively.
    pub fn split(&mut self) -> (Sender<'_, M, T>, Receiver<'_, M, T>) {
        self.channel.split()
    }

    /// Create a [`Receiver`] from an existing channel.
    ///
    /// Only one `Receiver` may be borrowed.
    pub fn receiver(&self) -> Option<Receiver<'_, M, T>> {
        self.channel.receiver()
    }

    /// Create a [`Sender`] from an existing channel.
    ///
    /// Only one `Sender` may be borrowed.
    pub fn sender(&self) -> Option<Sender<'_, M, T>> {
        self.channel.sender()
    }

    /// Clears all elements in the channel.
    pub fn clear(&mut self) {
        self.channel.clear()
    }

    /// Returns the number of elements currently in the channel.
    pub fn len(&self) -> usize {
        self.channel.len()
    }

    /// Returns whether the channel is empty.
    pub fn is_empty(&self) -> bool {
        self.channel.is_empty()
    }

    /// Returns whether the channel is full.
    pub fn is_full(&self) -> bool {
        self.channel.is_full()
    }
}

/// A bounded zero-copy channel for communicating between asynchronous tasks
/// with backpressure. Uses a local buffer.
///
/// The channel will buffer up to the provided number of messages.  Once the
/// buffer is full, attempts to `send` new messages will wait until a message is
/// received from the channel.
///
/// All data sent will become available in the same order as it was sent.
///
/// The channel uses an internal buffer of `N` elements, they must implement
/// `Default` for initial placeholders.
// TODO could make buf MaybeUninit then write through that?
pub struct FixedChannel<M: RawMutex, T, const N: usize> {
    channel: ChannelInner<M, T>,
    // Storage must not be accessed directly, only in update_ptr()
    storage: Storage<T, N>,
}

// Storage is always accessed locked by channel
//
// The storage is safe from aliasing because only a single Sender or Reciver
// can access any array element at a time.
//
// It is safe to implement Sync and Safe because any Sender or Receiver will
// borrow from the ChannelInner (having the same lifetime as storage), and storage
// is only manipulated through Sender and Receiver.
#[repr(transparent)]
struct Storage<T, const N: usize>(UnsafeCell<[T; N]>);
unsafe impl<T, const N: usize> Sync for Storage<T, N> {}
unsafe impl<T, const N: usize> Send for Storage<T, N> {}

impl<M: RawMutex, T: Default, const N: usize> FixedChannel<M, T, N> {
    /// Initialize a new [`FixedChannel`].
    pub fn new() -> Self {
        // Initial pointer is null, set before use with update_ptr()
        let channel = ChannelInner::new(N, BufferPtr(core::ptr::null_mut()));
        Self {
            channel,
            storage: Storage(UnsafeCell::new([(); N].map(|_| Default::default()))),
        }
    }
}

impl<M: RawMutex, T: Default, const N: usize> Default for FixedChannel<M, T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: RawMutex, T: Clone, const N: usize> FixedChannel<M, T, N> {
    /// Initialize a new [`FixedChannel`].
    ///
    /// This take an initial buffer value to clone.
    pub fn new_cloned(initial: &T) -> Self {
        // Initial pointer is null, set before use with update_ptr()
        let channel = ChannelInner::new(N, BufferPtr(core::ptr::null_mut()));
        Self {
            channel,
            storage: Storage(UnsafeCell::new([(); N].map(|_| initial.clone()))),
        }
    }
}

impl<M: RawMutex, T, const N: usize> FixedChannel<M, T, N> {
    /// Update the buf pointer.
    ///
    /// This must occur before each Sender/Receiver borrow to ensure it's not stale.
    /// The lifetime of Sender/Receiver guarantees that it won't go stale
    /// while one of them is active, and buf is only used by a Sender or Receiver.
    fn update_ptr(&self) {
        self.channel.state.lock(|s| {
            // Point to first storage array element
            s.borrow_mut().buf = BufferPtr(self.storage.0.get() as *mut T);
        });
    }

    /// Creates a [`Sender`] and [`Receiver`] from an existing channel.
    ///
    /// Further Senders and Receivers can be created through [`Sender::borrow`] and
    /// [`Receiver::borrow`] respectively.
    pub fn split(&mut self) -> (Sender<'_, M, T>, Receiver<'_, M, T>) {
        self.update_ptr();
        self.channel.split()
    }

    /// Create a [`Receiver`] from an existing channel.
    ///
    /// Only one `Receiver` may be borrowed.
    pub fn receiver(&self) -> Option<Receiver<'_, M, T>> {
        self.update_ptr();
        self.channel.receiver()
    }

    /// Create a [`Sender`] from an existing channel.
    ///
    /// Only one `Sender` may be borrowed.
    pub fn sender(&self) -> Option<Sender<'_, M, T>> {
        self.update_ptr();
        self.channel.sender()
    }

    /// Clears all elements in the channel.
    pub fn clear(&mut self) {
        self.channel.clear()
    }

    /// Returns the number of elements currently in the channel.
    pub fn len(&self) -> usize {
        self.channel.len()
    }

    /// Returns whether the channel is empty.
    pub fn is_empty(&self) -> bool {
        self.channel.is_empty()
    }

    /// Returns whether the channel is full.
    pub fn is_full(&self) -> bool {
        self.channel.is_full()
    }
}

/// Send-only access to a [`Channel`] or [`FixedChannel`].
pub struct Sender<'a, M: RawMutex, T> {
    channel: &'a ChannelInner<M, T>,
}

impl<'a, M: RawMutex, T> Sender<'a, M, T> {
    /// Creates one further [`Sender`] over the same channel.
    pub fn borrow(&mut self) -> Sender<'_, M, T> {
        Sender { channel: self.channel }
    }

    /// Attempts to send a value over the channel.
    pub fn try_send(&mut self) -> Option<&mut T> {
        self.channel.state.lock(|s| {
            let s = &mut *s.borrow_mut();
            match s.push_index() {
                Some(i) => Some(unsafe { &mut *s.buf.add(i) }),
                None => None,
            }
        })
    }

    /// Attempts to send a value over the channel.
    pub fn poll_send(&mut self, cx: &mut Context) -> Poll<&mut T> {
        self.channel.state.lock(|s| {
            let s = &mut *s.borrow_mut();
            match s.push_index() {
                Some(i) => Poll::Ready(unsafe { &mut *s.buf.add(i) }),
                None => {
                    s.receive_waker.register(cx.waker());
                    Poll::Pending
                }
            }
        })
    }

    /// Asynchronously send a value over the channel.
    pub fn send(&mut self) -> impl Future<Output = &mut T> {
        poll_fn(|cx| {
            self.channel.state.lock(|s| {
                let s = &mut *s.borrow_mut();
                match s.push_index() {
                    Some(i) => {
                        let r = unsafe { &mut *s.buf.add(i) };
                        Poll::Ready(r)
                    }
                    None => {
                        s.receive_waker.register(cx.waker());
                        Poll::Pending
                    }
                }
            })
        })
    }

    /// Notify the channel that the sending of the value has been finalized.
    pub fn send_done(&mut self) {
        self.channel.state.lock(|s| s.borrow_mut().push_done())
    }

    /// Clears all elements in the channel.
    pub fn clear(&mut self) {
        self.channel.state.lock(|s| {
            s.borrow_mut().clear();
        });
    }

    /// Returns the number of elements currently in the channel.
    pub fn len(&self) -> usize {
        self.channel.state.lock(|s| s.borrow().len())
    }

    /// Returns whether the channel is empty.
    pub fn is_empty(&self) -> bool {
        self.channel.state.lock(|s| s.borrow().is_empty())
    }

    /// Returns whether the channel is full.
    pub fn is_full(&self) -> bool {
        self.channel.state.lock(|s| s.borrow().is_full())
    }
}

impl<M: RawMutex, T> Drop for Sender<'_, M, T> {
    fn drop(&mut self) {
        self.channel.state.lock(|s| s.borrow_mut().remove_sender())
    }
}

/// Receive-only access to a [`Channel`] or [`FixedChannel`].
pub struct Receiver<'a, M: RawMutex, T> {
    channel: &'a ChannelInner<M, T>,
}

impl<'a, M: RawMutex, T> Receiver<'a, M, T> {
    /// Creates one further [`Receiver`] over the same channel.
    pub fn borrow(&mut self) -> Receiver<'_, M, T> {
        Receiver { channel: self.channel }
    }

    /// Attempts to receive a value over the channel.
    pub fn try_receive(&mut self) -> Option<&mut T> {
        self.channel.state.lock(|s| {
            let s = &mut *s.borrow_mut();
            match s.pop_index() {
                Some(i) => Some(unsafe { &mut *s.buf.add(i) }),
                None => None,
            }
        })
    }

    /// Attempts to asynchronously receive a value over the channel.
    pub fn poll_receive(&mut self, cx: &mut Context) -> Poll<&mut T> {
        self.channel.state.lock(|s| {
            let s = &mut *s.borrow_mut();
            match s.pop_index() {
                Some(i) => Poll::Ready(unsafe { &mut *s.buf.add(i) }),
                None => {
                    s.send_waker.register(cx.waker());
                    Poll::Pending
                }
            }
        })
    }

    /// Asynchronously receive a value over the channel.
    pub fn receive(&mut self) -> impl Future<Output = &mut T> {
        poll_fn(|cx| {
            self.channel.state.lock(|s| {
                let s = &mut *s.borrow_mut();
                match s.pop_index() {
                    Some(i) => {
                        let r = unsafe { &mut *s.buf.add(i) };
                        Poll::Ready(r)
                    }
                    None => {
                        s.send_waker.register(cx.waker());
                        Poll::Pending
                    }
                }
            })
        })
    }

    /// Notify the channel that the receiving of the value has been finalized.
    pub fn receive_done(&mut self) {
        self.channel.state.lock(|s| s.borrow_mut().pop_done())
    }

    /// Clears all elements in the channel.
    pub fn clear(&mut self) {
        self.channel.state.lock(|s| {
            s.borrow_mut().clear();
        });
    }

    /// Returns the number of elements currently in the channel.
    pub fn len(&self) -> usize {
        self.channel.state.lock(|s| s.borrow().len())
    }

    /// Returns whether the channel is empty.
    pub fn is_empty(&self) -> bool {
        self.channel.state.lock(|s| s.borrow().is_empty())
    }

    /// Returns whether the channel is full.
    pub fn is_full(&self) -> bool {
        self.channel.state.lock(|s| s.borrow().is_full())
    }
}

impl<M: RawMutex, T> Drop for Receiver<'_, M, T> {
    fn drop(&mut self) {
        self.channel.state.lock(|s| s.borrow_mut().remove_receiver())
    }
}

struct State<T> {
    /// Maximum number of elements the channel can hold.
    capacity: usize,

    /// Pointer to the channel's buffer.
    ///
    /// Will always/only be valid when a Sender or Receiver
    /// is borrowed.
    buf: BufferPtr<T>,

    /// Front index. Always 0..=(N-1)
    front: usize,
    /// Back index. Always 0..=(N-1).
    back: usize,

    /// Used to distinguish "empty" and "full" cases when `front == back`.
    /// May only be `true` if `front == back`, always `false` otherwise.
    full: bool,

    send_waker: WakerRegistration,
    receive_waker: WakerRegistration,

    have_receiver: bool,
    have_sender: bool,
}

impl<T> State<T> {
    fn increment(&self, i: usize) -> usize {
        if i + 1 == self.capacity {
            0
        } else {
            i + 1
        }
    }

    fn clear(&mut self) {
        if self.full {
            self.receive_waker.wake();
        }
        self.front = 0;
        self.back = 0;
        self.full = false;
    }

    fn len(&self) -> usize {
        if !self.full {
            if self.back >= self.front {
                self.back - self.front
            } else {
                self.capacity + self.back - self.front
            }
        } else {
            self.capacity
        }
    }

    fn is_full(&self) -> bool {
        self.full
    }

    fn is_empty(&self) -> bool {
        self.front == self.back && !self.full
    }

    fn push_index(&mut self) -> Option<usize> {
        match self.is_full() {
            true => None,
            false => Some(self.back),
        }
    }

    fn push_done(&mut self) {
        assert!(!self.is_full());
        self.back = self.increment(self.back);
        if self.back == self.front {
            self.full = true;
        }
        self.send_waker.wake();
    }

    fn pop_index(&mut self) -> Option<usize> {
        match self.is_empty() {
            true => None,
            false => Some(self.front),
        }
    }

    fn pop_done(&mut self) {
        assert!(!self.is_empty());
        self.front = self.increment(self.front);
        self.full = false;
        self.receive_waker.wake();
    }

    // Returns true if a sender was added, false if one already existed
    fn add_sender(&mut self) -> bool {
        !core::mem::replace(&mut self.have_sender, true)
    }

    fn remove_sender(&mut self) {
        debug_assert!(self.have_sender);
        self.have_sender = false;
    }

    // Returns true if a receiver was added, false if one already existed
    fn add_receiver(&mut self) -> bool {
        !core::mem::replace(&mut self.have_receiver, true)
    }

    fn remove_receiver(&mut self) {
        debug_assert!(self.have_receiver);
        self.have_receiver = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
    extern crate std;
    use core::ops::{Deref, DerefMut};
    use std::boxed::Box;

    #[test]
    fn split() {
        let mut buf = [0u32; 10];
        let mut c = Channel::<NoopRawMutex, _>::new(&mut buf);

        let (mut s, mut r) = c.split();

        *s.try_send().unwrap() = 4;
        s.send_done();

        let b = r.try_receive().unwrap();
        assert_eq!(*b, 4);
        let b = r.try_receive().unwrap();
        assert_eq!(*b, 4);
        r.receive_done();
        let b = r.try_receive();
        assert!(b.is_none());
    }

    #[test]
    fn sender_receiver() {
        let mut buf = [0u32; 10];
        let c = Channel::<NoopRawMutex, _>::new(&mut buf);

        let mut s = c.sender().unwrap();
        assert!(c.sender().is_none(), "can't borrow again");

        *s.try_send().unwrap() = 4;
        s.send_done();
        drop(s);

        // borrow again
        let mut s = c.sender().unwrap();
        *s.try_send().unwrap() = 5;
        s.send_done();

        let mut r = c.receiver().unwrap();
        assert!(c.receiver().is_none(), "can't borrow again");

        let b = r.try_receive().unwrap();
        assert_eq!(*b, 4);
        let b = r.try_receive().unwrap();
        assert_eq!(*b, 4);
        r.receive_done();

        drop(r);
        // borrow again
        let mut r = c.receiver().unwrap();
        let b = r.try_receive().unwrap();
        assert_eq!(*b, 5);
        r.receive_done();
        let b = r.try_receive();
        assert!(b.is_none());
    }

    #[test]
    fn fixed() {
        let c = FixedChannel::<NoopRawMutex, _, 2>::new();

        let mut s = c.sender().unwrap();
        assert!(c.sender().is_none(), "can't borrow again");
        assert!(s.is_empty());

        *s.try_send().unwrap() = 4;
        assert!(s.is_empty());
        s.send_done();
        assert!(!s.is_empty());
        drop(s);

        let mut s = c.sender().unwrap();
        *s.try_send().unwrap() = 5;
        assert!(!s.is_full());
        s.send_done();
        assert!(s.try_send().is_none(), "queue is full");
        assert!(s.is_full());

        let mut r = c.receiver().unwrap();
        assert!(c.receiver().is_none(), "can't borrow again");

        let b = r.try_receive().unwrap();
        assert_eq!(*b, 4);
        let b = r.try_receive().unwrap();
        assert_eq!(*b, 4);
        r.receive_done();

        drop(r);
        let mut r = c.receiver().unwrap();
        let b = r.try_receive().unwrap();
        assert_eq!(*b, 5);
        assert!(!s.is_empty());
        r.receive_done();
        assert!(s.is_empty());
        let b = r.try_receive();
        assert!(b.is_none());

        // Later send works
        *s.try_send().unwrap() = 6;
        assert!(r.is_empty());
        s.send_done();
        assert!(!r.is_empty());
    }

    #[test]
    fn fixed_move() {
        // Check that the buffer pointer updates if moved
        let c = FixedChannel::<CriticalSectionRawMutex, _, 40>::new_cloned(&123);

        let p1 = &c as *const _;
        let mut s = c.sender().unwrap();
        *s.try_send().unwrap() = 99u32;
        s.send_done();
        drop(s);

        let mut cbox = Box::new(Some(c));
        let c = cbox.deref().as_ref().unwrap();
        let p2 = c as *const _;

        let mut r = c.receiver().unwrap();
        let b = r.try_receive().unwrap();
        assert_eq!(*b, 99);
        r.receive_done();
        drop(r);

        let mut cbox = Box::new(cbox.take());
        let c = cbox.deref_mut().as_mut().unwrap();
        let p3 = c as *const _;

        let (mut s, mut r) = c.split();
        *s.try_send().unwrap() = 44;
        s.send_done();
        let b = r.try_receive().unwrap();
        assert_eq!(*b, 44);

        assert!(p1 != p2, "Ensure data moved");
        assert!(p1 != p3, "Ensure data moved");
        assert!(p2 != p3, "Ensure data moved");
    }
}
