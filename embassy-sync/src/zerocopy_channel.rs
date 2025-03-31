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

use core::cell::RefCell;
use core::future::{poll_fn, Future};
use core::marker::PhantomData;
use core::task::{Context, Poll};

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;
use crate::waitqueue::WakerRegistration;

/// A bounded zero-copy channel for communicating between asynchronous tasks
/// with backpressure.
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
    buf: BufferPtr<T>,
    phantom: PhantomData<&'a mut T>,
    state: Mutex<M, RefCell<State>>,
}

impl<'a, M: RawMutex, T> Channel<'a, M, T> {
    /// Initialize a new [`Channel`].
    ///
    /// The provided buffer will be used and reused by the channel's logic, and thus dictates the
    /// channel's capacity.
    pub fn new(buf: &'a mut [T]) -> Self {
        let len = buf.len();
        assert!(len != 0);

        Self {
            buf: BufferPtr(buf.as_mut_ptr()),
            phantom: PhantomData,
            state: Mutex::new(RefCell::new(State {
                capacity: len,
                front: 0,
                back: 0,
                full: false,
                send_waker: WakerRegistration::new(),
                receive_waker: WakerRegistration::new(),
            })),
        }
    }

    /// Creates a [`Sender`] and [`Receiver`] from an existing channel.
    ///
    /// Further Senders and Receivers can be created through [`Sender::borrow`] and
    /// [`Receiver::borrow`] respectively.
    pub fn split(&mut self) -> (Sender<'_, M, T>, Receiver<'_, M, T>) {
        (Sender { channel: self }, Receiver { channel: self })
    }

    /// Clears all elements in the channel.
    pub fn clear(&mut self) {
        self.state.lock(|s| {
            s.borrow_mut().clear();
        });
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

/// Send-only access to a [`Channel`].
pub struct Sender<'a, M: RawMutex, T> {
    channel: &'a Channel<'a, M, T>,
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
                Some(i) => Some(unsafe { &mut *self.channel.buf.add(i) }),
                None => None,
            }
        })
    }

    /// Attempts to send a value over the channel.
    pub fn poll_send(&mut self, cx: &mut Context) -> Poll<&mut T> {
        self.channel.state.lock(|s| {
            let s = &mut *s.borrow_mut();
            match s.push_index() {
                Some(i) => Poll::Ready(unsafe { &mut *self.channel.buf.add(i) }),
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
                        let r = unsafe { &mut *self.channel.buf.add(i) };
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

/// Receive-only access to a [`Channel`].
pub struct Receiver<'a, M: RawMutex, T> {
    channel: &'a Channel<'a, M, T>,
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
                Some(i) => Some(unsafe { &mut *self.channel.buf.add(i) }),
                None => None,
            }
        })
    }

    /// Attempts to asynchronously receive a value over the channel.
    pub fn poll_receive(&mut self, cx: &mut Context) -> Poll<&mut T> {
        self.channel.state.lock(|s| {
            let s = &mut *s.borrow_mut();
            match s.pop_index() {
                Some(i) => Poll::Ready(unsafe { &mut *self.channel.buf.add(i) }),
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
                        let r = unsafe { &mut *self.channel.buf.add(i) };
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

struct State {
    /// Maximum number of elements the channel can hold.
    capacity: usize,

    /// Front index. Always 0..=(N-1)
    front: usize,
    /// Back index. Always 0..=(N-1).
    back: usize,

    /// Used to distinguish "empty" and "full" cases when `front == back`.
    /// May only be `true` if `front == back`, always `false` otherwise.
    full: bool,

    send_waker: WakerRegistration,
    receive_waker: WakerRegistration,
}

impl State {
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
}
