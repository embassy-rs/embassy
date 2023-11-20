//! A queue for sending values between asynchronous tasks.
//!
//! It can be used concurrently by multiple producers (senders) and multiple
//! consumers (receivers), i.e. it is an  "MPMC channel".
//!
//! Receivers are competing for messages. So a message that is received by
//! one receiver is not received by any other.
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
//!

use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use heapless::Deque;

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;
use crate::waitqueue::WakerRegistration;

/// Send-only access to a [`Channel`].
pub struct Sender<'ch, M, T, const N: usize>
where
    M: RawMutex,
{
    channel: &'ch Channel<M, T, N>,
}

impl<'ch, M, T, const N: usize> Clone for Sender<'ch, M, T, N>
where
    M: RawMutex,
{
    fn clone(&self) -> Self {
        Sender { channel: self.channel }
    }
}

impl<'ch, M, T, const N: usize> Copy for Sender<'ch, M, T, N> where M: RawMutex {}

impl<'ch, M, T, const N: usize> Sender<'ch, M, T, N>
where
    M: RawMutex,
{
    /// Sends a value.
    ///
    /// See [`Channel::send()`]
    pub fn send(&self, message: T) -> SendFuture<'ch, M, T, N> {
        self.channel.send(message)
    }

    /// Attempt to immediately send a message.
    ///
    /// See [`Channel::send()`]
    pub fn try_send(&self, message: T) -> Result<(), TrySendError<T>> {
        self.channel.try_send(message)
    }

    /// Allows a poll_fn to poll until the channel is ready to send
    ///
    /// See [`Channel::poll_ready_to_send()`]
    pub fn poll_ready_to_send(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.channel.poll_ready_to_send(cx)
    }
}

/// Send-only access to a [`Channel`] without knowing channel size.
pub struct DynamicSender<'ch, T> {
    pub(crate) channel: &'ch dyn DynamicChannel<T>,
}

impl<'ch, T> Clone for DynamicSender<'ch, T> {
    fn clone(&self) -> Self {
        DynamicSender { channel: self.channel }
    }
}

impl<'ch, T> Copy for DynamicSender<'ch, T> {}

impl<'ch, M, T, const N: usize> From<Sender<'ch, M, T, N>> for DynamicSender<'ch, T>
where
    M: RawMutex,
{
    fn from(s: Sender<'ch, M, T, N>) -> Self {
        Self { channel: s.channel }
    }
}

impl<'ch, T> DynamicSender<'ch, T> {
    /// Sends a value.
    ///
    /// See [`Channel::send()`]
    pub fn send(&self, message: T) -> DynamicSendFuture<'ch, T> {
        DynamicSendFuture {
            channel: self.channel,
            message: Some(message),
        }
    }

    /// Attempt to immediately send a message.
    ///
    /// See [`Channel::send()`]
    pub fn try_send(&self, message: T) -> Result<(), TrySendError<T>> {
        self.channel.try_send_with_context(message, None)
    }

    /// Allows a poll_fn to poll until the channel is ready to send
    ///
    /// See [`Channel::poll_ready_to_send()`]
    pub fn poll_ready_to_send(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.channel.poll_ready_to_send(cx)
    }
}

/// Receive-only access to a [`Channel`].
pub struct Receiver<'ch, M, T, const N: usize>
where
    M: RawMutex,
{
    channel: &'ch Channel<M, T, N>,
}

impl<'ch, M, T, const N: usize> Clone for Receiver<'ch, M, T, N>
where
    M: RawMutex,
{
    fn clone(&self) -> Self {
        Receiver { channel: self.channel }
    }
}

impl<'ch, M, T, const N: usize> Copy for Receiver<'ch, M, T, N> where M: RawMutex {}

impl<'ch, M, T, const N: usize> Receiver<'ch, M, T, N>
where
    M: RawMutex,
{
    /// Receive the next value.
    ///
    /// See [`Channel::receive()`].
    pub fn receive(&self) -> ReceiveFuture<'_, M, T, N> {
        self.channel.receive()
    }

    /// Attempt to immediately receive the next value.
    ///
    /// See [`Channel::try_receive()`]
    pub fn try_receive(&self) -> Result<T, TryReceiveError> {
        self.channel.try_receive()
    }

    /// Allows a poll_fn to poll until the channel is ready to receive
    ///
    /// See [`Channel::poll_ready_to_receive()`]
    pub fn poll_ready_to_receive(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.channel.poll_ready_to_receive(cx)
    }

    /// Poll the channel for the next item
    ///
    /// See [`Channel::poll_receive()`]
    pub fn poll_receive(&self, cx: &mut Context<'_>) -> Poll<T> {
        self.channel.poll_receive(cx)
    }
}

