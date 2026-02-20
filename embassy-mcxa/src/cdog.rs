//! Code Watchdog (CDOG) driver for MCXA microcontrollers.
//!
//! The CDOG is a hardware watchdog that monitors code execution flow by tracking
//! a secure counter value and execute a timer. It can detect various fault conditions including:
//! - Timeout: Code execution takes too long
//! - Miscompare: Secure counter value doesn't match expected value
//! - Sequence: Invalid operation sequence
//! - State: Invalid state transitions
//! - Address: Invalid memory access

use core::marker::PhantomData;
use core::ops::{AddAssign, SubAssign};

use embassy_hal_internal::{Peri, PeripheralType};
use maitake_sync::WaitCell;
use paste::paste;

use crate::interrupt::typelevel::Interrupt;
use crate::pac::cdog::vals::{Ctrl, DebugHaltCtrl, IrqPause, LockCtrl};
use crate::{interrupt, pac};

/// Errors that can occur when configuring or using the CDOG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Watchdog is currently running and cannot be reconfigured.
    Running,

    /// Watchdog is currently *not* running. This is an error
    /// condition when attempting to issue a RESTART command.
    NotRunning,
}

/// Fault control configuration for different fault types.
///
/// Determines what action the CDOG takes when a fault is detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum FaultControl {
    /// Enable system reset on fault detection
    EnableReset = 1,
    /// Enable interrupt on fault detection
    EnableInterrupt = 2,
    #[default]
    /// Disable both reset and interrupt
    DisableBoth = 4,
}

impl From<FaultControl> for Ctrl {
    fn from(val: FaultControl) -> Self {
        match val {
            FaultControl::EnableReset => Ctrl::ENABLE_RESET,
            FaultControl::EnableInterrupt => Ctrl::ENABLE_INTERRUPT,
            FaultControl::DisableBoth => Ctrl::DISABLE_BOTH,
        }
    }
}

/// Timer pause control during special conditions.
///
/// Controls whether the instruction timer continues running or pauses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum PauseControl {
    #[default]
    /// Keep timer running during IRQ or debug halt
    RunTimer = 1,
    /// Pause timer during IRQ or debug halt
    PauseTimer = 2,
}

/// Lock control for CDOG configuration.
///
/// When locked, configuration registers cannot be modified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum LockControl {
    /// Lock configuration
    Locked = 1,
    #[default]
    /// Unlock configuration
    Unlocked = 2,
}

/// CDOG (Code Watchdog) configuration structure.
///
/// Defines the behavior of the watchdog for various fault conditions
/// and operational modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Config {
    /// The timeout period after which the watchdog will trigger
    pub timeout: FaultControl,
    pub miscompare: FaultControl,
    pub sequence: FaultControl,
    pub state: FaultControl,
    pub address: FaultControl,
    pub irq_pause: PauseControl,
    pub debug_halt: PauseControl,
    pub lock: LockControl,
}

/// Code Watchdog peripheral
pub struct Cdog<'d> {
    info: &'static Info,
    _phantom: PhantomData<&'d mut ()>,
}

impl<'d> Cdog<'d> {
    /// Creates a new CDOG instance with the given configuration.
    ///
    /// # Arguments
    /// * `_peri` - Peripheral ownership token
    /// * `_irq` - Interrupt binding for CDOG0 interrupt handler
    /// * `config` - Configuration for fault handling and operational modes
    ///
    /// # Returns
    /// * `Ok(Watchdog)` - Successfully configured watchdog
    /// * `Err(Error::Running)` - Watchdog is already running and cannot be reconfigured
    pub fn new<T: Instance>(
        _peri: Peri<'d, T>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        let mut inst = Self {
            info: T::info(),
            _phantom: PhantomData,
        };

        inst.set_configuration(&config)?;

        T::Interrupt::unpend();

        // Safety: `_irq` ensures an Interrupt Handler exists.
        unsafe { T::Interrupt::enable() };

