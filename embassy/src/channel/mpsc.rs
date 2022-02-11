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
//! This channel and its associated types were derived from <https://docs.rs/tokio/0.1.22/tokio/sync/mpsc/fn.channel.html>

use core::cell::RefCell;
use core::fmt;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;
use core::task::Waker;

use futures::Future;
use heapless::Deque;

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;
use crate::waitqueue::WakerRegistration;

/// Send values to the associated `Receiver`.
///
/// Instances are created by the [`split`](split) function.
pub struct Sender<'ch, M, T, const N: usize>
where
    M: RawMutex,
{
    channel: &'ch Channel<M, T, N>,
}

/// Receive values from the associated `Sender`.
///
/// Instances are created by the [`split`](split) function.
pub struct Receiver<'ch, M, T, const N: usize>
where
    M: RawMutex,
{
    channel: &'ch Channel<M, T, N>,
}

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
/// use embassy::channel::mpsc;
/// use embassy::channel::mpsc::{Channel, WithThreadModeOnly};
///
/// let (sender, receiver) = {
///    let mut channel = Channel::<WithThreadModeOnly, u32, 3>::with_thread_mode_only();
///     mpsc::split(&mut channel)
/// };
/// ```
pub fn split<M, T, const N: usize>(
    channel: &mut Channel<M, T, N>,
) -> (Sender<M, T, N>, Receiver<M, T, N>)
where
    M: RawMutex,
{
    let sender = Sender { channel };
    let receiver = Receiver { channel };
    channel.lock(|c| {
        c.register_receiver();
        c.register_sender();
    });
    (sender, receiver)
}

impl<'ch, M, T, const N: usize> Receiver<'ch, M, T, N>
where
    M: RawMutex,
{
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
    pub fn recv(&mut self) -> RecvFuture<'_, M, T, N> {
        RecvFuture {
            channel: self.channel,
        }
    }

    /// Attempts to immediately receive a message on this `Receiver`
    ///
    /// This method will either receive a message from the channel immediately or return an error
    /// if the channel is empty.
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.channel.lock(|c| c.try_recv())
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
        self.channel.lock(|c| c.close())
    }
}

impl<'ch, M, T, const N: usize> Drop for Receiver<'ch, M, T, N>
where
    M: RawMutex,
{
    fn drop(&mut self) {
        self.channel.lock(|c| c.deregister_receiver())
    }
}

pub struct RecvFuture<'ch, M, T, const N: usize>
where
    M: RawMutex,
{
    channel: &'ch Channel<M, T, N>,
}

impl<'ch, M, T, const N: usize> Future for RecvFuture<'ch, M, T, N>
where
    M: RawMutex,
{
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        self.channel
            .lock(|c| match c.try_recv_with_context(Some(cx)) {
                Ok(v) => Poll::Ready(Some(v)),
                Err(TryRecvError::Closed) => Poll::Ready(None),
                Err(TryRecvError::Empty) => Poll::Pending,
            })
    }
}

impl<'ch, M, T, const N: usize> Sender<'ch, M, T, N>
where
    M: RawMutex,
{
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
    pub fn send(&self, message: T) -> SendFuture<'ch, M, T, N> {
        SendFuture {
            channel: self.channel,
            message: Some(message),
        }
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
        self.channel.lock(|c| c.try_send(message))
    }

    /// Completes when the receiver has dropped.
    ///
    /// This allows the producers to get notified when interest in the produced
    /// values is canceled and immediately stop doing work.
    pub async fn closed(&self) {
        CloseFuture {
            channel: self.channel,
        }
        .await
    }

    /// Checks if the channel has been closed. This happens when the
    /// [`Receiver`] is dropped, or when the [`Receiver::close`] method is
    /// called.
    ///
    /// [`Receiver`]: Receiver
    /// [`Receiver::close`]: Receiver::close
    pub fn is_closed(&self) -> bool {
        self.channel.lock(|c| c.is_closed())
    }
}

pub struct SendFuture<'ch, M, T, const N: usize>
where
    M: RawMutex,
{
    channel: &'ch Channel<M, T, N>,
    message: Option<T>,
}

