//! Implementation of [PubSubChannel], a queue where published messages get received by all subscribers.

use core::cell::RefCell;
use core::fmt::Debug;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use heapless::Deque;

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;
use crate::waitqueue::WakerRegistration;

/// A broadcast channel implementation where multiple publishers can send messages to multiple subscribers
///
/// Any published message can be read by all subscribers.
/// A publisher can choose how it sends its message.
///
/// - With [Publisher::publish] the publisher has to wait until there is space in the internal message queue.
/// - With [Publisher::publish_immediate] the publisher doesn't await and instead lets the oldest message
/// in the queue drop if necessary. This will cause any [Subscriber] that missed the message to receive
/// an error to indicate that it has lagged.
pub struct PubSubChannel<M: RawMutex, T: Clone, const CAP: usize, const SUBS: usize, const PUBS: usize> {
    inner: Mutex<M, RefCell<PubSubState<T, CAP, SUBS, PUBS>>>,
}

impl<M: RawMutex, T: Clone, const CAP: usize, const SUBS: usize, const PUBS: usize>
    PubSubChannel<M, T, CAP, SUBS, PUBS>
{
    /// Create a new channel
    pub const fn new() -> Self {
        Self {
            inner: Mutex::const_new(M::INIT, RefCell::new(PubSubState::new())),
        }
    }

    /// Create a new subscriber. It will only receive messages that are published after its creation.
    ///
    /// If there are no subscriber slots left, an error will be returned.
    pub fn subscriber(&self) -> Result<Subscriber<'_, T>, Error> {
        self.inner.lock(|inner| {
            let mut s = inner.borrow_mut();

            // Search for an empty subscriber spot
            for (i, sub_spot) in s.subscriber_wakers.iter_mut().enumerate() {
                if sub_spot.is_none() {
                    // We've found a spot, so now fill it and create the subscriber
                    *sub_spot = Some(WakerRegistration::new());
                    return Ok(Subscriber {
                        subscriber_index: i,
                        next_message_id: s.next_message_id,
                        channel: self,
                    });
                }
            }

            // No spot was found, we're full
            Err(Error::MaximumSubscribersReached)
        })
    }

    /// Create a new publisher
    ///
    /// If there are no publisher slots left, an error will be returned.
    pub fn publisher(&self) -> Result<Publisher<'_, T>, Error> {
        self.inner.lock(|inner| {
            let mut s = inner.borrow_mut();

            // Search for an empty publisher spot
            for (i, pub_spot) in s.publisher_wakers.iter_mut().enumerate() {
                if pub_spot.is_none() {
                    // We've found a spot, so now fill it and create the subscriber
                    *pub_spot = Some(WakerRegistration::new());
                    return Ok(Publisher {
                        publisher_index: i,
                        channel: self,
                    });
                }
            }

            // No spot was found, we're full
            Err(Error::MaximumPublishersReached)
        })
    }

    /// Create a new publisher that can only send immediate messages.
    /// This kind of publisher does not take up a publisher slot.
    pub fn immediate_publisher(&self) -> ImmediatePublisher<'_, T> {
        ImmediatePublisher { channel: self }
    }
}

