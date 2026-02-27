//! Code Watchdog (CDOG) driver for MCXA microcontrollers.
//!
//! The CDOG is a hardware watchdog that monitors code execution flow by tracking
//! a secure counter value and execute a timer. It can detect various fault conditions including:
//! - Timeout: Code execution takes too long
//! - Miscompare: Secure counter value doesn't match expected value
//! - Sequence: Invalid operation sequence
//! - State: Invalid state transitions
//! - Address: Invalid memory access

use embassy_hal_internal::Peri;
use embassy_hal_internal::interrupt::InterruptExt;

use crate::interrupt::typelevel::{self, Handler};
use crate::pac::cdog::vals::{Ctrl, DebugHaltCtrl, IrqPause, LockCtrl};
use crate::pac::{self};
use crate::peripherals::CDOG0;

/// Shorthand for `Result<T>`.
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can occur when configuring or using the CDOG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Watchdog is currently running and cannot be reconfigured
    WatchdogRunning,
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
pub struct Watchdog<'d> {
    _peri: Peri<'d, CDOG0>,
    // The register block of the CDOG instance
    info: pac::cdog::Cdog,
    /// Software-tracked secure counter value
    secure_counter: u32,
}

impl<'d> Watchdog<'d> {
    /// Creates a new CDOG instance with the given configuration.
    ///
    /// # Arguments
    /// * `_peri` - Peripheral ownership token
    /// * `_irq` - Interrupt binding for CDOG0 interrupt handler
    /// * `config` - Configuration for fault handling and operational modes
    ///
    /// # Returns
    /// * `Ok(Watchdog)` - Successfully configured watchdog
    /// * `Err(Error::WatchdogRunning)` - Watchdog is already running and cannot be reconfigured
    pub fn new(
        _peri: Peri<'d, CDOG0>,
        _irq: impl crate::interrupt::typelevel::Binding<typelevel::CDOG0, InterruptHandler> + 'd,
        config: Config,
    ) -> Result<Self> {
        let info = pac::CDOG0;

        // Ensure that CDOG is in IDLE mode otherwise writing to CONTROL register will trigger a fault.
        if info.status().read().curst() == 0xA {
            return Err(Error::WatchdogRunning);
        }

        // Clear all pending error flags to prevent immediate reset after enable.
        // The clearing method depends on whether the module is locked:
        // - Unlocked (LOCK_CTRL = 10b): Write flag values directly
        // - Locked (LOCK_CTRL = 01b): Write '1' to clear individual flags
        let b = info.control().read().lock_ctrl() == LockCtrl::LOCKED;
        // Locked mode: write '1' to clear each flag
        info.flags().write(|w| {
            w.set_to_flag(b);
            w.set_miscom_flag(b);
            w.set_seq_flag(b);
            w.set_cnt_flag(b);
            w.set_state_flag(b);
            w.set_addr_flag(b);
            w.set_por_flag(b);
        });

        // Configure CONTROL register with the provided config
        info.control().write(|w| {
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

        crate::pac::Interrupt::CDOG0.unpend();

        // Safety: `_irq` ensures an Interrupt Handler exists.
        unsafe { crate::pac::Interrupt::CDOG0.enable() };

        Ok(Self {
            _peri,
            info,
            secure_counter: 0,
        })
    }

    /// Starts the watchdog with specified timer and counter values.
    ///
    /// # Arguments
    /// * `instruction_timer_value` - Number of clock cycles before timeout
    /// * `secure_counter_value` - Initial value of the secure counter
    ///
    /// # Note
    /// If the watchdog is already running, this will stop it first.
    pub fn start(&mut self, instruction_timer_value: u32, secure_counter_value: u32) {
        // Ensure the CDOG is in IDLE mode before starting
        // Status value 0xA indicates ACTIVE state
        while self.info.status().read().curst() == 0xA {
            self.stop();
        }

        // Update internal secure counter tracking
        self.secure_counter = secure_counter_value;

        // Set the instruction timer reload value (timeout period)
        self.info.reload().write(|w| w.set_rload(instruction_timer_value));
        // Start the watchdog with initial secure counter value
        self.info.start().write(|w| w.set_strt(secure_counter_value));
    }

    /// Adds a value to the secure counter.
    ///
    /// # Arguments
    /// * `add` - Value to add to the secure counter
    pub fn add(&mut self, add: u32) {
        self.secure_counter = self.secure_counter.wrapping_add(add);
        self.info.add().write(|w| w.set_ad(add));
    }

    // Subtracts a value from the secure counter.
    ///
    /// # Arguments
    /// * `sub` - Value to subtract from the secure counter
    pub fn sub(&mut self, sub: u32) {
        self.secure_counter = self.secure_counter.wrapping_sub(sub);
        self.info.sub().write(|w| w.set_sb(sub));
    }

    /// Checks the secure counter value and restarts the watchdog.
    ///
    /// This stops the watchdog, verifies the secure counter matches the expected
    /// value, and restarts with the same instruction timer reload value.
    ///
    /// # Arguments
    /// * `check` - Expected secure counter value
    ///
    /// # Note
    /// If the counter doesn't match, a miscompare fault may be triggered
    /// depending on configuration.
    pub fn check(&mut self, check: u32) {
        self.secure_counter = check;
        self.info.stop().write(|w| w.set_stp(self.secure_counter));
        let reload = self.info.reload().read().rload();
        self.info.reload().write(|w| w.set_rload(reload));
        self.info.start().write(|w| w.set_strt(self.secure_counter));
    }

    /// Stops the watchdog timer.
    ///
    /// # Note
    /// This is a private method. The watchdog is stopped by writing the
    /// current secure counter value to the STOP register.
    fn stop(&mut self) {
        self.info.stop().write(|w| w.set_stp(self.secure_counter));
    }

    /// Reads the current instruction timer value.
    ///
    /// # Returns
    /// Current countdown value of the instruction timer.
    pub fn get_instruction_timer(&self) -> u32 {
        self.info.instruction_timer().read().instim()
    }

    // Gets the current secure counter value.
    ///
    /// # Returns
    /// The software-tracked secure counter value.
    pub fn get_secure_counter(&self) -> u32 {
        self.secure_counter
    }

    /// Updates the instruction timer reload value.
    ///
    /// # Arguments
    /// * `instruction_timer_value` - New timeout period in clock cycles
    pub fn update_instruction_timer(&mut self, instruction_timer_value: u32) {
        self.info.reload().write(|w| w.set_rload(instruction_timer_value));
    }

    /// Sets a persistent value in the CDOG peripheral.
    ///
    /// This value is stored in the 32 bits PERSISTENT register and persist through resets other than a Power-On Reset (POR).
    ///
    /// # Arguments
    /// * `value` - The 32-bit value to store in the persistent register
    pub fn set_persistent_value(&mut self, value: u32) {
        self.info.persistent().write(|w| w.set_persis(value));
    }

    /// Gets the persistent value from the CDOG peripheral.
    ///
    /// # Returns
    /// The 32-bit value stored in the persistent register
    pub fn get_persistent_value(&self) -> u32 {
        self.info.persistent().read().persis()
    }
}

/// CDOG0 interrupt handler.
///
/// This handler is called when any cdog interrupt fires.
/// When reset happens, the interrupt handler will never be reached.
pub struct InterruptHandler;

impl Handler<typelevel::CDOG0> for InterruptHandler {
    unsafe fn on_interrupt() {
        crate::perf_counters::incr_interrupt_cdog0();
        let cdog0 = pac::CDOG0;

        // Print all flags at once using the Debug implementation
        #[cfg(feature = "defmt")]
        defmt::trace!("CDOG0 flags {}", cdog0.flags().read());

        // Stop the cdog
        cdog0.stop().write(|w| w.set_stp(0));

        // Clear all flags by writing 0
        cdog0.flags().write(|w| w.0 = 0);
    }
}
