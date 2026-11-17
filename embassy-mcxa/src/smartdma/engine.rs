//! One-shot execution with permanently retained resources.

use core::mem::ManuallyDrop;
use core::sync::atomic::{Ordering, compiler_fence};

use super::loader::LoadedProgram;
use crate::clocks::periph_helpers::SmartDmaConfig;
use crate::clocks::{ClockError, Gate, WakeGuard, enable_and_reset};
use crate::gpio::{AnyPin, GpioPin, Pull, SealedPin};
use crate::interrupt::typelevel::Interrupt;
#[cfg(feature = "_mcxa5xx")]
use crate::pac::smartdma::Smartdma as Registers;
#[cfg(feature = "_mcxa2xx")]
use crate::pac::smartdma0::Smartdma0 as Registers;
use crate::{Peri, PeripheralType, interrupt, pac};

/// Engine or pin construction failed.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum SetupError {
    /// The system clock is unavailable.
    Clock(ClockError),
    /// Another owned pin already uses this internal PIO bit.
    PinInUse,
}

/// Launch validation failed before START was written.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum StartError {
    /// The requested exported entry does not exist.
    UnknownEntry,
    /// The entry does not begin with the explicit NOP required by this launch ABI.
    MissingEntryNop,
    /// Stack/parameter storage is empty or outside ordinary SRAM.
    InvalidStorage,
    /// No output pins were supplied.
    NoPins,
}

pub(crate) struct Info {
    pub(crate) regs: Registers,
    pub(crate) disable_clock: unsafe fn(),
    pub(crate) assert_reset: unsafe fn(),
}

unsafe impl Sync for Info {}

pub(crate) trait SealedInstance: Gate<MrccPeriphConfig = SmartDmaConfig> {
    fn info() -> &'static Info;
}

/// The selected chip's single SmartDMA engine.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {}

pub(crate) trait SealedSmartDmaPin {
    const PIO: u8;
    const MUX: pac::port::Mux;
}

/// A pin with a chip-metadata-checked SmartDMA PIO alternate function.
#[allow(private_bounds)]
pub trait Pin: GpioPin + SealedSmartDmaPin {}
impl<T: GpioPin + SealedSmartDmaPin> Pin for T {}

/// An exclusively owned physical pin connected to one SmartDMA PIO bit.
///
/// Ownership is permanent in this first one-shot API. Dropping this handle does
/// not return a peripheral token or restore GPIO configuration.
pub struct SmartDmaPin {
    pin: Peri<'static, AnyPin>,
    pio: u8,
}

impl SmartDmaPin {
    /// Bit mask to use in application-authored GPO/GPD instructions.
    pub fn mask(&self) -> u32 {
        1 << self.pio
    }
}

/// Idle single-engine owner. Construction does not start instruction execution.
///
/// This initial implementation requires genuinely static tokens. It deliberately
/// cannot be constructed from a short reborrow that could be reused after a
/// forgotten running handle. Launch is one-way until a full device reset or
/// power cycle, not a CPU-only debugger restart.
pub struct SmartDma {
    info: &'static Info,
    frequency: u32,
    pin_mask: u32,
    _wake_guard: Option<WakeGuard>,
}

impl SmartDma {
    /// Enable/reset the idle engine and capture its system-clock frequency.
    pub fn new<T: Instance>(_peripheral: Peri<'static, T>) -> Result<Self, SetupError> {
        let parts = critical_section::with(|_| {
            interrupt::typelevel::SMARTDMA::disable();
            interrupt::typelevel::SMARTDMA::unpend();
            // SAFETY: the static singleton token is consumed, execution has not
            // started, and shared MRCC read-modify-writes are serialized here.
            unsafe { enable_and_reset::<T>(&SmartDmaConfig) }
        })
        .map_err(SetupError::Clock)?;
        Ok(Self {
            info: T::info(),
            frequency: parts.freq,
            pin_mask: 0,
            _wake_guard: parts.wake_guard,
        })
    }