/// Receive-only access to a [`Channel`] without knowing channel size.
pub struct DynamicReceiver<'ch, T> {
    pub(crate) channel: &'ch dyn DynamicChannel<T>,
}

impl<'ch, T> Clone for DynamicReceiver<'ch, T> {
    fn clone(&self) -> Self {
        DynamicReceiver { channel: self.channel }
    }
}

impl<'ch, T> Copy for DynamicReceiver<'ch, T> {}

impl<'ch, T> DynamicReceiver<'ch, T> {
    /// Receive the next value.
    ///
    /// See [`Channel::receive()`].
    pub fn receive(&self) -> DynamicReceiveFuture<'_, T> {
        DynamicReceiveFuture { channel: self.channel }
    }

    /// Attempt to immediately receive the next value.
    ///
    /// See [`Channel::try_receive()`]
    pub fn try_receive(&self) -> Result<T, TryReceiveError> {
        self.channel.try_receive_with_context(None)
    }

    /// Allows a poll_fn to poll until the channel is ready to receive
    ///
    /// See [`Channel::poll_ready_to_receive()`]
    pub fn poll_ready_to_receive(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.channel.poll_ready_to_receive(cx)
    }

    /// Poll the channel for the next item
    ///
    /// See [`Channel::poll_receive()`]
    pub fn poll_receive(&self, cx: &mut Context<'_>) -> Poll<T> {
        self.channel.poll_receive(cx)
    }
}

impl<'ch, M, T, const N: usize> From<Receiver<'ch, M, T, N>> for DynamicReceiver<'ch, T>
where
    M: RawMutex,
{
    fn from(s: Receiver<'ch, M, T, N>) -> Self {
        Self { channel: s.channel }
    }
}

/// Future returned by [`Channel::receive`] and  [`Receiver::receive`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ReceiveFuture<'ch, M, T, const N: usize>
where
    M: RawMutex,
{
    channel: &'ch Channel<M, T, N>,
}

impl<'ch, M, T, const N: usize> Future for ReceiveFuture<'ch, M, T, N>
where
    M: RawMutex,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        self.channel.poll_receive(cx)
    }
}

/// Future returned by [`DynamicReceiver::receive`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct DynamicReceiveFuture<'ch, T> {
    channel: &'ch dyn DynamicChannel<T>,
}

impl<'ch, T> Future for DynamicReceiveFuture<'ch, T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        match self.channel.try_receive_with_context(Some(cx)) {
            Ok(v) => Poll::Ready(v),
            Err(TryReceiveError::Empty) => Poll::Pending,
        }
    }
}

/// Future returned by [`Channel::send`] and  [`Sender::send`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
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
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.message.take() {
            Some(m) => match self.channel.try_send_with_context(m, Some(cx)) {
                Ok(..) => Poll::Ready(()),
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

/// Future returned by [`DynamicSender::send`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct DynamicSendFuture<'ch, T> {
    channel: &'ch dyn DynamicChannel<T>,
    message: Option<T>,
}

impl<'ch, T> Future for DynamicSendFuture<'ch, T> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.message.take() {
            Some(m) => match self.channel.try_send_with_context(m, Some(cx)) {
                Ok(..) => Poll::Ready(()),
                Err(TrySendError::Full(m)) => {
                    self.message = Some(m);
                    Poll::Pending
                }
            },
            None => panic!("Message cannot be None"),
        }
    }
}

impl<'ch, T> Unpin for DynamicSendFuture<'ch, T> {}

