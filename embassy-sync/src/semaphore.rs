//! A synchronization primitive for controlling access to a pool of resources.
use core::cell::{Cell, RefCell};
use core::convert::Infallible;
use core::future::{poll_fn, Future};
use core::task::{Poll, Waker};

use heapless::Deque;

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;
use crate::waitqueue::WakerRegistration;

/// An asynchronous semaphore.
///
/// A semaphore tracks a number of permits, typically representing a pool of shared resources.
/// Users can acquire permits to synchronize access to those resources. The semaphore does not
/// contain the resources themselves, only the count of available permits.
pub trait Semaphore: Sized {
    /// The error returned when the semaphore is unable to acquire the requested permits.
    type Error;

    /// Asynchronously acquire one or more permits from the semaphore.
    async fn acquire(&self, permits: usize) -> Result<SemaphoreReleaser<'_, Self>, Self::Error>;

    /// Try to immediately acquire one or more permits from the semaphore.
    fn try_acquire(&self, permits: usize) -> Option<SemaphoreReleaser<'_, Self>>;

    /// Asynchronously acquire all permits controlled by the semaphore.
    ///
    /// This method will wait until at least `min` permits are available, then acquire all available permits
    /// from the semaphore. Note that other tasks may have already acquired some permits which could be released
    /// back to the semaphore at any time. The number of permits actually acquired may be determined by calling
    /// [`SemaphoreReleaser::permits`].
    async fn acquire_all(&self, min: usize) -> Result<SemaphoreReleaser<'_, Self>, Self::Error>;

    /// Try to immediately acquire all available permits from the semaphore, if at least `min` permits are available.
    fn try_acquire_all(&self, min: usize) -> Option<SemaphoreReleaser<'_, Self>>;

    /// Release `permits` back to the semaphore, making them available to be acquired.
    fn release(&self, permits: usize);

    /// Reset the number of available permints in the semaphore to `permits`.
    fn set(&self, permits: usize);
}

/// A representation of a number of acquired permits.
///
/// The acquired permits will be released back to the [`Semaphore`] when this is dropped.
pub struct SemaphoreReleaser<'a, S: Semaphore> {
    semaphore: &'a S,
    permits: usize,
}

impl<'a, S: Semaphore> Drop for SemaphoreReleaser<'a, S> {
    fn drop(&mut self) {
        self.semaphore.release(self.permits);
    }
}

impl<'a, S: Semaphore> SemaphoreReleaser<'a, S> {
    /// The number of acquired permits.
    pub fn permits(&self) -> usize {
        self.permits
    }

    /// Prevent the acquired permits from being released on drop.
    ///
    /// Returns the number of acquired permits.
    pub fn disarm(self) -> usize {
        let permits = self.permits;
        core::mem::forget(self);
        permits
    }
}

/// A greedy [`Semaphore`] implementation.
///
/// Tasks can acquire permits as soon as they become available, even if another task
/// is waiting on a larger number of permits.
pub struct GreedySemaphore<M: RawMutex> {
    state: Mutex<M, Cell<SemaphoreState>>,
}

impl<M: RawMutex> Default for GreedySemaphore<M> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<M: RawMutex> GreedySemaphore<M> {
    /// Create a new `Semaphore`.
    pub const fn new(permits: usize) -> Self {
        Self {
            state: Mutex::new(Cell::new(SemaphoreState {
                permits,
                waker: WakerRegistration::new(),
            })),
        }
    }

    #[cfg(test)]
    fn permits(&self) -> usize {
        self.state.lock(|cell| {
            let state = cell.replace(SemaphoreState::EMPTY);
            let permits = state.permits;
            cell.replace(state);
            permits
        })
    }

    fn poll_acquire(
        &self,
        permits: usize,
        acquire_all: bool,
        waker: Option<&Waker>,
    ) -> Poll<Result<SemaphoreReleaser<'_, Self>, Infallible>> {
        self.state.lock(|cell| {
            let mut state = cell.replace(SemaphoreState::EMPTY);
            if let Some(permits) = state.take(permits, acquire_all) {
                cell.set(state);
                Poll::Ready(Ok(SemaphoreReleaser {
                    semaphore: self,
                    permits,
                }))
            } else {
                if let Some(waker) = waker {
                    state.register(waker);
                }
                cell.set(state);
                Poll::Pending
            }
        })
    }
}

impl<M: RawMutex> Semaphore for GreedySemaphore<M> {
    type Error = Infallible;

