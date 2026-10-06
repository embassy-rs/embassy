//! Basic timers (BSTIMER0 / BSTIMER1).
//!
//! Register programming follows the vendor `tremo_bstimer` driver and the
//! `bstimer/onepulse` / DAC trigger examples.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_hal_internal::interrupt::Priority;
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::{Binding, Handler, Interrupt as TypelevelInterrupt};
use crate::mode::{Async, Blocking, Mode};
use crate::rcc::{self, Peripheral};
use crate::{Peri, PeripheralType, interrupt, pac, peripherals};

const INSTANCE_COUNT: usize = 2;

const CR2_MMS_MASK: u32 = 0x70;
const SR_UIF: u32 = 1 << 0;

static WAKERS: [AtomicWaker; INSTANCE_COUNT] = [const { AtomicWaker::new() }; INSTANCE_COUNT];
static UPDATE_PENDING: [AtomicBool; INSTANCE_COUNT] = [const { AtomicBool::new(false) }; INSTANCE_COUNT];

/// Master-mode selection for the TRGO output (`CR2.MMS`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
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

mod sealed {
    use crate::pac::bstimer0::RegisterBlock;
    use crate::rcc::Peripheral;

    pub trait Instance {
        fn regs() -> &'static RegisterBlock;
        fn peripheral() -> Peripheral;
        fn number() -> usize;
        fn sr_ptr() -> *mut u32;
    }
}

/// BSTIMER peripheral instance.
#[allow(private_bounds)]
pub trait Instance: sealed::Instance + PeripheralType + 'static {
    /// Update interrupt for this timer.
    type Interrupt: TypelevelInterrupt;
}

macro_rules! impl_instance {
    ($name:ident, $pac:ident, $interrupt:ident, $peripheral:ident, $number:expr, $base:expr) => {
        impl sealed::Instance for peripherals::$name {
            #[inline]
            fn regs() -> &'static pac::bstimer0::RegisterBlock {
                unsafe { &*pac::$pac::PTR }
            }

            #[inline]
            fn peripheral() -> Peripheral {
                Peripheral::$peripheral
            }

            #[inline]
            fn number() -> usize {
                $number
            }

            #[inline]
            fn sr_ptr() -> *mut u32 {
                ($base + 0x10) as *mut u32
            }
        }

        impl Instance for peripherals::$name {
            type Interrupt = interrupt::typelevel::$interrupt;
        }
    };
}

impl_instance!(BSTIMER0, Bstimer0, BSTIMER0, Bstimer0, 0, 0x4000_c000);
impl_instance!(BSTIMER1, Bstimer1, BSTIMER1, Bstimer1, 1, 0x4001_c000);

/// Interrupt handler for one BSTIMER instance.
///
/// Bind this to `BSTIMER0` / `BSTIMER1` with [`crate::bind_interrupts!`] before
/// constructing an asynchronous timer.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();
        if !regs.sr().read().uif().bit_is_set() {
            return;
        }

        // Clear UIF (PAC marks `SR` read-only; GPTIMER uses the same write-0 clear).
        clear_update_flag_raw(T::sr_ptr());
        regs.dier().modify(|_, w| w.uie().clear_bit());
        UPDATE_PENDING[T::number()].store(true, Ordering::Release);
        WAKERS[T::number()].wake();
    }
}

/// Owned basic timer.
pub struct BsTimer<'d, T: Instance, M: Mode = Blocking> {
    _peri: Peri<'d, T>,
    _mode: PhantomData<M>,
}

impl<'d, T: Instance> BsTimer<'d, T, Blocking> {
    /// Enable clocks, release reset, apply `config`, and leave the timer stopped.
    pub fn new(peri: Peri<'d, T>, config: Config) -> Self {
        let this = Self::create(peri);
        this.configure(config);
        this
    }

    /// Busy-wait until the update interrupt flag is set, then clear it.
    pub fn wait_for_update(&mut self) {
        while !self.update_flag() {}
        self.clear_update_flag();
    }
}

impl<'d, T: Instance> BsTimer<'d, T, Async> {
    /// Enable clocks, release reset, apply `config`, and bind the update IRQ.
    pub fn new(peri: Peri<'d, T>, _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd, config: Config) -> Self {
        UPDATE_PENDING[T::number()].store(false, Ordering::Release);
        T::Interrupt::unpend();
        T::Interrupt::set_priority(Priority::P3);
        unsafe { T::Interrupt::enable() };

        let this = Self::create(peri);
        this.configure(config);
        this
    }