impl<M: RawMutex, T: Clone, const CAP: usize, const SUBS: usize, const PUBS: usize> PubSubBehavior<T>
    for PubSubChannel<M, T, CAP, SUBS, PUBS>
{
    fn get_message_with_context(
        &self,
        next_message_id: &mut u64,
        subscriber_index: usize,
        cx: Option<&mut Context<'_>>,
    ) -> Poll<WaitResult<T>> {
        self.inner.lock(|s| {
            let mut s = s.borrow_mut();

            // Check if we can read a message
            match s.get_message(*next_message_id) {
                // Yes, so we are done polling
                Some(WaitResult::Message(message)) => {
                    *next_message_id += 1;
                    Poll::Ready(WaitResult::Message(message))
                }
                // No, so we need to reregister our waker and sleep again
                None => {
                    if let Some(cx) = cx {
                        s.register_subscriber_waker(subscriber_index, cx.waker());
                    }
                    Poll::Pending
                }
                // We missed a couple of messages. We must do our internal bookkeeping and return that we lagged
                Some(WaitResult::Lagged(amount)) => {
                    *next_message_id += amount;
                    Poll::Ready(WaitResult::Lagged(amount))
                }
            }
        })
    }

    fn publish_with_context(&self, message: T, publisher_index: usize, cx: Option<&mut Context<'_>>) -> Result<(), T> {
        self.inner.lock(|s| {
            let mut s = s.borrow_mut();
            // Try to publish the message
            match s.try_publish(message) {
                // We did it, we are ready
                Ok(()) => Ok(()),
                // The queue is full, so we need to reregister our waker and go to sleep
                Err(message) => {
                    if let Some(cx) = cx {
                        s.register_publisher_waker(publisher_index, cx.waker());
                    }
                    Err(message)
                }
            }
        })
    }

    fn publish_immediate(&self, message: T) {
        self.inner.lock(|s| {
            let mut s = s.borrow_mut();
            s.publish_immediate(message)
        })
    }

    fn unregister_subscriber(&self, subscriber_index: usize, subscriber_next_message_id: u64) {
        self.inner.lock(|s| {
            let mut s = s.borrow_mut();
            s.unregister_subscriber(subscriber_index, subscriber_next_message_id)
        })
    }

    fn unregister_publisher(&self, publisher_index: usize) {
        self.inner.lock(|s| {
            let mut s = s.borrow_mut();
            s.unregister_publisher(publisher_index)
        })
    }
}

/// Internal state for the PubSub channel
struct PubSubState<T: Clone, const CAP: usize, const SUBS: usize, const PUBS: usize> {
    /// The queue contains the last messages that have been published and a countdown of how many subscribers are yet to read it
    queue: Deque<(T, usize), CAP>,
    /// Every message has an id.
    /// Don't worry, we won't run out.
    /// If a million messages were published every second, then the ID's would run out in about 584942 years.
    next_message_id: u64,
    /// Collection of wakers for Subscribers that are waiting.  
    /// The [Subscriber::subscriber_index] field indexes into this array.
    subscriber_wakers: [Option<WakerRegistration>; SUBS],
    /// Collection of wakers for Publishers that are waiting.  
    /// The [Publisher::publisher_index] field indexes into this array.
    publisher_wakers: [Option<WakerRegistration>; PUBS],
}

impl<T: Clone, const CAP: usize, const SUBS: usize, const PUBS: usize> PubSubState<T, CAP, SUBS, PUBS> {
    /// Create a new internal channel state
    const fn new() -> Self {
        const WAKER_INIT: Option<WakerRegistration> = None;
        Self {
            queue: Deque::new(),
            next_message_id: 0,
            subscriber_wakers: [WAKER_INIT; SUBS],
            publisher_wakers: [WAKER_INIT; PUBS],
        }
    }

    fn try_publish(&mut self, message: T) -> Result<(), T> {
        let active_subscriber_count = self.subscriber_wakers.iter().flatten().count();

        if active_subscriber_count == 0 {
            // We don't need to publish anything because there is no one to receive it
            return Ok(());
        }

        if self.queue.is_full() {
            return Err(message);
        }
        // We just did a check for this
        self.queue.push_back((message, active_subscriber_count)).ok().unwrap();

        self.next_message_id += 1;

        // Wake all of the subscribers
        for active_subscriber in self.subscriber_wakers.iter_mut().flatten() {
            active_subscriber.wake()
        }

        Ok(())
    }

    fn publish_immediate(&mut self, message: T) {
        // Make space in the queue if required
        if self.queue.is_full() {
            self.queue.pop_front();
        }

        // This will succeed because we made sure there is space
        self.try_publish(message).ok().unwrap();
    }

