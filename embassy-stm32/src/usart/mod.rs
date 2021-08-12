#![macro_use]

#[cfg_attr(usart_v1, path = "v1.rs")]
#[cfg_attr(usart_v2, path = "v2.rs")]
mod _version;
use crate::{dma, peripherals};
pub use _version::*;

use crate::gpio::Pin;
use crate::rcc::RccPeripheral;
use embassy::interrupt::Interrupt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DataBits {
    DataBits8,
    DataBits9,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Parity {
    ParityNone,
    ParityEven,
    ParityOdd,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StopBits {
    #[doc = "1 stop bit"]
    STOP1,
    #[doc = "0.5 stop bits"]
    STOP0P5,
    #[doc = "2 stop bits"]
    STOP2,
    #[doc = "1.5 stop bits"]
    STOP1P5,
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Config {
    pub baudrate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            baudrate: 115200,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::STOP1,
            parity: Parity::ParityNone,
        }
    }
}

/// Serial error
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Framing error
    Framing,
    /// Noise error
    Noise,
    /// RX buffer overrun
    Overrun,
    /// Parity check error
    Parity,
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        fn regs(&self) -> crate::pac::usart::Usart;
    }
    pub trait RxPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }
    pub trait TxPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }
    pub trait CtsPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }
    pub trait RtsPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }
    pub trait CkPin<T: Instance>: Pin {
        fn af_num(&self) -> u8;
    }

    pub trait RxDma<T: Instance> {
        fn request(&self) -> dma::Request;
    }

    pub trait TxDma<T: Instance> {
        fn request(&self) -> dma::Request;
    }
}

pub trait Instance: sealed::Instance + RccPeripheral {
    type Interrupt: Interrupt;
}
pub trait RxPin<T: Instance>: sealed::RxPin<T> {}
pub trait TxPin<T: Instance>: sealed::TxPin<T> {}
pub trait CtsPin<T: Instance>: sealed::CtsPin<T> {}
pub trait RtsPin<T: Instance>: sealed::RtsPin<T> {}
pub trait CkPin<T: Instance>: sealed::CkPin<T> {}
pub trait RxDma<T: Instance>: sealed::RxDma<T> + dma::Channel {}
pub trait TxDma<T: Instance>: sealed::TxDma<T> + dma::Channel {}

crate::pac::interrupts!(
    ($inst:ident, usart, $block:ident, $signal_name:ident, $irq:ident) => {
        impl sealed::Instance for peripherals::$inst {
            fn regs(&self) -> crate::pac::usart::Usart {
                crate::pac::$inst
            }
        }

        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::$irq;
        }

    };
);

macro_rules! impl_pin {
    ($inst:ident, $pin:ident, $signal:ident, $af:expr) => {
        impl sealed::$signal<peripherals::$inst> for peripherals::$pin {
            fn af_num(&self) -> u8 {
                $af
            }
        }

        impl $signal<peripherals::$inst> for peripherals::$pin {}
    };
}

crate::pac::peripheral_pins!(

    // USART
    ($inst:ident, usart, USART, $pin:ident, TX, $af:expr) => {
        impl_pin!($inst, $pin, TxPin, $af);
    };
    ($inst:ident, usart, USART, $pin:ident, RX, $af:expr) => {
        impl_pin!($inst, $pin, RxPin, $af);
    };
    ($inst:ident, usart, USART, $pin:ident, CTS, $af:expr) => {
        impl_pin!($inst, $pin, CtsPin, $af);
    };
    ($inst:ident, usart, USART, $pin:ident, RTS, $af:expr) => {
        impl_pin!($inst, $pin, RtsPin, $af);
    };
    ($inst:ident, usart, USART, $pin:ident, CK, $af:expr) => {
        impl_pin!($inst, $pin, CkPin, $af);
    };

    // UART
    ($inst:ident, uart, UART, $pin:ident, TX, $af:expr) => {
        impl_pin!($inst, $pin, TxPin, $af);
    };
    ($inst:ident, uart, UART, $pin:ident, RX, $af:expr) => {
        impl_pin!($inst, $pin, RxPin, $af);
    };
    ($inst:ident, uart, UART, $pin:ident, CTS, $af:expr) => {
        impl_pin!($inst, $pin, CtsPin, $af);
    };
    ($inst:ident, uart, UART, $pin:ident, RTS, $af:expr) => {
        impl_pin!($inst, $pin, RtsPin, $af);
    };
    ($inst:ident, uart, UART, $pin:ident, CK, $af:expr) => {
        impl_pin!($inst, $pin, CkPin, $af);
    };
);

macro_rules! impl_dma {
    ($inst:ident, {dmamux: $dmamux:ident}, $signal:ident, $request:expr) => {
        impl<T> sealed::$signal<peripherals::$inst> for T
        where
            T: crate::dma::MuxChannel<Mux = crate::dma::$dmamux>,
        {
            fn request(&self) -> dma::Request {
                $request
            }
        }

        impl<T> $signal<peripherals::$inst> for T where
            T: crate::dma::MuxChannel<Mux = crate::dma::$dmamux>
        {
        }
    };
    ($inst:ident, {channel: $channel:ident}, $signal:ident, $request:expr) => {
        impl sealed::$signal<peripherals::$inst> for peripherals::$channel {
            fn request(&self) -> dma::Request {
                $request
            }
        }

        impl $signal<peripherals::$inst> for peripherals::$channel {}
    };
}

crate::pac::peripheral_dma_channels! {
    ($peri:ident, usart, $kind:ident, RX, $channel:tt, $request:expr) => {
        impl_dma!($peri, $channel, RxDma, $request);
    };
    ($peri:ident, usart, $kind:ident, TX, $channel:tt, $request:expr) => {
        impl_dma!($peri, $channel, TxDma, $request);
    };
    ($peri:ident, uart, $kind:ident, RX, $channel:tt, $request:expr) => {
        impl_dma!($peri, $channel, RxDma, $request);
    };
    ($peri:ident, uart, $kind:ident, TX, $channel:tt, $request:expr) => {
        impl_dma!($peri, $channel, TxDma, $request);
    };
}
