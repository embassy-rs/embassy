//! Minimal interrupt helpers mirroring embassy-imxrt style for OS_EVENT and LPUART2.
//! Type-level interrupt traits and bindings are provided by the
//! `embassy_hal_internal::interrupt_mod!` macro via the generated module below.

// TODO(AJM): As of 2025-11-13, we need to do a pass to ensure safety docs
// are complete prior to release.
#![allow(clippy::missing_safety_doc)]

mod generated {
    #[rustfmt::skip]
    embassy_hal_internal::interrupt_mod!(
        ADC0,
        ADC1,
        ADC2,
        ADC3,
        DMA_CH0,
        DMA_CH1,
        DMA_CH2,
        DMA_CH3,
        DMA_CH4,
        DMA_CH5,
        DMA_CH6,
        DMA_CH7,
        GPIO0,
        GPIO1,
        GPIO2,
        GPIO3,
        GPIO4,
        LPI2C0,
        LPI2C1,
        LPI2C2,
        LPI2C3,
        LPUART0,
        LPUART1,
        LPUART2,
        LPUART3,
        LPUART4,
        LPUART5,
        OS_EVENT,
        RTC,
        TRNG0
    );
}

use core::sync::atomic::{AtomicU16, AtomicU32, Ordering};

pub use generated::interrupt::{Priority, typelevel};

use crate::pac::Interrupt;

/// Trait for configuring and controlling interrupts.
///
/// This trait provides a consistent interface for interrupt management across
/// different interrupt sources, similar to embassy-imxrt's InterruptExt.
pub trait InterruptExt {
    /// Clear any pending interrupt in NVIC.
    fn unpend(&self);

    /// Set NVIC priority for this interrupt.
    fn set_priority(&self, priority: Priority);

    /// Enable this interrupt in NVIC.
    ///
    /// # Safety
    /// This function is unsafe because it can enable interrupts that may not be
    /// properly configured, potentially leading to undefined behavior.
    unsafe fn enable(&self);

    /// Disable this interrupt in NVIC.
    ///
    /// # Safety
    /// This function is unsafe because disabling interrupts may leave the system
    /// in an inconsistent state if the interrupt was expected to fire.
    unsafe fn disable(&self);

    /// Check if the interrupt is pending in NVIC.
    fn is_pending(&self) -> bool;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultHandlerSnapshot {
    pub vector: u16,
    pub count: u32,
    pub cfsr: u32,
    pub hfsr: u32,
    pub stacked_pc: u32,
    pub stacked_lr: u32,
    pub stacked_sp: u32,
}

static LAST_DEFAULT_VECTOR: AtomicU16 = AtomicU16::new(0);
static LAST_DEFAULT_COUNT: AtomicU32 = AtomicU32::new(0);
static LAST_DEFAULT_CFSR: AtomicU32 = AtomicU32::new(0);
static LAST_DEFAULT_HFSR: AtomicU32 = AtomicU32::new(0);
static LAST_DEFAULT_PC: AtomicU32 = AtomicU32::new(0);
static LAST_DEFAULT_LR: AtomicU32 = AtomicU32::new(0);
static LAST_DEFAULT_SP: AtomicU32 = AtomicU32::new(0);

#[inline]
pub fn default_handler_snapshot() -> DefaultHandlerSnapshot {
    DefaultHandlerSnapshot {
        vector: LAST_DEFAULT_VECTOR.load(Ordering::Relaxed),
        count: LAST_DEFAULT_COUNT.load(Ordering::Relaxed),
        cfsr: LAST_DEFAULT_CFSR.load(Ordering::Relaxed),
        hfsr: LAST_DEFAULT_HFSR.load(Ordering::Relaxed),
        stacked_pc: LAST_DEFAULT_PC.load(Ordering::Relaxed),
        stacked_lr: LAST_DEFAULT_LR.load(Ordering::Relaxed),
        stacked_sp: LAST_DEFAULT_SP.load(Ordering::Relaxed),
    }
}

#[inline]
pub fn clear_default_handler_snapshot() {
    LAST_DEFAULT_VECTOR.store(0, Ordering::Relaxed);
    LAST_DEFAULT_COUNT.store(0, Ordering::Relaxed);
    LAST_DEFAULT_CFSR.store(0, Ordering::Relaxed);
    LAST_DEFAULT_HFSR.store(0, Ordering::Relaxed);
    LAST_DEFAULT_PC.store(0, Ordering::Relaxed);
    LAST_DEFAULT_LR.store(0, Ordering::Relaxed);
    LAST_DEFAULT_SP.store(0, Ordering::Relaxed);
}

/// OS_EVENT interrupt helper with methods similar to embassy-imxrt's InterruptExt.
pub struct OsEvent;
pub const OS_EVENT: OsEvent = OsEvent;

impl InterruptExt for OsEvent {
    /// Clear any pending OS_EVENT in NVIC.
    #[inline]
    fn unpend(&self) {
        cortex_m::peripheral::NVIC::unpend(Interrupt::OS_EVENT);
    }

