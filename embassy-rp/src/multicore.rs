//! Multicore support
//!
//! This module handles setup of the 2nd cpu core on the rp2040, which we refer to as core1.
//! It provides functionality for setting up the stack, and starting core1.
//!
//! The entrypoint for core1 can be any function that never returns, including closures.
//!
//! Enable the `critical-section-impl` feature in embassy-rp when sharing data across cores using
//! the `embassy-sync` primitives and `CriticalSectionRawMutex`.
//!
//! # Usage
//!
//! ```no_run
//! # #![feature(type_alias_impl_trait)]
//! use embassy_rp::multicore::Stack;
//! use static_cell::StaticCell;
//! use embassy_executor::Executor;
//!
//! static mut CORE1_STACK: Stack<4096> = Stack::new();
//! static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
//! static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
//!
//! # // workaround weird error: `main` function not found in crate `rust_out`
//! # let _ = ();
//!
//! #[embassy_executor::task]
//! async fn core0_task() {
//!     // ...
//! }
//!
//! #[embassy_executor::task]
//! async fn core1_task() {
//!     // ...
//! }
//!
//! #[cortex_m_rt::entry]
//! fn main() -> ! {
//!     let p = embassy_rp::init(Default::default());
//!
//!     embassy_rp::multicore::spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
//!         let executor1 = EXECUTOR1.init(Executor::new());
//!         executor1.run(|spawner| spawner.spawn(core1_task()).unwrap());
//!     });
//!
//!     let executor0 = EXECUTOR0.init(Executor::new());
//!     executor0.run(|spawner| spawner.spawn(core0_task()).unwrap())
//! }
//! ```

use core::mem::ManuallyDrop;
use core::sync::atomic::{compiler_fence, AtomicBool, Ordering};

use crate::interrupt::InterruptExt;
use crate::peripherals::CORE1;
use crate::{gpio, install_stack_guard, interrupt, pac};

const PAUSE_TOKEN: u32 = 0xDEADBEEF;
const RESUME_TOKEN: u32 = !0xDEADBEEF;
static IS_CORE1_INIT: AtomicBool = AtomicBool::new(false);

#[inline(always)]
fn core1_setup(stack_bottom: *mut usize) {
    if let Err(_) = install_stack_guard(stack_bottom) {
        // currently only happens if the MPU was already set up, which
        // would indicate that the core is already in use from outside
        // embassy, somehow. trap if so since we can't deal with that.
        cortex_m::asm::udf();
    }
    unsafe {
        gpio::init();
    }
}

/// Data type for a properly aligned stack of N bytes
#[repr(C, align(32))]
pub struct Stack<const SIZE: usize> {
    /// Memory to be used for the stack
    pub mem: [u8; SIZE],
}

impl<const SIZE: usize> Stack<SIZE> {
    /// Construct a stack of length SIZE, initialized to 0
    pub const fn new() -> Stack<SIZE> {
        Stack { mem: [0_u8; SIZE] }
    }
}

#[cfg(feature = "rt")]
#[interrupt]
#[link_section = ".data.ram_func"]
unsafe fn SIO_IRQ_PROC1() {
    let sio = pac::SIO;
    // Clear IRQ
    sio.fifo().st().write(|w| w.set_wof(false));

    while sio.fifo().st().read().vld() {
        // Pause CORE1 execution and disable interrupts
        if fifo_read_wfe() == PAUSE_TOKEN {
            cortex_m::interrupt::disable();
            // Signal to CORE0 that execution is paused
            fifo_write(PAUSE_TOKEN);
            // Wait for `resume` signal from CORE0
            while fifo_read_wfe() != RESUME_TOKEN {
                cortex_m::asm::nop();
            }
            cortex_m::interrupt::enable();
            // Signal to CORE0 that execution is resumed
            fifo_write(RESUME_TOKEN);
        }
    }
}