pub(crate) trait DynamicChannel<T> {
    fn try_send_with_context(&self, message: T, cx: Option<&mut Context<'_>>) -> Result<(), TrySendError<T>>;

    fn try_receive_with_context(&self, cx: Option<&mut Context<'_>>) -> Result<T, TryReceiveError>;

    fn poll_ready_to_send(&self, cx: &mut Context<'_>) -> Poll<()>;
    fn poll_ready_to_receive(&self, cx: &mut Context<'_>) -> Poll<()>;

    fn poll_receive(&self, cx: &mut Context<'_>) -> Poll<T>;
}

/// Error returned by [`try_receive`](Channel::try_receive).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TryReceiveError {
    /// A message could not be received because the channel is empty.
    Empty,
}

/// Error returned by [`try_send`](Channel::try_send).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TrySendError<T> {
    /// The data could not be sent on the channel because the channel is
    /// currently full and sending would require blocking.
    Full(T),
}

struct ChannelState<T, const N: usize> {
    queue: Deque<T, N>,
    receiver_waker: WakerRegistration,
    senders_waker: WakerRegistration,
}

impl<T, const N: usize> ChannelState<T, N> {
    const fn new() -> Self {
        ChannelState {
            queue: Deque::new(),
            receiver_waker: WakerRegistration::new(),
            senders_waker: WakerRegistration::new(),
        }
    }

    fn try_receive(&mut self) -> Result<T, TryReceiveError> {
        self.try_receive_with_context(None)
    }

    fn try_receive_with_context(&mut self, cx: Option<&mut Context<'_>>) -> Result<T, TryReceiveError> {
        if self.queue.is_full() {
            self.senders_waker.wake();
        }

        if let Some(message) = self.queue.pop_front() {
            Ok(message)
        } else {
            if let Some(cx) = cx {
                self.receiver_waker.register(cx.waker());
            }
            Err(TryReceiveError::Empty)
        }
    }

    fn poll_receive(&mut self, cx: &mut Context<'_>) -> Poll<T> {
        if self.queue.is_full() {
            self.senders_waker.wake();
        }

        if let Some(message) = self.queue.pop_front() {
            Poll::Ready(message)
        } else {
            self.receiver_waker.register(cx.waker());
            Poll::Pending
        }
    }

    fn poll_ready_to_receive(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        self.receiver_waker.register(cx.waker());

        if !self.queue.is_empty() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    fn try_send(&mut self, message: T) -> Result<(), TrySendError<T>> {
        self.try_send_with_context(message, None)
    }

    fn try_send_with_context(&mut self, message: T, cx: Option<&mut Context<'_>>) -> Result<(), TrySendError<T>> {
        match self.queue.push_back(message) {
            Ok(()) => {
                self.receiver_waker.wake();
                Ok(())
            }
            Err(message) => {
                if let Some(cx) = cx {
                    self.senders_waker.register(cx.waker());
                }
                Err(TrySendError::Full(message))
            }
        }
    }

    fn poll_ready_to_send(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        self.senders_waker.register(cx.waker());

        if !self.queue.is_full() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

/// A bounded channel for communicating between asynchronous tasks
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
    /// use embassy_sync::channel::Channel;
    /// use embassy_sync::blocking_mutex::raw::NoopRawMutex;
    ///
    /// // Declare a bounded channel of 3 u32s.
    /// let mut channel = Channel::<NoopRawMutex, u32, 3>::new();
    /// ```
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(RefCell::new(ChannelState::new())),
        }
    }

    fn lock<R>(&self, f: impl FnOnce(&mut ChannelState<T, N>) -> R) -> R {
        self.inner.lock(|rc| f(&mut *unwrap!(rc.try_borrow_mut())))
    }

    fn try_receive_with_context(&self, cx: Option<&mut Context<'_>>) -> Result<T, TryReceiveError> {
        self.lock(|c| c.try_receive_with_context(cx))
    }

    /// Poll the channel for the next message
    pub fn poll_receive(&self, cx: &mut Context<'_>) -> Poll<T> {
        self.lock(|c| c.poll_receive(cx))
    }

    fn try_send_with_context(&self, m: T, cx: Option<&mut Context<'_>>) -> Result<(), TrySendError<T>> {
        self.lock(|c| c.try_send_with_context(m, cx))
    }

    /// Allows a poll_fn to poll until the channel is ready to receive
    pub fn poll_ready_to_receive(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.lock(|c| c.poll_ready_to_receive(cx))
    }

    /// Allows a poll_fn to poll until the channel is ready to send
    pub fn poll_ready_to_send(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.lock(|c| c.poll_ready_to_send(cx))
    }

    /// Get a sender for this channel.
    pub fn sender(&self) -> Sender<'_, M, T, N> {
        Sender { channel: self }
    }

    /// Get a receiver for this channel.
    pub fn receiver(&self) -> Receiver<'_, M, T, N> {
        Receiver { channel: self }
    }

    /// Send a value, waiting until there is capacity.
    ///
    /// Sending completes when the value has been pushed to the channel's queue.
    /// This doesn't mean the value has been received yet.
    pub fn send(&self, message: T) -> SendFuture<'_, M, T, N> {
        SendFuture {
            channel: self,
            message: Some(message),
        }
    }

    /// Attempt to immediately send a message.
    ///
    /// This method differs from [`send`](Channel::send) by returning immediately if the channel's
    /// buffer is full, instead of waiting.
    ///
    /// # Errors
    ///
    /// If the channel capacity has been reached, i.e., the channel has `n`
    /// buffered values where `n` is the argument passed to [`Channel`], then an
    /// error is returned.
    pub fn try_send(&self, message: T) -> Result<(), TrySendError<T>> {
        self.lock(|c| c.try_send(message))
    }