    /// Set NVIC priority for OS_EVENT.
    #[inline]
    fn set_priority(&self, priority: Priority) {
        unsafe {
            let mut nvic = cortex_m::peripheral::Peripherals::steal().NVIC;
            nvic.set_priority(Interrupt::OS_EVENT, u8::from(priority));
        }
    }

    /// Enable OS_EVENT in NVIC.
    #[inline]
    unsafe fn enable(&self) {
        cortex_m::peripheral::NVIC::unmask(Interrupt::OS_EVENT);
    }

    /// Disable OS_EVENT in NVIC.
    #[inline]
    unsafe fn disable(&self) {
        cortex_m::peripheral::NVIC::mask(Interrupt::OS_EVENT);
    }

    /// Check if OS_EVENT is pending in NVIC.
    #[inline]
    fn is_pending(&self) -> bool {
        cortex_m::peripheral::NVIC::is_pending(Interrupt::OS_EVENT)
    }
}

impl OsEvent {
    /// Configure OS_EVENT interrupt for timer operation.
    /// This sets up the NVIC priority, enables the interrupt, and ensures global interrupts are enabled.
    /// Also performs a software event to wake any pending WFE.
    pub fn configure_for_timer(&self, priority: Priority) {
        // Configure NVIC
        self.unpend();
        self.set_priority(priority);
        unsafe {
            self.enable();
        }

        // Ensure global interrupts are enabled in no-reset scenarios (e.g., cargo run)
        // Debuggers typically perform a reset which leaves PRIMASK=0; cargo run may not.
        unsafe {
            cortex_m::interrupt::enable();
        }

        // Wake any executor WFE that might be sleeping when we armed the first deadline
        cortex_m::asm::sev();
    }
}

/// LPUART2 interrupt helper with methods similar to embassy-imxrt's InterruptExt.
pub struct Lpuart2;
pub const LPUART2: Lpuart2 = Lpuart2;

impl InterruptExt for Lpuart2 {
    /// Clear any pending LPUART2 in NVIC.
    #[inline]
    fn unpend(&self) {
        cortex_m::peripheral::NVIC::unpend(Interrupt::LPUART2);
    }

    /// Set NVIC priority for LPUART2.
    #[inline]
    fn set_priority(&self, priority: Priority) {
        unsafe {
            let mut nvic = cortex_m::peripheral::Peripherals::steal().NVIC;
            nvic.set_priority(Interrupt::LPUART2, u8::from(priority));
        }
    }

    /// Enable LPUART2 in NVIC.
    #[inline]
    unsafe fn enable(&self) {
        cortex_m::peripheral::NVIC::unmask(Interrupt::LPUART2);
    }

    /// Disable LPUART2 in NVIC.
    #[inline]
    unsafe fn disable(&self) {
        cortex_m::peripheral::NVIC::mask(Interrupt::LPUART2);
    }

