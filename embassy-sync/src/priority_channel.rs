//! A queue for sending values between asynchronous tasks.
//!
//! Similar to a [`Channel`](crate::channel::Channel), however [`PriorityChannel`] sifts higher priority items to the front of the queue.
//! Priority is determined by the `Ord` trait. Priority behavior is determined by the [`Kind`](heapless::binary_heap::Kind) parameter of the channel.

use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub use heapless::binary_heap::{Kind, Max, Min};
use heapless::BinaryHeap;

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;
use crate::channel::{DynamicChannel, DynamicReceiver, DynamicSender, TryReceiveError, TrySendError};
use crate::waitqueue::WakerRegistration;

/// Send-only access to a [`PriorityChannel`].
pub struct Sender<'ch, M, T, K, const N: usize>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
    channel: &'ch PriorityChannel<M, T, K, N>,
}

impl<'ch, M, T, K, const N: usize> Clone for Sender<'ch, M, T, K, N>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
    fn clone(&self) -> Self {
        Sender { channel: self.channel }
    }
}

impl<'ch, M, T, K, const N: usize> Copy for Sender<'ch, M, T, K, N>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
}

impl<'ch, M, T, K, const N: usize> Sender<'ch, M, T, K, N>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
    /// Sends a value.
    ///
    /// See [`PriorityChannel::send()`]
    pub fn send(&self, message: T) -> SendFuture<'ch, M, T, K, N> {
        self.channel.send(message)
    }

    /// Attempt to immediately send a message.
    ///
    /// See [`PriorityChannel::send()`]
    pub fn try_send(&self, message: T) -> Result<(), TrySendError<T>> {
        self.channel.try_send(message)
    }

    /// Allows a poll_fn to poll until the channel is ready to send
    ///
    /// See [`PriorityChannel::poll_ready_to_send()`]
    pub fn poll_ready_to_send(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.channel.poll_ready_to_send(cx)
    }
}

impl<'ch, M, T, K, const N: usize> From<Sender<'ch, M, T, K, N>> for DynamicSender<'ch, T>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
    fn from(s: Sender<'ch, M, T, K, N>) -> Self {
        Self { channel: s.channel }
    }
}

/// Receive-only access to a [`PriorityChannel`].
pub struct Receiver<'ch, M, T, K, const N: usize>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
    channel: &'ch PriorityChannel<M, T, K, N>,
}

impl<'ch, M, T, K, const N: usize> Clone for Receiver<'ch, M, T, K, N>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
    fn clone(&self) -> Self {
        Receiver { channel: self.channel }
    }
}

impl<'ch, M, T, K, const N: usize> Copy for Receiver<'ch, M, T, K, N>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
}

impl<'ch, M, T, K, const N: usize> Receiver<'ch, M, T, K, N>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
    /// Receive the next value.
    ///
    /// See [`PriorityChannel::receive()`].
    pub fn receive(&self) -> ReceiveFuture<'_, M, T, K, N> {
        self.channel.receive()
    }

    /// Attempt to immediately receive the next value.
    ///
    /// See [`PriorityChannel::try_receive()`]
    pub fn try_receive(&self) -> Result<T, TryReceiveError> {
        self.channel.try_receive()
    }

    /// Allows a poll_fn to poll until the channel is ready to receive
    ///
    /// See [`PriorityChannel::poll_ready_to_receive()`]
    pub fn poll_ready_to_receive(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.channel.poll_ready_to_receive(cx)
    }

    /// Poll the channel for the next item
    ///
    /// See [`PriorityChannel::poll_receive()`]
    pub fn poll_receive(&self, cx: &mut Context<'_>) -> Poll<T> {
        self.channel.poll_receive(cx)
    }
}

impl<'ch, M, T, K, const N: usize> From<Receiver<'ch, M, T, K, N>> for DynamicReceiver<'ch, T>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
    fn from(s: Receiver<'ch, M, T, K, N>) -> Self {
        Self { channel: s.channel }
    }
}

/// Future returned by [`PriorityChannel::receive`] and  [`Receiver::receive`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ReceiveFuture<'ch, M, T, K, const N: usize>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
    channel: &'ch PriorityChannel<M, T, K, N>,
}

impl<'ch, M, T, K, const N: usize> Future for ReceiveFuture<'ch, M, T, K, N>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        self.channel.poll_receive(cx)
    }
}

/// Future returned by [`PriorityChannel::send`] and  [`Sender::send`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct SendFuture<'ch, M, T, K, const N: usize>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
    channel: &'ch PriorityChannel<M, T, K, N>,
    message: Option<T>,
}

