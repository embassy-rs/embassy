use core::marker::PhantomData;
use core::task::Poll;

use crate::gpio::sealed::Pin as __GpioPin;
use crate::gpio::Pin as GpioPin;
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::util::Unborrow;
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::unborrow;
use futures::future::poll_fn;

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

pub struct Dcmi<'d, T: Instance, Dma: FrameDma> {
    inner: T,
    dma: Dma,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T, Dma> Dcmi<'d, T, Dma>
where
    T: Instance,
    Dma: FrameDma,
{
    pub fn new(
        peri: impl Unborrow<Target = T> + 'd,
        dma: impl Unborrow<Target = Dma> + 'd,
        vsync_level: VSyncDataInvalidLevel,
        hsync_level: HSyncDataInvalidLevel,
        pixclk_polarity: PixelClockPolarity,
        use_embedded_synchronization: bool,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
        d0: impl Unborrow<Target = impl D0Pin> + 'd,
        d1: impl Unborrow<Target = impl D1Pin> + 'd,
        d2: impl Unborrow<Target = impl D2Pin> + 'd,
        d3: impl Unborrow<Target = impl D3Pin> + 'd,
        d4: impl Unborrow<Target = impl D4Pin> + 'd,
        d5: impl Unborrow<Target = impl D5Pin> + 'd,
        d6: impl Unborrow<Target = impl D6Pin> + 'd,
        d7: impl Unborrow<Target = impl D7Pin> + 'd,
        d8: impl Unborrow<Target = impl D8Pin> + 'd,
        d9: impl Unborrow<Target = impl D9Pin> + 'd,
        d10: impl Unborrow<Target = impl D10Pin> + 'd,
        d11: impl Unborrow<Target = impl D11Pin> + 'd,
        d12: impl Unborrow<Target = impl D12Pin> + 'd,
        d13: impl Unborrow<Target = impl D13Pin> + 'd,
        v_sync: impl Unborrow<Target = impl VSyncPin> + 'd,
        h_sync: impl Unborrow<Target = impl HSyncPin> + 'd,
        pixclk: impl Unborrow<Target = impl PixClkPin> + 'd,
    ) -> Self {
        T::reset();
        T::enable();

        unborrow!(
            peri, dma, irq, d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12, d13, v_sync,
            h_sync, pixclk
        );

        d0.configure();
        d1.configure();
        d2.configure();
        d3.configure();
        d4.configure();
        d5.configure();
        d6.configure();
        d7.configure();
        d8.configure();
        d9.configure();
        d10.configure();
        d11.configure();
        d12.configure();
        d13.configure();

        v_sync.configure();
        h_sync.configure();
        pixclk.configure();

        let edm = match (
            d8.pin().is_some(),
            d9.pin().is_some(),
            d10.pin().is_some(),
            d11.pin().is_some(),
            d12.pin().is_some(),
            d13.pin().is_some(),
        ) {
            (true, true, true, true, true, true) => 0b11, // 14 bits
            (true, true, true, true, false, false) => 0b10, // 12 bits
            (true, true, false, false, false, false) => 0b01, // 10 bits
            (false, false, false, false, false, false) => 0b00, // 8 bits
            _ => {
                panic!("Invalid pin configuration.");
            }
        };

        unsafe {
            peri.regs().cr().modify(|r| {
                r.set_cm(true); // disable continuous mode (snapshot mode)
                r.set_ess(use_embedded_synchronization);
                r.set_pckpol(pixclk_polarity == PixelClockPolarity::RisingEdge);
                r.set_vspol(vsync_level == VSyncDataInvalidLevel::High);
                r.set_hspol(hsync_level == HSyncDataInvalidLevel::High);
                r.set_fcrc(0x00); // capture every frame
                r.set_edm(edm); // extended data mode
            });
        }

        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        Self {
            inner: peri,
            dma,
            phantom: PhantomData,
        }
    }

    unsafe fn on_interrupt(_: *mut ()) {
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

    unsafe fn toggle(enable: bool) {
        crate::pac::DCMI.cr().modify(|r| {
            r.set_enable(enable);
            r.set_capture(enable);
        })
    }

    fn enable_irqs() {
        unsafe {
            crate::pac::DCMI.ier().modify(|r| {
                r.set_err_ie(true);
                r.set_ovr_ie(true);
                r.set_frame_ie(true);
            });
        }
    }

    fn clear_interrupt_flags() {
        unsafe {
            crate::pac::DCMI.icr().write(|r| {
                r.set_ovr_isc(true);
                r.set_err_isc(true);
                r.set_frame_isc(true);
            })
        }
    }

    /// This method starts the capture and finishes when both the dma transfer and DCMI finish the frame transfer.
    /// The implication is that the input buffer size must be exactly the size of the captured frame.
    pub async fn capture(&mut self, buffer: &mut [u32]) -> Result<(), Error> {
        let channel = &mut self.dma;
        let request = channel.request();

        let r = self.inner.regs();
        let src = r.dr().ptr() as *mut u32;
        let dma_read = crate::dma::read(channel, request, src, buffer);

        Self::clear_interrupt_flags();
        Self::enable_irqs();

        unsafe { Self::toggle(true) };

        let result = poll_fn(|cx| {
            STATE.waker.register(cx.waker());

            let ris = unsafe { crate::pac::DCMI.ris().read() };
            if ris.err_ris() {
                unsafe {
                    crate::pac::DCMI.icr().write(|r| {
                        r.set_err_isc(true);
                    })
                };
                Poll::Ready(Err(Error::PeripheralError))
            } else if ris.ovr_ris() {
                unsafe {
                    crate::pac::DCMI.icr().write(|r| {
                        r.set_ovr_isc(true);
                    })
                };
                Poll::Ready(Err(Error::Overrun))
            } else if ris.frame_ris() {
                unsafe {
                    crate::pac::DCMI.icr().write(|r| {
                        r.set_frame_isc(true);
                    })
                };
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        });

        let (_, result) = futures::future::join(dma_read, result).await;

        unsafe { Self::toggle(false) };

        result
    }
}

mod sealed {
    use super::*;
    use crate::rcc::RccPeripheral;

    pub trait Instance: RccPeripheral {
        fn regs(&self) -> crate::pac::dcmi::Dcmi;
    }

    pub trait FrameDma {
        fn request(&self) -> crate::dma::Request;
    }

    macro_rules! pin {
        ($name:ident) => {
            pub trait $name: GpioPin {
                fn configure(&mut self);
            }
        };
    }

    macro_rules! optional_pin {
        ($name:ident) => {
            pub trait $name: crate::gpio::OptionalPin {
                fn configure(&mut self);
            }
        };
    }

    pin!(D0Pin);
    pin!(D1Pin);
    pin!(D2Pin);
    pin!(D3Pin);
    pin!(D4Pin);
    pin!(D5Pin);
    pin!(D6Pin);
    pin!(D7Pin);
    optional_pin!(D8Pin);
    optional_pin!(D9Pin);
    optional_pin!(D10Pin);
    optional_pin!(D11Pin);
    optional_pin!(D12Pin);
    optional_pin!(D13Pin);

    optional_pin!(HSyncPin);
    optional_pin!(VSyncPin);
    pin!(PixClkPin);
}

pub trait Instance: sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

pub trait FrameDma: sealed::FrameDma + crate::dma::Channel {}

macro_rules! pin {
    ($name:ident) => {
        pub trait $name: sealed::$name + 'static {}
    };
}

pin!(D0Pin);
pin!(D1Pin);
pin!(D2Pin);
pin!(D3Pin);
pin!(D4Pin);
pin!(D5Pin);
pin!(D6Pin);
pin!(D7Pin);
pin!(D8Pin);
pin!(D9Pin);
pin!(D10Pin);
pin!(D11Pin);
pin!(D12Pin);
pin!(D13Pin);

pin!(HSyncPin);
pin!(VSyncPin);
pin!(PixClkPin);

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
            type Interrupt = crate::interrupt::$irq;
        }
    };
}

