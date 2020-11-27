use core::cell::Cell;
use core::cell::UnsafeCell;
use core::future::Future;
use core::marker::PhantomData;
use core::mem;
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::ptr;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, AtomicU32, Ordering};
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

//=============
// UninitCell

struct UninitCell<T>(MaybeUninit<UnsafeCell<T>>);
impl<T> UninitCell<T> {
    const fn uninit() -> Self {
        Self(MaybeUninit::uninit())
    }

    unsafe fn as_mut_ptr(&self) -> *mut T {
        (*self.0.as_ptr()).get()
    }

    unsafe fn as_mut(&self) -> &mut T {
        &mut *self.as_mut_ptr()
    }

    unsafe fn write(&self, val: T) {
        ptr::write(self.as_mut_ptr(), val)
    }

    unsafe fn drop_in_place(&self) {
        ptr::drop_in_place(self.as_mut_ptr())
    }
}

impl<T: Copy> UninitCell<T> {
    unsafe fn read(&self) -> T {
        ptr::read(self.as_mut_ptr())
    }
}

//=============
// Data structures

const STATE_RUNNING: u32 = 1 << 0;
const STATE_QUEUED: u32 = 1 << 1;

struct Header {
    state: AtomicU32,
    next: AtomicPtr<Header>,
    executor: Cell<*const Executor>,
    poll_fn: UninitCell<unsafe fn(*mut Header)>, // Valid if STATE_RUNNING
}

// repr(C) is needed to guarantee that header is located at offset 0
// This makes it safe to cast between Header and Task pointers.
#[repr(C)]
pub struct Task<F: Future + 'static> {
    header: Header,
    future: UninitCell<F>, // Valid if STATE_RUNNING
}

#[derive(Copy, Clone, Debug, defmt::Format)]
pub enum SpawnError {
    Busy,
}

//=============
// Atomic task queue using a very, very simple lock-free linked-list queue:
//
// To enqueue a task, task.next is set to the old head, and head is atomically set to task.
//
// Dequeuing is done in batches: the queue is emptied by atomically replacing head with
// null. Then the batch is iterated following the next pointers until null is reached.
//
// Note that batches will be iterated in the opposite order as they were enqueued. This should
// be OK for our use case. Hopefully it doesn't create executor fairness problems.

struct Queue {
    head: AtomicPtr<Header>,
}

impl Queue {
    const fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Enqueues an item. Returns true if the queue was empty.
    unsafe fn enqueue(&self, item: *mut Header) -> bool {
        let mut prev = self.head.load(Ordering::Acquire);
        loop {
            (*item).next.store(prev, Ordering::Relaxed);
            match self
                .head
                .compare_exchange_weak(prev, item, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => break,
                Err(next_prev) => prev = next_prev,
            }
        }

        prev.is_null()
    }

    unsafe fn dequeue_all(&self, on_task: impl Fn(*mut Header)) {
        let mut task = self.head.swap(ptr::null_mut(), Ordering::AcqRel);

        while !task.is_null() {
            // If the task re-enqueues itself, the `next` pointer will get overwritten.
            // Therefore, first read the next pointer, and only then process the task.
            let next = (*task).next.load(Ordering::Relaxed);

            on_task(task);

            task = next
        }
    }
}

//=============
// Waker

static WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(waker_clone, waker_wake, waker_wake, waker_drop);

unsafe fn waker_clone(p: *const ()) -> RawWaker {
    RawWaker::new(p, &WAKER_VTABLE)
}

unsafe fn waker_wake(p: *const ()) {
    let header = &*(p as *const Header);

    let mut current = header.state.load(Ordering::Acquire);
    loop {
        // If already scheduled, or if not started,
        if (current & STATE_QUEUED != 0) || (current & STATE_RUNNING == 0) {
            return;
        }

        // Mark it as scheduled
        let new = current | STATE_QUEUED;

        match header
            .state
            .compare_exchange_weak(current, new, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => break,
            Err(next_current) => current = next_current,
        }
    }

    // We have just marked the task as scheduled, so enqueue it.
    let executor = &*header.executor.get();
    executor.enqueue(p as *mut Header);
}