    /// SmartDMA's actual configured system-clock frequency, in Hz.
    pub fn frequency(&self) -> u32 {
        self.frequency
    }

    /// Reserve and mux one compatible pin, enabling its input readback.
    ///
    /// Output state/direction is program-controlled; no glitch-free initial level
    /// is promised. Disconnect other devices that might drive this signal.
    pub fn make_pin<P: Pin>(&mut self, pin: Peri<'static, P>) -> Result<SmartDmaPin, SetupError> {
        let mask = 1u32 << P::PIO;
        if self.pin_mask & mask != 0 {
            return Err(SetupError::PinInUse);
        }
        critical_section::with(|_| {
            pin.set_pull(Pull::Disabled);
            pin.set_enable_input_buffer(true);
            pin.gpio()
                .pidr()
                .modify(|w| w.set_pid(pin.pin() as usize, pac::gpio::Pid::Pid0));
            pin.gpio()
                .pddr()
                .modify(|w| w.set_pdd(pin.pin() as usize, pac::gpio::Pdd::Pdd0));
            pin.set_function(P::MUX);
        });
        self.pin_mask |= mask;
        Ok(SmartDmaPin {
            pin: pin.into(),
            pio: P::PIO,
        })
    }

    /// Launch once, permanently retaining the engine, power vote and resources.
    ///
    /// Parameter word zero is overwritten with the bottom address of `stack`.
    /// The entry must begin with an explicit `nop`, then read ARM2EZH
    /// (`per_read ..., 0xe040`), clear its
    /// low two bits and load internal SP from parameter word zero before using
    /// the stack. No hidden bootstrap instructions are inserted. Other parameter
    /// words belong to the application ABI. The EZH stack grows upward.
    ///
    /// # Safety
    ///
    /// The caller must establish instruction-fetch accessibility under existing
    /// memory permissions and noncacheable placement for all supplied storage.
    /// Every reachable instruction/access must be valid: stay inside owned live
    /// memory/stack, avoid CPU/SmartDMA data races and immutable-code writes, and
    /// access only authorized peripherals and owned pins. Pointers embedded in
    /// parameters must remain valid permanently, not merely until notification.
    /// A valid encoding, IRQ, stable PC or HOLD does not prove memory safety or
    /// bus quiescence. No cancellation, stop, safe-low recovery, or reclamation
    /// is provided, including after faults or dropping/forgetting the result.
    pub unsafe fn start(
        self,
        program: LoadedProgram,
        entry: &str,
        stack: &'static mut [u32],
        parameters: &'static mut [u32],
        pins: &'static mut [SmartDmaPin],
    ) -> Result<Running, StartError> {
        let entry = program.entry_address(entry).ok_or(StartError::UnknownEntry)?;
        if program.words()[((entry - program.address()) / 4) as usize] != 0x12 {
            return Err(StartError::MissingEntryNop);
        }
        if !ordinary_storage(stack) || !ordinary_storage(parameters) {
            return Err(StartError::InvalidStorage);
        }
        if pins.is_empty() {
            return Err(StartError::NoPins);
        }
        let stack_address = stack.as_mut_ptr() as u32;
        parameters[0] = stack_address;
        let parameter_address = parameters.as_mut_ptr() as u32;
        // Establish non-reclaiming ownership BEFORE any START write. Suppressing
        // only Running::drop would still drop its fields and release WakeGuard.
        let running = Running {
            engine: ManuallyDrop::new(self),
            _program: program,
            _stack: stack.as_mut_ptr(),
            _parameters: parameters.as_mut_ptr(),
            pins,
        };
        let regs = running.engine.info.regs;
        regs.arm2ezh().write(|w| {
            w.set_gp(parameter_address >> 2);
            w.set_ie(2);
        });
        regs.pendtrap().write(|w| {
            w.set_status(0);
            w.set_pol(0);
            w.set_en(0);
        });
        // PAC fields start at bit2: supply word addresses, not byte addresses.
        regs.bootadr().write(|w| w.set_addr(entry >> 2));
        compiler_fence(Ordering::Release);
        cortex_m::asm::dsb();
        regs.ctrl().write(|w| {
            w.set_wkey(0xc0de);
            w.set_syncen(true);
            w.set_bufen(false);
            w.set_errdis(false);
            w.set_start(true);
        });
        Ok(running)
    }
}