    async fn acquire(&self, permits: usize) -> Result<SemaphoreReleaser<'_, Self>, Self::Error> {
        poll_fn(|cx| self.poll_acquire(permits, false, Some(cx.waker()))).await
    }

    fn try_acquire(&self, permits: usize) -> Option<SemaphoreReleaser<'_, Self>> {
        match self.poll_acquire(permits, false, None) {
            Poll::Ready(Ok(n)) => Some(n),
            _ => None,
        }
    }

    async fn acquire_all(&self, min: usize) -> Result<SemaphoreReleaser<'_, Self>, Self::Error> {
        poll_fn(|cx| self.poll_acquire(min, true, Some(cx.waker()))).await
    }

    fn try_acquire_all(&self, min: usize) -> Option<SemaphoreReleaser<'_, Self>> {
        match self.poll_acquire(min, true, None) {
            Poll::Ready(Ok(n)) => Some(n),
            _ => None,
        }
    }

    fn release(&self, permits: usize) {
        if permits > 0 {
            self.state.lock(|cell| {
                let mut state = cell.replace(SemaphoreState::EMPTY);
                state.permits += permits;
                state.wake();
                cell.set(state);
            });
        }
    }

    fn set(&self, permits: usize) {
        self.state.lock(|cell| {
            let mut state = cell.replace(SemaphoreState::EMPTY);
            if permits > state.permits {
                state.wake();
            }
            state.permits = permits;
            cell.set(state);
        });
    }
}

struct SemaphoreState {
    permits: usize,
    waker: WakerRegistration,
}

impl SemaphoreState {
    const EMPTY: SemaphoreState = SemaphoreState {
        permits: 0,
        waker: WakerRegistration::new(),
    };

    fn register(&mut self, w: &Waker) {
        self.waker.register(w);
    }

    fn take(&mut self, mut permits: usize, acquire_all: bool) -> Option<usize> {
        if self.permits < permits {
            None
        } else {
            if acquire_all {
                permits = self.permits;
            }
            self.permits -= permits;
            Some(permits)
        }
    }

    fn wake(&mut self) {
        self.waker.wake();
    }
}

/// A fair [`Semaphore`] implementation.
///
/// Tasks are allowed to acquire permits in FIFO order. A task waiting to acquire
/// a large number of permits will prevent other tasks from acquiring any permits
/// until its request is satisfied.
///
/// Up to `N` tasks may attempt to acquire permits concurrently. If additional
/// tasks attempt to acquire a permit, a [`WaitQueueFull`] error will be returned.
pub struct FairSemaphore<M, const N: usize>
where
    M: RawMutex,
{
    state: Mutex<M, RefCell<FairSemaphoreState<N>>>,
}

impl<M, const N: usize> Default for FairSemaphore<M, N>
where
    M: RawMutex,
{
    fn default() -> Self {
        Self::new(0)
    }
}

impl<M, const N: usize> FairSemaphore<M, N>
where
    M: RawMutex,
{
    /// Create a new `FairSemaphore`.
    pub const fn new(permits: usize) -> Self {
        Self {
            state: Mutex::new(RefCell::new(FairSemaphoreState::new(permits))),
        }
    }

    #[cfg(test)]
    fn permits(&self) -> usize {
        self.state.lock(|cell| cell.borrow().permits)
    }

    fn poll_acquire(
        &self,
        permits: usize,
        acquire_all: bool,
        cx: Option<(&mut Option<usize>, &Waker)>,
    ) -> Poll<Result<SemaphoreReleaser<'_, Self>, WaitQueueFull>> {
        let ticket = cx.as_ref().map(|(x, _)| **x).unwrap_or(None);
        self.state.lock(|cell| {
            let mut state = cell.borrow_mut();
            if let Some(permits) = state.take(ticket, permits, acquire_all) {
                Poll::Ready(Ok(SemaphoreReleaser {
                    semaphore: self,
                    permits,
                }))
            } else if let Some((ticket_ref, waker)) = cx {
                match state.register(ticket, waker) {
                    Ok(ticket) => {
                        *ticket_ref = Some(ticket);
                        Poll::Pending
                    }
                    Err(err) => Poll::Ready(Err(err)),
                }
            } else {
                Poll::Pending
            }
        })
    }
}

/// An error indicating the [`FairSemaphore`]'s wait queue is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WaitQueueFull;

impl<M: RawMutex, const N: usize> Semaphore for FairSemaphore<M, N> {
    type Error = WaitQueueFull;

