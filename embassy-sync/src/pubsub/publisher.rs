//! Implementation of anything directly publisher related

use core::future::Future;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::task::{Context, Poll};

use super::{PubSubBehavior, PubSubChannel};
use crate::blocking_mutex::raw::RawMutex;

/// A publisher to a channel
pub struct Pub<'a, PSB: PubSubBehavior<T> + ?Sized, T: Clone> {
    /// The channel we are a publisher for
    channel: &'a PSB,
    _phantom: PhantomData<T>,
}

impl<'a, PSB: PubSubBehavior<T> + ?Sized, T: Clone> Pub<'a, PSB, T> {
    pub(super) fn new(channel: &'a PSB) -> Self {
        Self {
            channel,
            _phantom: Default::default(),
        }
    }

    /// Publish a message right now even when the queue is full.
    /// This may cause a subscriber to miss an older message.
    pub fn publish_immediate(&self, message: T) {
        self.channel.publish_immediate(message)
    }

    /// Publish a message. But if the message queue is full, wait for all subscribers to have read the last message
    pub fn publish<'s>(&'s self, message: T) -> PublisherWaitFuture<'s, 'a, PSB, T> {
        PublisherWaitFuture {
            message: Some(message),
            publisher: self,
        }
    }

    /// Publish a message if there is space in the message queue
    pub fn try_publish(&self, message: T) -> Result<(), T> {
        self.channel.publish_with_context(message, None)
    }

    /// The amount of messages that can still be published without having to wait or without having to lag the subscribers
    ///
    /// *Note: In the time between checking this and a publish action, other publishers may have had time to publish something.
    /// So checking doesn't give any guarantees.*
    pub fn space(&self) -> usize {
        self.channel.space()
    }
}

impl<'a, PSB: PubSubBehavior<T> + ?Sized, T: Clone> Drop for Pub<'a, PSB, T> {
    fn drop(&mut self) {
        self.channel.unregister_publisher()
    }
}

/// A publisher that holds a dynamic reference to the channel
pub struct DynPublisher<'a, T: Clone>(pub(super) Pub<'a, dyn PubSubBehavior<T> + 'a, T>);

impl<'a, T: Clone> Deref for DynPublisher<'a, T> {
    type Target = Pub<'a, dyn PubSubBehavior<T> + 'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: Clone> DerefMut for DynPublisher<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A publisher that holds a generic reference to the channel
pub struct Publisher<'a, M: RawMutex, T: Clone, const CAP: usize, const SUBS: usize, const PUBS: usize>(
    pub(super) Pub<'a, PubSubChannel<M, T, CAP, SUBS, PUBS>, T>,
);

impl<'a, M: RawMutex, T: Clone, const CAP: usize, const SUBS: usize, const PUBS: usize> Deref
    for Publisher<'a, M, T, CAP, SUBS, PUBS>
{
    type Target = Pub<'a, PubSubChannel<M, T, CAP, SUBS, PUBS>, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, M: RawMutex, T: Clone, const CAP: usize, const SUBS: usize, const PUBS: usize> DerefMut
    for Publisher<'a, M, T, CAP, SUBS, PUBS>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A publisher that can only use the `publish_immediate` function, but it doesn't have to be registered with the channel.
/// (So an infinite amount is possible)
pub struct ImmediatePub<'a, PSB: PubSubBehavior<T> + ?Sized, T: Clone> {
    /// The channel we are a publisher for
    channel: &'a PSB,
    _phantom: PhantomData<T>,
}

impl<'a, PSB: PubSubBehavior<T> + ?Sized, T: Clone> ImmediatePub<'a, PSB, T> {
    pub(super) fn new(channel: &'a PSB) -> Self {
        Self {
            channel,
            _phantom: Default::default(),
        }
    }
    /// Publish the message right now even when the queue is full.
    /// This may cause a subscriber to miss an older message.
    pub fn publish_immediate(&self, message: T) {
        self.channel.publish_immediate(message)
    }

    /// Publish a message if there is space in the message queue
    pub fn try_publish(&self, message: T) -> Result<(), T> {
        self.channel.publish_with_context(message, None)
    }
}

/// An immediate publisher that holds a dynamic reference to the channel
pub struct DynImmediatePublisher<'a, T: Clone>(pub(super) ImmediatePub<'a, dyn PubSubBehavior<T> + 'a, T>);

impl<'a, T: Clone> Deref for DynImmediatePublisher<'a, T> {
    type Target = ImmediatePub<'a, dyn PubSubBehavior<T> + 'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: Clone> DerefMut for DynImmediatePublisher<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// An immediate publisher that holds a generic reference to the channel
pub struct ImmediatePublisher<'a, M: RawMutex, T: Clone, const CAP: usize, const SUBS: usize, const PUBS: usize>(
    pub(super) ImmediatePub<'a, PubSubChannel<M, T, CAP, SUBS, PUBS>, T>,
);

impl<'a, M: RawMutex, T: Clone, const CAP: usize, const SUBS: usize, const PUBS: usize> Deref
    for ImmediatePublisher<'a, M, T, CAP, SUBS, PUBS>
{
    type Target = ImmediatePub<'a, PubSubChannel<M, T, CAP, SUBS, PUBS>, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, M: RawMutex, T: Clone, const CAP: usize, const SUBS: usize, const PUBS: usize> DerefMut
    for ImmediatePublisher<'a, M, T, CAP, SUBS, PUBS>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Future for the publisher wait action
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct PublisherWaitFuture<'s, 'a, PSB: PubSubBehavior<T> + ?Sized, T: Clone> {
    /// The message we need to publish
    message: Option<T>,
    publisher: &'s Pub<'a, PSB, T>,
}

impl<'s, 'a, PSB: PubSubBehavior<T> + ?Sized, T: Clone> Future for PublisherWaitFuture<'s, 'a, PSB, T> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let message = self.message.take().unwrap();
        match self.publisher.channel.publish_with_context(message, Some(cx)) {
            Ok(()) => Poll::Ready(()),
            Err(message) => {
                self.message = Some(message);
                Poll::Pending
            }
        }
    }
}

impl<'s, 'a, PSB: PubSubBehavior<T> + ?Sized, T: Clone> Unpin for PublisherWaitFuture<'s, 'a, PSB, T> {}
