//! A multi-producer, single-consumer queue for sending values between
//! asynchronous tasks. This queue takes a Mutex type so that various
//! targets can be attained. For example, a ThreadModeMutex can be used
//! for single-core Cortex-M targets where messages are only passed
//! between tasks running in thread mode. Similarly, a CriticalSectionMutex
//! can also be used for single-core targets where messages are to be
//! passed from exception mode e.g. out of an interrupt handler.
//!
//! This module provides a bounded channel that has a limit on the number of
//! messages that it can store, and if this limit is reached, trying to send
//! another message will result in an error being returned.
//!
//! Similar to the `mpsc` channels provided by `std`, the channel constructor
//! functions provide separate send and receive handles, [`Sender`] and
//! [`Receiver`]. If there is no message to read, the current task will be
//! notified when a new value is sent. [`Sender`] allows sending values into
//! the channel. If the bounded channel is at capacity, the send is rejected.
//!
//! # Disconnection
//!
//! When all [`Sender`] handles have been dropped, it is no longer
//! possible to send values into the channel. This is considered the termination
//! event of the stream.
//!
//! If the [`Receiver`] handle is dropped, then messages can no longer
//! be read out of the channel. In this case, all further attempts to send will
//! result in an error.
//!
//! # Clean Shutdown
//!
//! When the [`Receiver`] is dropped, it is possible for unprocessed messages to
//! remain in the channel. Instead, it is usually desirable to perform a "clean"
//! shutdown. To do this, the receiver first calls `close`, which will prevent
//! any further messages to be sent into the channel. Then, the receiver
//! consumes the channel to completion, at which point the receiver can be
//! dropped.
//!
//! This channel and its associated types were derived from https://docs.rs/tokio/0.1.22/tokio/sync/mpsc/fn.channel.html

use core::cell::UnsafeCell;
use core::fmt;
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;
use core::task::Waker;

use futures::Future;

use super::CriticalSectionMutex;
use super::Mutex;
use super::ThreadModeMutex;

/// A ChannelCell permits a channel to be shared between senders and their receivers.
// Derived from UnsafeCell.
#[repr(transparent)]
pub struct ChannelCell<T: ?Sized> {
    _value: T,
}

impl<T> ChannelCell<T> {
    #[inline(always)]
    pub const fn new<U>(value: T) -> ChannelCell<T>
    where
        T: ChannelLike<U>,
    {
        ChannelCell { _value: value }
    }
}

impl<T: ?Sized> ChannelCell<T> {
    #[inline(always)]
    const fn get(&self) -> *mut T {
        // As per UnsafeCell:
        // We can just cast the pointer from `ChannelCell<T>` to `T` because of
        // #[repr(transparent)]. This exploits libstd's special status, there is
        // no guarantee for user code that this will work in future versions of the compiler!
        self as *const ChannelCell<T> as *const T as *mut T
    }
}

/// Send values to the associated `Receiver`.
///
/// Instances are created by the [`split`](split) function.
pub struct Sender<'ch, T> {
    channel: &'ch ChannelCell<dyn ChannelLike<T>>,
}

// Safe to pass the sender around
unsafe impl<'ch, T> Send for Sender<'ch, T> {}
unsafe impl<'ch, T> Sync for Sender<'ch, T> {}

/// Receive values from the associated `Sender`.
///
/// Instances are created by the [`split`](split) function.
pub struct Receiver<'ch, T> {
    channel: &'ch ChannelCell<dyn ChannelLike<T>>,
}

// Safe to pass the receiver around
unsafe impl<'ch, T> Send for Receiver<'ch, T> {}
unsafe impl<'ch, T> Sync for Receiver<'ch, T> {}