    /// Check if LPUART2 is pending in NVIC.
    #[inline]
    fn is_pending(&self) -> bool {
        cortex_m::peripheral::NVIC::is_pending(Interrupt::LPUART2)
    }
}

impl Lpuart2 {
    /// Configure LPUART2 interrupt for UART operation.
    /// This sets up the NVIC priority, enables the interrupt, and ensures global interrupts are enabled.
    pub fn configure_for_uart(&self, priority: Priority) {
        // Configure NVIC
        self.unpend();
        self.set_priority(priority);
        unsafe {
            self.enable();
        }

        // Ensure global interrupts are enabled in no-reset scenarios (e.g., cargo run)
        // Debuggers typically perform a reset which leaves PRIMASK=0; cargo run may not.
        unsafe {
            cortex_m::interrupt::enable();
        }
    }

    /// Install LPUART2 handler into the RAM vector table.
    /// Safety: See `install_irq_handler`.
    pub unsafe fn install_handler(&self, handler: unsafe extern "C" fn()) {
        install_irq_handler(Interrupt::LPUART2, handler);
    }
}

pub struct Rtc;
pub const RTC: Rtc = Rtc;

impl InterruptExt for Rtc {
    /// Clear any pending RTC in NVIC.
    #[inline]
    fn unpend(&self) {
        cortex_m::peripheral::NVIC::unpend(Interrupt::RTC);
    }

    /// Set NVIC priority for RTC.
    #[inline]
    fn set_priority(&self, priority: Priority) {
        unsafe {
            let mut nvic = cortex_m::peripheral::Peripherals::steal().NVIC;
            nvic.set_priority(Interrupt::RTC, u8::from(priority));
        }
    }

    /// Enable RTC in NVIC.
    #[inline]
    unsafe fn enable(&self) {
        cortex_m::peripheral::NVIC::unmask(Interrupt::RTC);
    }

    /// Disable RTC in NVIC.
    #[inline]
    unsafe fn disable(&self) {
        cortex_m::peripheral::NVIC::mask(Interrupt::RTC);
    }

    /// Check if RTC is pending in NVIC.
    #[inline]
    fn is_pending(&self) -> bool {
        cortex_m::peripheral::NVIC::is_pending(Interrupt::RTC)
    }
}

pub struct Gpio0;
pub const GPIO0: Gpio0 = Gpio0;

impl InterruptExt for Gpio0 {
    /// Clear any pending GPIO0 in NVIC.
    #[inline]
    fn unpend(&self) {
        cortex_m::peripheral::NVIC::unpend(Interrupt::GPIO0);
    }

    /// Set NVIC priority for GPIO0.
    #[inline]
    fn set_priority(&self, priority: Priority) {
        unsafe {
            let mut nvic = cortex_m::peripheral::Peripherals::steal().NVIC;
            nvic.set_priority(Interrupt::GPIO0, u8::from(priority));
        }
    }

    /// Enable GPIO0 in NVIC.
    #[inline]
    unsafe fn enable(&self) {
        cortex_m::peripheral::NVIC::unmask(Interrupt::GPIO0);
    }

    /// Disable GPIO0 in NVIC.
    #[inline]
    unsafe fn disable(&self) {
        cortex_m::peripheral::NVIC::mask(Interrupt::GPIO0);
    }

    /// Check if GPIO0 is pending in NVIC.
    #[inline]
    fn is_pending(&self) -> bool {
        cortex_m::peripheral::NVIC::is_pending(Interrupt::GPIO0)
    }
}

pub struct Gpio1;
pub const GPIO1: Gpio1 = Gpio1;

impl InterruptExt for Gpio1 {
    /// Clear any pending GPIO1 in NVIC.
    #[inline]
    fn unpend(&self) {
        cortex_m::peripheral::NVIC::unpend(Interrupt::GPIO1);
    }

    /// Set NVIC priority for GPIO1.
    #[inline]
    fn set_priority(&self, priority: Priority) {
        unsafe {
            let mut nvic = cortex_m::peripheral::Peripherals::steal().NVIC;
            nvic.set_priority(Interrupt::GPIO1, u8::from(priority));
        }
    }

