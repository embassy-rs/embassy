//! Optional tracing for [`Channel`](crate::channel::Channel) and [`Mutex`](crate::mutex::Mutex).
//!
//! Enable `trace`, implement [`Trace`], and register it once with [`trace_impl!`](crate::trace_impl).
//! An ID identifies internal state; it need not equal the address of the public object.
//! It can change when the primitive moves
//! and may be reused after it is dropped. Keep traced objects in place to correlate events.
//! Callbacks run synchronously, including inside raw mutexes and interrupt handlers. They must
//! be bounded, never panic, and never reenter the primitive or wake or poll other tasks.
//! A wait describes one poll of one operation. It can repeat, be cancelled, or be one branch
//! of a larger future that completes in the same poll. Wake notifications can occur before
//! the corresponding operation callback.

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

// Bump the symbol version for changes to signatures or event discriminants, including additions.
unitrait::unitrait! {
    /// Synchronization trace hooks. Implement all callbacks.
    #[symbol_prefix = "_embassy_channel_mutex_trace_v1"]
    pub trait Trace {
        /// A future registered its waker and will report pending.
        fn wait(id: usize, operation: Wait, waker: &Waker);
        /// Channel occupancy after the operation, in items.
        fn channel(id: usize, event: ChannelEvent, len: usize, capacity: usize);
        /// A mutex was acquired, released, or found busy by an immediate operation.
        fn mutex(id: usize, event: MutexEvent);
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

    #[derive(Debug, PartialEq)]
    enum Event {
        Wait(Wait),
        Channel(ChannelEvent, usize, usize),
        Mutex(MutexEvent),
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
}
