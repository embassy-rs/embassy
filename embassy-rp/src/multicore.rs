//! Multicore support
//!
//! This module handles setup of the 2nd cpu core on the rp2040, which we refer to as core1.
//! It provides functionality for setting up the stack, and starting core1.
//!
//! The entrypoint for core1 can be any function that never returns, including closures.
//!
//! Enable the `critical-section-impl` feature in embassy-rp when sharing data across cores using
//! the `embassy-sync` primitives and `CriticalSectionRawMutex`.

use core::mem::ManuallyDrop;
use core::sync::atomic::{compiler_fence, Ordering};

use crate::pac;

/// Errors for multicore operations.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Operation is invalid on this core.
    InvalidCore,
    /// Core was unresponsive to commands.
    Unresponsive,
}

/// Core ID
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CoreId {
    Core0,
    Core1,
}

#[inline(always)]
fn install_stack_guard(stack_bottom: *mut usize) {
    let core = unsafe { cortex_m::Peripherals::steal() };

    // Trap if MPU is already configured
    if core.MPU.ctrl.read() != 0 {
        cortex_m::asm::udf();
    }

    // The minimum we can protect is 32 bytes on a 32 byte boundary, so round up which will
    // just shorten the valid stack range a tad.
    let addr = (stack_bottom as u32 + 31) & !31;
    // Mask is 1 bit per 32 bytes of the 256 byte range... clear the bit for the segment we want
    let subregion_select = 0xff ^ (1 << ((addr >> 5) & 7));
    unsafe {
        core.MPU.ctrl.write(5); // enable mpu with background default map
        core.MPU.rbar.write((addr & !0xff) | 0x8);
        core.MPU.rasr.write(
            1 // enable region
               | (0x7 << 1) // size 2^(7 + 1) = 256
               | (subregion_select << 8)
               | 0x10000000, // XN = disable instruction fetch; no other bits means no permissions
        );
    }
}

#[inline(always)]
fn core1_setup(stack_bottom: *mut usize) {
    install_stack_guard(stack_bottom);
}

/// Multicore execution management.
pub struct Multicore {
    cores: (Core, Core),
}

/// Data type for a properly aligned stack of N 32-bit (usize) words
#[repr(C, align(32))]
pub struct Stack<const SIZE: usize> {
    /// Memory to be used for the stack
    pub mem: [usize; SIZE],
}

impl<const SIZE: usize> Stack<SIZE> {
    /// Construct a stack of length SIZE, initialized to 0
    pub const fn new() -> Stack<SIZE> {
        Stack { mem: [0; SIZE] }
    }
}

impl Multicore {
    /// Create a new |Multicore| instance.
    pub fn new() -> Self {
        Self {
            cores: (Core { id: CoreId::Core0 }, Core { id: CoreId::Core1 }),
        }
    }

    /// Get the available |Core| instances.
    pub fn cores(&mut self) -> &mut (Core, Core) {
        &mut self.cores
    }
}

/// A handle for controlling a logical core.
pub struct Core {
    pub id: CoreId,
}

impl Core {
    /// Spawn a function on this core.
    pub fn spawn<F>(&mut self, stack: &'static mut [usize], entry: F) -> Result<(), Error>
    where
        F: FnOnce() -> bad::Never + Send + 'static,
    {
        fn fifo_write(value: u32) {
            unsafe {
                let sio = pac::SIO;
                // Wait for the FIFO to have some space
                while !sio.fifo().st().read().rdy() {
                    cortex_m::asm::nop();
                }
                // Signal that it's safe for core 0 to get rid of the original value now.
                sio.fifo().wr().write_value(value);
            }

            // Fire off an event to the other core.
            // This is required as the other core may be `wfe` (waiting for event)
            cortex_m::asm::sev();
        }

        fn fifo_read() -> u32 {
            unsafe {
                let sio = pac::SIO;
                // Keep trying until FIFO has data
                loop {
                    if sio.fifo().st().read().vld() {
                        return sio.fifo().rd().read();
                    } else {
                        // We expect the sending core to `sev` on write.
                        cortex_m::asm::wfe();
                    }
                }
            }
        }

        fn fifo_drain() {
            unsafe {
                let sio = pac::SIO;
                while sio.fifo().st().read().vld() {
                    let _ = sio.fifo().rd().read();
                }
            }
        }

        match self.id {
            CoreId::Core1 => {
                // The first two ignored `u64` parameters are there to take up all of the registers,
                // which means that the rest of the arguments are taken from the stack,
                // where we're able to put them from core 0.
                extern "C" fn core1_startup<F: FnOnce() -> bad::Never>(
                    _: u64,
                    _: u64,
                    entry: &mut ManuallyDrop<F>,
                    stack_bottom: *mut usize,
                ) -> ! {
                    core1_setup(stack_bottom);
                    let entry = unsafe { ManuallyDrop::take(entry) };
                    // Signal that it's safe for core 0 to get rid of the original value now.
                    fifo_write(1);
                    entry()
                }

                // Reset the core
                unsafe {
                    let psm = pac::PSM;
                    psm.frce_off().modify(|w| w.set_proc1(true));
                    while !psm.frce_off().read().proc1() {
                        cortex_m::asm::nop();
                    }
                    psm.frce_off().modify(|w| w.set_proc1(false));
                }

                // Set up the stack
                let mut stack_ptr = unsafe { stack.as_mut_ptr().add(stack.len()) };

                // We don't want to drop this, since it's getting moved to the other core.
                let mut entry = ManuallyDrop::new(entry);

                // Push the arguments to `core1_startup` onto the stack.
                unsafe {
                    // Push `stack_bottom`.
                    stack_ptr = stack_ptr.sub(1);
                    stack_ptr.cast::<*mut usize>().write(stack.as_mut_ptr());

                    // Push `entry`.
                    stack_ptr = stack_ptr.sub(1);
                    stack_ptr.cast::<&mut ManuallyDrop<F>>().write(&mut entry);
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
                            // The second core isn't responding, and isn't going to take the entrypoint,
                            // so we have to drop it ourselves.
                            drop(ManuallyDrop::into_inner(entry));
                            return Err(Error::Unresponsive);
                        }
                    }
                    if seq >= cmd_seq.len() {
                        break;
                    }
                }

                // Wait until the other core has copied `entry` before returning.
                fifo_read();

                Ok(())
            }
            _ => Err(Error::InvalidCore),
        }
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
