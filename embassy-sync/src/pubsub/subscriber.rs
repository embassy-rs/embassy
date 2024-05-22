//! Implementation of anything directly subscriber related

use core::future::Future;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::task::{Context, Poll};

use super::{PubSubBehavior, PubSubChannel, WaitResult};
use crate::blocking_mutex::raw::RawMutex;

/// A subscriber to a channel
pub struct Sub<'a, PSB: PubSubBehavior<T> + ?Sized, T: Clone> {
    /// The message id of the next message we are yet to receive
    next_message_id: u64,
    /// The channel we are a subscriber to
    channel: &'a PSB,
    _phantom: PhantomData<T>,
}

impl<'a, PSB: PubSubBehavior<T> + ?Sized, T: Clone> Sub<'a, PSB, T> {
    pub(super) fn new(next_message_id: u64, channel: &'a PSB) -> Self {
        Self {
            next_message_id,
            channel,
            _phantom: Default::default(),
        }
    }

    /// Wait for a published message
    pub fn next_message<'s>(&'s mut self) -> SubscriberWaitFuture<'s, 'a, PSB, T> {
        SubscriberWaitFuture { subscriber: self }
    }

    /// Wait for a published message (ignoring lag results)
    pub async fn next_message_pure(&mut self) -> T {
        loop {
            match self.next_message().await {
                WaitResult::Lagged(_) => continue,
                WaitResult::Message(message) => break message,
            }
        }
    }

    /// Try to see if there's a published message we haven't received yet.
    ///
    /// This function does not peek. The message is received if there is one.
    pub fn try_next_message(&mut self) -> Option<WaitResult<T>> {
        match self.channel.get_message_with_context(&mut self.next_message_id, None) {
            Poll::Ready(result) => Some(result),
            Poll::Pending => None,
        }
    }

    /// Try to see if there's a published message we haven't received yet (ignoring lag results).
    ///
    /// This function does not peek. The message is received if there is one.
    pub fn try_next_message_pure(&mut self) -> Option<T> {
        loop {
            match self.try_next_message() {
                Some(WaitResult::Lagged(_)) => continue,
                Some(WaitResult::Message(message)) => break Some(message),
                None => break None,
            }
        }
    }

    /// The amount of messages this subscriber hasn't received yet. This is like [Self::len] but specifically
    /// for this subscriber.
    pub fn available(&self) -> u64 {
        self.channel.available(self.next_message_id)
    }

    /// Returns the maximum number of elements the ***channel*** can hold.
    pub fn capacity(&self) -> usize {
        self.channel.capacity()
    }

    /// Returns the free capacity of the ***channel***.
    ///
    /// This is equivalent to `capacity() - len()`
    pub fn free_capacity(&self) -> usize {
        self.channel.free_capacity()
    }

    /// Clears all elements in the ***channel***.
    pub fn clear(&self) {
        self.channel.clear();
    }

    /// Returns the number of elements currently in the ***channel***.
    /// See [Self::available] for how many messages are available for this subscriber.
    pub fn len(&self) -> usize {
        self.channel.len()
    }

    /// Returns whether the ***channel*** is empty.
    pub fn is_empty(&self) -> bool {
        self.channel.is_empty()
    }

    /// Returns whether the ***channel*** is full.
    pub fn is_full(&self) -> bool {
        self.channel.is_full()
    }
}

impl<'a, PSB: PubSubBehavior<T> + ?Sized, T: Clone> Drop for Sub<'a, PSB, T> {
    fn drop(&mut self) {
        self.channel.unregister_subscriber(self.next_message_id)
    }
}

impl<'a, PSB: PubSubBehavior<T> + ?Sized, T: Clone> Unpin for Sub<'a, PSB, T> {}

/// Warning: The stream implementation ignores lag results and returns all messages.
/// This might miss some messages without you knowing it.
impl<'a, PSB: PubSubBehavior<T> + ?Sized, T: Clone> futures_util::Stream for Sub<'a, PSB, T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self
            .channel
            .get_message_with_context(&mut self.next_message_id, Some(cx))
        {
            Poll::Ready(WaitResult::Message(message)) => Poll::Ready(Some(message)),
            Poll::Ready(WaitResult::Lagged(_)) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A subscriber that holds a dynamic reference to the channel
pub struct DynSubscriber<'a, T: Clone>(pub(super) Sub<'a, dyn PubSubBehavior<T> + 'a, T>);

impl<'a, T: Clone> Deref for DynSubscriber<'a, T> {
    type Target = Sub<'a, dyn PubSubBehavior<T> + 'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: Clone> DerefMut for DynSubscriber<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A subscriber that holds a generic reference to the channel
pub struct Subscriber<'a, M: RawMutex, T: Clone, const CAP: usize, const SUBS: usize, const PUBS: usize>(
    pub(super) Sub<'a, PubSubChannel<M, T, CAP, SUBS, PUBS>, T>,
);

impl<'a, M: RawMutex, T: Clone, const CAP: usize, const SUBS: usize, const PUBS: usize> Deref
    for Subscriber<'a, M, T, CAP, SUBS, PUBS>
{
    type Target = Sub<'a, PubSubChannel<M, T, CAP, SUBS, PUBS>, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, M: RawMutex, T: Clone, const CAP: usize, const SUBS: usize, const PUBS: usize> DerefMut
    for Subscriber<'a, M, T, CAP, SUBS, PUBS>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Future for the subscriber wait action
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct SubscriberWaitFuture<'s, 'a, PSB: PubSubBehavior<T> + ?Sized, T: Clone> {
    subscriber: &'s mut Sub<'a, PSB, T>,
}

impl<'s, 'a, PSB: PubSubBehavior<T> + ?Sized, T: Clone> Future for SubscriberWaitFuture<'s, 'a, PSB, T> {
    type Output = WaitResult<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.subscriber
            .channel
            .get_message_with_context(&mut self.subscriber.next_message_id, Some(cx))
    }
}

impl<'s, 'a, PSB: PubSubBehavior<T> + ?Sized, T: Clone> Unpin for SubscriberWaitFuture<'s, 'a, PSB, T> {}
