//! Digital Camera Interface (DCMI)
use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::Transfer;
use crate::gpio::{AfType, Pull};
use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, rcc, Peri};

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let ris = crate::pac::DCMI.ris().read();
        if ris.err_ris() {
            trace!("DCMI IRQ: Error.");
            crate::pac::DCMI.ier().modify(|ier| ier.set_err_ie(false));
        }
        if ris.ovr_ris() {
            trace!("DCMI IRQ: Overrun.");
            crate::pac::DCMI.ier().modify(|ier| ier.set_ovr_ie(false));
        }
        if ris.frame_ris() {
            trace!("DCMI IRQ: Frame captured.");
            crate::pac::DCMI.ier().modify(|ier| ier.set_frame_ie(false));
        }
        STATE.waker.wake();
    }
}

/// The level on the VSync pin when the data is not valid on the parallel interface.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq)]
pub enum VSyncDataInvalidLevel {
    Low,
    High,
}

/// The level on the VSync pin when the data is not valid on the parallel interface.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq)]
pub enum HSyncDataInvalidLevel {
    Low,
    High,
}

#[derive(Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum PixelClockPolarity {
    RisingEdge,
    FallingEdge,
}

struct State {
    waker: AtomicWaker,
}

impl State {
    const fn new() -> State {
        State {
            waker: AtomicWaker::new(),
        }
    }
}

static STATE: State = State::new();

/// DCMI error.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Overrun error: the hardware generated data faster than we could read it.
    Overrun,
    /// Internal peripheral error.
    PeripheralError,
}

/// DCMI configuration.
#[non_exhaustive]
pub struct Config {
    /// VSYNC level.
    pub vsync_level: VSyncDataInvalidLevel,
    /// HSYNC level.
    pub hsync_level: HSyncDataInvalidLevel,
    /// PIXCLK polarity.
    pub pixclk_polarity: PixelClockPolarity,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vsync_level: VSyncDataInvalidLevel::High,
            hsync_level: HSyncDataInvalidLevel::Low,
            pixclk_polarity: PixelClockPolarity::RisingEdge,
        }
    }
}

macro_rules! config_pins {
    ($($pin:ident),*) => {
                critical_section::with(|_| {
            $(
                $pin.set_as_af($pin.af_num(), AfType::input(Pull::None));
            )*
        })
    };
}

/// DCMI driver.
pub struct Dcmi<'d, T: Instance, Dma: FrameDma<T>> {
    inner: Peri<'d, T>,
    dma: Peri<'d, Dma>,
}

