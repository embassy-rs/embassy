mod i2c;
mod i2c_slave;

use core::marker::PhantomData;

use embassy_sync::waitqueue::AtomicWaker;
pub use i2c::{Config, I2c};
pub use i2c_slave::{Command, I2cSlave, ReadStatus, SlaveConfig};

use crate::{interrupt, pac, peripherals};

const FIFO_SIZE: u8 = 16;

/// I2C error abort reason
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AbortReason {
    /// A bus operation was not acknowledged, e.g. due to the addressed device
    /// not being available on the bus or the device not being ready to process
    /// requests at the moment
    NoAcknowledge,
    /// The arbitration was lost, e.g. electrical problems with the clock signal
    ArbitrationLoss,
    /// Transmit ended with data still in fifo
    TxNotEmpty(u16),
    Other(u32),
}

/// I2C error
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// I2C abort with error
    Abort(AbortReason),
    /// User passed in a read buffer that was 0 length
    InvalidReadBufferLength,
    /// User passed in a write buffer that was 0 length
    InvalidWriteBufferLength,
    /// Target i2c address is out of range
    AddressOutOfRange(u16),
    /// Target i2c address is reserved
    AddressReserved(u16),
}

pub struct InterruptHandler<T: Instance> {
    _uart: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    // Mask interrupts and wake any task waiting for this interrupt
    unsafe fn on_interrupt() {
        let i2c = T::regs();
        i2c.ic_intr_mask().write_value(pac::i2c::regs::IcIntrMask::default());

        T::waker().wake();
    }
}

fn i2c_reserved_addr(addr: u16) -> bool {
    ((addr & 0x78) == 0 || (addr & 0x78) == 0x78) && addr != 0
}

mod sealed {
    use embassy_sync::waitqueue::AtomicWaker;

    use crate::interrupt;

    pub trait Instance {
        const TX_DREQ: u8;
        const RX_DREQ: u8;

        type Interrupt: interrupt::typelevel::Interrupt;

        fn regs() -> crate::pac::i2c::I2c;
        fn reset() -> crate::pac::resets::regs::Peripherals;
        fn waker() -> &'static AtomicWaker;
    }

    pub trait Mode {}

    pub trait SdaPin<T: Instance> {}
    pub trait SclPin<T: Instance> {}
}

pub trait Mode: sealed::Mode {}

macro_rules! impl_mode {
    ($name:ident) => {
        impl sealed::Mode for $name {}
        impl Mode for $name {}
    };
}

pub struct Blocking;
pub struct Async;

impl_mode!(Blocking);
impl_mode!(Async);

pub trait Instance: sealed::Instance {}

macro_rules! impl_instance {
    ($type:ident, $irq:ident, $reset:ident, $tx_dreq:expr, $rx_dreq:expr) => {
        impl sealed::Instance for peripherals::$type {
            const TX_DREQ: u8 = $tx_dreq;
            const RX_DREQ: u8 = $rx_dreq;

            type Interrupt = crate::interrupt::typelevel::$irq;

            #[inline]
            fn regs() -> pac::i2c::I2c {
                pac::$type
            }

            #[inline]
            fn reset() -> pac::resets::regs::Peripherals {
                let mut ret = pac::resets::regs::Peripherals::default();
                ret.$reset(true);
                ret
            }

            #[inline]
            fn waker() -> &'static AtomicWaker {
                static WAKER: AtomicWaker = AtomicWaker::new();

                &WAKER
            }
        }
        impl Instance for peripherals::$type {}
    };
}

impl_instance!(I2C0, I2C0_IRQ, set_i2c0, 32, 33);
impl_instance!(I2C1, I2C1_IRQ, set_i2c1, 34, 35);

pub trait SdaPin<T: Instance>: sealed::SdaPin<T> + crate::gpio::Pin {}
pub trait SclPin<T: Instance>: sealed::SclPin<T> + crate::gpio::Pin {}

macro_rules! impl_pin {
    ($pin:ident, $instance:ident, $function:ident) => {
        impl sealed::$function<peripherals::$instance> for peripherals::$pin {}
        impl $function<peripherals::$instance> for peripherals::$pin {}
    };
}

impl_pin!(PIN_0, I2C0, SdaPin);
impl_pin!(PIN_1, I2C0, SclPin);
impl_pin!(PIN_2, I2C1, SdaPin);
impl_pin!(PIN_3, I2C1, SclPin);
impl_pin!(PIN_4, I2C0, SdaPin);
impl_pin!(PIN_5, I2C0, SclPin);
impl_pin!(PIN_6, I2C1, SdaPin);
impl_pin!(PIN_7, I2C1, SclPin);
impl_pin!(PIN_8, I2C0, SdaPin);
impl_pin!(PIN_9, I2C0, SclPin);
impl_pin!(PIN_10, I2C1, SdaPin);
impl_pin!(PIN_11, I2C1, SclPin);
impl_pin!(PIN_12, I2C0, SdaPin);
impl_pin!(PIN_13, I2C0, SclPin);
impl_pin!(PIN_14, I2C1, SdaPin);
impl_pin!(PIN_15, I2C1, SclPin);
impl_pin!(PIN_16, I2C0, SdaPin);
impl_pin!(PIN_17, I2C0, SclPin);
impl_pin!(PIN_18, I2C1, SdaPin);
impl_pin!(PIN_19, I2C1, SclPin);
impl_pin!(PIN_20, I2C0, SdaPin);
impl_pin!(PIN_21, I2C0, SclPin);
impl_pin!(PIN_22, I2C1, SdaPin);
impl_pin!(PIN_23, I2C1, SclPin);
impl_pin!(PIN_24, I2C0, SdaPin);
impl_pin!(PIN_25, I2C0, SclPin);
impl_pin!(PIN_26, I2C1, SdaPin);
impl_pin!(PIN_27, I2C1, SclPin);
impl_pin!(PIN_28, I2C0, SdaPin);
impl_pin!(PIN_29, I2C0, SclPin);