unsafe fn waker_drop(_: *const ()) {
    // nop
}

//=============
// Task

impl<F: Future + 'static> Task<F> {
    pub const fn new() -> Self {
        Self {
            header: Header {
                state: AtomicU32::new(0),
                next: AtomicPtr::new(ptr::null_mut()),
                executor: Cell::new(ptr::null()),
                poll_fn: UninitCell::uninit(),
            },
            future: UninitCell::uninit(),
        }
    }

    pub unsafe fn spawn(pool: &'static [Self], future: impl FnOnce() -> F) -> SpawnToken {
        for task in pool {
            let state = STATE_RUNNING | STATE_QUEUED;
            if task
                .header
                .state
                .compare_and_swap(0, state, Ordering::AcqRel)
                == 0
            {
                // Initialize the task
                task.header.poll_fn.write(Self::poll);
                task.future.write(future());

                return SpawnToken {
                    header: Some(NonNull::new_unchecked(&task.header as *const Header as _)),
                };
            }
        }

        return SpawnToken { header: None };
    }

    unsafe fn poll(p: *mut Header) {
        let this = &*(p as *const Task<F>);

        let future = Pin::new_unchecked(this.future.as_mut());
        let waker = Waker::from_raw(RawWaker::new(p as _, &WAKER_VTABLE));
        let mut cx = Context::from_waker(&waker);
        match future.poll(&mut cx) {
            Poll::Ready(_) => {
                this.future.drop_in_place();
                this.header
                    .state
                    .fetch_and(!STATE_RUNNING, Ordering::AcqRel);
            }
            Poll::Pending => {}
        }
    }
}

unsafe impl<F: Future + 'static> Sync for Task<F> {}

//=============
// Spawn token

#[must_use = "Calling a task function does nothing on its own. To spawn a task, pass the result to Executor::spawn()"]
pub struct SpawnToken {
    header: Option<NonNull<Header>>,
}

impl Drop for SpawnToken {
    fn drop(&mut self) {
        // TODO maybe we can deallocate the task instead.
        panic!("Please do not drop SpawnToken instances")
    }
}

//=============
// Executor

pub struct Executor {
    queue: Queue,
    signal_fn: fn(),
    not_send: PhantomData<*mut ()>,
}

impl Executor {
    pub const fn new(signal_fn: fn()) -> Self {
        Self {
            queue: Queue::new(),
            signal_fn: signal_fn,
            not_send: PhantomData,
        }
    }

    unsafe fn enqueue(&self, item: *mut Header) {
        if self.queue.enqueue(item) {
            (self.signal_fn)()
        }
    }

    /// Spawn a future on this executor.
    pub fn spawn(&'static self, token: SpawnToken) -> Result<(), SpawnError> {
        let header = token.header;
        mem::forget(token);

        match header {
            Some(header) => unsafe {
                let header = header.as_ref();
                header.executor.set(self);
                self.enqueue(header as *const _ as _);
                Ok(())
            },
            None => Err(SpawnError::Busy),
        }
    }

    /// Runs the executor until the queue is empty.
    pub fn run(&self) {
        unsafe {
            self.queue.dequeue_all(|p| {
                let header = &*p;

                let state = header.state.fetch_and(!STATE_QUEUED, Ordering::AcqRel);
                if state & STATE_RUNNING == 0 {
                    // If task is not running, ignore it. This can happen in the following scenario:
                    //   - Task gets dequeued, poll starts
                    //   - While task is being polled, it gets woken. It gets placed in the queue.
                    //   - Task poll finishes, returning done=true
                    //   - RUNNING bit is cleared, but the task is already in the queue.
                    return;
                }

                // Run the task
                header.poll_fn.read()(p as _);
            });
        }
    }
}