impl<'ch, M, T, K, const N: usize> Future for SendFuture<'ch, M, T, K, N>
where
    T: Ord,
    K: Kind,
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

impl<'ch, M, T, K, const N: usize> Unpin for SendFuture<'ch, M, T, K, N>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
}

struct ChannelState<T, K, const N: usize> {
    queue: BinaryHeap<T, K, N>,
    receiver_waker: WakerRegistration,
    senders_waker: WakerRegistration,
}

impl<T, K, const N: usize> ChannelState<T, K, N>
where
    T: Ord,
    K: Kind,
{
    const fn new() -> Self {
        ChannelState {
            queue: BinaryHeap::new(),
            receiver_waker: WakerRegistration::new(),
            senders_waker: WakerRegistration::new(),
        }
    }

    fn try_receive(&mut self) -> Result<T, TryReceiveError> {
        self.try_receive_with_context(None)
    }

    fn try_receive_with_context(&mut self, cx: Option<&mut Context<'_>>) -> Result<T, TryReceiveError> {
        if self.queue.len() == self.queue.capacity() {
            self.senders_waker.wake();
        }

        if let Some(message) = self.queue.pop() {
            Ok(message)
        } else {
            if let Some(cx) = cx {
                self.receiver_waker.register(cx.waker());
            }
            Err(TryReceiveError::Empty)
        }
    }

    fn poll_receive(&mut self, cx: &mut Context<'_>) -> Poll<T> {
        if self.queue.len() == self.queue.capacity() {
            self.senders_waker.wake();
        }

        if let Some(message) = self.queue.pop() {
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
        match self.queue.push(message) {
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

        if !self.queue.len() == self.queue.capacity() {
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
/// Sent data may be reordered based on their priorty within the channel.
/// For example, in a [`Max`](heapless::binary_heap::Max) [`PriorityChannel`]
/// containing `u32`'s, data sent in the following order `[1, 2, 3]` will be received as `[3, 2, 1]`.
pub struct PriorityChannel<M, T, K, const N: usize>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
    inner: Mutex<M, RefCell<ChannelState<T, K, N>>>,
}

impl<M, T, K, const N: usize> PriorityChannel<M, T, K, N>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
    /// Establish a new bounded channel. For example, to create one with a NoopMutex:
    ///
    /// ```
    /// use embassy_sync::priority_channel::{PriorityChannel, Max};
    /// use embassy_sync::blocking_mutex::raw::NoopRawMutex;
    ///
    /// // Declare a bounded channel of 3 u32s.
    /// let mut channel = PriorityChannel::<NoopRawMutex, u32, Max, 3>::new();
    /// ```
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(RefCell::new(ChannelState::new())),
        }
    }

    fn lock<R>(&self, f: impl FnOnce(&mut ChannelState<T, K, N>) -> R) -> R {
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
    pub fn sender(&self) -> Sender<'_, M, T, K, N> {
        Sender { channel: self }
    }

    /// Get a receiver for this channel.
    pub fn receiver(&self) -> Receiver<'_, M, T, K, N> {
        Receiver { channel: self }
    }

    /// Send a value, waiting until there is capacity.
    ///
    /// Sending completes when the value has been pushed to the channel's queue.
    /// This doesn't mean the value has been received yet.
    pub fn send(&self, message: T) -> SendFuture<'_, M, T, K, N> {
        SendFuture {
            channel: self,
            message: Some(message),
        }
    }

    /// Attempt to immediately send a message.
    ///
    /// This method differs from [`send`](PriorityChannel::send) by returning immediately if the channel's
    /// buffer is full, instead of waiting.
    ///
    /// # Errors
    ///
    /// If the channel capacity has been reached, i.e., the channel has `n`
    /// buffered values where `n` is the argument passed to [`PriorityChannel`], then an
    /// error is returned.
    pub fn try_send(&self, message: T) -> Result<(), TrySendError<T>> {
        self.lock(|c| c.try_send(message))
    }

    /// Receive the next value.
    ///
    /// If there are no messages in the channel's buffer, this method will
    /// wait until a message is sent.
    pub fn receive(&self) -> ReceiveFuture<'_, M, T, K, N> {
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
impl<M, T, K, const N: usize> DynamicChannel<T> for PriorityChannel<M, T, K, N>
where
    T: Ord,
    K: Kind,
    M: RawMutex,
{
    fn try_send_with_context(&self, m: T, cx: Option<&mut Context<'_>>) -> Result<(), TrySendError<T>> {
        PriorityChannel::try_send_with_context(self, m, cx)
    }

    fn try_receive_with_context(&self, cx: Option<&mut Context<'_>>) -> Result<T, TryReceiveError> {
        PriorityChannel::try_receive_with_context(self, cx)
    }

    fn poll_ready_to_send(&self, cx: &mut Context<'_>) -> Poll<()> {
        PriorityChannel::poll_ready_to_send(self, cx)
    }

    fn poll_ready_to_receive(&self, cx: &mut Context<'_>) -> Poll<()> {
        PriorityChannel::poll_ready_to_receive(self, cx)
    }

    fn poll_receive(&self, cx: &mut Context<'_>) -> Poll<T> {
        PriorityChannel::poll_receive(self, cx)
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use futures_executor::ThreadPool;
    use futures_timer::Delay;
    use futures_util::task::SpawnExt;
    use heapless::binary_heap::{Kind, Max};
    use static_cell::StaticCell;

    use super::*;
    use crate::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};

    fn capacity<T, K, const N: usize>(c: &ChannelState<T, K, N>) -> usize
    where
        T: Ord,
        K: Kind,
    {
        c.queue.capacity() - c.queue.len()
    }

    #[test]
    fn sending_once() {
        let mut c = ChannelState::<u32, Max, 3>::new();
        assert!(c.try_send(1).is_ok());
        assert_eq!(capacity(&c), 2);
    }

    #[test]
    fn sending_when_full() {
        let mut c = ChannelState::<u32, Max, 3>::new();
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
    fn send_priority() {
        // Prio channel with kind `Max` sifts larger numbers to the front of the queue
        let mut c = ChannelState::<u32, Max, 3>::new();
        assert!(c.try_send(1).is_ok());
        assert!(c.try_send(2).is_ok());
        assert!(c.try_send(3).is_ok());
        assert_eq!(c.try_receive().unwrap(), 3);
        assert_eq!(c.try_receive().unwrap(), 2);
        assert_eq!(c.try_receive().unwrap(), 1);
    }

    #[test]
    fn receiving_once_with_one_send() {
        let mut c = ChannelState::<u32, Max, 3>::new();
        assert!(c.try_send(1).is_ok());
        assert_eq!(c.try_receive().unwrap(), 1);
        assert_eq!(capacity(&c), 3);
    }

    #[test]
    fn receiving_when_empty() {
        let mut c = ChannelState::<u32, Max, 3>::new();
        match c.try_receive() {
            Err(TryReceiveError::Empty) => assert!(true),
            _ => assert!(false),
        }
        assert_eq!(capacity(&c), 3);
    }

    #[test]
    fn simple_send_and_receive() {
        let c = PriorityChannel::<NoopRawMutex, u32, Max, 3>::new();
        assert!(c.try_send(1).is_ok());
        assert_eq!(c.try_receive().unwrap(), 1);
    }

    #[test]
    fn cloning() {
        let c = PriorityChannel::<NoopRawMutex, u32, Max, 3>::new();
        let r1 = c.receiver();
        let s1 = c.sender();

        let _ = r1.clone();
        let _ = s1.clone();
    }

    #[test]
    fn dynamic_dispatch() {
        let c = PriorityChannel::<NoopRawMutex, u32, Max, 3>::new();
        let s: DynamicSender<'_, u32> = c.sender().into();
        let r: DynamicReceiver<'_, u32> = c.receiver().into();

        assert!(s.try_send(1).is_ok());
        assert_eq!(r.try_receive().unwrap(), 1);
    }

    #[futures_test::test]
    async fn receiver_receives_given_try_send_async() {
        let executor = ThreadPool::new().unwrap();

        static CHANNEL: StaticCell<PriorityChannel<CriticalSectionRawMutex, u32, Max, 3>> = StaticCell::new();
        let c = &*CHANNEL.init(PriorityChannel::new());
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
        let c = PriorityChannel::<CriticalSectionRawMutex, u32, Max, 1>::new();
        c.send(1).await;
        assert_eq!(c.receive().await, 1);
    }

    #[futures_test::test]
    async fn senders_sends_wait_until_capacity() {
        let executor = ThreadPool::new().unwrap();

        static CHANNEL: StaticCell<PriorityChannel<CriticalSectionRawMutex, u32, Max, 1>> = StaticCell::new();
        let c = &*CHANNEL.init(PriorityChannel::new());
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