impl<'d, T, Dma> Dcmi<'d, T, Dma>
where
    T: Instance,
    Dma: FrameDma<T>,
{
    /// Create a new DCMI driver with 8 data bits.
    pub fn new_8bit(
        peri: Peri<'d, T>,
        dma: Peri<'d, Dma>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        v_sync: Peri<'d, impl VSyncPin<T>>,
        h_sync: Peri<'d, impl HSyncPin<T>>,
        pixclk: Peri<'d, impl PixClkPin<T>>,
        config: Config,
    ) -> Self {
        config_pins!(d0, d1, d2, d3, d4, d5, d6, d7);
        config_pins!(v_sync, h_sync, pixclk);

        Self::new_inner(peri, dma, config, false, 0b00)
    }

    /// Create a new DCMI driver with 10 data bits.
    pub fn new_10bit(
        peri: Peri<'d, T>,
        dma: Peri<'d, Dma>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        d8: Peri<'d, impl D8Pin<T>>,
        d9: Peri<'d, impl D9Pin<T>>,
        v_sync: Peri<'d, impl VSyncPin<T>>,
        h_sync: Peri<'d, impl HSyncPin<T>>,
        pixclk: Peri<'d, impl PixClkPin<T>>,
        config: Config,
    ) -> Self {
        config_pins!(d0, d1, d2, d3, d4, d5, d6, d7, d8, d9);
        config_pins!(v_sync, h_sync, pixclk);

        Self::new_inner(peri, dma, config, false, 0b01)
    }

    /// Create a new DCMI driver with 12 data bits.
    pub fn new_12bit(
        peri: Peri<'d, T>,
        dma: Peri<'d, Dma>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        d8: Peri<'d, impl D8Pin<T>>,
        d9: Peri<'d, impl D9Pin<T>>,
        d10: Peri<'d, impl D10Pin<T>>,
        d11: Peri<'d, impl D11Pin<T>>,
        v_sync: Peri<'d, impl VSyncPin<T>>,
        h_sync: Peri<'d, impl HSyncPin<T>>,
        pixclk: Peri<'d, impl PixClkPin<T>>,
        config: Config,
    ) -> Self {
        config_pins!(d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11);
        config_pins!(v_sync, h_sync, pixclk);

        Self::new_inner(peri, dma, config, false, 0b10)
    }

    /// Create a new DCMI driver with 14 data bits.
    pub fn new_14bit(
        peri: Peri<'d, T>,
        dma: Peri<'d, Dma>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        d8: Peri<'d, impl D8Pin<T>>,
        d9: Peri<'d, impl D9Pin<T>>,
        d10: Peri<'d, impl D10Pin<T>>,
        d11: Peri<'d, impl D11Pin<T>>,
        d12: Peri<'d, impl D12Pin<T>>,
        d13: Peri<'d, impl D13Pin<T>>,
        v_sync: Peri<'d, impl VSyncPin<T>>,
        h_sync: Peri<'d, impl HSyncPin<T>>,
        pixclk: Peri<'d, impl PixClkPin<T>>,
        config: Config,
    ) -> Self {
        config_pins!(d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12, d13);
        config_pins!(v_sync, h_sync, pixclk);

        Self::new_inner(peri, dma, config, false, 0b11)
    }

    /// Create a new DCMI driver with 8 data bits, with embedded synchronization.
    pub fn new_es_8bit(
        peri: Peri<'d, T>,
        dma: Peri<'d, Dma>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        pixclk: Peri<'d, impl PixClkPin<T>>,
        config: Config,
    ) -> Self {
        config_pins!(d0, d1, d2, d3, d4, d5, d6, d7);
        config_pins!(pixclk);

        Self::new_inner(peri, dma, config, true, 0b00)
    }

    /// Create a new DCMI driver with 10 data bits, with embedded synchronization.
    pub fn new_es_10bit(
        peri: Peri<'d, T>,
        dma: Peri<'d, Dma>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        d8: Peri<'d, impl D8Pin<T>>,
        d9: Peri<'d, impl D9Pin<T>>,
        pixclk: Peri<'d, impl PixClkPin<T>>,
        config: Config,
    ) -> Self {
        config_pins!(d0, d1, d2, d3, d4, d5, d6, d7, d8, d9);
        config_pins!(pixclk);

        Self::new_inner(peri, dma, config, true, 0b01)
    }

    /// Create a new DCMI driver with 12 data bits, with embedded synchronization.
    pub fn new_es_12bit(
        peri: Peri<'d, T>,
        dma: Peri<'d, Dma>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        d8: Peri<'d, impl D8Pin<T>>,
        d9: Peri<'d, impl D9Pin<T>>,
        d10: Peri<'d, impl D10Pin<T>>,
        d11: Peri<'d, impl D11Pin<T>>,
        pixclk: Peri<'d, impl PixClkPin<T>>,
        config: Config,
    ) -> Self {
        config_pins!(d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11);
        config_pins!(pixclk);

        Self::new_inner(peri, dma, config, true, 0b10)
    }

    /// Create a new DCMI driver with 14 data bits, with embedded synchronization.
    pub fn new_es_14bit(
        peri: Peri<'d, T>,
        dma: Peri<'d, Dma>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        d8: Peri<'d, impl D8Pin<T>>,
        d9: Peri<'d, impl D9Pin<T>>,
        d10: Peri<'d, impl D10Pin<T>>,
        d11: Peri<'d, impl D11Pin<T>>,
        d12: Peri<'d, impl D12Pin<T>>,
        d13: Peri<'d, impl D13Pin<T>>,
        pixclk: Peri<'d, impl PixClkPin<T>>,
        config: Config,
    ) -> Self {
        config_pins!(d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12, d13);
        config_pins!(pixclk);

        Self::new_inner(peri, dma, config, true, 0b11)
    }

    fn new_inner(
        peri: Peri<'d, T>,
        dma: Peri<'d, Dma>,
        config: Config,
        use_embedded_synchronization: bool,
        edm: u8,
    ) -> Self {
        rcc::enable_and_reset::<T>();

        peri.regs().cr().modify(|r| {
            r.set_cm(true); // disable continuous mode (snapshot mode)
            r.set_ess(use_embedded_synchronization);
            r.set_pckpol(config.pixclk_polarity == PixelClockPolarity::RisingEdge);
            r.set_vspol(config.vsync_level == VSyncDataInvalidLevel::High);
            r.set_hspol(config.hsync_level == HSyncDataInvalidLevel::High);
            r.set_fcrc(0x00); // capture every frame
            r.set_edm(edm); // extended data mode
        });

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self { inner: peri, dma }
    }

    fn toggle(enable: bool) {
        crate::pac::DCMI.cr().modify(|r| {
            r.set_enable(enable);
            r.set_capture(enable);
        })
    }

    fn enable_irqs() {
        crate::pac::DCMI.ier().modify(|r| {
            r.set_err_ie(true);
            r.set_ovr_ie(true);
            r.set_frame_ie(true);
        });
    }

    fn clear_interrupt_flags() {
        crate::pac::DCMI.icr().write(|r| {
            r.set_ovr_isc(true);
            r.set_err_isc(true);
            r.set_frame_isc(true);
        })
    }

    /// This method starts the capture and finishes when both the dma transfer and DCMI finish the frame transfer.
    /// The implication is that the input buffer size must be exactly the size of the captured frame.
    pub async fn capture(&mut self, buffer: &mut [u32]) -> Result<(), Error> {
        let r = self.inner.regs();
        let src = r.dr().as_ptr() as *mut u32;
        let request = self.dma.request();
        let dma_read = unsafe { Transfer::new_read(self.dma.reborrow(), request, src, buffer, Default::default()) };

        Self::clear_interrupt_flags();
        Self::enable_irqs();

        Self::toggle(true);

        let result = poll_fn(|cx| {
            STATE.waker.register(cx.waker());

            let ris = crate::pac::DCMI.ris().read();
            if ris.err_ris() {
                crate::pac::DCMI.icr().write(|r| r.set_err_isc(true));
                Poll::Ready(Err(Error::PeripheralError))
            } else if ris.ovr_ris() {
                crate::pac::DCMI.icr().write(|r| r.set_ovr_isc(true));
                Poll::Ready(Err(Error::Overrun))
            } else if ris.frame_ris() {
                crate::pac::DCMI.icr().write(|r| r.set_frame_isc(true));
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        });

        let (_, result) = embassy_futures::join::join(dma_read, result).await;

        Self::toggle(false);

        result
    }
}

trait SealedInstance: crate::rcc::RccPeripheral {
    fn regs(&self) -> crate::pac::dcmi::Dcmi;
}

/// DCMI instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// Interrupt for this instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(D0Pin, Instance);
pin_trait!(D1Pin, Instance);
pin_trait!(D2Pin, Instance);
pin_trait!(D3Pin, Instance);
pin_trait!(D4Pin, Instance);
pin_trait!(D5Pin, Instance);
pin_trait!(D6Pin, Instance);
pin_trait!(D7Pin, Instance);
pin_trait!(D8Pin, Instance);
pin_trait!(D9Pin, Instance);
pin_trait!(D10Pin, Instance);
pin_trait!(D11Pin, Instance);
pin_trait!(D12Pin, Instance);
pin_trait!(D13Pin, Instance);
pin_trait!(HSyncPin, Instance);
pin_trait!(VSyncPin, Instance);
pin_trait!(PixClkPin, Instance);

// allow unused as U5 sources do not contain interrupt nor dma data
#[allow(unused)]
macro_rules! impl_peripheral {
    ($inst:ident, $irq:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn regs(&self) -> crate::pac::dcmi::Dcmi {
                crate::pac::$inst
            }
        }

        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}

foreach_interrupt! {
    ($inst:ident, dcmi, $block:ident, GLOBAL, $irq:ident) => {
        impl_peripheral!($inst, $irq);
    };
}

dma_trait!(FrameDma, Instance);