impl<'ch, M, T, const N: usize> Future for SendFuture<'ch, M, T, N>
where
    M: RawMutex,
{
    type Output = Result<(), SendError<T>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.message.take() {
            Some(m) => match self.channel.lock(|c| c.try_send_with_context(m, Some(cx))) {
                Ok(..) => Poll::Ready(Ok(())),
                Err(TrySendError::Closed(m)) => Poll::Ready(Err(SendError(m))),
                Err(TrySendError::Full(m)) => {
                    self.message = Some(m);
                    Poll::Pending
                }
            },
            None => panic!("Message cannot be None"),
        }
    }
}

impl<'ch, M, T, const N: usize> Unpin for SendFuture<'ch, M, T, N> where M: RawMutex {}

struct CloseFuture<'ch, M, T, const N: usize>
where
    M: RawMutex,
{
    channel: &'ch Channel<M, T, N>,
}

impl<'ch, M, T, const N: usize> Future for CloseFuture<'ch, M, T, N>
where
    M: RawMutex,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.channel.lock(|c| c.is_closed_with_context(Some(cx))) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl<'ch, M, T, const N: usize> Drop for Sender<'ch, M, T, N>
where
    M: RawMutex,
{
    fn drop(&mut self) {
        self.channel.lock(|c| c.deregister_sender())
    }
}

impl<'ch, M, T, const N: usize> Clone for Sender<'ch, M, T, N>
where
    M: RawMutex,
{
    fn clone(&self) -> Self {
        self.channel.lock(|c| c.register_sender());
        Sender {
            channel: self.channel,
        }
    }
}

/// An error returned from the [`try_recv`] method.
///
/// [`try_recv`]: Receiver::try_recv
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

#[cfg(feature = "defmt")]
impl<T> defmt::Format for SendError<T> {
    fn format(&self, fmt: defmt::Formatter<'_>) {
        defmt::write!(fmt, "channel closed")
    }
}

/// This enumeration is the list of the possible error outcomes for the
/// [try_send](Sender::try_send) method.
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

#[cfg(feature = "defmt")]
impl<T> defmt::Format for TrySendError<T> {
    fn format(&self, fmt: defmt::Formatter<'_>) {
        match self {
            TrySendError::Full(..) => defmt::write!(fmt, "no available capacity"),
            TrySendError::Closed(..) => defmt::write!(fmt, "channel closed"),
        }
    }
}

struct ChannelState<T, const N: usize> {
    queue: Deque<T, N>,
    closed: bool,
    receiver_registered: bool,
    senders_registered: u32,
    receiver_waker: WakerRegistration,
    senders_waker: WakerRegistration,
}

impl<T, const N: usize> ChannelState<T, N> {
    const fn new() -> Self {
        ChannelState {
            queue: Deque::new(),
            closed: false,
            receiver_registered: false,
            senders_registered: 0,
            receiver_waker: WakerRegistration::new(),
            senders_waker: WakerRegistration::new(),
        }
    }

    fn try_recv(&mut self) -> Result<T, TryRecvError> {
        self.try_recv_with_context(None)
    }

    fn try_recv_with_context(&mut self, cx: Option<&mut Context<'_>>) -> Result<T, TryRecvError> {
        if self.queue.is_full() {
            self.senders_waker.wake();
        }

        if let Some(message) = self.queue.pop_front() {
            Ok(message)
        } else if !self.closed {
            if let Some(cx) = cx {
                self.set_receiver_waker(cx.waker());
            }
            Err(TryRecvError::Empty)
        } else {
            Err(TryRecvError::Closed)
        }
    }

    fn try_send(&mut self, message: T) -> Result<(), TrySendError<T>> {
        self.try_send_with_context(message, None)
    }

    fn try_send_with_context(
        &mut self,
        message: T,
        cx: Option<&mut Context<'_>>,
    ) -> Result<(), TrySendError<T>> {
        if self.closed {
            return Err(TrySendError::Closed(message));
        }

        match self.queue.push_back(message) {
            Ok(()) => {
                self.receiver_waker.wake();

                Ok(())
            }
            Err(message) => {
                cx.into_iter()
                    .for_each(|cx| self.set_senders_waker(cx.waker()));
                Err(TrySendError::Full(message))
            }
        }
    }

    fn close(&mut self) {
        self.receiver_waker.wake();
        self.closed = true;
    }

    fn is_closed(&mut self) -> bool {
        self.is_closed_with_context(None)
    }

    fn is_closed_with_context(&mut self, cx: Option<&mut Context<'_>>) -> bool {
        if self.closed {
            cx.into_iter()
                .for_each(|cx| self.set_senders_waker(cx.waker()));
            true
        } else {
            false
        }
    }