/// Splits a bounded mpsc channel into a `Sender` and `Receiver`.
///
/// All data sent on `Sender` will become available on `Receiver` in the same
/// order as it was sent.
///
/// The `Sender` can be cloned to `send` to the same channel from multiple code
/// locations. Only one `Receiver` is valid.
///
/// If the `Receiver` is disconnected while trying to `send`, the `send` method
/// will return a `SendError`. Similarly, if `Sender` is disconnected while
/// trying to `recv`, the `recv` method will return a `RecvError`.
///
/// Note that when splitting the channel, the sender and receiver cannot outlive
/// their channel. The following will therefore fail compilation:
////
/// ```compile_fail
/// use embassy::util::mpsc;
/// use embassy::util::mpsc::{Channel, ChannelCell, WithThreadModeOnly};
///
/// let (sender, receiver) = {
///    let mut channel = ChannelCell::new(Channel::<WithThreadModeOnly, u32, 3>::with_thread_mode_only());
///     mpsc::split(&channel)
/// };
/// ```
pub fn split<T>(channel: &ChannelCell<dyn ChannelLike<T>>) -> (Sender<T>, Receiver<T>) {
    let sender = Sender { channel: &channel };
    let receiver = Receiver { channel: &channel };
    {
        let c = unsafe { &mut *channel.get() };
        c.register_receiver();
        c.register_sender();
    }
    (sender, receiver)
}

impl<'ch, T> Receiver<'ch, T> {
    /// Receives the next value for this receiver.
    ///
    /// This method returns `None` if the channel has been closed and there are
    /// no remaining messages in the channel's buffer. This indicates that no
    /// further values can ever be received from this `Receiver`. The channel is
    /// closed when all senders have been dropped, or when [`close`] is called.
    ///
    /// If there are no messages in the channel's buffer, but the channel has
    /// not yet been closed, this method will sleep until a message is sent or
    /// the channel is closed.
    ///
    /// Note that if [`close`] is called, but there are still outstanding
    /// messages from before it was closed, the channel is not considered
    /// closed by `recv` until they are all consumed.
    ///
    /// [`close`]: Self::close
    pub async fn recv(&mut self) -> Option<T> {
        self.await
    }

    /// Attempts to immediately receive a message on this `Receiver`
    ///
    /// This method will either receive a message from the channel immediately or return an error
    /// if the channel is empty.
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        unsafe { &mut *self.channel.get() }.try_recv()
    }

    /// Closes the receiving half of a channel without dropping it.
    ///
    /// This prevents any further messages from being sent on the channel while
    /// still enabling the receiver to drain messages that are buffered.
    ///
    /// To guarantee that no messages are dropped, after calling `close()`,
    /// `recv()` must be called until `None` is returned. If there are
    /// outstanding messages, the `recv` method will not return `None`
    /// until those are released.
    ///
    pub fn close(&mut self) {
        unsafe { &mut *self.channel.get() }.close()
    }
}

