use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;
use embedded_hal_02::adc::{Channel, OneShot};

use crate::gpio::Pin;
use crate::interrupt::typelevel::Binding;
use crate::interrupt::InterruptExt;
use crate::peripherals::ADC;
use crate::{interrupt, pac, peripherals, Peripheral};
static WAKER: AtomicWaker = AtomicWaker::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    // No errors for now
}

#[non_exhaustive]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}
pub struct Adc<'d> {
    phantom: PhantomData<&'d ADC>,
}

impl<'d> Adc<'d> {
    #[inline]
    fn regs() -> pac::adc::Adc {
        pac::ADC
    }

    #[inline]
    fn reset() -> pac::resets::regs::Peripherals {
        let mut ret = pac::resets::regs::Peripherals::default();
        ret.set_adc(true);
        ret
    }

    pub fn new(
        _inner: impl Peripheral<P = ADC> + 'd,
        _irq: impl Binding<interrupt::typelevel::ADC_IRQ_FIFO, InterruptHandler>,
        _config: Config,
    ) -> Self {
        unsafe {
            let reset = Self::reset();
            crate::reset::reset(reset);
            crate::reset::unreset_wait(reset);
            let r = Self::regs();
            // Enable ADC
            r.cs().write(|w| w.set_en(true));
            // Wait for ADC ready
            while !r.cs().read().ready() {}
        }

        // Setup IRQ
        interrupt::ADC_IRQ_FIFO.unpend();
        unsafe { interrupt::ADC_IRQ_FIFO.enable() };

        Self { phantom: PhantomData }
    }

    async fn wait_for_ready() {
        let r = Self::regs();
        unsafe {
            r.inte().write(|w| w.set_fifo(true));
            compiler_fence(Ordering::SeqCst);
            poll_fn(|cx| {
                WAKER.register(cx.waker());
                if r.cs().read().ready() {
                    return Poll::Ready(());
                }
                Poll::Pending
            })
            .await;
        }
    }

    pub async fn read<PIN: Channel<Adc<'d>, ID = u8> + Pin>(&mut self, pin: &mut PIN) -> u16 {
        let r = Self::regs();
        unsafe {
            // disable pull-down and pull-up resistors
            // pull-down resistors are enabled by default
            pin.pad_ctrl().modify(|w| {
                w.set_ie(true);
                let (pu, pd) = (false, false);
                w.set_pue(pu);
                w.set_pde(pd);
            });
            r.cs().modify(|w| {
                w.set_ainsel(PIN::channel());
                w.set_start_once(true)
            });
            Self::wait_for_ready().await;
            r.result().read().result().into()
        }
    }

    pub async fn read_temperature(&mut self) -> u16 {
        let r = Self::regs();
        unsafe {
            r.cs().modify(|w| w.set_ts_en(true));
            if !r.cs().read().ready() {
                Self::wait_for_ready().await;
            }
            r.cs().modify(|w| {
                w.set_ainsel(4);
                w.set_start_once(true)
            });
            Self::wait_for_ready().await;
            r.result().read().result().into()
        }
    }

    pub fn blocking_read<PIN: Channel<Adc<'d>, ID = u8>>(&mut self, _pin: &mut PIN) -> u16 {
        let r = Self::regs();
        unsafe {
            r.cs().modify(|w| {
                w.set_ainsel(PIN::channel());
                w.set_start_once(true)
            });
            while !r.cs().read().ready() {}
            r.result().read().result().into()
        }
    }

    pub fn blocking_read_temperature(&mut self) -> u16 {
        let r = Self::regs();
        unsafe {
            r.cs().modify(|w| w.set_ts_en(true));
            while !r.cs().read().ready() {}
            r.cs().modify(|w| {
                w.set_ainsel(4);
                w.set_start_once(true)
            });
            while !r.cs().read().ready() {}
            r.result().read().result().into()
        }
    }
}

macro_rules! impl_pin {
    ($pin:ident, $channel:expr) => {
        impl Channel<Adc<'static>> for peripherals::$pin {
            type ID = u8;
            fn channel() -> u8 {
                $channel
            }
        }
    };
}

pub struct InterruptHandler {
    _empty: (),
}

impl interrupt::typelevel::Handler<interrupt::typelevel::ADC_IRQ_FIFO> for InterruptHandler {
    unsafe fn on_interrupt() {
        let r = Adc::regs();
        r.inte().write(|w| w.set_fifo(false));
        WAKER.wake();
    }
}

impl_pin!(PIN_26, 0);
impl_pin!(PIN_27, 1);
impl_pin!(PIN_28, 2);
impl_pin!(PIN_29, 3);

impl<WORD, PIN> OneShot<Adc<'static>, WORD, PIN> for Adc<'static>
where
    WORD: From<u16>,
    PIN: Channel<Adc<'static>, ID = u8>,
{
    type Error = ();
    fn read(&mut self, pin: &mut PIN) -> nb::Result<WORD, Self::Error> {
        Ok(self.blocking_read(pin).into())
    }
}
