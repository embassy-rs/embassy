//! Optional tracing for [`Channel`](crate::channel::Channel), [`Mutex`](crate::mutex::Mutex),
//! and the greedy and fair [semaphores](crate::semaphore).
//!
//! Enable `trace`, implement [`Trace`], and register it once with [`trace_impl!`](crate::trace_impl).
//! An ID identifies internal state; it need not equal the address of the public object.
//! It can change when the primitive moves
//! and may be reused after it is dropped. Keep traced objects in place to correlate events.
//! Callbacks run synchronously, including inside raw mutexes and interrupt handlers. They must
//! be bounded, never panic, and never reenter the primitive or wake or poll other tasks.
//! A wait describes one poll of one operation. It can repeat, be cancelled, or be one branch
//! of a larger future that completes in the same poll. Wake notifications can occur before
//! the corresponding operation callback. Dropping a pending acquisition can remove a fair
//! semaphore ticket; no cancellation or acquisition event is emitted.

pub use core::task::Waker;

/// An operation that could not make progress.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum Wait {
    /// A send or a send-readiness poll needs space.
    ChannelSend,
    /// A receive, peek or receive-readiness poll needs an item.
    ChannelReceive,
    /// The mutex is locked.
    MutexLock,
    /// The semaphore could not grant the requested permits.
    SemaphoreAcquire,
}

/// A bounded FIFO channel operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum ChannelEvent {
    /// An item was enqueued.
    Sent,
    /// An item was dequeued.
    Received,
    /// An immediate send failed because the channel was full.
    Full,
    /// An immediate receive failed because the channel was empty.
    Empty,
    /// The channel was cleared.
    Cleared,
}

/// An async mutex operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum MutexEvent {
    /// A guard was acquired.
    Locked,
    /// The guard released the lock.
    Unlocked,
    /// An immediate lock attempt found the mutex locked.
    Busy,
}

/// A greedy or fair semaphore operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum SemaphoreEvent {
    /// A request acquired permits.
    Acquired,
    /// Permits were returned.
    Released,
    /// The available permit count was set.
    Set,
    /// An immediate acquisition could not obtain permits.
    Unavailable,
    /// A fair semaphore could not register a waiter.
    WaitQueueFull,
}

// Bump the symbol version for changes to signatures or event discriminants, including additions.
unitrait::unitrait! {
    /// Synchronization trace hooks. Implement all callbacks.
    #[symbol_prefix = "_embassy_sync_trace_v1"]
    pub trait Trace {
        /// A future registered its waker and will report pending.
        fn wait(id: usize, operation: Wait, waker: &Waker);
        /// Channel occupancy after the operation, in items.
        fn channel(id: usize, event: ChannelEvent, len: usize, capacity: usize);
        /// A mutex was acquired, released, or found busy by an immediate operation.
        fn mutex(id: usize, event: MutexEvent);
        /// Number of permits requested or affected, and permits available after the operation.
        fn semaphore(id: usize, event: SemaphoreEvent, permits: usize, available: usize);
    }
    /// The registered trace implementation.
    pub(crate) struct TraceImpl;
    /// Register an implementation of the synchronization trace hooks.
    macro trace_impl(path = $crate::trace);
}

#[cfg(test)]
mod tests {
    extern crate std;

    use core::future::Future;
    use core::pin::pin;
    use core::task::Context;
    use std::cell::RefCell;
    use std::vec::Vec;

    use super::*;
    use crate::blocking_mutex::raw::NoopRawMutex;
    use crate::channel::Channel;
    use crate::mutex::{Mutex, MutexGuard};
    use crate::semaphore::{FairSemaphore, GreedySemaphore, Semaphore};

    #[derive(Debug, PartialEq)]
    enum Event {
        Wait(Wait),
        Channel(ChannelEvent, usize, usize),
        Mutex(MutexEvent),
        Semaphore(SemaphoreEvent, usize, usize),
    }

    std::thread_local! { static EVENTS: RefCell<Vec<(usize, Event)>> = const { RefCell::new(Vec::new()) }; }

    struct Recorder;
    trace_impl!(Recorder);

    impl Trace for Recorder {
        fn wait(id: usize, operation: Wait, _: &Waker) {
            EVENTS.with_borrow_mut(|e| e.push((id, Event::Wait(operation))));
        }
        fn channel(id: usize, event: ChannelEvent, len: usize, capacity: usize) {
            EVENTS.with_borrow_mut(|e| e.push((id, Event::Channel(event, len, capacity))));
        }
        fn mutex(id: usize, event: MutexEvent) {
            EVENTS.with_borrow_mut(|e| e.push((id, Event::Mutex(event))));
        }
        fn semaphore(id: usize, event: SemaphoreEvent, permits: usize, available: usize) {
            EVENTS.with_borrow_mut(|e| e.push((id, Event::Semaphore(event, permits, available))));
        }
    }