    fn register_receiver(&mut self) {
        assert!(!self.receiver_registered);
        self.receiver_registered = true;
    }

    fn deregister_receiver(&mut self) {
        if self.receiver_registered {
            self.closed = true;
            self.senders_waker.wake();
        }
        self.receiver_registered = false;
    }

    fn register_sender(&mut self) {
        self.senders_registered += 1;
    }

    fn deregister_sender(&mut self) {
        assert!(self.senders_registered > 0);
        self.senders_registered -= 1;
        if self.senders_registered == 0 {
            self.receiver_waker.wake();
            self.closed = true;
        }
    }

    fn set_receiver_waker(&mut self, receiver_waker: &Waker) {
        self.receiver_waker.register(receiver_waker);
    }

    fn set_senders_waker(&mut self, senders_waker: &Waker) {
        // Dispose of any existing sender causing them to be polled again.
        // This could cause a spin given multiple concurrent senders, however given that
        // most sends only block waiting for the receiver to become active, this should
        // be a short-lived activity. The upside is a greatly simplified implementation
        // that avoids the need for intrusive linked-lists and unsafe operations on pinned
        // pointers.
        self.senders_waker.wake();
        self.senders_waker.register(senders_waker);
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
    M: RawMutex,
{
    inner: Mutex<M, RefCell<ChannelState<T, N>>>,
}

impl<M, T, const N: usize> Channel<M, T, N>
where
    M: RawMutex,
{
    /// Establish a new bounded channel. For example, to create one with a NoopMutex:
    ///
    /// ```
    /// use embassy::channel::mpsc;
    /// use embassy::blocking_mutex::raw::NoopRawMutex;
    /// use embassy::channel::mpsc::Channel;
    ///
    /// // Declare a bounded channel of 3 u32s.
    /// let mut channel = Channel::<NoopRawMutex, u32, 3>::new();
    /// // once we have a channel, obtain its sender and receiver
    /// let (sender, receiver) = mpsc::split(&mut channel);
    /// ```
    #[cfg(feature = "nightly")]
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(RefCell::new(ChannelState::new())),
        }
    }