    fn acquire(&self, permits: usize) -> impl Future<Output = Result<SemaphoreReleaser<'_, Self>, Self::Error>> {
        FairAcquire {
            sema: self,
            permits,
            ticket: None,
        }
    }

    fn try_acquire(&self, permits: usize) -> Option<SemaphoreReleaser<'_, Self>> {
        match self.poll_acquire(permits, false, None) {
            Poll::Ready(Ok(x)) => Some(x),
            _ => None,
        }
    }

    fn acquire_all(&self, min: usize) -> impl Future<Output = Result<SemaphoreReleaser<'_, Self>, Self::Error>> {
        FairAcquireAll {
            sema: self,
            min,
            ticket: None,
        }
    }

    fn try_acquire_all(&self, min: usize) -> Option<SemaphoreReleaser<'_, Self>> {
        match self.poll_acquire(min, true, None) {
            Poll::Ready(Ok(x)) => Some(x),
            _ => None,
        }
    }

    fn release(&self, permits: usize) {
        if permits > 0 {
            self.state.lock(|cell| {
                let mut state = cell.borrow_mut();
                state.permits += permits;
                state.wake();
            });
        }
    }

    fn set(&self, permits: usize) {
        self.state.lock(|cell| {
            let mut state = cell.borrow_mut();
            if permits > state.permits {
                state.wake();
            }
            state.permits = permits;
        });
    }
}

struct FairAcquire<'a, M: RawMutex, const N: usize> {
    sema: &'a FairSemaphore<M, N>,
    permits: usize,
    ticket: Option<usize>,
}

impl<'a, M: RawMutex, const N: usize> Drop for FairAcquire<'a, M, N> {
    fn drop(&mut self) {
        self.sema
            .state
            .lock(|cell| cell.borrow_mut().cancel(self.ticket.take()));
    }
}

impl<'a, M: RawMutex, const N: usize> core::future::Future for FairAcquire<'a, M, N> {
    type Output = Result<SemaphoreReleaser<'a, FairSemaphore<M, N>>, WaitQueueFull>;

    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        self.sema
            .poll_acquire(self.permits, false, Some((&mut self.ticket, cx.waker())))
    }
}

struct FairAcquireAll<'a, M: RawMutex, const N: usize> {
    sema: &'a FairSemaphore<M, N>,
    min: usize,
    ticket: Option<usize>,
}

impl<'a, M: RawMutex, const N: usize> Drop for FairAcquireAll<'a, M, N> {
    fn drop(&mut self) {
        self.sema
            .state
            .lock(|cell| cell.borrow_mut().cancel(self.ticket.take()));
    }
}

impl<'a, M: RawMutex, const N: usize> core::future::Future for FairAcquireAll<'a, M, N> {
    type Output = Result<SemaphoreReleaser<'a, FairSemaphore<M, N>>, WaitQueueFull>;

    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        self.sema
            .poll_acquire(self.min, true, Some((&mut self.ticket, cx.waker())))
    }
}

struct FairSemaphoreState<const N: usize> {
    permits: usize,
    next_ticket: usize,
    wakers: Deque<Option<Waker>, N>,
}

impl<const N: usize> FairSemaphoreState<N> {
    /// Create a new empty instance
    const fn new(permits: usize) -> Self {
        Self {
            permits,
            next_ticket: 0,
            wakers: Deque::new(),
        }
    }

    /// Register a waker. If the queue is full the function returns an error
    fn register(&mut self, ticket: Option<usize>, w: &Waker) -> Result<usize, WaitQueueFull> {
        self.pop_canceled();

        match ticket {
            None => {
                let ticket = self.next_ticket.wrapping_add(self.wakers.len());
                self.wakers.push_back(Some(w.clone())).or(Err(WaitQueueFull))?;
                Ok(ticket)
            }
            Some(ticket) => {
                self.set_waker(ticket, Some(w.clone()));
                Ok(ticket)
            }
        }
    }

    fn cancel(&mut self, ticket: Option<usize>) {
        if let Some(ticket) = ticket {
            self.set_waker(ticket, None);
        }
    }

    fn set_waker(&mut self, ticket: usize, waker: Option<Waker>) {
        let i = ticket.wrapping_sub(self.next_ticket);
        if i < self.wakers.len() {
            let (a, b) = self.wakers.as_mut_slices();
            let x = if i < a.len() { &mut a[i] } else { &mut b[i - a.len()] };
            *x = waker;
        }
    }

    fn take(&mut self, ticket: Option<usize>, mut permits: usize, acquire_all: bool) -> Option<usize> {
        self.pop_canceled();

        if permits > self.permits {
            return None;
        }

        match ticket {
            Some(n) if n != self.next_ticket => return None,
            None if !self.wakers.is_empty() => return None,
            _ => (),
        }

        if acquire_all {
            permits = self.permits;
        }
        self.permits -= permits;

        if ticket.is_some() {
            self.pop();
            if self.permits > 0 {
                self.wake();
            }
        }

        Some(permits)
    }

