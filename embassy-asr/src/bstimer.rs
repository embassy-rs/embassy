//! Basic timers (BSTIMER0 / BSTIMER1).
//!
//! Register programming follows the vendor `tremo_bstimer` driver and the
//! `bstimer/onepulse` / DAC trigger examples.
//!
//! All register fields are accessed through the PAC. `SR.UIF` is modeled as
//! zero-to-clear, matching the vendor write-0 clear sequence.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_hal_internal::impl_peripheral;
use embassy_hal_internal::interrupt::{InterruptExt, Priority};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::{Binding, Handler, Interrupt as TypelevelInterrupt};
use crate::mode::{Async, Blocking, Mode};
use crate::rcc::{self, Peripheral};
use crate::{Peri, PeripheralType, interrupt, pac, peripherals};

const INSTANCE_COUNT: usize = 2;

static WAKERS: [AtomicWaker; INSTANCE_COUNT] = [const { AtomicWaker::new() }; INSTANCE_COUNT];
static UPDATE_PENDING: [AtomicBool; INSTANCE_COUNT] = [const { AtomicBool::new(false) }; INSTANCE_COUNT];

/// Master-mode selection for the TRGO output (`CR2.MMS`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
#[non_exhaustive]
pub enum MasterMode {
    /// `EGR.UG` is used as the trigger output.
    Reset = 0x00,
    /// `CR1.CEN` is used as the trigger output.
    Enable = 0x10,
    /// The update event is used as the trigger output.
    Update = 0x20,
}

/// BSTIMER configuration matching `bstimer_init_t`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config {
    /// Prescaler loaded into `PSC` (`0..=0xFFFF`). Counter clock is
    /// `timer_clk / (prescaler + 1)`.
    pub prescaler: u16,
    /// Auto-reload value loaded into `ARR` (`0..=0xFFFF`).
    pub period: u16,
    /// Master-mode selection for TRGO.
    pub master_mode: MasterMode,
    /// When true, `ARR` is buffered (`CR1.ARPE`).
    pub autoreload_preload: bool,
}

impl Config {
    /// Create a default configuration (free-running, update TRGO, no preload).
    pub const fn new() -> Self {
        Self {
            prescaler: 0,
            period: 0xffff,
            master_mode: MasterMode::Update,
            autoreload_preload: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

struct Info {
    regs: *const pac::bstim0::RegisterBlock,
    peripheral: Peripheral,
    number: usize,
    interrupt: pac::Interrupt,
}

impl Info {
    #[inline]
    fn regs(&self) -> &'static pac::bstim0::RegisterBlock {
        unsafe { &*self.regs }
    }
}

// SAFETY: `Info` only contains read-only register addresses.
unsafe impl Sync for Info {}

trait SealedInstance {
    fn info() -> &'static Info;
}

/// BSTIMER peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + Into<AnyBsTimer> + 'static + Send {
    /// Update interrupt for this timer.
    type Interrupt: TypelevelInterrupt;
}

/// Type-erased BSTIMER token.
pub struct AnyBsTimer {
    _private: (),
}

impl_peripheral!(AnyBsTimer);

macro_rules! impl_instance {
    ($name:ident, $pac:ident, $interrupt:ident, $peripheral:ident, $number:expr) => {
        impl SealedInstance for peripherals::$name {
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    regs: pac::$pac::PTR,
                    peripheral: Peripheral::$peripheral,
                    number: $number,
                    interrupt: pac::Interrupt::$interrupt,
                };
                &INFO
            }
        }

        impl From<peripherals::$name> for AnyBsTimer {
            fn from(_: peripherals::$name) -> Self {
                Self { _private: () }
            }
        }

        impl Instance for peripherals::$name {
            type Interrupt = interrupt::typelevel::$interrupt;
        }
    };
}

impl_instance!(BSTIMER0, Bstim0, BSTIM0, Bstimer0, 0);
impl_instance!(BSTIMER1, Bstim1, BSTIM1, Bstimer1, 1);