crate::pac::interrupts! {
    ($inst:ident, dcmi, $block:ident, GLOBAL, $irq:ident) => {
        impl_peripheral!($inst, $irq);
    };
}

// allow unused as U5 sources do not contain interrupt nor dma data
#[allow(unused)]
macro_rules! impl_dma {
    ($inst:ident, {dmamux: $dmamux:ident}, $signal:ident, $request:expr) => {
        impl<T> sealed::$signal for T
        where
            T: crate::dma::MuxChannel<Mux = crate::dma::$dmamux>,
        {
            fn request(&self) -> crate::dma::Request {
                $request
            }
        }

        impl<T> $signal for T where T: crate::dma::MuxChannel<Mux = crate::dma::$dmamux> {}
    };
    ($inst:ident, {channel: $channel:ident}, $signal:ident, $request:expr) => {
        impl sealed::$signal for crate::peripherals::$channel {
            fn request(&self) -> crate::dma::Request {
                $request
            }
        }

        impl $signal for crate::peripherals::$channel {}
    };
}

crate::pac::peripheral_dma_channels! {
    ($peri:ident, dcmi, $kind:ident, PSSI, $channel:tt, $request:expr) => {
        impl_dma!($peri, $channel, FrameDma, $request);
    };
    ($peri:ident, dcmi, $kind:ident, DCMI, $channel:tt, $request:expr) => {
        impl_dma!($peri, $channel, FrameDma, $request);
    };
}

