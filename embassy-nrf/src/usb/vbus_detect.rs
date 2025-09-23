//! Trait and implementations for performing VBUS detection.

use core::future::{poll_fn, Future};
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;

use super::BUS_WAKER;
use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, pac};

/// Trait for detecting USB VBUS power.
///
/// There are multiple ways to detect USB power. The behavior
/// here provides a hook into determining whether it is.
pub trait VbusDetect {
    /// Report whether power is detected.
    ///
    /// This is indicated by the `USBREGSTATUS.VBUSDETECT` register, or the
    /// `USBDETECTED`, `USBREMOVED` events from the `POWER` peripheral.
    fn is_usb_detected(&self) -> bool;

    /// Wait until USB power is ready.
    ///
    /// USB power ready is indicated by the `USBREGSTATUS.OUTPUTRDY` register, or the
    /// `USBPWRRDY` event from the `POWER` peripheral.
    async fn wait_power_ready(&mut self) -> Result<(), ()>;
}

#[cfg(not(feature = "_nrf5340"))]
type UsbRegIrq = interrupt::typelevel::CLOCK_POWER;
#[cfg(feature = "_nrf5340")]
type UsbRegIrq = interrupt::typelevel::USBREGULATOR;

#[cfg(not(feature = "_nrf5340"))]
const USB_REG_PERI: pac::power::Power = pac::POWER;
#[cfg(feature = "_nrf5340")]
const USB_REG_PERI: pac::usbreg::Usbreg = pac::USBREGULATOR;

/// Interrupt handler.
pub struct InterruptHandler {
    _private: (),
}

impl interrupt::typelevel::Handler<UsbRegIrq> for InterruptHandler {
    unsafe fn on_interrupt() {
        let regs = USB_REG_PERI;

        if regs.events_usbdetected().read() != 0 {
            regs.events_usbdetected().write_value(0);
            BUS_WAKER.wake();
        }

        if regs.events_usbremoved().read() != 0 {
            regs.events_usbremoved().write_value(0);
            BUS_WAKER.wake();
            POWER_WAKER.wake();
        }

        if regs.events_usbpwrrdy().read() != 0 {
            regs.events_usbpwrrdy().write_value(0);
            POWER_WAKER.wake();
        }
    }
}

/// [`VbusDetect`] implementation using the native hardware POWER peripheral.
///
/// Unsuitable for usage with the nRF softdevice, since it reserves exclusive acces
/// to POWER. In that case, use [SoftwareVbusDetect].
pub struct HardwareVbusDetect {
    _private: (),
}

static POWER_WAKER: AtomicWaker = AtomicWaker::new();

impl HardwareVbusDetect {
    /// Create a new `VbusDetectNative`.
    pub fn new(_irq: impl interrupt::typelevel::Binding<UsbRegIrq, InterruptHandler> + 'static) -> Self {
        let regs = USB_REG_PERI;

        UsbRegIrq::unpend();
        unsafe { UsbRegIrq::enable() };

        regs.intenset().write(|w| {
            w.set_usbdetected(true);
            w.set_usbremoved(true);
            w.set_usbpwrrdy(true);
        });

        Self { _private: () }
    }
}

impl VbusDetect for HardwareVbusDetect {
    fn is_usb_detected(&self) -> bool {
        let regs = USB_REG_PERI;
        regs.usbregstatus().read().vbusdetect()
    }

    fn wait_power_ready(&mut self) -> impl Future<Output = Result<(), ()>> {
        poll_fn(|cx| {
            POWER_WAKER.register(cx.waker());
            let regs = USB_REG_PERI;

            if regs.usbregstatus().read().outputrdy() {
                Poll::Ready(Ok(()))
            } else if !self.is_usb_detected() {
                Poll::Ready(Err(()))
            } else {
                Poll::Pending
            }
        })
    }
}

/// Software-backed [`VbusDetect`] implementation.
///
/// This implementation does not interact with the hardware, it allows user code
/// to notify the power events by calling functions instead.
///
/// This is suitable for use with the nRF softdevice, by calling the functions
/// when the softdevice reports power-related events.
pub struct SoftwareVbusDetect {
    usb_detected: AtomicBool,
    power_ready: AtomicBool,
}

impl SoftwareVbusDetect {
    /// Create a new `SoftwareVbusDetect`.
    pub fn new(usb_detected: bool, power_ready: bool) -> Self {
        BUS_WAKER.wake();

        Self {
            usb_detected: AtomicBool::new(usb_detected),
            power_ready: AtomicBool::new(power_ready),
        }
    }

    /// Report whether power was detected.
    ///
    /// Equivalent to the `USBDETECTED`, `USBREMOVED` events from the `POWER` peripheral.
    pub fn detected(&self, detected: bool) {
        self.usb_detected.store(detected, Ordering::Relaxed);
        self.power_ready.store(false, Ordering::Relaxed);
        BUS_WAKER.wake();
        POWER_WAKER.wake();
    }

    /// Report when USB power is ready.
    ///
    /// Equivalent to the `USBPWRRDY` event from the `POWER` peripheral.
    pub fn ready(&self) {
        self.power_ready.store(true, Ordering::Relaxed);
        POWER_WAKER.wake();
    }
}

impl VbusDetect for &SoftwareVbusDetect {
    fn is_usb_detected(&self) -> bool {
        self.usb_detected.load(Ordering::Relaxed)
    }

    fn wait_power_ready(&mut self) -> impl Future<Output = Result<(), ()>> {
        poll_fn(move |cx| {
            POWER_WAKER.register(cx.waker());

            if self.power_ready.load(Ordering::Relaxed) {
                Poll::Ready(Ok(()))
            } else if !self.usb_detected.load(Ordering::Relaxed) {
                Poll::Ready(Err(()))
            } else {
                Poll::Pending
            }
        })
    }
}