    /// Enable GPIO1 in NVIC.
    #[inline]
    unsafe fn enable(&self) {
        cortex_m::peripheral::NVIC::unmask(Interrupt::GPIO1);
    }

    /// Disable GPIO1 in NVIC.
    #[inline]
    unsafe fn disable(&self) {
        cortex_m::peripheral::NVIC::mask(Interrupt::GPIO1);
    }

    /// Check if GPIO1 is pending in NVIC.
    #[inline]
    fn is_pending(&self) -> bool {
        cortex_m::peripheral::NVIC::is_pending(Interrupt::GPIO1)
    }
}

pub struct Gpio2;
pub const GPIO2: Gpio2 = Gpio2;

impl InterruptExt for Gpio2 {
    /// Clear any pending GPIO2 in NVIC.
    #[inline]
    fn unpend(&self) {
        cortex_m::peripheral::NVIC::unpend(Interrupt::GPIO2);
    }

    /// Set NVIC priority for GPIO2.
    #[inline]
    fn set_priority(&self, priority: Priority) {
        unsafe {
            let mut nvic = cortex_m::peripheral::Peripherals::steal().NVIC;
            nvic.set_priority(Interrupt::GPIO2, u8::from(priority));
        }
    }

    /// Enable GPIO2 in NVIC.
    #[inline]
    unsafe fn enable(&self) {
        cortex_m::peripheral::NVIC::unmask(Interrupt::GPIO2);
    }

    /// Disable GPIO2 in NVIC.
    #[inline]
    unsafe fn disable(&self) {
        cortex_m::peripheral::NVIC::mask(Interrupt::GPIO2);
    }

    /// Check if GPIO2 is pending in NVIC.
    #[inline]
    fn is_pending(&self) -> bool {
        cortex_m::peripheral::NVIC::is_pending(Interrupt::GPIO2)
    }
}

pub struct Gpio3;
pub const GPIO3: Gpio3 = Gpio3;

impl InterruptExt for Gpio3 {
    /// Clear any pending GPIO3 in NVIC.
    #[inline]
    fn unpend(&self) {
        cortex_m::peripheral::NVIC::unpend(Interrupt::GPIO3);
    }

    /// Set NVIC priority for GPIO3.
    #[inline]
    fn set_priority(&self, priority: Priority) {
        unsafe {
            let mut nvic = cortex_m::peripheral::Peripherals::steal().NVIC;
            nvic.set_priority(Interrupt::GPIO3, u8::from(priority));
        }
    }

    /// Enable GPIO3 in NVIC.
    #[inline]
    unsafe fn enable(&self) {
        cortex_m::peripheral::NVIC::unmask(Interrupt::GPIO3);
    }

    /// Disable GPIO3 in NVIC.
    #[inline]
    unsafe fn disable(&self) {
        cortex_m::peripheral::NVIC::mask(Interrupt::GPIO3);
    }

    /// Check if GPIO3 is pending in NVIC.
    #[inline]
    fn is_pending(&self) -> bool {
        cortex_m::peripheral::NVIC::is_pending(Interrupt::GPIO3)
    }
}

pub struct Gpio4;
pub const GPIO4: Gpio4 = Gpio4;

impl InterruptExt for Gpio4 {
    /// Clear any pending GPIO4 in NVIC.
    #[inline]
    fn unpend(&self) {
        cortex_m::peripheral::NVIC::unpend(Interrupt::GPIO4);
    }

    /// Set NVIC priority for GPIO4.
    #[inline]
    fn set_priority(&self, priority: Priority) {
        unsafe {
            let mut nvic = cortex_m::peripheral::Peripherals::steal().NVIC;
            nvic.set_priority(Interrupt::GPIO4, u8::from(priority));
        }
    }

    /// Enable GPIO4 in NVIC.
    #[inline]
    unsafe fn enable(&self) {
        cortex_m::peripheral::NVIC::unmask(Interrupt::GPIO4);
    }