impl Drop for SmartDma {
    fn drop(&mut self) {
        // This destructor is reachable ONLY before launch; Running retains a
        // ManuallyDrop<SmartDma>. No bus-drain guarantee is inferred from reset.
        critical_section::with(|_| {
            interrupt::typelevel::SMARTDMA::disable();
            interrupt::typelevel::SMARTDMA::unpend();
            // SAFETY: this idle engine has never started; no instruction/data
            // accesses exist, and no driver ISR can resume after the gate closes.
            unsafe {
                (self.info.assert_reset)();
                (self.info.disable_clock)();
            }
        });
    }
}

fn ordinary_storage(storage: &[u32]) -> bool {
    let address = storage.as_ptr() as usize;
    !storage.is_empty()
        && address >= 0x2000_0000
        && storage
            .len()
            .checked_mul(4)
            .and_then(|length| address.checked_add(length))
            .is_some_and(|end| end <= 0x2008_0000)
}

/// Read-only engine diagnostics. None of these fields proves completion.
#[derive(Clone, Copy, Debug)]
pub struct Status {
    /// Raw keyed control-register readback (write key reads as zero).
    pub control: u32,
    /// Hardware's pipeline-biased program counter.
    pub pc: u32,
    /// Internal stack pointer.
    pub sp: u32,
    /// Program-defined EZH2ARM mailbox value; not an acknowledgement or drain.
    pub notification: u32,
}

/// A running engine with permanently retained resources.
///
/// Dropping or forgetting this handle leaves execution and its active-only clock
/// vote intact. Static storage and peripheral tokens cannot be recovered through
/// this API. Use a full device reset/power cycle before replacing the firmware.
/// There is deliberately no in-process stop/restart/unload operation yet.
pub struct Running {
    engine: ManuallyDrop<SmartDma>,
    _program: LoadedProgram,
    _stack: *mut u32,
    _parameters: *mut u32,
    pins: &'static mut [SmartDmaPin],
}

impl Running {
    /// Snapshot read-only diagnostics while the retained clock remains enabled.
    pub fn status(&self) -> Status {
        let regs = self.engine.info.regs;
        Status {
            control: regs.ctrl().read().0,
            pc: regs.pc().read().pc(),
            sp: regs.sp().read().sp(),
            notification: regs.ezh2arm().read().gp(),
        }
    }

    /// Sample an owned physical pin's input buffer, not the GPO output latch.
    pub fn pin_is_high(&self, index: usize) -> Option<bool> {
        let pin = &self.pins.get(index)?.pin;
        Some(pin.gpio().pdir().read().pdi(pin.pin_index() as usize))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_smartdma_instance {
    ($name:ident) => {
        impl crate::smartdma::engine::SealedInstance for crate::peripherals::$name {
            fn info() -> &'static crate::smartdma::engine::Info {
                static INFO: crate::smartdma::engine::Info = crate::smartdma::engine::Info {
                    regs: crate::pac::$name,
                    disable_clock: crate::clocks::disable::<crate::peripherals::$name>,
                    assert_reset: crate::clocks::assert_reset::<crate::peripherals::$name>,
                };
                &INFO
            }
        }
        impl crate::smartdma::engine::Instance for crate::peripherals::$name {}
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_smartdma_pin {
    ($pin:ident, $pio:literal, $mux:ident) => {
        impl crate::smartdma::engine::SealedSmartDmaPin for crate::peripherals::$pin {
            const PIO: u8 = $pio;
            const MUX: crate::pac::port::Mux = crate::pac::port::Mux::$mux;
        }
    };
}