    /// Receive the next value.
    ///
    /// If there are no messages in the channel's buffer, this method will
    /// wait until a message is sent.
    pub fn receive(&self) -> ReceiveFuture<'_, M, T, N> {
        ReceiveFuture { channel: self }
    }

    /// Attempt to immediately receive a message.
    ///
    /// This method will either receive a message from the channel immediately or return an error
    /// if the channel is empty.
    pub fn try_receive(&self) -> Result<T, TryReceiveError> {
        self.lock(|c| c.try_receive())
    }
}

/// Implements the DynamicChannel to allow creating types that are unaware of the queue size with the
/// tradeoff cost of dynamic dispatch.
impl<M, T, const N: usize> DynamicChannel<T> for Channel<M, T, N>
where
    M: RawMutex,
{
    fn try_send_with_context(&self, m: T, cx: Option<&mut Context<'_>>) -> Result<(), TrySendError<T>> {
        Channel::try_send_with_context(self, m, cx)
    }

    fn try_receive_with_context(&self, cx: Option<&mut Context<'_>>) -> Result<T, TryReceiveError> {
        Channel::try_receive_with_context(self, cx)
    }

    fn poll_ready_to_send(&self, cx: &mut Context<'_>) -> Poll<()> {
        Channel::poll_ready_to_send(self, cx)
    }

    fn poll_ready_to_receive(&self, cx: &mut Context<'_>) -> Poll<()> {
        Channel::poll_ready_to_receive(self, cx)
    }

    fn poll_receive(&self, cx: &mut Context<'_>) -> Poll<T> {
        Channel::poll_receive(self, cx)
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use futures_executor::ThreadPool;
    use futures_timer::Delay;
    use futures_util::task::SpawnExt;
    use static_cell::StaticCell;

    use super::*;
    use crate::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};

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
    fn receiving_once_with_one_send() {
        let mut c = ChannelState::<u32, 3>::new();
        assert!(c.try_send(1).is_ok());
        assert_eq!(c.try_receive().unwrap(), 1);
        assert_eq!(capacity(&c), 3);
    }

    #[test]
    fn receiving_when_empty() {
        let mut c = ChannelState::<u32, 3>::new();
        match c.try_receive() {
            Err(TryReceiveError::Empty) => assert!(true),
            _ => assert!(false),
        }
        assert_eq!(capacity(&c), 3);
    }

    #[test]
    fn simple_send_and_receive() {
        let c = Channel::<NoopRawMutex, u32, 3>::new();
        assert!(c.try_send(1).is_ok());
        assert_eq!(c.try_receive().unwrap(), 1);
    }

    #[test]
    fn cloning() {
        let c = Channel::<NoopRawMutex, u32, 3>::new();
        let r1 = c.receiver();
        let s1 = c.sender();

        let _ = r1.clone();
        let _ = s1.clone();
    }

    #[test]
    fn dynamic_dispatch() {
        let c = Channel::<NoopRawMutex, u32, 3>::new();
        let s: DynamicSender<'_, u32> = c.sender().into();
        let r: DynamicReceiver<'_, u32> = c.receiver().into();

        assert!(s.try_send(1).is_ok());
        assert_eq!(r.try_receive().unwrap(), 1);
    }

    #[futures_test::test]
    async fn receiver_receives_given_try_send_async() {
        let executor = ThreadPool::new().unwrap();

        static CHANNEL: StaticCell<Channel<CriticalSectionRawMutex, u32, 3>> = StaticCell::new();
        let c = &*CHANNEL.init(Channel::new());
        let c2 = c;
        assert!(executor
            .spawn(async move {
                assert!(c2.try_send(1).is_ok());
            })
            .is_ok());
        assert_eq!(c.receive().await, 1);
    }

    #[futures_test::test]
    async fn sender_send_completes_if_capacity() {
        let c = Channel::<CriticalSectionRawMutex, u32, 1>::new();
        c.send(1).await;
        assert_eq!(c.receive().await, 1);
    }

    #[futures_test::test]
    async fn senders_sends_wait_until_capacity() {
        let executor = ThreadPool::new().unwrap();

        static CHANNEL: StaticCell<Channel<CriticalSectionRawMutex, u32, 1>> = StaticCell::new();
        let c = &*CHANNEL.init(Channel::new());
        assert!(c.try_send(1).is_ok());

        let c2 = c;
        let send_task_1 = executor.spawn_with_handle(async move { c2.send(2).await });
        let c2 = c;
        let send_task_2 = executor.spawn_with_handle(async move { c2.send(3).await });
        // Wish I could think of a means of determining that the async send is waiting instead.
        // However, I've used the debugger to observe that the send does indeed wait.
        Delay::new(Duration::from_millis(500)).await;
        assert_eq!(c.receive().await, 1);
        assert!(executor
            .spawn(async move {
                loop {
                    c.receive().await;
                }
            })
            .is_ok());
        send_task_1.unwrap().await;
        send_task_2.unwrap().await;
    }
}