impl<'ch, T> Future for Receiver<'ch, T> {
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.try_recv() {
            Ok(v) => Poll::Ready(Some(v)),
            Err(TryRecvError::Closed) => Poll::Ready(None),
            Err(TryRecvError::Empty) => {
                unsafe { &mut *self.channel.get() }.set_receiver_waker(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl<'ch, T> Drop for Receiver<'ch, T> {
    fn drop(&mut self) {
        unsafe { &mut *self.channel.get() }.deregister_receiver()
    }
}

impl<'ch, T> Sender<'ch, T> {
    /// Sends a value, waiting until there is capacity.
    ///
    /// A successful send occurs when it is determined that the other end of the
    /// channel has not hung up already. An unsuccessful send would be one where
    /// the corresponding receiver has already been closed. Note that a return
    /// value of `Err` means that the data will never be received, but a return
    /// value of `Ok` does not mean that the data will be received. It is
    /// possible for the corresponding receiver to hang up immediately after
    /// this function returns `Ok`.
    ///
    /// # Errors
    ///
    /// If the receive half of the channel is closed, either due to [`close`]
    /// being called or the [`Receiver`] handle dropping, the function returns
    /// an error. The error includes the value passed to `send`.
    ///
    /// [`close`]: Receiver::close
    /// [`Receiver`]: Receiver
    pub async fn send(&self, message: T) -> Result<(), SendError<T>> {
        SendFuture {
            sender: self.clone(),
            message: UnsafeCell::new(message),
        }
        .await
    }

    /// Attempts to immediately send a message on this `Sender`
    ///
    /// This method differs from [`send`] by returning immediately if the channel's
    /// buffer is full or no receiver is waiting to acquire some data. Compared
    /// with [`send`], this function has two failure cases instead of one (one for
    /// disconnection, one for a full buffer).
    ///
    /// # Errors
    ///
    /// If the channel capacity has been reached, i.e., the channel has `n`
    /// buffered values where `n` is the argument passed to [`channel`], then an
    /// error is returned.
    ///
    /// If the receive half of the channel is closed, either due to [`close`]
    /// being called or the [`Receiver`] handle dropping, the function returns
    /// an error. The error includes the value passed to `send`.
    ///
    /// [`send`]: Sender::send
    /// [`channel`]: channel
    /// [`close`]: Receiver::close
    pub fn try_send(&self, message: T) -> Result<(), TrySendError<T>> {
        unsafe { &mut *self.channel.get() }.try_send(message)
    }

    /// Completes when the receiver has dropped.
    ///
    /// This allows the producers to get notified when interest in the produced
    /// values is canceled and immediately stop doing work.
    pub async fn closed(&self) {
        CloseFuture {
            sender: self.clone(),
        }
        .await
    }

    /// Checks if the channel has been closed. This happens when the
    /// [`Receiver`] is dropped, or when the [`Receiver::close`] method is
    /// called.
    ///
    /// [`Receiver`]: crate::sync::mpsc::Receiver
    /// [`Receiver::close`]: crate::sync::mpsc::Receiver::close
    pub fn is_closed(&self) -> bool {
        unsafe { &mut *self.channel.get() }.is_closed()
    }
}

struct SendFuture<'ch, T> {
    sender: Sender<'ch, T>,
    message: UnsafeCell<T>,
}

impl<'ch, T> Future for SendFuture<'ch, T> {
    type Output = Result<(), SendError<T>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.sender.try_send(unsafe { self.message.get().read() }) {
            Ok(..) => Poll::Ready(Ok(())),
            Err(TrySendError::Closed(m)) => Poll::Ready(Err(SendError(m))),
            Err(TrySendError::Full(..)) => {
                unsafe { &mut *self.sender.channel.get() }.set_senders_waker(cx.waker().clone());
                Poll::Pending
                // Note we leave the existing UnsafeCell contents - they still
                // contain the original message. We could create another UnsafeCell
                // with the message of Full, but there's no real need.
            }
        }
    }
}

struct CloseFuture<'ch, T> {
    sender: Sender<'ch, T>,
}

impl<'ch, T> Future for CloseFuture<'ch, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.sender.is_closed() {
            Poll::Ready(())
        } else {
            unsafe { &mut *self.sender.channel.get() }.set_senders_waker(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl<'ch, T> Drop for Sender<'ch, T> {
    fn drop(&mut self) {
        unsafe { &mut *self.channel.get() }.deregister_sender()
    }
}

impl<'ch, T> Clone for Sender<'ch, T> {
    #[allow(clippy::clone_double_ref)]
    fn clone(&self) -> Self {
        unsafe { &mut *self.channel.get() }.register_sender();
        Sender {
            channel: self.channel.clone(),
        }
    }
}

/// An error returned from the [`try_recv`] method.
///
/// [`try_recv`]: super::Receiver::try_recv
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TryRecvError {
    /// A message could not be received because the channel is empty.
    Empty,

    /// The message could not be received because the channel is empty and closed.
    Closed,
}

/// Error returned by the `Sender`.
#[derive(Debug)]
pub struct SendError<T>(pub T);

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "channel closed")
    }
}

/// This enumeration is the list of the possible error outcomes for the
/// [try_send](super::Sender::try_send) method.
#[derive(Debug)]
pub enum TrySendError<T> {
    /// The data could not be sent on the channel because the channel is
    /// currently full and sending would require blocking.
    Full(T),

    /// The receive half of the channel was explicitly closed or has been
    /// dropped.
    Closed(T),
}

impl<T> fmt::Display for TrySendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                TrySendError::Full(..) => "no available capacity",
                TrySendError::Closed(..) => "channel closed",
            }
        )
    }
}

pub trait ChannelLike<T> {
    fn try_recv(&mut self) -> Result<T, TryRecvError>;

    fn try_send(&mut self, message: T) -> Result<(), TrySendError<T>>;