    fn pop_canceled(&mut self) {
        while let Some(None) = self.wakers.front() {
            self.pop();
        }
    }

    /// Panics if `self.wakers` is empty
    fn pop(&mut self) {
        self.wakers.pop_front().unwrap();
        self.next_ticket = self.next_ticket.wrapping_add(1);
    }

    fn wake(&mut self) {
        self.pop_canceled();

        if let Some(Some(waker)) = self.wakers.front() {
            waker.wake_by_ref();
        }
    }
}

#[cfg(test)]
mod tests {
    mod greedy {
        use core::pin::pin;

        use futures_util::poll;

        use super::super::*;
        use crate::blocking_mutex::raw::NoopRawMutex;

        #[test]
        fn try_acquire() {
            let semaphore = GreedySemaphore::<NoopRawMutex>::new(3);

            let a = semaphore.try_acquire(1).unwrap();
            assert_eq!(a.permits(), 1);
            assert_eq!(semaphore.permits(), 2);

            core::mem::drop(a);
            assert_eq!(semaphore.permits(), 3);
        }

        #[test]
        fn disarm() {
            let semaphore = GreedySemaphore::<NoopRawMutex>::new(3);

            let a = semaphore.try_acquire(1).unwrap();
            assert_eq!(a.disarm(), 1);
            assert_eq!(semaphore.permits(), 2);
        }

        #[futures_test::test]
        async fn acquire() {
            let semaphore = GreedySemaphore::<NoopRawMutex>::new(3);

            let a = semaphore.acquire(1).await.unwrap();
            assert_eq!(a.permits(), 1);
            assert_eq!(semaphore.permits(), 2);

            core::mem::drop(a);
            assert_eq!(semaphore.permits(), 3);
        }

        #[test]
        fn try_acquire_all() {
            let semaphore = GreedySemaphore::<NoopRawMutex>::new(3);

            let a = semaphore.try_acquire_all(1).unwrap();
            assert_eq!(a.permits(), 3);
            assert_eq!(semaphore.permits(), 0);
        }

        #[futures_test::test]
        async fn acquire_all() {
            let semaphore = GreedySemaphore::<NoopRawMutex>::new(3);

            let a = semaphore.acquire_all(1).await.unwrap();
            assert_eq!(a.permits(), 3);
            assert_eq!(semaphore.permits(), 0);
        }

        #[test]
        fn release() {
            let semaphore = GreedySemaphore::<NoopRawMutex>::new(3);
            assert_eq!(semaphore.permits(), 3);
            semaphore.release(2);
            assert_eq!(semaphore.permits(), 5);
        }

        #[test]
        fn set() {
            let semaphore = GreedySemaphore::<NoopRawMutex>::new(3);
            assert_eq!(semaphore.permits(), 3);
            semaphore.set(2);
            assert_eq!(semaphore.permits(), 2);
        }

        #[test]
        fn contested() {
            let semaphore = GreedySemaphore::<NoopRawMutex>::new(3);

            let a = semaphore.try_acquire(1).unwrap();
            let b = semaphore.try_acquire(3);
            assert!(b.is_none());

            core::mem::drop(a);

            let b = semaphore.try_acquire(3);
            assert!(b.is_some());
        }

        #[futures_test::test]
        async fn greedy() {
            let semaphore = GreedySemaphore::<NoopRawMutex>::new(3);

            let a = semaphore.try_acquire(1).unwrap();

            let b_fut = semaphore.acquire(3);
            let mut b_fut = pin!(b_fut);
            let b = poll!(b_fut.as_mut());
            assert!(b.is_pending());

            // Succeed even through `b` is waiting
            let c = semaphore.try_acquire(1);
            assert!(c.is_some());

            let b = poll!(b_fut.as_mut());
            assert!(b.is_pending());

            core::mem::drop(a);

            let b = poll!(b_fut.as_mut());
            assert!(b.is_pending());

            core::mem::drop(c);

            let b = poll!(b_fut.as_mut());
            assert!(b.is_ready());
        }
    }

    mod fair {
        use core::pin::pin;
        use core::time::Duration;

        use futures_executor::ThreadPool;
        use futures_timer::Delay;
        use futures_util::poll;
        use futures_util::task::SpawnExt;
        use static_cell::StaticCell;

        use super::super::*;
        use crate::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};

        #[test]
        fn try_acquire() {
            let semaphore = FairSemaphore::<NoopRawMutex, 2>::new(3);

            let a = semaphore.try_acquire(1).unwrap();
            assert_eq!(a.permits(), 1);
            assert_eq!(semaphore.permits(), 2);

            core::mem::drop(a);
            assert_eq!(semaphore.permits(), 3);
        }