/// Interrupt handler for one BSTIMER instance.
///
/// Bind this to `BSTIMER0` / `BSTIMER1` with [`crate::bind_interrupts!`] before
/// constructing an asynchronous timer.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let info = T::info();
        if !info.regs().sr().read().uif().bit_is_set() {
            return;
        }

        // Clear UIF (GPTIMER uses the same write-0 clear).
        clear_update_flag(info);
        info.regs().dier().modify(|_, w| w.uie().clear_bit());
        UPDATE_PENDING[info.number].store(true, Ordering::Release);
        WAKERS[info.number].wake();
    }
}

/// Owned basic timer.
pub struct BsTimer<'d, M: Mode = Blocking> {
    _peri: Peri<'d, AnyBsTimer>,
    info: &'static Info,
    _mode: PhantomData<M>,
}

impl<'d> BsTimer<'d, Blocking> {
    /// Enable clocks, release reset, apply `config`, and leave the timer stopped.
    pub fn new_blocking<T: Instance>(peri: Peri<'d, T>, config: Config) -> Self {
        let mut this = Self::create(peri);
        this.configure(config);
        this
    }

    /// Busy-wait until the update interrupt flag is set, then clear it.
    pub fn wait_for_update(&mut self) {
        while !self.update_flag() {}
        self.clear_update_flag();
    }
}

impl<'d> BsTimer<'d, Async> {
    /// Enable clocks, release reset, apply `config`, and bind the update IRQ.
    pub fn new<T: Instance>(
        peri: Peri<'d, T>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Self {
        let info = T::info();
        UPDATE_PENDING[info.number].store(false, Ordering::Release);
        info.interrupt.unpend();
        info.interrupt.set_priority(Priority::P3);
        unsafe { info.interrupt.enable() };

        let mut this = Self::create(peri);
        this.configure(config);
        this
    }

