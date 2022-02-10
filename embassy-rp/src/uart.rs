use core::marker::PhantomData;

use embassy::util::Unborrow;
use embassy_hal_common::unborrow;
use gpio::Pin;

use crate::{gpio, pac, peripherals};

#[non_exhaustive]
pub struct Config {
    pub baudrate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            baudrate: 115200,
            data_bits: 8,
            stop_bits: 1,
        }
    }
}

pub struct Uart<'d, T: Instance> {
    inner: T,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Uart<'d, T> {
    pub fn new(
        inner: impl Unborrow<Target = T> + 'd,
        tx: impl Unborrow<Target = impl TxPin<T>> + 'd,
        rx: impl Unborrow<Target = impl RxPin<T>> + 'd,
        cts: impl Unborrow<Target = impl CtsPin<T>> + 'd,
        rts: impl Unborrow<Target = impl RtsPin<T>> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(inner, tx, rx, cts, rts);

        unsafe {
            let p = inner.regs();

            let clk_base = crate::clocks::clk_peri_freq();

            let baud_rate_div = (8 * clk_base) / config.baudrate;
            let mut baud_ibrd = baud_rate_div >> 7;
            let mut baud_fbrd = ((baud_rate_div & 0x7f) + 1) / 2;

            if baud_ibrd == 0 {
                baud_ibrd = 1;
                baud_fbrd = 0;
            } else if baud_ibrd >= 65535 {
                baud_ibrd = 65535;
                baud_fbrd = 0;
            }

            // Load PL011's baud divisor registers
            p.uartibrd()
                .write_value(pac::uart::regs::Uartibrd(baud_ibrd));
            p.uartfbrd()
                .write_value(pac::uart::regs::Uartfbrd(baud_fbrd));

            p.uartlcr_h().write(|w| {
                w.set_wlen(config.data_bits - 5);
                w.set_stp2(config.stop_bits == 2);
                w.set_pen(false);
                w.set_eps(false);
                w.set_fen(true);
            });

            p.uartcr().write(|w| {
                w.set_uarten(true);
                w.set_rxe(true);
                w.set_txe(true);
            });

            tx.io().ctrl().write(|w| w.set_funcsel(2));
            rx.io().ctrl().write(|w| w.set_funcsel(2));
            cts.io().ctrl().write(|w| w.set_funcsel(2));
            rts.io().ctrl().write(|w| w.set_funcsel(2));
        }
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    pub fn send(&mut self, data: &[u8]) {
        unsafe {
            let p = self.inner.regs();

            for &byte in data {
                if !p.uartfr().read().txff() {
                    p.uartdr().write(|w| w.set_data(byte));
                }
            }
        }
    }
}

mod sealed {
    use super::*;

    pub trait Instance {
        fn regs(&self) -> pac::uart::Uart;
    }
    pub trait TxPin<T: Instance> {}
    pub trait RxPin<T: Instance> {}
    pub trait CtsPin<T: Instance> {}
    pub trait RtsPin<T: Instance> {}
}

pub trait Instance: sealed::Instance {}

macro_rules! impl_instance {
    ($type:ident, $irq:ident) => {
        impl sealed::Instance for peripherals::$type {
            fn regs(&self) -> pac::uart::Uart {
                pac::$type
            }
        }
        impl Instance for peripherals::$type {}
    };
}

impl_instance!(UART0, UART0);
impl_instance!(UART1, UART1);

pub trait TxPin<T: Instance>: sealed::TxPin<T> + Pin {}
pub trait RxPin<T: Instance>: sealed::RxPin<T> + Pin {}
pub trait CtsPin<T: Instance>: sealed::CtsPin<T> + Pin {}
pub trait RtsPin<T: Instance>: sealed::RtsPin<T> + Pin {}

macro_rules! impl_pin {
    ($pin:ident, $instance:ident, $function:ident) => {
        impl sealed::$function<peripherals::$instance> for peripherals::$pin {}
        impl $function<peripherals::$instance> for peripherals::$pin {}
    };
}

impl_pin!(PIN_0, UART0, TxPin);
impl_pin!(PIN_1, UART0, RxPin);
impl_pin!(PIN_2, UART0, CtsPin);
impl_pin!(PIN_3, UART0, RtsPin);
impl_pin!(PIN_4, UART1, TxPin);
impl_pin!(PIN_5, UART1, RxPin);
impl_pin!(PIN_6, UART1, CtsPin);
impl_pin!(PIN_7, UART1, RtsPin);
impl_pin!(PIN_8, UART1, TxPin);
impl_pin!(PIN_9, UART1, RxPin);
impl_pin!(PIN_10, UART1, CtsPin);
impl_pin!(PIN_11, UART1, RtsPin);
impl_pin!(PIN_12, UART0, TxPin);
impl_pin!(PIN_13, UART0, RxPin);
impl_pin!(PIN_14, UART0, CtsPin);
impl_pin!(PIN_15, UART0, RtsPin);
impl_pin!(PIN_16, UART0, TxPin);
impl_pin!(PIN_17, UART0, RxPin);
impl_pin!(PIN_18, UART0, CtsPin);
impl_pin!(PIN_19, UART0, RtsPin);
impl_pin!(PIN_20, UART1, TxPin);
impl_pin!(PIN_21, UART1, RxPin);
impl_pin!(PIN_22, UART1, CtsPin);
impl_pin!(PIN_23, UART1, RtsPin);
impl_pin!(PIN_24, UART1, TxPin);
impl_pin!(PIN_25, UART1, RxPin);
impl_pin!(PIN_26, UART1, CtsPin);
impl_pin!(PIN_27, UART1, RtsPin);
impl_pin!(PIN_28, UART0, TxPin);
impl_pin!(PIN_29, UART0, RxPin);