    fn close(&mut self);

    fn is_closed(&mut self) -> bool;

    fn register_receiver(&mut self);

    fn deregister_receiver(&mut self);

    fn register_sender(&mut self);

    fn deregister_sender(&mut self);

    fn set_receiver_waker(&mut self, receiver_waker: Waker);

    fn set_senders_waker(&mut self, senders_waker: Waker);
}

pub struct ChannelState<T, const N: usize> {
    buf: [MaybeUninit<UnsafeCell<T>>; N],
    read_pos: usize,
    write_pos: usize,
    full: bool,
    closing: bool,
    closed: bool,
    receiver_registered: bool,
    senders_registered: u32,
    receiver_waker: Option<Waker>,
    senders_waker: Option<Waker>,
}

impl<T, const N: usize> ChannelState<T, N> {
    const INIT: MaybeUninit<UnsafeCell<T>> = MaybeUninit::uninit();

    const fn new() -> Self {
        let buf = [Self::INIT; N];
        let read_pos = 0;
        let write_pos = 0;
        let full = false;
        let closing = false;
        let closed = false;
        let receiver_registered = false;
        let senders_registered = 0;
        let receiver_waker = None;
        let senders_waker = None;
        ChannelState {
            buf,
            read_pos,
            write_pos,
            full,
            closing,
            closed,
            receiver_registered,
            senders_registered,
            receiver_waker,
            senders_waker,
        }
    }
}

/// A a bounded mpsc channel for communicating between asynchronous tasks
/// with backpressure.
///
/// The channel will buffer up to the provided number of messages.  Once the
/// buffer is full, attempts to `send` new messages will wait until a message is
/// received from the channel.
///
/// All data sent will become available in the same order as it was sent.
pub struct Channel<M, T, const N: usize>
where
    M: Mutex<Data = ()>,
{
    mutex: M,
    state: ChannelState<T, N>,
}

pub type WithCriticalSections = CriticalSectionMutex<()>;

impl<T, const N: usize> Channel<WithCriticalSections, T, N> {
    /// Establish a new bounded channel using critical sections. Critical sections
    /// should be used only single core targets where communication is required
    /// from exception mode e.g. interrupt handlers. To create one:
    ///
    /// ```
    /// use embassy::util::mpsc;
    /// use embassy::util::mpsc::{Channel, ChannelCell, WithCriticalSections};
    ///
    /// // Declare a bounded channel of 3 u32s.
    /// let mut channel = ChannelCell::new(mpsc::Channel::<WithCriticalSections, u32, 3>::with_critical_sections());
    /// // once we have a channel, obtain its sender and receiver
    /// let (sender, receiver) = mpsc::split(&channel);
    /// ```
    pub const fn with_critical_sections() -> Self {
        let mutex = CriticalSectionMutex::new(());
        let state = ChannelState::new();
        Channel { mutex, state }
    }
}

pub type WithThreadModeOnly = ThreadModeMutex<()>;

impl<T, const N: usize> Channel<WithThreadModeOnly, T, N> {
    /// Establish a new bounded channel for use in Cortex-M thread mode. Thread
    /// mode is intended for application threads on a single core, not interrupts.
    /// As such, only one task at a time can acquire a resource and so this
    /// channel avoids all locks. To create one:
    ///
    /// ``` no_run
    /// use embassy::util::mpsc;
    /// use embassy::util::mpsc::{Channel, ChannelCell, WithThreadModeOnly};
    ///
    /// // Declare a bounded channel of 3 u32s.
    /// let mut channel = ChannelCell::new(Channel::<WithThreadModeOnly, u32, 3>::with_thread_mode_only());
    /// // once we have a channel, obtain its sender and receiver
    /// let (sender, receiver) = mpsc::split(&channel);
    /// ```
    pub const fn with_thread_mode_only() -> Self {
        let mutex = ThreadModeMutex::new(());
        let state = ChannelState::new();
        Channel { mutex, state }
    }
}