    fn get_message(&mut self, message_id: u64) -> Option<WaitResult<T>> {
        let start_id = self.next_message_id - self.queue.len() as u64;

        if message_id < start_id {
            return Some(WaitResult::Lagged(start_id - message_id));
        }

        let current_message_index = (message_id - start_id) as usize;

        if current_message_index >= self.queue.len() {
            return None;
        }

        // We've checked that the index is valid
        let queue_item = self.queue.iter_mut().nth(current_message_index).unwrap();

        // We're reading this item, so decrement the counter
        queue_item.1 -= 1;
        let message = queue_item.0.clone();

        if current_message_index == 0 && queue_item.1 == 0 {
            self.queue.pop_front();
            self.publisher_wakers.iter_mut().flatten().for_each(|w| w.wake());
        }

        Some(WaitResult::Message(message))
    }

    fn register_subscriber_waker(&mut self, subscriber_index: usize, waker: &Waker) {
        self.subscriber_wakers[subscriber_index]
            .as_mut()
            .unwrap()
            .register(waker);
    }

    fn register_publisher_waker(&mut self, publisher_index: usize, waker: &Waker) {
        self.publisher_wakers[publisher_index].as_mut().unwrap().register(waker);
    }

    fn unregister_subscriber(&mut self, subscriber_index: usize, subscriber_next_message_id: u64) {
        // Remove the subscriber from the wakers
        self.subscriber_wakers[subscriber_index] = None;

        // All messages that haven't been read yet by this subscriber must have their counter decremented
        let start_id = self.next_message_id - self.queue.len() as u64;
        if subscriber_next_message_id >= start_id {
            let current_message_index = (subscriber_next_message_id - start_id) as usize;
            self.queue
                .iter_mut()
                .skip(current_message_index)
                .for_each(|(_, counter)| *counter -= 1);
        }
    }

    fn unregister_publisher(&mut self, publisher_index: usize) {
        // Remove the publisher from the wakers
        self.publisher_wakers[publisher_index] = None;
    }
}

/// A subscriber to a channel
///
/// This instance carries a reference to the channel, but uses a trait object for it so that the channel's
/// generics are erased on this subscriber
pub struct Subscriber<'a, T: Clone> {
    /// Our index into the channel
    subscriber_index: usize,
    /// The message id of the next message we are yet to receive
    next_message_id: u64,
    /// The channel we are a subscriber to
    channel: &'a dyn PubSubBehavior<T>,
}

impl<'a, T: Clone> Subscriber<'a, T> {
    /// Wait for a published message
    pub fn next_message<'s>(&'s mut self) -> SubscriberWaitFuture<'s, 'a, T> {
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
        match self
            .channel
            .get_message_with_context(&mut self.next_message_id, self.subscriber_index, None)
        {
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
}

impl<'a, T: Clone> Drop for Subscriber<'a, T> {
    fn drop(&mut self) {
        self.channel
            .unregister_subscriber(self.subscriber_index, self.next_message_id)
    }
}

/// Warning: The stream implementation ignores lag results and returns all messages.
/// This might miss some messages without you knowing it.
impl<'a, T: Clone> futures::Stream for Subscriber<'a, T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = unsafe { self.get_unchecked_mut() };

        match this
            .channel
            .get_message_with_context(&mut this.next_message_id, this.subscriber_index, Some(cx))
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

/// A publisher to a channel
///
/// This instance carries a reference to the channel, but uses a trait object for it so that the channel's
/// generics are erased on this subscriber
pub struct Publisher<'a, T: Clone> {
    /// Our index into the channel
    publisher_index: usize,
    /// The channel we are a publisher for
    channel: &'a dyn PubSubBehavior<T>,
}

impl<'a, T: Clone> Publisher<'a, T> {
    /// Publish a message right now even when the queue is full.
    /// This may cause a subscriber to miss an older message.
    pub fn publish_immediate(&self, message: T) {
        self.channel.publish_immediate(message)
    }