macro_rules! impl_pin {
    ($pin:ident, $signal:ident, $af:expr) => {
        impl sealed::$signal for crate::peripherals::$pin {
            fn configure(&mut self) {
                // NOTE(unsafe) Exclusive access to the registers
                critical_section::with(|_| unsafe {
                    self.set_as_af($af, crate::gpio::sealed::AFType::Input);
                    self.block().ospeedr().modify(|w| {
                        w.set_ospeedr(
                            self.pin() as usize,
                            crate::pac::gpio::vals::Ospeedr::VERYHIGHSPEED,
                        )
                    });
                })
            }
        }

        impl $signal for crate::peripherals::$pin {}
    };
}

macro_rules! impl_no_pin {
    ($signal:ident) => {
        impl sealed::$signal for crate::gpio::NoPin {
            fn configure(&mut self) {}
        }
        impl $signal for crate::gpio::NoPin {}
    };
}

impl_no_pin!(D8Pin);
impl_no_pin!(D9Pin);
impl_no_pin!(D10Pin);
impl_no_pin!(D11Pin);
impl_no_pin!(D12Pin);
impl_no_pin!(D13Pin);
impl_no_pin!(HSyncPin);
impl_no_pin!(VSyncPin);

crate::pac::peripheral_pins!(
    ($inst:ident, dcmi, DCMI, $pin:ident, D0, $af:expr) => {
        impl_pin!($pin, D0Pin, $af);
    };
    ($inst:ident, dcmi, DCMI, $pin:ident, D1, $af:expr) => {
        impl_pin!($pin, D1Pin, $af);
    };
    ($inst:ident, dcmi, DCMI, $pin:ident, D2, $af:expr) => {
        impl_pin!($pin, D2Pin, $af);
    };
    ($inst:ident, dcmi, DCMI, $pin:ident, D3, $af:expr) => {
        impl_pin!($pin, D3Pin, $af);
    };
    ($inst:ident, dcmi, DCMI, $pin:ident, D4, $af:expr) => {
        impl_pin!($pin, D4Pin, $af);
    };
    ($inst:ident, dcmi, DCMI, $pin:ident, D5, $af:expr) => {
        impl_pin!($pin, D5Pin, $af);
    };
    ($inst:ident, dcmi, DCMI, $pin:ident, D6, $af:expr) => {
        impl_pin!($pin, D6Pin, $af);
    };
    ($inst:ident, dcmi, DCMI, $pin:ident, D7, $af:expr) => {
        impl_pin!($pin, D7Pin, $af);
    };
    ($inst:ident, dcmi, DCMI, $pin:ident, D8, $af:expr) => {
        impl_pin!($pin, D8Pin, $af);
    };
    ($inst:ident, dcmi, DCMI, $pin:ident, D9, $af:expr) => {
        impl_pin!($pin, D9Pin, $af);
    };
    ($inst:ident, dcmi, DCMI, $pin:ident, D10, $af:expr) => {
        impl_pin!($pin, D10Pin, $af);
    };
    ($inst:ident, dcmi, DCMI, $pin:ident, D11, $af:expr) => {
        impl_pin!($pin, D11Pin, $af);
    };
    ($inst:ident, dcmi, DCMI, $pin:ident, D12, $af:expr) => {
        impl_pin!($pin, D12Pin, $af);
    };
    ($inst:ident, dcmi, DCMI, $pin:ident, D13, $af:expr) => {
        impl_pin!($pin, D13Pin, $af);
    };
    ($inst:ident, dcmi, DCMI, $pin:ident, HSYNC, $af:expr) => {
        impl_pin!($pin, HSyncPin, $af);
    };
    ($inst:ident, dcmi, DCMI, $pin:ident, VSYNC, $af:expr) => {
        impl_pin!($pin, VSyncPin, $af);
    };
    ($inst:ident, dcmi, DCMI, $pin:ident, PIXCLK, $af:expr) => {
        impl_pin!($pin, PixClkPin, $af);
    };
);