    /// Wait for the next update event.
    ///
    /// Enables `DIER.UIE` until [`InterruptHandler`] observes `SR.UIF`.
    pub async fn wait_for_update(&mut self) {
        let info = self.info;
        UPDATE_PENDING[info.number].store(false, Ordering::Release);
        clear_update_flag(info);

        let _uie = UpdateInterruptGuard::arm(info);

        poll_fn(|cx| {
            WAKERS[info.number].register(cx.waker());

            if UPDATE_PENDING[info.number].swap(false, Ordering::AcqRel) {
                return Poll::Ready(());
            }

            // Already latched before / while UIE was armed.
            if info.regs().sr().read().uif().bit_is_set() {
                clear_update_flag(info);
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }
}

impl<'d, M: Mode> BsTimer<'d, M> {
    fn create<T: Instance>(peri: Peri<'d, T>) -> Self {
        let info = T::info();
        let _ = rcc::enable_peripheral(info.peripheral);
        let _ = rcc::reset_peripheral(info.peripheral);
        Self {
            _peri: peri.into(),
            info,
            _mode: PhantomData,
        }
    }

    /// Apply configuration while the counter is left as-is (vendor `bstimer_init`).
    pub fn configure(&mut self, config: Config) {
        let regs = self.info.regs();

        regs.cr2().modify(|_, w| match config.master_mode {
            MasterMode::Reset => w.mms().reset(),
            MasterMode::Enable => w.mms().enable(),
            MasterMode::Update => w.mms().update(),
        });

        // CR1 ARPE has field accessor.
        regs.cr1().modify(|_, w| w.arpe().bit(config.autoreload_preload));

        // ARR/PSC `write_with_zero` is unsafe in the PAC; values are
        // range-checked by their `u16` types.
        unsafe {
            regs.arr().write_with_zero(|w| w.arr().bits(config.period));
            regs.psc().write_with_zero(|w| w.psc().bits(config.prescaler));
        }

        // PSC is shadowed until an update event; without this the first
        // period runs with the old prescaler (reset value 0, i.e. a ~40 us
        // "period" instead of the configured one). The forced event also
        // latches UIF, so clear it to avoid a spurious first update.
        self.generate_update();
        self.clear_update_flag();
    }

    /// Start the counter (`CR1.CEN`).
    pub fn start(&mut self) {
        self.info.regs().cr1().modify(|_, w| w.cen().set_bit());
    }

    /// Stop the counter (`CR1.CEN`).
    pub fn stop(&mut self) {
        self.info.regs().cr1().modify(|_, w| w.cen().clear_bit());
    }

    /// Enable or disable one-pulse mode (`CR1.OPM`).
    pub fn set_one_pulse(&mut self, enabled: bool) {
        self.info.regs().cr1().modify(|_, w| w.opm().bit(enabled));
    }

    /// When enabled, only counter overflow/underflow generates update events
    /// that set UIF (`CR1.URS`, vendor `bstimer_config_overflow_update`).
    pub fn set_overflow_update(&mut self, enabled: bool) {
        self.info.regs().cr1().modify(|_, w| w.urs().bit(enabled));
    }

    /// Disable generation of update events (`CR1.UDIS`).
    pub fn set_update_disabled(&mut self, disabled: bool) {
        self.info.regs().cr1().modify(|_, w| w.udis().bit(disabled));
    }

    /// Enable or disable DMA requests on update (`DIER.UDE`).
    pub fn set_dma_request(&mut self, enabled: bool) {
        self.info.regs().dier().modify(|_, w| w.ude().bit(enabled));
    }

    /// Enable or disable the update interrupt (`DIER.UIE`).
    pub fn set_update_interrupt(&mut self, enabled: bool) {
        self.info.regs().dier().modify(|_, w| w.uie().bit(enabled));
    }

    /// Whether `SR.UIF` is set.
    pub fn update_flag(&self) -> bool {
        self.info.regs().sr().read().uif().bit_is_set()
    }

    /// Clear `SR.UIF`.
    ///
    /// The vendor BSTIMER driver never clears this flag; GPTIMER does via
    /// `SR &= ~UIF`. `SR.UIF` is a zero-to-clear field, so this matches
    /// GPTIMER's `timer_clear_status` (write-0 to the flag).
    pub fn clear_update_flag(&mut self) {
        clear_update_flag(self.info);
    }

    /// Generate an update event (`EGR.UG`), reloading the prescaler shadow.
    /// EGR has field accessor for UG.
    pub fn generate_update(&self) {
        unsafe {
            self.info.regs().egr().write_with_zero(|w| w.ug().set_bit());
        }
    }

    /// Current counter value.
    pub fn counter(&self) -> u32 {
        u32::from(self.info.regs().cnt().read().cnt().bits())
    }

    /// Write the counter register.
    pub fn set_counter(&mut self, value: u32) {
        // CNT is wider than the 16-bit value field; truncate explicitly.
        unsafe {
            self.info.regs().cnt().write_with_zero(|w| w.cnt().bits(value as u16));
        }
    }

    /// Current auto-reload value.
    pub fn period(&self) -> u32 {
        u32::from(self.info.regs().arr().read().arr().bits())
    }

    /// Current prescaler value.
    pub fn prescaler(&self) -> u32 {
        u32::from(self.info.regs().psc().read().psc().bits())
    }

    /// Configure one-pulse mode as in the vendor `bstimer/onepulse` example:
    /// OPM + overflow-only updates, then force an update so `PSC` takes effect
    /// on the first period. Does not start the counter or enable the IRQ.
    pub fn setup_one_pulse(&mut self) {
        self.set_one_pulse(true);
        self.set_overflow_update(true);
        self.generate_update();
    }
}

impl<'d, M: Mode> embassy_embedded_hal::SetConfig for BsTimer<'d, M> {
    type Config = Config;
    type ConfigError = core::convert::Infallible;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.configure(*config);
        Ok(())
    }
}

impl<'d, M: Mode> Drop for BsTimer<'d, M> {
    fn drop(&mut self) {
        self.stop();
        self.set_update_interrupt(false);
        self.clear_update_flag();
        self.info.interrupt.disable();
        UPDATE_PENDING[self.info.number].store(false, Ordering::Release);
        let _ = rcc::disable_peripheral(self.info.peripheral);
    }
}

struct UpdateInterruptGuard {
    info: &'static Info,
}

impl UpdateInterruptGuard {
    fn arm(info: &'static Info) -> Self {
        info.regs().dier().modify(|_, w| w.uie().set_bit());
        Self { info }
    }
}

impl Drop for UpdateInterruptGuard {
    fn drop(&mut self) {
        self.info.regs().dier().modify(|_, w| w.uie().clear_bit());
    }
}

#[inline]
fn clear_update_flag(info: &Info) {
    // Mirror GPTIMER `timer_clear_status`: write 0 to the flag being cleared.
    info.regs().sr().modify(|_, w| w.uif().clear_bit());
}