    /// Publish a message. But if the message queue is full, wait for all subscribers to have read the last message
    pub fn publish<'s>(&'s self, message: T) -> PublisherWaitFuture<'s, 'a, T> {
        PublisherWaitFuture {
            message: Some(message),
            publisher: self,
        }
    }

    /// Publish a message if there is space in the message queue
    pub fn try_publish(&self, message: T) -> Result<(), T> {
        self.channel.publish_with_context(message, self.publisher_index, None)
    }
}

impl<'a, T: Clone> Drop for Publisher<'a, T> {
    fn drop(&mut self) {
        self.channel.unregister_publisher(self.publisher_index)
    }
}

/// A publisher that can only use the `publish_immediate` function, but it doesn't have to be registered with the channel.
/// (So an infinite amount is possible)
pub struct ImmediatePublisher<'a, T: Clone> {
    /// The channel we are a publisher for
    channel: &'a dyn PubSubBehavior<T>,
}

impl<'a, T: Clone> ImmediatePublisher<'a, T> {
    /// Publish the message right now even when the queue is full.
    /// This may cause a subscriber to miss an older message.
    pub fn publish_immediate(&mut self, message: T) {
        self.channel.publish_immediate(message)
    }

    /// Publish a message if there is space in the message queue
    pub fn try_publish(&self, message: T) -> Result<(), T> {
        self.channel.publish_with_context(message, usize::MAX, None)
    }
}

/// Error type for the [PubSubChannel]
#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    /// All subscriber slots are used. To add another subscriber, first another subscriber must be dropped or
    /// the capacity of the channels must be increased.
    MaximumSubscribersReached,
    /// All publisher slots are used. To add another publisher, first another publisher must be dropped or
    /// the capacity of the channels must be increased.
    MaximumPublishersReached,
}

trait PubSubBehavior<T> {
    fn get_message_with_context(
        &self,
        next_message_id: &mut u64,
        subscriber_index: usize,
        cx: Option<&mut Context<'_>>,
    ) -> Poll<WaitResult<T>>;

    fn publish_with_context(&self, message: T, publisher_index: usize, cx: Option<&mut Context<'_>>) -> Result<(), T>;

    fn publish_immediate(&self, message: T);

    fn unregister_subscriber(&self, subscriber_index: usize, subscriber_next_message_id: u64);

    fn unregister_publisher(&self, publisher_index: usize);
}

/// Future for the subscriber wait action
pub struct SubscriberWaitFuture<'s, 'a, T: Clone> {
    subscriber: &'s mut Subscriber<'a, T>,
}

impl<'s, 'a, T: Clone> Future for SubscriberWaitFuture<'s, 'a, T> {
    type Output = WaitResult<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let sub_index = self.subscriber.subscriber_index;
        self.subscriber
            .channel
            .get_message_with_context(&mut self.subscriber.next_message_id, sub_index, Some(cx))
    }
}

impl<'s, 'a, T: Clone> Unpin for SubscriberWaitFuture<'s, 'a, T> {}

/// Future for the publisher wait action
pub struct PublisherWaitFuture<'s, 'a, T: Clone> {
    /// The message we need to publish
    message: Option<T>,
    publisher: &'s Publisher<'a, T>,
}

impl<'s, 'a, T: Clone> Future for PublisherWaitFuture<'s, 'a, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        let message = this.message.take().unwrap();
        match this
            .publisher
            .channel
            .publish_with_context(message, this.publisher.publisher_index, Some(cx))
        {
            Ok(()) => Poll::Ready(()),
            Err(message) => {
                this.message = Some(message);
                Poll::Pending
            }
        }
    }
}

impl<'s, 'a, T: Clone> Unpin for PublisherWaitFuture<'s, 'a, T> {}

/// The result of the subscriber wait procedure
#[derive(Debug, Clone, PartialEq)]
pub enum WaitResult<T> {
    /// The subscriber did not receive all messages and lagged by the given amount of messages.
    /// (This is the amount of messages that were missed)
    Lagged(u64),
    /// A message was received
    Message(T),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocking_mutex::raw::NoopRawMutex;