/// Spawn a function on this core
pub fn spawn_core1<F, const SIZE: usize>(_core1: CORE1, stack: &'static mut Stack<SIZE>, entry: F)
where
    F: FnOnce() -> bad::Never + Send + 'static,
{
    // The first two ignored `u64` parameters are there to take up all of the registers,
    // which means that the rest of the arguments are taken from the stack,
    // where we're able to put them from core 0.
    extern "C" fn core1_startup<F: FnOnce() -> bad::Never>(
        _: u64,
        _: u64,
        entry: *mut ManuallyDrop<F>,
        stack_bottom: *mut usize,
    ) -> ! {
        core1_setup(stack_bottom);

        let entry = unsafe { ManuallyDrop::take(&mut *entry) };

        // make sure the preceding read doesn't get reordered past the following fifo write
        compiler_fence(Ordering::SeqCst);

        // Signal that it's safe for core 0 to get rid of the original value now.
        fifo_write(1);

        IS_CORE1_INIT.store(true, Ordering::Release);
        // Enable fifo interrupt on CORE1 for `pause` functionality.
        unsafe { interrupt::SIO_IRQ_PROC1.enable() };

        entry()
    }

    // Reset the core
    let psm = pac::PSM;
    psm.frce_off().modify(|w| w.set_proc1(true));
    while !psm.frce_off().read().proc1() {
        cortex_m::asm::nop();
    }
    psm.frce_off().modify(|w| w.set_proc1(false));

    // The ARM AAPCS ABI requires 8-byte stack alignment.
    // #[align] on `struct Stack` ensures the bottom is aligned, but the top could still be
    // unaligned if the user chooses a stack size that's not multiple of 8.
    // So, we round down to the next multiple of 8.
    let stack_words = stack.mem.len() / 8 * 2;
    let mem = unsafe { core::slice::from_raw_parts_mut(stack.mem.as_mut_ptr() as *mut usize, stack_words) };

    // Set up the stack
    let mut stack_ptr = unsafe { mem.as_mut_ptr().add(mem.len()) };

    // We don't want to drop this, since it's getting moved to the other core.
    let mut entry = ManuallyDrop::new(entry);

    // Push the arguments to `core1_startup` onto the stack.
    unsafe {
        // Push `stack_bottom`.
        stack_ptr = stack_ptr.sub(1);
        stack_ptr.cast::<*mut usize>().write(mem.as_mut_ptr());

        // Push `entry`.
        stack_ptr = stack_ptr.sub(1);
        stack_ptr.cast::<*mut ManuallyDrop<F>>().write(&mut entry);
    }

    // Make sure the compiler does not reorder the stack writes after to after the
    // below FIFO writes, which would result in them not being seen by the second
    // core.
    //
    // From the compiler perspective, this doesn't guarantee that the second core
    // actually sees those writes. However, we know that the RP2040 doesn't have
    // memory caches, and writes happen in-order.
    compiler_fence(Ordering::Release);

    let p = unsafe { cortex_m::Peripherals::steal() };
    let vector_table = p.SCB.vtor.read();

    // After reset, core 1 is waiting to receive commands over FIFO.
    // This is the sequence to have it jump to some code.
    let cmd_seq = [
        0,
        0,
        1,
        vector_table as usize,
        stack_ptr as usize,
        core1_startup::<F> as usize,
    ];

    let mut seq = 0;
    let mut fails = 0;
    loop {
        let cmd = cmd_seq[seq] as u32;
        if cmd == 0 {
            fifo_drain();
            cortex_m::asm::sev();
        }
        fifo_write(cmd);

        let response = fifo_read();
        if cmd == response {
            seq += 1;
        } else {
            seq = 0;
            fails += 1;
            if fails > 16 {
                // The second core isn't responding, and isn't going to take the entrypoint
                panic!("CORE1 not responding");
            }
        }
        if seq >= cmd_seq.len() {
            break;
        }
    }

    // Wait until the other core has copied `entry` before returning.
    fifo_read();
}

/// Pause execution on CORE1.
pub fn pause_core1() {
    if IS_CORE1_INIT.load(Ordering::Acquire) {
        fifo_write(PAUSE_TOKEN);
        // Wait for CORE1 to signal it has paused execution.
        while fifo_read() != PAUSE_TOKEN {}
    }
}

/// Resume CORE1 execution.
pub fn resume_core1() {
    if IS_CORE1_INIT.load(Ordering::Acquire) {
        fifo_write(RESUME_TOKEN);
        // Wait for CORE1 to signal it has resumed execution.
        while fifo_read() != RESUME_TOKEN {}
    }
}

// Push a value to the inter-core FIFO, block until space is available
#[inline(always)]
fn fifo_write(value: u32) {
    let sio = pac::SIO;
    // Wait for the FIFO to have enough space
    while !sio.fifo().st().read().rdy() {
        cortex_m::asm::nop();
    }
    sio.fifo().wr().write_value(value);
    // Fire off an event to the other core.
    // This is required as the other core may be `wfe` (waiting for event)
    cortex_m::asm::sev();
}

// Pop a value from inter-core FIFO, block until available
#[inline(always)]
fn fifo_read() -> u32 {
    let sio = pac::SIO;
    // Wait until FIFO has data
    while !sio.fifo().st().read().vld() {
        cortex_m::asm::nop();
    }
    sio.fifo().rd().read()
}

// Pop a value from inter-core FIFO, `wfe` until available
#[inline(always)]
#[allow(unused)]
fn fifo_read_wfe() -> u32 {
    let sio = pac::SIO;
    // Wait until FIFO has data
    while !sio.fifo().st().read().vld() {
        cortex_m::asm::wfe();
    }
    sio.fifo().rd().read()
}

// Drain inter-core FIFO
#[inline(always)]
fn fifo_drain() {
    let sio = pac::SIO;
    while sio.fifo().st().read().vld() {
        let _ = sio.fifo().rd().read();
    }
}

// https://github.com/nvzqz/bad-rs/blob/master/src/never.rs
mod bad {
    pub(crate) type Never = <F as HasOutput>::Output;

    pub trait HasOutput {
        type Output;
    }

    impl<O> HasOutput for fn() -> O {
        type Output = O;
    }

    type F = fn() -> !;
}