    /// Wait for the next update event.
    ///
    /// Enables `DIER.UIE` until [`InterruptHandler`] observes `SR.UIF`.
    pub async fn wait_for_update(&mut self) {
        let index = T::number();
        UPDATE_PENDING[index].store(false, Ordering::Release);
        clear_update_flag_raw(T::sr_ptr());

        let _uie = UpdateInterruptGuard::<T>::arm();

        poll_fn(|cx| {
            WAKERS[index].register(cx.waker());

            if UPDATE_PENDING[index].swap(false, Ordering::AcqRel) {
                return Poll::Ready(());
            }

            // Already latched before / while UIE was armed.
            if T::regs().sr().read().uif().bit_is_set() {
                clear_update_flag_raw(T::sr_ptr());
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }
}

impl<'d, T: Instance, M: Mode> BsTimer<'d, T, M> {
    fn create(peri: Peri<'d, T>) -> Self {
        let _ = rcc::enable_peripheral(T::peripheral());
        let _ = rcc::reset_peripheral(T::peripheral());
        Self {
            _peri: peri,
            _mode: PhantomData,
        }
    }

    /// Apply configuration while the counter is left as-is (vendor `bstimer_init`).
    pub fn configure(&self, config: Config) {
        let regs = T::regs();

        regs.cr2().modify(|r, w| unsafe {
            let bits = (r.bits() & !CR2_MMS_MASK) | (config.master_mode as u32);
            w.bits(bits)
        });

        regs.cr1().modify(|_, w| w.arpe().bit(config.autoreload_preload));

        unsafe {
            regs.arr().write_with_zero(|w| w.bits(u32::from(config.period)));
            regs.psc().write_with_zero(|w| w.bits(u32::from(config.prescaler)));
        }
    }

    /// Start the counter (`CR1.CEN`).
    pub fn start(&self) {
        T::regs().cr1().modify(|_, w| w.cen().set_bit());
    }

    /// Stop the counter (`CR1.CEN`).
    pub fn stop(&self) {
        T::regs().cr1().modify(|_, w| w.cen().clear_bit());
    }

    /// Enable or disable one-pulse mode (`CR1.OPM`).
    pub fn set_one_pulse(&self, enabled: bool) {
        T::regs().cr1().modify(|_, w| w.opm().bit(enabled));
    }

    /// When enabled, only counter overflow/underflow generates update events
    /// that set UIF (`CR1.URS`, vendor `bstimer_config_overflow_update`).
    pub fn set_overflow_update(&self, enabled: bool) {
        T::regs().cr1().modify(|_, w| w.urs().bit(enabled));
    }

    /// Disable generation of update events (`CR1.UDIS`).
    pub fn set_update_disabled(&self, disabled: bool) {
        T::regs().cr1().modify(|_, w| w.udis().bit(disabled));
    }

    /// Enable or disable DMA requests on update (`DIER.UDE`).
    pub fn set_dma_request(&self, enabled: bool) {
        T::regs().dier().modify(|_, w| w.ude().bit(enabled));
    }

    /// Enable or disable the update interrupt (`DIER.UIE`).
    pub fn set_update_interrupt(&self, enabled: bool) {
        T::regs().dier().modify(|_, w| w.uie().bit(enabled));
    }

    /// Whether `SR.UIF` is set.
    pub fn update_flag(&self) -> bool {
        T::regs().sr().read().uif().bit_is_set()
    }

    /// Clear `SR.UIF`.
    ///
    /// The vendor BSTIMER driver never clears this flag; GPTIMER does via
    /// `SR &= ~UIF`. This uses the same write-clear sequence through a raw
    /// pointer because the PAC marks `SR` read-only.
    pub fn clear_update_flag(&self) {
        clear_update_flag_raw(T::sr_ptr());
    }

    /// Generate an update event (`EGR.UG`), reloading the prescaler shadow.
    pub fn generate_update(&self) {
        unsafe {
            T::regs().egr().write_with_zero(|w| w.ug().set_bit());
        }
    }

    /// Current counter value.
    pub fn counter(&self) -> u32 {
        T::regs().cnt().read().bits()
    }

    /// Write the counter register.
    pub fn set_counter(&self, value: u32) {
        unsafe {
            T::regs().cnt().write_with_zero(|w| w.bits(value));
        }
    }

    /// Current auto-reload value.
    pub fn period(&self) -> u32 {
        T::regs().arr().read().bits()
    }

    /// Current prescaler value.
    pub fn prescaler(&self) -> u32 {
        T::regs().psc().read().bits()
    }

    /// Configure one-pulse mode as in the vendor `bstimer/onepulse` example:
    /// OPM + overflow-only updates, then force an update so `PSC` takes effect
    /// on the first period. Does not start the counter or enable the IRQ.
    pub fn setup_one_pulse(&self) {
        self.set_one_pulse(true);
        self.set_overflow_update(true);
        self.generate_update();
    }
}

impl<'d, T: Instance, M: Mode> Drop for BsTimer<'d, T, M> {
    fn drop(&mut self) {
        self.stop();
        self.set_update_interrupt(false);
        self.clear_update_flag();
        T::Interrupt::disable();
        UPDATE_PENDING[T::number()].store(false, Ordering::Release);
        let _ = rcc::disable_peripheral(T::peripheral());
    }
}

struct UpdateInterruptGuard<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> UpdateInterruptGuard<T> {
    fn arm() -> Self {
        T::regs().dier().modify(|_, w| w.uie().set_bit());
        Self { _phantom: PhantomData }
    }
}

impl<T: Instance> Drop for UpdateInterruptGuard<T> {
    fn drop(&mut self) {
        T::regs().dier().modify(|_, w| w.uie().clear_bit());
    }
}

#[inline]
fn clear_update_flag_raw(sr: *mut u32) {
    // Mirror GPTIMER `timer_clear_status` / embassy-asr `timer.rs`: write 0 to
    // the flag being cleared (rc_w0). BSTIMER PAC `SR` is read-only.
    unsafe {
        core::ptr::write_volatile(sr, !SR_UIF);
    }
}