    #[futures_test::test]
    async fn all_subscribers_receive() {
        let channel = PubSubChannel::<NoopRawMutex, u32, 4, 4, 4>::new();

        let mut sub0 = channel.subscriber().unwrap();
        let mut sub1 = channel.subscriber().unwrap();
        let pub0 = channel.publisher().unwrap();

        pub0.publish(42).await;

        assert_eq!(sub0.next_message().await, WaitResult::Message(42));
        assert_eq!(sub1.next_message().await, WaitResult::Message(42));

        assert_eq!(sub0.try_next_message(), None);
        assert_eq!(sub1.try_next_message(), None);
    }

    #[futures_test::test]
    async fn lag_when_queue_full_on_immediate_publish() {
        let channel = PubSubChannel::<NoopRawMutex, u32, 4, 4, 4>::new();

        let mut sub0 = channel.subscriber().unwrap();
        let pub0 = channel.publisher().unwrap();

        pub0.publish_immediate(42);
        pub0.publish_immediate(43);
        pub0.publish_immediate(44);
        pub0.publish_immediate(45);
        pub0.publish_immediate(46);
        pub0.publish_immediate(47);

        assert_eq!(sub0.try_next_message(), Some(WaitResult::Lagged(2)));
        assert_eq!(sub0.next_message().await, WaitResult::Message(44));
        assert_eq!(sub0.next_message().await, WaitResult::Message(45));
        assert_eq!(sub0.next_message().await, WaitResult::Message(46));
        assert_eq!(sub0.next_message().await, WaitResult::Message(47));
        assert_eq!(sub0.try_next_message(), None);
    }

    #[test]
    fn limited_subs_and_pubs() {
        let channel = PubSubChannel::<NoopRawMutex, u32, 4, 4, 4>::new();

        let sub0 = channel.subscriber();
        let sub1 = channel.subscriber();
        let sub2 = channel.subscriber();
        let sub3 = channel.subscriber();
        let sub4 = channel.subscriber();

        assert!(sub0.is_ok());
        assert!(sub1.is_ok());
        assert!(sub2.is_ok());
        assert!(sub3.is_ok());
        assert_eq!(sub4.err().unwrap(), Error::MaximumSubscribersReached);

        drop(sub0);

        let sub5 = channel.subscriber();
        assert!(sub5.is_ok());

        // publishers

        let pub0 = channel.publisher();
        let pub1 = channel.publisher();
        let pub2 = channel.publisher();
        let pub3 = channel.publisher();
        let pub4 = channel.publisher();

        assert!(pub0.is_ok());
        assert!(pub1.is_ok());
        assert!(pub2.is_ok());
        assert!(pub3.is_ok());
        assert_eq!(pub4.err().unwrap(), Error::MaximumPublishersReached);

        drop(pub0);

        let pub5 = channel.publisher();
        assert!(pub5.is_ok());
    }

    #[test]
    fn publisher_wait_on_full_queue() {
        let channel = PubSubChannel::<NoopRawMutex, u32, 4, 4, 4>::new();

        let pub0 = channel.publisher().unwrap();

        // There are no subscribers, so the queue will never be full
        assert_eq!(pub0.try_publish(0), Ok(()));
        assert_eq!(pub0.try_publish(0), Ok(()));
        assert_eq!(pub0.try_publish(0), Ok(()));
        assert_eq!(pub0.try_publish(0), Ok(()));
        assert_eq!(pub0.try_publish(0), Ok(()));

        let sub0 = channel.subscriber().unwrap();

        assert_eq!(pub0.try_publish(0), Ok(()));
        assert_eq!(pub0.try_publish(0), Ok(()));
        assert_eq!(pub0.try_publish(0), Ok(()));
        assert_eq!(pub0.try_publish(0), Ok(()));
        assert_eq!(pub0.try_publish(0), Err(0));

        drop(sub0);
    }
}