        Ok(inst)
    }

    fn set_configuration(&mut self, config: &Config) -> Result<(), Error> {
        // Ensure that CDOG is in IDLE mode otherwise writing to CONTROL register will trigger a fault.
        if self.info.regs().status().read().curst() == 0xA {
            return Err(Error::Running);
        }

        // Clear all pending error flags to prevent immediate reset after enable.
        // The clearing method depends on whether the module is locked:
        // - Unlocked (LOCK_CTRL = 10b): Write flag values directly
        // - Locked (LOCK_CTRL = 01b): Write '1' to clear individual flags
        let b = self.info.regs().control().read().lock_ctrl() == LockCtrl::LOCKED;
        // Locked mode: write '1' to clear each flag
        self.info.regs().flags().write(|w| {
            w.set_to_flag(b);
            w.set_miscom_flag(b);
            w.set_seq_flag(b);
            w.set_cnt_flag(b);
            w.set_state_flag(b);
            w.set_addr_flag(b);
            w.set_por_flag(b);
        });

        // Configure CONTROL register with the provided config
        self.info.regs().control().write(|w| {
            w.set_timeout_ctrl(config.timeout.into());
            w.set_miscompare_ctrl(config.miscompare.into());
            w.set_sequence_ctrl(config.sequence.into());
            w.set_state_ctrl(config.state.into());
            w.set_address_ctrl(config.address.into());

            // IRQ pause control
            match config.irq_pause {
                PauseControl::RunTimer => w.set_irq_pause(IrqPause::RUN_TIMER),
                PauseControl::PauseTimer => w.set_irq_pause(IrqPause::PAUSE_TIMER),
            };

            // Debug halt control
            match config.debug_halt {
                PauseControl::RunTimer => w.set_debug_halt_ctrl(DebugHaltCtrl::RUN_TIMER),
                PauseControl::PauseTimer => w.set_debug_halt_ctrl(DebugHaltCtrl::PAUSE_TIMER),
            };

            // Lock control
            match config.lock {
                LockControl::Locked => w.set_lock_ctrl(LockCtrl::LOCKED),
                LockControl::Unlocked => w.set_lock_ctrl(LockCtrl::UNLOCKED),
            }
        });

        Ok(())
    }

    /// Starts the watchdog with specified timer and counter values.
    ///
    /// # Arguments
    /// * `instruction_timer` - Number of clock cycles before timeout
    /// * `secure_counter` - Initial value of the secure counter
    ///
    /// # Note
    /// If the watchdog is already running, this will return a `Running` error.
    pub fn start(&mut self, instruction_timer: u32, secure_counter: u32) -> Result<(), Error> {
        if is_running(self.info) {
            return Err(Error::Running);
        }

        // Set the instruction timer reload value (timeout period)
        self.info.regs().reload().write(|w| w.set_rload(instruction_timer));

        // Start the watchdog with initial secure counter value
        self.info.regs().start().write(|w| w.set_strt(secure_counter));

        Ok(())
    }

    /// Restart the watchdog while comparing against the secure counter.
    ///
    /// If the watchdog is not yet running, this produces a `NotRunning` error.
    pub fn restart(&mut self, check: u32) -> Result<(), Error> {
        if is_idle(self.info) {
            return Err(Error::NotRunning);
        }

        self.info.regs().restart().write(|w| w.set_rstrt(check));

        Ok(())
    }

    /// Stops the watchdog timer.
    ///
    /// If the watchdog is already stopped, this will return a `NotRunning` error.
    pub fn stop(&mut self, check: u32) -> Result<(), Error> {
        if is_idle(self.info) {
            return Err(Error::NotRunning);
        }

        self.info.regs().stop().write(|w| w.set_stp(check));

        Ok(())
    }

    /// Produces a handle to operate on the internal secure counter
    /// using regular + and - operators.
    pub fn secure_counter(&self) -> SecureCounter {
        SecureCounter::new(self.info)
    }

    /// Produces a handle to operate on the instruction timer.
    pub fn instruction_timer(&self) -> InstructionTimer {
        InstructionTimer::new(self.info)
    }

    /// Produces a handle to operate on the persistent value.
    pub fn persistent_value(&self) -> PersistentValue {
        PersistentValue::new(self.info)
    }
}

/// CDOG Persistent Value
pub struct PersistentValue {
    info: &'static Info,
}

impl PersistentValue {
    fn new(info: &'static Info) -> Self {
        Self { info }
    }

    /// Read the current value stored in persistent value
    pub fn read(&self) -> u32 {
        self.info.regs().persistent().read().persis()
    }