        #[test]
        fn disarm() {
            let semaphore = FairSemaphore::<NoopRawMutex, 2>::new(3);

            let a = semaphore.try_acquire(1).unwrap();
            assert_eq!(a.disarm(), 1);
            assert_eq!(semaphore.permits(), 2);
        }

        #[futures_test::test]
        async fn acquire() {
            let semaphore = FairSemaphore::<NoopRawMutex, 2>::new(3);

            let a = semaphore.acquire(1).await.unwrap();
            assert_eq!(a.permits(), 1);
            assert_eq!(semaphore.permits(), 2);

            core::mem::drop(a);
            assert_eq!(semaphore.permits(), 3);
        }

        #[test]
        fn try_acquire_all() {
            let semaphore = FairSemaphore::<NoopRawMutex, 2>::new(3);

            let a = semaphore.try_acquire_all(1).unwrap();
            assert_eq!(a.permits(), 3);
            assert_eq!(semaphore.permits(), 0);
        }

        #[futures_test::test]
        async fn acquire_all() {
            let semaphore = FairSemaphore::<NoopRawMutex, 2>::new(3);

            let a = semaphore.acquire_all(1).await.unwrap();
            assert_eq!(a.permits(), 3);
            assert_eq!(semaphore.permits(), 0);
        }

        #[test]
        fn release() {
            let semaphore = FairSemaphore::<NoopRawMutex, 2>::new(3);
            assert_eq!(semaphore.permits(), 3);
            semaphore.release(2);
            assert_eq!(semaphore.permits(), 5);
        }

        #[test]
        fn set() {
            let semaphore = FairSemaphore::<NoopRawMutex, 2>::new(3);
            assert_eq!(semaphore.permits(), 3);
            semaphore.set(2);
            assert_eq!(semaphore.permits(), 2);
        }

        #[test]
        fn contested() {
            let semaphore = FairSemaphore::<NoopRawMutex, 2>::new(3);

            let a = semaphore.try_acquire(1).unwrap();
            let b = semaphore.try_acquire(3);
            assert!(b.is_none());

            core::mem::drop(a);

            let b = semaphore.try_acquire(3);
            assert!(b.is_some());
        }

        #[futures_test::test]
        async fn fairness() {
            let semaphore = FairSemaphore::<NoopRawMutex, 2>::new(3);

            let a = semaphore.try_acquire(1);
            assert!(a.is_some());

            let b_fut = semaphore.acquire(3);
            let mut b_fut = pin!(b_fut);
            let b = poll!(b_fut.as_mut()); // Poll `b_fut` once so it is registered
            assert!(b.is_pending());

            let c = semaphore.try_acquire(1);
            assert!(c.is_none());

            let c_fut = semaphore.acquire(1);
            let mut c_fut = pin!(c_fut);
            let c = poll!(c_fut.as_mut()); // Poll `c_fut` once so it is registered
            assert!(c.is_pending()); // `c` is blocked behind `b`

            let d = semaphore.acquire(1).await;
            assert!(matches!(d, Err(WaitQueueFull)));

            core::mem::drop(a);

            let c = poll!(c_fut.as_mut());
            assert!(c.is_pending()); // `c` is still blocked behind `b`

            let b = poll!(b_fut.as_mut());
            assert!(b.is_ready());

            let c = poll!(c_fut.as_mut());
            assert!(c.is_pending()); // `c` is still blocked behind `b`

            core::mem::drop(b);

            let c = poll!(c_fut.as_mut());
            assert!(c.is_ready());
        }

        #[futures_test::test]
        async fn wakers() {
            let executor = ThreadPool::new().unwrap();

            static SEMAPHORE: StaticCell<FairSemaphore<CriticalSectionRawMutex, 2>> = StaticCell::new();
            let semaphore = &*SEMAPHORE.init(FairSemaphore::new(3));

            let a = semaphore.try_acquire(2);
            assert!(a.is_some());

            let b_task = executor
                .spawn_with_handle(async move { semaphore.acquire(2).await })
                .unwrap();
            while semaphore.state.lock(|x| x.borrow().wakers.is_empty()) {
                Delay::new(Duration::from_millis(50)).await;
            }

            let c_task = executor
                .spawn_with_handle(async move { semaphore.acquire(1).await })
                .unwrap();

            core::mem::drop(a);

            let b = b_task.await.unwrap();
            assert_eq!(b.permits(), 2);

            let c = c_task.await.unwrap();
            assert_eq!(c.permits(), 1);
        }
    }
}