    /// Disable GPIO4 in NVIC.
    #[inline]
    unsafe fn disable(&self) {
        cortex_m::peripheral::NVIC::mask(Interrupt::GPIO4);
    }

    /// Check if GPIO4 is pending in NVIC.
    #[inline]
    fn is_pending(&self) -> bool {
        cortex_m::peripheral::NVIC::is_pending(Interrupt::GPIO4)
    }
}

/// Set VTOR (Vector Table Offset) to a RAM-based vector table.
/// Pass a pointer to the first word in the RAM table (stack pointer slot 0).
/// Safety: Caller must ensure the RAM table is valid and aligned as required by the core.
pub unsafe fn vtor_set_ram_vector_base(base: *const u32) {
    core::ptr::write_volatile(0xE000_ED08 as *mut u32, base as u32);
}

/// Install an interrupt handler into the current VTOR-based vector table.
/// This writes the function pointer at index 16 + irq number.
/// Safety: Caller must ensure VTOR points at a writable RAM table and that `handler`
/// has the correct ABI and lifetime.
pub unsafe fn install_irq_handler(irq: Interrupt, handler: unsafe extern "C" fn()) {
    let vtor_base = core::ptr::read_volatile(0xE000_ED08 as *const u32) as *mut u32;
    let idx = 16 + (irq as isize as usize);
    core::ptr::write_volatile(vtor_base.add(idx), handler as usize as u32);
}

impl OsEvent {
    /// Convenience to install the OS_EVENT handler into the RAM vector table.
    /// Safety: See `install_irq_handler`.
    pub unsafe fn install_handler(&self, handler: extern "C" fn()) {
        install_irq_handler(Interrupt::OS_EVENT, handler);
    }
}

/// Install OS_EVENT handler by raw address. Useful to avoid fn pointer type mismatches.
/// Safety: Caller must ensure the address is a valid `extern "C" fn()` handler.
pub unsafe fn os_event_install_handler_raw(handler_addr: usize) {
    let vtor_base = core::ptr::read_volatile(0xE000_ED08 as *const u32) as *mut u32;
    let idx = 16 + (Interrupt::OS_EVENT as isize as usize);
    core::ptr::write_volatile(vtor_base.add(idx), handler_addr as u32);
}

/// Provide a conservative default IRQ handler that avoids wedging the system.
/// It clears all NVIC pending bits and returns, so spurious or reserved IRQs
/// don’t trap the core in an infinite WFI loop during bring-up.
#[no_mangle]
pub unsafe extern "C" fn DefaultHandler() {
    let active = core::ptr::read_volatile(0xE000_ED04 as *const u32) & 0x1FF;
    let cfsr = core::ptr::read_volatile(0xE000_ED28 as *const u32);
    let hfsr = core::ptr::read_volatile(0xE000_ED2C as *const u32);

    let sp = cortex_m::register::msp::read();
    let stacked = sp as *const u32;
    // Stacked registers follow ARMv8-M procedure call standard order
    let stacked_pc = unsafe { stacked.add(6).read() };
    let stacked_lr = unsafe { stacked.add(5).read() };

    LAST_DEFAULT_VECTOR.store(active as u16, Ordering::Relaxed);
    LAST_DEFAULT_CFSR.store(cfsr, Ordering::Relaxed);
    LAST_DEFAULT_HFSR.store(hfsr, Ordering::Relaxed);
    LAST_DEFAULT_COUNT.fetch_add(1, Ordering::Relaxed);
    LAST_DEFAULT_PC.store(stacked_pc, Ordering::Relaxed);
    LAST_DEFAULT_LR.store(stacked_lr, Ordering::Relaxed);
    LAST_DEFAULT_SP.store(sp, Ordering::Relaxed);

    // Do nothing here: on some MCUs/TrustZone setups, writing NVIC from a spurious
    // handler can fault if targeting the Secure bank. Just return.
    cortex_m::asm::dsb();
    cortex_m::asm::isb();
}