impl<M, T, const N: usize> ChannelLike<T> for Channel<M, T, N>
where
    M: Mutex<Data = ()>,
{
    fn try_recv(&mut self) -> Result<T, TryRecvError> {
        let state = &mut self.state;
        self.mutex.lock(|_| {
            if !state.closed {
                if state.read_pos != state.write_pos || state.full {
                    if state.full {
                        state.full = false;
                        if let Some(w) = state.senders_waker.take() {
                            w.wake();
                        }
                    }
                    let message =
                        unsafe { (state.buf[state.read_pos]).assume_init_mut().get().read() };
                    state.read_pos = (state.read_pos + 1) % state.buf.len();
                    Ok(message)
                } else if !state.closing {
                    Err(TryRecvError::Empty)
                } else {
                    state.closed = true;
                    if let Some(w) = state.senders_waker.take() {
                        w.wake();
                    }
                    Err(TryRecvError::Closed)
                }
            } else {
                Err(TryRecvError::Closed)
            }
        })
    }

    fn try_send(&mut self, message: T) -> Result<(), TrySendError<T>> {
        let state = &mut self.state;
        self.mutex.lock(|_| {
            if !state.closed {
                if !state.full {
                    state.buf[state.write_pos] = MaybeUninit::new(message.into());
                    state.write_pos = (state.write_pos + 1) % state.buf.len();
                    if state.write_pos == state.read_pos {
                        state.full = true;
                    }
                    if let Some(w) = state.receiver_waker.take() {
                        w.wake();
                    }
                    Ok(())
                } else {
                    Err(TrySendError::Full(message))
                }
            } else {
                Err(TrySendError::Closed(message))
            }
        })
    }

    fn close(&mut self) {
        let state = &mut self.state;
        self.mutex.lock(|_| {
            if let Some(w) = state.receiver_waker.take() {
                w.wake();
            }
            state.closing = true;
        });
    }

    fn is_closed(&mut self) -> bool {
        let state = &self.state;
        self.mutex.lock(|_| state.closing || state.closed)
    }

    fn register_receiver(&mut self) {
        let state = &mut self.state;
        self.mutex.lock(|_| {
            assert!(!state.receiver_registered);
            state.receiver_registered = true;
        });
    }

    fn deregister_receiver(&mut self) {
        let state = &mut self.state;
        self.mutex.lock(|_| {
            if state.receiver_registered {
                state.closed = true;
                if let Some(w) = state.senders_waker.take() {
                    w.wake();
                }
            }
            state.receiver_registered = false;
        })
    }

    fn register_sender(&mut self) {
        let state = &mut self.state;
        self.mutex.lock(|_| {
            state.senders_registered += 1;
        })
    }

    fn deregister_sender(&mut self) {
        let state = &mut self.state;
        self.mutex.lock(|_| {
            assert!(state.senders_registered > 0);
            state.senders_registered -= 1;
            if state.senders_registered == 0 {
                if let Some(w) = state.receiver_waker.take() {
                    w.wake();
                }
                state.closing = true;
            }
        })
    }

    fn set_receiver_waker(&mut self, receiver_waker: Waker) {
        let state = &mut self.state;
        self.mutex.lock(|_| {
            state.receiver_waker = Some(receiver_waker);
        })
    }

    fn set_senders_waker(&mut self, senders_waker: Waker) {
        let state = &mut self.state;
        self.mutex.lock(|_| {

            // Dispose of any existing sender causing them to be polled again.
            // This could cause a spin given multiple concurrent senders, however given that
            // most sends only block waiting for the receiver to become active, this should
            // be a short-lived activity. The upside is a greatly simplified implementation
            // that avoids the need for intrusive linked-lists and unsafe operations on pinned
            // pointers.
            if let Some(waker) = state.senders_waker.clone() {
                if !senders_waker.will_wake(&waker) {
                    trace!("Waking an an active send waker due to being superseded with a new one. While benign, please report this.");
                    waker.wake();
                }
            }
            state.senders_waker = Some(senders_waker);
        })
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use futures::task::SpawnExt;
    use futures_executor::ThreadPool;
    use futures_timer::Delay;

    use super::*;

    fn capacity<M, T, const N: usize>(c: &Channel<M, T, N>) -> usize
    where
        M: Mutex<Data = ()>,
    {
        if !c.state.full {
            if c.state.write_pos > c.state.read_pos {
                (c.state.buf.len() - c.state.write_pos) + c.state.read_pos
            } else {
                (c.state.buf.len() - c.state.read_pos) + c.state.write_pos
            }
        } else {
            0
        }
    }

    /// A mutex that does nothing - useful for our testing purposes
    pub struct NoopMutex<T> {
        inner: UnsafeCell<T>,
    }

    impl<T> NoopMutex<T> {
        pub const fn new(value: T) -> Self {
            NoopMutex {
                inner: UnsafeCell::new(value),
            }
        }
    }

    impl<T> NoopMutex<T> {
        pub fn borrow(&self) -> &T {
            unsafe { &*self.inner.get() }
        }
    }

    impl<T> Mutex for NoopMutex<T> {
        type Data = T;

        fn lock<R>(&mut self, f: impl FnOnce(&Self::Data) -> R) -> R {
            f(self.borrow())
        }
    }

    pub type WithNoThreads = NoopMutex<()>;

    impl<T, const N: usize> Channel<WithNoThreads, T, N> {
        pub const fn with_no_threads() -> Self {
            let mutex = NoopMutex::new(());
            let state = ChannelState::new();
            Channel { mutex, state }
        }
    }

    #[test]
    fn sending_once() {
        let mut c = Channel::<WithNoThreads, u32, 3>::with_no_threads();
        assert!(c.try_send(1).is_ok());
        assert_eq!(capacity(&c), 2);
    }

    #[test]
    fn sending_when_full() {
        let mut c = Channel::<WithNoThreads, u32, 3>::with_no_threads();
        let _ = c.try_send(1);
        let _ = c.try_send(1);
        let _ = c.try_send(1);
        match c.try_send(2) {
            Err(TrySendError::Full(2)) => assert!(true),
            _ => assert!(false),
        }
        assert_eq!(capacity(&c), 0);
    }

    #[test]
    fn sending_when_closed() {
        let mut c = Channel::<WithNoThreads, u32, 3>::with_no_threads();
        c.state.closed = true;
        match c.try_send(2) {
            Err(TrySendError::Closed(2)) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn receiving_once_with_one_send() {
        let mut c = Channel::<WithNoThreads, u32, 3>::with_no_threads();
        assert!(c.try_send(1).is_ok());
        assert_eq!(c.try_recv().unwrap(), 1);
        assert_eq!(capacity(&c), 3);
    }

    #[test]
    fn receiving_when_empty() {
        let mut c = Channel::<WithNoThreads, u32, 3>::with_no_threads();
        match c.try_recv() {
            Err(TryRecvError::Empty) => assert!(true),
            _ => assert!(false),
        }
        assert_eq!(capacity(&c), 3);
    }

    #[test]
    fn receiving_when_closed() {
        let mut c = Channel::<WithNoThreads, u32, 3>::with_no_threads();
        c.state.closed = true;
        match c.try_recv() {
            Err(TryRecvError::Closed) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn simple_send_and_receive() {
        let c = ChannelCell::new(Channel::<WithNoThreads, u32, 3>::with_no_threads());
        let (s, r) = split(&c);
        assert!(s.clone().try_send(1).is_ok());
        assert_eq!(r.try_recv().unwrap(), 1);
    }

    #[test]
    fn should_close_without_sender() {
        let c = ChannelCell::new(Channel::<WithNoThreads, u32, 3>::with_no_threads());
        let (s, r) = split(&c);
        drop(s);
        match r.try_recv() {
            Err(TryRecvError::Closed) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn should_close_once_drained() {
        let c = ChannelCell::new(Channel::<WithNoThreads, u32, 3>::with_no_threads());
        let (s, r) = split(&c);
        assert!(s.try_send(1).is_ok());
        drop(s);
        assert_eq!(r.try_recv().unwrap(), 1);
        match r.try_recv() {
            Err(TryRecvError::Closed) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn should_reject_send_when_receiver_dropped() {
        let c = ChannelCell::new(Channel::<WithNoThreads, u32, 3>::with_no_threads());
        let (s, r) = split(&c);
        drop(r);
        match s.try_send(1) {
            Err(TrySendError::Closed(1)) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn should_reject_send_when_channel_closed() {
        let c = ChannelCell::new(Channel::<WithNoThreads, u32, 3>::with_no_threads());
        let (s, mut r) = split(&c);
        assert!(s.try_send(1).is_ok());
        r.close();
        assert_eq!(r.try_recv().unwrap(), 1);
        match r.try_recv() {
            Err(TryRecvError::Closed) => assert!(true),
            _ => assert!(false),
        }
        assert!(s.is_closed());
    }

    #[futures_test::test]
    async fn receiver_closes_when_sender_dropped_async() {
        let executor = ThreadPool::new().unwrap();

        static mut CHANNEL: ChannelCell<Channel<WithCriticalSections, u32, 3>> =
            ChannelCell::new(Channel::with_critical_sections());
        let (s, mut r) = split(unsafe { &CHANNEL });
        assert!(executor
            .spawn(async move {
                drop(s);
            })
            .is_ok());
        assert_eq!(r.recv().await, None);
    }

    #[futures_test::test]
    async fn receiver_receives_given_try_send_async() {
        let executor = ThreadPool::new().unwrap();

        static mut CHANNEL: ChannelCell<Channel<WithCriticalSections, u32, 3>> =
            ChannelCell::new(Channel::with_critical_sections());
        let (s, mut r) = split(unsafe { &CHANNEL });
        assert!(executor
            .spawn(async move {
                assert!(s.try_send(1).is_ok());
            })
            .is_ok());
        assert_eq!(r.recv().await, Some(1));
    }

    #[futures_test::test]
    async fn sender_send_completes_if_capacity() {
        static mut CHANNEL: ChannelCell<Channel<WithCriticalSections, u32, 1>> =
            ChannelCell::new(Channel::with_critical_sections());
        let (s, mut r) = split(unsafe { &CHANNEL });
        assert!(s.send(1).await.is_ok());
        assert_eq!(r.recv().await, Some(1));
    }

    #[futures_test::test]
    async fn sender_send_completes_if_closed() {
        static mut CHANNEL: ChannelCell<Channel<WithCriticalSections, u32, 1>> =
            ChannelCell::new(Channel::with_critical_sections());
        let (s, r) = split(unsafe { &CHANNEL });
        drop(r);
        match s.send(1).await {
            Err(SendError(1)) => assert!(true),
            _ => assert!(false),
        }
    }

    #[futures_test::test]
    async fn senders_sends_wait_until_capacity() {
        let executor = ThreadPool::new().unwrap();

        static mut CHANNEL: ChannelCell<Channel<WithCriticalSections, u32, 1>> =
            ChannelCell::new(Channel::with_critical_sections());
        let (s0, mut r) = split(unsafe { &CHANNEL });
        assert!(s0.try_send(1).is_ok());
        let s1 = s0.clone();
        let send_task_1 = executor.spawn_with_handle(async move { s0.send(2).await });
        let send_task_2 = executor.spawn_with_handle(async move { s1.send(3).await });
        // Wish I could think of a means of determining that the async send is waiting instead.
        // However, I've used the debugger to observe that the send does indeed wait.
        assert!(Delay::new(Duration::from_millis(500)).await.is_ok());
        assert_eq!(r.recv().await, Some(1));
        assert!(executor
            .spawn(async move { while let Some(_) = r.recv().await {} })
            .is_ok());
        assert!(send_task_1.unwrap().await.is_ok());
        assert!(send_task_2.unwrap().await.is_ok());
    }

    #[futures_test::test]
    async fn sender_close_completes_if_closing() {
        static mut CHANNEL: ChannelCell<Channel<WithCriticalSections, u32, 1>> =
            ChannelCell::new(Channel::with_critical_sections());
        let (s, mut r) = split(unsafe { &CHANNEL });
        r.close();
        s.closed().await;
    }

    #[futures_test::test]
    async fn sender_close_completes_if_closed() {
        static mut CHANNEL: ChannelCell<Channel<WithCriticalSections, u32, 1>> =
            ChannelCell::new(Channel::with_critical_sections());
        let (s, r) = split(unsafe { &CHANNEL });
        drop(r);
        s.closed().await;
    }
}