    /// Write a new value to persistent value
    pub fn write(&mut self, value: u32) {
        self.info.regs().persistent().write(|w| w.set_persis(value));
    }
}

/// CDOG Instruction Timer
pub struct InstructionTimer {
    info: &'static Info,
}

impl InstructionTimer {
    fn new(info: &'static Info) -> Self {
        Self { info }
    }

    /// Read the current instruction counter value.
    pub fn read(&self) -> u32 {
        self.info.regs().instruction_timer().read().instim()
    }

    /// Update the instruction timer with a new counter value.
    pub fn update(&mut self, value: u32) {
        self.info.regs().reload().write(|w| w.set_rload(value));
    }
}

/// CDOG Persistent Value
pub struct SecureCounter {
    info: &'static Info,
}

impl SecureCounter {
    fn new(info: &'static Info) -> Self {
        Self { info }
    }

    /// Validate secure counter
    pub fn validate(&mut self, check: u32) -> Result<(), Error> {
        if is_idle(self.info) {
            return Err(Error::NotRunning);
        }

        self.info.regs().stop().write(|w| w.set_stp(check));
        let reload = self.info.regs().reload().read().rload();
        self.info.regs().reload().write(|w| w.set_rload(reload));
        self.info.regs().start().write(|w| w.set_strt(check));

        Ok(())
    }
}

impl AddAssign<u32> for SecureCounter {
    fn add_assign(&mut self, rhs: u32) {
        self.info.regs().add().write(|w| w.set_ad(rhs));
    }
}

impl SubAssign<u32> for SecureCounter {
    fn sub_assign(&mut self, rhs: u32) {
        self.info.regs().sub().write(|w| w.set_sb(rhs));
    }
}

fn is_running(info: &'static Info) -> bool {
    info.regs().status().read().curst() == 0x0a
}

fn is_idle(info: &'static Info) -> bool {
    info.regs().status().read().curst() == 0x05
}

/// CDOG interrupt handler.
///
/// This handler is called when any cdog interrupt fires.
/// When reset happens, the interrupt handler will never be reached.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::PERF_INT_INCR();

        // Print all flags at once using the Debug implementation
        #[cfg(feature = "defmt")]
        defmt::trace!("CDOG0 flags {}", T::info().regs().flags().read());

        // Stop the cdog
        T::info().regs().stop().write(|w| w.set_stp(0));

        // Clear all flags by writing 0
        T::info().regs().flags().write(|w| w.0 = 0);
    }
}

trait SealedInstance {
    fn info() -> &'static Info;
}

/// I2C Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this CDOG instance.
    type Interrupt: interrupt::typelevel::Interrupt;
    const PERF_INT_INCR: fn();
    const PERF_INT_WAKE_INCR: fn();
}

struct Info {
    regs: pac::cdog::Cdog,
    wait_cell: WaitCell,
}

impl Info {
    #[inline(always)]
    fn regs(&self) -> pac::cdog::Cdog {
        self.regs
    }

    #[inline(always)]
    #[allow(dead_code)]
    fn wait_cell(&self) -> &WaitCell {
        &self.wait_cell
    }
}

unsafe impl Sync for Info {}

macro_rules! impl_instance {
    ($($n:literal),*) => {
        $(
            paste!{
                impl SealedInstance for crate::peripherals::[<CDOG $n>] {
                    fn info() -> &'static Info {
                        static INFO: Info = Info {
                            regs: pac::[<CDOG $n>],
                            wait_cell: WaitCell::new(),
                        };
                        &INFO
                    }
                }

                impl Instance for crate::peripherals::[<CDOG $n>] {
                    type Interrupt = crate::interrupt::typelevel::[<CDOG $n>];
                    const PERF_INT_INCR: fn() = crate::perf_counters::[<incr_interrupt_cdog $n>];
                    const PERF_INT_WAKE_INCR: fn() = crate::perf_counters::[<incr_interrupt_cdog $n _wake>];
                }
            }
        )*
    };
}

impl_instance!(0, 1);

impl<'d> embassy_embedded_hal::SetConfig for Cdog<'d> {
    type Config = Config;
    type ConfigError = Error;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_configuration(config)
    }
}