    /// Establish a new bounded channel. For example, to create one with a NoopMutex:
    ///
    /// ```
    /// use embassy::channel::mpsc;
    /// use embassy::blocking_mutex::raw::NoopRawMutex;
    /// use embassy::channel::mpsc::Channel;
    ///
    /// // Declare a bounded channel of 3 u32s.
    /// let mut channel = Channel::<NoopRawMutex, u32, 3>::new();
    /// // once we have a channel, obtain its sender and receiver
    /// let (sender, receiver) = mpsc::split(&mut channel);
    /// ```
    #[cfg(not(feature = "nightly"))]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(RefCell::new(ChannelState::new())),
        }
    }

    fn lock<R>(&self, f: impl FnOnce(&mut ChannelState<T, N>) -> R) -> R {
        self.inner.lock(|rc| f(&mut *rc.borrow_mut()))
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use futures::task::SpawnExt;
    use futures_executor::ThreadPool;
    use futures_timer::Delay;

    use crate::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
    use crate::util::Forever;

    use super::*;

    fn capacity<T, const N: usize>(c: &ChannelState<T, N>) -> usize {
        c.queue.capacity() - c.queue.len()
    }

    #[test]
    fn sending_once() {
        let mut c = ChannelState::<u32, 3>::new();
        assert!(c.try_send(1).is_ok());
        assert_eq!(capacity(&c), 2);
    }

    #[test]
    fn sending_when_full() {
        let mut c = ChannelState::<u32, 3>::new();
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
        let mut c = ChannelState::<u32, 3>::new();
        c.closed = true;
        match c.try_send(2) {
            Err(TrySendError::Closed(2)) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn receiving_once_with_one_send() {
        let mut c = ChannelState::<u32, 3>::new();
        assert!(c.try_send(1).is_ok());
        assert_eq!(c.try_recv().unwrap(), 1);
        assert_eq!(capacity(&c), 3);
    }

    #[test]
    fn receiving_when_empty() {
        let mut c = ChannelState::<u32, 3>::new();
        match c.try_recv() {
            Err(TryRecvError::Empty) => assert!(true),
            _ => assert!(false),
        }
        assert_eq!(capacity(&c), 3);
    }

    #[test]
    fn receiving_when_closed() {
        let mut c = ChannelState::<u32, 3>::new();
        c.closed = true;
        match c.try_recv() {
            Err(TryRecvError::Closed) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn simple_send_and_receive() {
        let mut c = Channel::<NoopRawMutex, u32, 3>::new();
        let (s, r) = split(&mut c);
        assert!(s.clone().try_send(1).is_ok());
        assert_eq!(r.try_recv().unwrap(), 1);
    }

    #[test]
    fn should_close_without_sender() {
        let mut c = Channel::<NoopRawMutex, u32, 3>::new();
        let (s, r) = split(&mut c);
        drop(s);
        match r.try_recv() {
            Err(TryRecvError::Closed) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn should_close_once_drained() {
        let mut c = Channel::<NoopRawMutex, u32, 3>::new();
        let (s, r) = split(&mut c);
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
        let mut c = Channel::<NoopRawMutex, u32, 3>::new();
        let (s, r) = split(&mut c);
        drop(r);
        match s.try_send(1) {
            Err(TrySendError::Closed(1)) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn should_reject_send_when_channel_closed() {
        let mut c = Channel::<NoopRawMutex, u32, 3>::new();
        let (s, mut r) = split(&mut c);
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

        static CHANNEL: Forever<Channel<CriticalSectionRawMutex, u32, 3>> = Forever::new();
        let c = CHANNEL.put(Channel::new());
        let (s, mut r) = split(c);
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

        static CHANNEL: Forever<Channel<CriticalSectionRawMutex, u32, 3>> = Forever::new();
        let c = CHANNEL.put(Channel::new());
        let (s, mut r) = split(c);
        assert!(executor
            .spawn(async move {
                assert!(s.try_send(1).is_ok());
            })
            .is_ok());
        assert_eq!(r.recv().await, Some(1));
    }

    #[futures_test::test]
    async fn sender_send_completes_if_capacity() {
        let mut c = Channel::<CriticalSectionRawMutex, u32, 1>::new();
        let (s, mut r) = split(&mut c);
        assert!(s.send(1).await.is_ok());
        assert_eq!(r.recv().await, Some(1));
    }

    #[futures_test::test]
    async fn sender_send_completes_if_closed() {
        static CHANNEL: Forever<Channel<CriticalSectionRawMutex, u32, 1>> = Forever::new();
        let c = CHANNEL.put(Channel::new());
        let (s, r) = split(c);
        drop(r);
        match s.send(1).await {
            Err(SendError(1)) => assert!(true),
            _ => assert!(false),
        }
    }

    #[futures_test::test]
    async fn senders_sends_wait_until_capacity() {
        let executor = ThreadPool::new().unwrap();

        static CHANNEL: Forever<Channel<CriticalSectionRawMutex, u32, 1>> = Forever::new();
        let c = CHANNEL.put(Channel::new());
        let (s0, mut r) = split(c);
        assert!(s0.try_send(1).is_ok());
        let s1 = s0.clone();
        let send_task_1 = executor.spawn_with_handle(async move { s0.send(2).await });
        let send_task_2 = executor.spawn_with_handle(async move { s1.send(3).await });
        // Wish I could think of a means of determining that the async send is waiting instead.
        // However, I've used the debugger to observe that the send does indeed wait.
        Delay::new(Duration::from_millis(500)).await;
        assert_eq!(r.recv().await, Some(1));
        assert!(executor
            .spawn(async move { while let Some(_) = r.recv().await {} })
            .is_ok());
        assert!(send_task_1.unwrap().await.is_ok());
        assert!(send_task_2.unwrap().await.is_ok());
    }

    #[futures_test::test]
    async fn sender_close_completes_if_closing() {
        static CHANNEL: Forever<Channel<CriticalSectionRawMutex, u32, 1>> = Forever::new();
        let c = CHANNEL.put(Channel::new());
        let (s, mut r) = split(c);
        r.close();
        s.closed().await;
    }

    #[futures_test::test]
    async fn sender_close_completes_if_closed() {
        static CHANNEL: Forever<Channel<CriticalSectionRawMutex, u32, 1>> = Forever::new();
        let c = CHANNEL.put(Channel::new());
        let (s, r) = split(c);
        drop(r);
        s.closed().await;
    }
}
