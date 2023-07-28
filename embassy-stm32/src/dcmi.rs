use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::Transfer;
use crate::gpio::sealed::AFType;
use crate::gpio::Speed;
use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, Peripheral};

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
#[derive(Clone, Copy, PartialEq)]
pub enum VSyncDataInvalidLevel {
    Low,
    High,
}

/// The level on the VSync pin when the data is not valid on the parallel interface.
#[derive(Clone, Copy, PartialEq)]
pub enum HSyncDataInvalidLevel {
    Low,
    High,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PixelClockPolarity {
    RisingEdge,
    FallingEdge,
}

pub struct State {
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

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    Overrun,
    PeripheralError,
}

#[non_exhaustive]
pub struct Config {
    pub vsync_level: VSyncDataInvalidLevel,
    pub hsync_level: HSyncDataInvalidLevel,
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
        into_ref!($($pin),*);
        critical_section::with(|_| {
            $(
                $pin.set_as_af($pin.af_num(), AFType::Input);
                $pin.set_speed(Speed::VeryHigh);
            )*
        })
    };
}

pub struct Dcmi<'d, T: Instance, Dma: FrameDma<T>> {
    inner: PeripheralRef<'d, T>,
    dma: PeripheralRef<'d, Dma>,
}

impl<'d, T, Dma> Dcmi<'d, T, Dma>
where
    T: Instance,
    Dma: FrameDma<T>,
{
    pub fn new_8bit(
        peri: impl Peripheral<P = T> + 'd,
        dma: impl Peripheral<P = Dma> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        d4: impl Peripheral<P = impl D4Pin<T>> + 'd,
        d5: impl Peripheral<P = impl D5Pin<T>> + 'd,
        d6: impl Peripheral<P = impl D6Pin<T>> + 'd,
        d7: impl Peripheral<P = impl D7Pin<T>> + 'd,
        v_sync: impl Peripheral<P = impl VSyncPin<T>> + 'd,
        h_sync: impl Peripheral<P = impl HSyncPin<T>> + 'd,
        pixclk: impl Peripheral<P = impl PixClkPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, dma);
        config_pins!(d0, d1, d2, d3, d4, d5, d6, d7);
        config_pins!(v_sync, h_sync, pixclk);

        Self::new_inner(peri, dma, config, false, 0b00)
    }

    pub fn new_10bit(
        peri: impl Peripheral<P = T> + 'd,
        dma: impl Peripheral<P = Dma> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        d4: impl Peripheral<P = impl D4Pin<T>> + 'd,
        d5: impl Peripheral<P = impl D5Pin<T>> + 'd,
        d6: impl Peripheral<P = impl D6Pin<T>> + 'd,
        d7: impl Peripheral<P = impl D7Pin<T>> + 'd,
        d8: impl Peripheral<P = impl D8Pin<T>> + 'd,
        d9: impl Peripheral<P = impl D9Pin<T>> + 'd,
        v_sync: impl Peripheral<P = impl VSyncPin<T>> + 'd,
        h_sync: impl Peripheral<P = impl HSyncPin<T>> + 'd,
        pixclk: impl Peripheral<P = impl PixClkPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, dma);
        config_pins!(d0, d1, d2, d3, d4, d5, d6, d7, d8, d9);
        config_pins!(v_sync, h_sync, pixclk);

        Self::new_inner(peri, dma, config, false, 0b01)
    }

    pub fn new_12bit(
        peri: impl Peripheral<P = T> + 'd,
        dma: impl Peripheral<P = Dma> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        d4: impl Peripheral<P = impl D4Pin<T>> + 'd,
        d5: impl Peripheral<P = impl D5Pin<T>> + 'd,
        d6: impl Peripheral<P = impl D6Pin<T>> + 'd,
        d7: impl Peripheral<P = impl D7Pin<T>> + 'd,
        d8: impl Peripheral<P = impl D8Pin<T>> + 'd,
        d9: impl Peripheral<P = impl D9Pin<T>> + 'd,
        d10: impl Peripheral<P = impl D10Pin<T>> + 'd,
        d11: impl Peripheral<P = impl D11Pin<T>> + 'd,
        v_sync: impl Peripheral<P = impl VSyncPin<T>> + 'd,
        h_sync: impl Peripheral<P = impl HSyncPin<T>> + 'd,
        pixclk: impl Peripheral<P = impl PixClkPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, dma);
        config_pins!(d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11);
        config_pins!(v_sync, h_sync, pixclk);

        Self::new_inner(peri, dma, config, false, 0b10)
    }

    pub fn new_14bit(
        peri: impl Peripheral<P = T> + 'd,
        dma: impl Peripheral<P = Dma> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        d4: impl Peripheral<P = impl D4Pin<T>> + 'd,
        d5: impl Peripheral<P = impl D5Pin<T>> + 'd,
        d6: impl Peripheral<P = impl D6Pin<T>> + 'd,
        d7: impl Peripheral<P = impl D7Pin<T>> + 'd,
        d8: impl Peripheral<P = impl D8Pin<T>> + 'd,
        d9: impl Peripheral<P = impl D9Pin<T>> + 'd,
        d10: impl Peripheral<P = impl D10Pin<T>> + 'd,
        d11: impl Peripheral<P = impl D11Pin<T>> + 'd,
        d12: impl Peripheral<P = impl D12Pin<T>> + 'd,
        d13: impl Peripheral<P = impl D13Pin<T>> + 'd,
        v_sync: impl Peripheral<P = impl VSyncPin<T>> + 'd,
        h_sync: impl Peripheral<P = impl HSyncPin<T>> + 'd,
        pixclk: impl Peripheral<P = impl PixClkPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, dma);
        config_pins!(d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12, d13);
        config_pins!(v_sync, h_sync, pixclk);

        Self::new_inner(peri, dma, config, false, 0b11)
    }

    pub fn new_es_8bit(
        peri: impl Peripheral<P = T> + 'd,
        dma: impl Peripheral<P = Dma> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        d4: impl Peripheral<P = impl D4Pin<T>> + 'd,
        d5: impl Peripheral<P = impl D5Pin<T>> + 'd,
        d6: impl Peripheral<P = impl D6Pin<T>> + 'd,
        d7: impl Peripheral<P = impl D7Pin<T>> + 'd,
        pixclk: impl Peripheral<P = impl PixClkPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, dma);
        config_pins!(d0, d1, d2, d3, d4, d5, d6, d7);
        config_pins!(pixclk);

        Self::new_inner(peri, dma, config, true, 0b00)
    }

    pub fn new_es_10bit(
        peri: impl Peripheral<P = T> + 'd,
        dma: impl Peripheral<P = Dma> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        d4: impl Peripheral<P = impl D4Pin<T>> + 'd,
        d5: impl Peripheral<P = impl D5Pin<T>> + 'd,
        d6: impl Peripheral<P = impl D6Pin<T>> + 'd,
        d7: impl Peripheral<P = impl D7Pin<T>> + 'd,
        d8: impl Peripheral<P = impl D8Pin<T>> + 'd,
        d9: impl Peripheral<P = impl D9Pin<T>> + 'd,
        pixclk: impl Peripheral<P = impl PixClkPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, dma);
        config_pins!(d0, d1, d2, d3, d4, d5, d6, d7, d8, d9);
        config_pins!(pixclk);

        Self::new_inner(peri, dma, config, true, 0b01)
    }

    pub fn new_es_12bit(
        peri: impl Peripheral<P = T> + 'd,
        dma: impl Peripheral<P = Dma> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        d4: impl Peripheral<P = impl D4Pin<T>> + 'd,
        d5: impl Peripheral<P = impl D5Pin<T>> + 'd,
        d6: impl Peripheral<P = impl D6Pin<T>> + 'd,
        d7: impl Peripheral<P = impl D7Pin<T>> + 'd,
        d8: impl Peripheral<P = impl D8Pin<T>> + 'd,
        d9: impl Peripheral<P = impl D9Pin<T>> + 'd,
        d10: impl Peripheral<P = impl D10Pin<T>> + 'd,
        d11: impl Peripheral<P = impl D11Pin<T>> + 'd,
        pixclk: impl Peripheral<P = impl PixClkPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, dma);
        config_pins!(d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11);
        config_pins!(pixclk);

        Self::new_inner(peri, dma, config, true, 0b10)
    }

    pub fn new_es_14bit(
        peri: impl Peripheral<P = T> + 'd,
        dma: impl Peripheral<P = Dma> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        d4: impl Peripheral<P = impl D4Pin<T>> + 'd,
        d5: impl Peripheral<P = impl D5Pin<T>> + 'd,
        d6: impl Peripheral<P = impl D6Pin<T>> + 'd,
        d7: impl Peripheral<P = impl D7Pin<T>> + 'd,
        d8: impl Peripheral<P = impl D8Pin<T>> + 'd,
        d9: impl Peripheral<P = impl D9Pin<T>> + 'd,
        d10: impl Peripheral<P = impl D10Pin<T>> + 'd,
        d11: impl Peripheral<P = impl D11Pin<T>> + 'd,
        d12: impl Peripheral<P = impl D12Pin<T>> + 'd,
        d13: impl Peripheral<P = impl D13Pin<T>> + 'd,
        pixclk: impl Peripheral<P = impl PixClkPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(peri, dma);
        config_pins!(d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12, d13);
        config_pins!(pixclk);

        Self::new_inner(peri, dma, config, true, 0b11)
    }

    fn new_inner(
        peri: PeripheralRef<'d, T>,
        dma: PeripheralRef<'d, Dma>,
        config: Config,
        use_embedded_synchronization: bool,
        edm: u8,
    ) -> Self {
        T::reset();
        T::enable();

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
    ///
    /// Note that when `buffer.len() > 0xffff` the capture future requires some real-time guarantees to be upheld
    /// (must be polled fast enough so the buffers get switched before data is overwritten).
    /// It is therefore recommended that it is run on higher priority executor.
    pub async fn capture(&mut self, buffer: &mut [u32]) -> Result<(), Error> {
        if buffer.len() <= 0xffff {
            return self.capture_small(buffer).await;
        } else {
            return self.capture_giant(buffer).await;
        }
    }

    async fn capture_small(&mut self, buffer: &mut [u32]) -> Result<(), Error> {
        let r = self.inner.regs();
        let src = r.dr().as_ptr() as *mut u32;
        let request = self.dma.request();
        let dma_read = unsafe { Transfer::new_read(&mut self.dma, request, src, buffer, Default::default()) };

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

    #[cfg(not(dma))]
    async fn capture_giant(&mut self, _buffer: &mut [u32]) -> Result<(), Error> {
        panic!("capturing to buffers larger than 0xffff is only supported on DMA for now, not on BDMA or GPDMA.");
    }

    #[cfg(dma)]
    async fn capture_giant(&mut self, buffer: &mut [u32]) -> Result<(), Error> {
        use crate::dma::TransferOptions;

        let data_len = buffer.len();
        let chunk_estimate = data_len / 0xffff;

        let mut chunks = chunk_estimate + 1;
        while data_len % chunks != 0 {
            chunks += 1;
        }

        let chunk_size = data_len / chunks;

        let mut remaining_chunks = chunks - 2;

        let mut m0ar = buffer.as_mut_ptr();
        let mut m1ar = unsafe { buffer.as_mut_ptr().add(chunk_size) };

        let channel = &mut self.dma;
        let request = channel.request();

        let r = self.inner.regs();
        let src = r.dr().as_ptr() as *mut u32;

        let mut transfer = unsafe {
            crate::dma::DoubleBuffered::new_read(
                &mut self.dma,
                request,
                src,
                m0ar,
                m1ar,
                chunk_size,
                TransferOptions::default(),
            )
        };

        let mut last_chunk_set_for_transfer = false;
        let mut buffer0_last_accessible = false;
        let dma_result = poll_fn(|cx| {
            transfer.set_waker(cx.waker());

            let buffer0_currently_accessible = transfer.is_buffer0_accessible();

            // check if the accessible buffer changed since last poll
            if buffer0_last_accessible == buffer0_currently_accessible {
                return Poll::Pending;
            }
            buffer0_last_accessible = !buffer0_last_accessible;

            if remaining_chunks != 0 {
                if remaining_chunks % 2 == 0 && buffer0_currently_accessible {
                    m0ar = unsafe { m0ar.add(2 * chunk_size) };
                    unsafe { transfer.set_buffer0(m0ar) }
                    remaining_chunks -= 1;
                } else if !buffer0_currently_accessible {
                    m1ar = unsafe { m1ar.add(2 * chunk_size) };
                    unsafe { transfer.set_buffer1(m1ar) };
                    remaining_chunks -= 1;
                }
            } else {
                if buffer0_currently_accessible {
                    unsafe { transfer.set_buffer0(buffer.as_mut_ptr()) }
                } else {
                    unsafe { transfer.set_buffer1(buffer.as_mut_ptr()) }
                }
                if last_chunk_set_for_transfer {
                    transfer.request_stop();
                    return Poll::Ready(());
                }
                last_chunk_set_for_transfer = true;
            }
            Poll::Pending
        });

        Self::clear_interrupt_flags();
        Self::enable_irqs();

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

        Self::toggle(true);

        let (_, result) = embassy_futures::join::join(dma_result, result).await;

        Self::toggle(false);

        result
    }
}

mod sealed {
    pub trait Instance: crate::rcc::RccPeripheral {
        fn regs(&self) -> crate::pac::dcmi::Dcmi;
    }
}

pub trait Instance: sealed::Instance + 'static {
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
        impl sealed::Instance for crate::peripherals::$inst {
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