    fn expect_one_resource(expected: &[Event]) {
        let events = EVENTS.take();
        assert_eq!(events.len(), expected.len());
        let id = events[0].0;
        for ((actual_id, actual), expected) in events.into_iter().zip(expected) {
            assert_eq!(actual_id, id);
            assert_eq!(&actual, expected);
        }
    }

    #[test]
    fn channel_observes_native_and_dynamic_handles() {
        let channel = Channel::<NoopRawMutex, u8, 1>::new();
        let mut cx = Context::from_waker(Waker::noop());
        assert!(channel.try_receive().is_err());
        assert!(
            pin!(channel.dyn_receiver().receive())
                .as_mut()
                .poll(&mut cx)
                .is_pending()
        );
        channel.try_send(1).unwrap();
        assert!(channel.try_send(2).is_err());
        assert!(pin!(channel.send(3)).as_mut().poll(&mut cx).is_pending());
        assert!(channel.poll_ready_to_receive(&mut cx).is_ready());
        assert_eq!(channel.dyn_receiver().try_receive(), Ok(1));
        channel.dyn_sender().try_send(4).unwrap();
        channel.clear();
        expect_one_resource(&[
            Event::Channel(ChannelEvent::Empty, 0, 1),
            Event::Wait(Wait::ChannelReceive),
            Event::Channel(ChannelEvent::Sent, 1, 1),
            Event::Channel(ChannelEvent::Full, 1, 1),
            Event::Wait(Wait::ChannelSend),
            Event::Channel(ChannelEvent::Received, 0, 1),
            Event::Channel(ChannelEvent::Sent, 1, 1),
            Event::Channel(ChannelEvent::Cleared, 0, 1),
        ]);
    }

    #[test]
    fn mapped_guard_releases_the_same_mutex_once() {
        let mutex = Mutex::<NoopRawMutex, _>::new((1, 2));
        let guard = mutex.try_lock().unwrap();
        assert!(mutex.try_lock().is_err());
        assert!(
            pin!(mutex.lock())
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
        drop(MutexGuard::map(guard, |v| &mut v.0));
        expect_one_resource(&[
            Event::Mutex(MutexEvent::Locked),
            Event::Mutex(MutexEvent::Busy),
            Event::Wait(Wait::MutexLock),
            Event::Mutex(MutexEvent::Unlocked),
        ]);
    }

    #[test]
    fn semaphore_reports_actual_permits_and_wait_queue_failure() {
        let greedy = GreedySemaphore::<NoopRawMutex>::new(3);
        let permit = greedy.try_acquire_all(1).unwrap();
        assert!(greedy.try_acquire(1).is_none());
        let mut cx = Context::from_waker(Waker::noop());
        assert!(pin!(greedy.acquire(1)).as_mut().poll(&mut cx).is_pending());
        drop(permit);
        greedy.set(5);
        expect_one_resource(&[
            Event::Semaphore(SemaphoreEvent::Acquired, 3, 0),
            Event::Semaphore(SemaphoreEvent::Unavailable, 1, 0),
            Event::Wait(Wait::SemaphoreAcquire),
            Event::Semaphore(SemaphoreEvent::Released, 3, 3),
            Event::Semaphore(SemaphoreEvent::Set, 5, 5),
        ]);
        let fair = FairSemaphore::<NoopRawMutex, 1>::new(0);
        let mut pending = pin!(fair.acquire(1));
        assert!(pending.as_mut().poll(&mut cx).is_pending());
        assert!(matches!(
            pin!(fair.acquire(2)).as_mut().poll(&mut cx),
            core::task::Poll::Ready(Err(_))
        ));
        fair.release(1);
        assert!(pending.as_mut().poll(&mut cx).is_ready());
        expect_one_resource(&[
            Event::Wait(Wait::SemaphoreAcquire),
            Event::Semaphore(SemaphoreEvent::WaitQueueFull, 2, 0),
            Event::Semaphore(SemaphoreEvent::Released, 1, 1),
            Event::Semaphore(SemaphoreEvent::Acquired, 1, 0),
            Event::Semaphore(SemaphoreEvent::Released, 1, 1),
        ]);
    }
}
