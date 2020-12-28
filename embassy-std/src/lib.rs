use embassy::executor::Executor;
use embassy::time::TICKS_PER_SECOND;
use embassy::time::{Alarm, Clock};
use embassy::util::Forever;
use rand_core::{OsRng, RngCore};
use std::mem::MaybeUninit;
use std::sync::{Condvar, Mutex};
use std::time::{Duration as StdDuration, Instant as StdInstant};

static mut CLOCK_ZERO: MaybeUninit<StdInstant> = MaybeUninit::uninit();
struct StdClock;
impl Clock for StdClock {
    fn now(&self) -> u64 {
        let zero = unsafe { CLOCK_ZERO.as_ptr().read() };
        let dur = StdInstant::now().duration_since(zero);
        dur.as_secs() * (TICKS_PER_SECOND as u64)
            + (dur.subsec_nanos() as u64) * (TICKS_PER_SECOND as u64) / 1_000_000_000
    }
}

struct StdRand;
impl embassy::rand::Rand for StdRand {
    fn rand(&self, buf: &mut [u8]) {
        OsRng.fill_bytes(buf);
    }
}

static mut ALARM_AT: u64 = u64::MAX;

pub struct StdAlarm;
impl Alarm for StdAlarm {
    fn set_callback(&self, _callback: fn()) {}
    fn set(&self, timestamp: u64) {
        unsafe { ALARM_AT = timestamp }
    }

    fn clear(&self) {
        unsafe { ALARM_AT = u64::MAX }
    }
}

static EXECUTOR: Forever<Executor> = Forever::new();

lazy_static::lazy_static! {
    static ref MUTEX: Mutex<bool> = Mutex::new(false);
    static ref CONDVAR: Condvar = Condvar::new();
}

pub fn init() -> &'static Executor {
    unsafe {
        CLOCK_ZERO.as_mut_ptr().write(StdInstant::now());
        embassy::time::set_clock(&StdClock);
        embassy::rand::set_rand(&StdRand);

        EXECUTOR.put(Executor::new_with_alarm(&StdAlarm, || {
            let mut signaled = MUTEX.lock().unwrap();
            *signaled = true;
            CONDVAR.notify_one();
        }))
    }
}

pub fn run(executor: &'static Executor) -> ! {
    unsafe {
        loop {
            executor.run();

            let mut signaled = MUTEX.lock().unwrap();
            while !*signaled {
                let alarm_at = ALARM_AT;
                if alarm_at == u64::MAX {
                    signaled = CONDVAR.wait(signaled).unwrap();
                } else {
                    let now = StdClock.now();
                    if now >= alarm_at {
                        break;
                    }

                    let left = alarm_at - now;
                    let dur = StdDuration::new(
                        left / (TICKS_PER_SECOND as u64),
                        (left % (TICKS_PER_SECOND as u64) * 1_000_000_000
                            / (TICKS_PER_SECOND as u64)) as u32,
                    );
                    let (signaled2, timeout) = CONDVAR.wait_timeout(signaled, dur).unwrap();
                    signaled = signaled2;
                    if timeout.timed_out() {
                        break;
                    }
                }
            }
            *signaled = false;
        }
    }
}
