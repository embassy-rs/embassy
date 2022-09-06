//! Generic timer queue implementation
use core::cell::RefCell;
use core::cmp::Ordering;
use core::task::Waker;

use atomic_polyfill::{AtomicBool, Ordering as AtomicOrdering};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, RawMutex};
use embassy_sync::blocking_mutex::Mutex;
use heapless::sorted_linked_list::{LinkedIndexU8, Min, SortedLinkedList};

use crate::driver::{allocate_alarm, set_alarm, set_alarm_callback, AlarmHandle};
use crate::Instant;

#[derive(Debug)]
struct Timer {
    at: Instant,
    waker: Waker,
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.at == other.at
    }
}

impl Eq for Timer {}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.at.partial_cmp(&other.at)
    }
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.at.cmp(&other.at)
    }
}

struct InnerQueue<const N: usize> {
    queue: SortedLinkedList<Timer, LinkedIndexU8, Min, N>,
    alarm_at: Instant,
    alarm: Option<AlarmHandle>,
}

impl<const N: usize> InnerQueue<N> {
    const fn new() -> Self {
        Self {
            queue: SortedLinkedList::new_u8(),
            alarm_at: Instant::MAX,
            alarm: None,
        }
    }

    fn schedule(&mut self, at: Instant, waker: &Waker) {
        self.queue
            .find_mut(|timer| timer.waker.will_wake(waker))
            .map(|mut timer| {
                timer.waker = waker.clone();
                timer.at = at;

                timer.finish();
            })
            .unwrap_or_else(|| {
                let mut timer = Timer {
                    waker: waker.clone(),
                    at,
                };

                loop {
                    match self.queue.push(timer) {
                        Ok(()) => break,
                        Err(e) => timer = e,
                    }

                    self.queue.pop().unwrap().waker.wake();
                }
            });

        // Don't wait for the alarm callback to trigger and directly
        // dispatch all timers that are already due
        //
        // Then update the alarm if necessary
        self.dispatch();
    }

    fn dispatch(&mut self) {
        let now = Instant::now();

        while self.queue.peek().filter(|timer| timer.at <= now).is_some() {
            self.queue.pop().unwrap().waker.wake();
        }

        self.update_alarm();
    }

    fn update_alarm(&mut self) {
        if let Some(timer) = self.queue.peek() {
            let new_at = timer.at;

            if self.alarm_at != new_at {
                self.alarm_at = new_at;
                set_alarm(self.alarm.unwrap(), new_at.as_ticks());
            }
        } else {
            self.alarm_at = Instant::MAX;
        }
    }

    fn handle_alarm(&mut self) {
        self.alarm_at = Instant::MAX;

        self.dispatch();
    }
}

/// TODO: Doc
pub struct Queue<const N: usize = 128, R: RawMutex = CriticalSectionRawMutex> {
    initialized: AtomicBool,
    inner: Mutex<R, RefCell<InnerQueue<N>>>,
}

impl<const N: usize, R: RawMutex + 'static> Queue<N, R> {
    /// TODO: Doc
    pub const fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
            inner: Mutex::new(RefCell::new(InnerQueue::<N>::new())),
        }
    }

    /// TODO: Doc
    pub unsafe fn initialize(&'static self) {
        if self.initialized.load(AtomicOrdering::SeqCst) {
            panic!("Queue already initialized");
        }

        let handle = allocate_alarm().unwrap();
        self.inner.lock(|inner| inner.borrow_mut().alarm = Some(handle));

        set_alarm_callback(handle, Self::handle_alarm, self as *const _ as _);

        self.initialized.store(true, AtomicOrdering::SeqCst);
    }

    /// TODO: Doc
    pub fn schedule(&'static self, at: Instant, waker: &Waker) {
        self.check_initialized();

        self.inner.lock(|inner| inner.borrow_mut().schedule(at, waker));
    }

    fn check_initialized(&self) {
        if !self.initialized.load(AtomicOrdering::SeqCst) {
            panic!("Queue is not initialized");
        }
    }

    fn handle_alarm(ctx: *mut ()) {
        let this = unsafe { (ctx as *const Self).as_ref().unwrap() };

        this.check_initialized();
        this.inner.lock(|inner| inner.borrow_mut().handle_alarm());
    }
}

/// TODO: Doc
pub unsafe fn initialize() {
    extern "Rust" {
        fn _embassy_time_generic_queue_initialize();
    }

    _embassy_time_generic_queue_initialize();
}

/// TODO: Doc
#[macro_export]
macro_rules! generic_queue {
    (static $name:ident: $t: ty = $val:expr) => {
        static $name: $t = $val;

        #[no_mangle]
        fn _embassy_time_generic_queue_initialize() {
            unsafe {
                $crate::queue::Queue::initialize(&$name);
            }
        }

        #[no_mangle]
        fn _embassy_time_schedule_wake(at: $crate::Instant, waker: &core::task::Waker) {
            $crate::queue::Queue::schedule(&$name, at, waker);
        }
    };
}

#[cfg(feature = "generic-queue")]
generic_queue!(static QUEUE: Queue = Queue::new());
