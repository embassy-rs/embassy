use core::marker::PhantomData;

use embassy_hal_common::into_ref;
use pac::i2c;

use crate::{pac, peripherals, Peripheral};

/// I2C error
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// I2C abort with error
    Abort(u32),
    /// User passed in a read buffer that was 0 length
    InvalidReadBufferLength,
    /// User passed in a write buffer that was 0 length
    InvalidWriteBufferLength,
    /// Target i2c address is out of range
    AddressOutOfRange(u16),
    /// Target i2c address is reserved
    AddressReserved(u16),
}

#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct Config {
    pub frequency: u32,
    pub sda_pullup: bool,
    pub scl_pullup: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: 100_000,
            sda_pullup: false,
            scl_pullup: false,
        }
    }
}

pub struct I2c<'d, T: Instance, M: Mode> {
    phantom: PhantomData<(&'d mut T, M)>,
}

impl<'d, T: Instance> I2c<'d, T, Master> {
    pub fn new_master(
        _peri: impl Peripheral<P = T> + 'd,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(_peri, scl, sda);

        assert!(config.frequency <= 1_000_000);
        assert!(config.frequency > 0);

        let p = T::regs();

        unsafe {
            p.ic_enable().write(|w| w.set_enable(false));

            // select controller mode & speed
            p.ic_con().write(|w| {
                // Always use "fast" mode (<= 400 kHz, works fine for standard mode too)
                w.set_speed(i2c::vals::Speed::FAST);
                w.set_master_mode(true);
                w.set_ic_slave_disable(true);
                w.set_ic_restart_en(true);
                w.set_tx_empty_ctrl(true);
            });

            // Clear FIFO threshold
            p.ic_tx_tl().write(|w| w.set_tx_tl(0));
            p.ic_rx_tl().write(|w| w.set_rx_tl(0));

            // Configure SCL & SDA pins
            scl.io().ctrl().write(|w| w.set_funcsel(3));
            sda.io().ctrl().write(|w| w.set_funcsel(3));

            scl.pad_ctrl().write(|w| {
                w.set_schmitt(true);
                w.set_pue(config.scl_pullup);
            });
            sda.pad_ctrl().write(|w| {
                w.set_schmitt(true);
                w.set_pue(config.sda_pullup);
            });

            // Configure baudrate

            // There are some subtleties to I2C timing which we are completely ignoring here
            // See: https://github.com/raspberrypi/pico-sdk/blob/bfcbefafc5d2a210551a4d9d80b4303d4ae0adf7/src/rp2_common/hardware_i2c/i2c.c#L69
            let clk_base = crate::clocks::clk_sys_freq();

            let period = (clk_base + config.frequency / 2) / config.frequency;
            let lcnt = period * 3 / 5; // spend 3/5 (60%) of the period low
            let hcnt = period - lcnt; // and 2/5 (40%) of the period high

            // Check for out-of-range divisors:
            assert!(hcnt <= 0xffff);
            assert!(lcnt <= 0xffff);
            assert!(hcnt >= 8);
            assert!(lcnt >= 8);

            // Per I2C-bus specification a device in standard or fast mode must
            // internally provide a hold time of at least 300ns for the SDA signal to
            // bridge the undefined region of the falling edge of SCL. A smaller hold
            // time of 120ns is used for fast mode plus.
            let sda_tx_hold_count = if config.frequency < 1_000_000 {
                // sda_tx_hold_count = clk_base [cycles/s] * 300ns * (1s / 1e9ns)
                // Reduce 300/1e9 to 3/1e7 to avoid numbers that don't fit in uint.
                // Add 1 to avoid division truncation.
                ((clk_base * 3) / 10_000_000) + 1
            } else {
                // fast mode plus requires a clk_base > 32MHz
                assert!(clk_base >= 32_000_000);

                // sda_tx_hold_count = clk_base [cycles/s] * 120ns * (1s / 1e9ns)
                // Reduce 120/1e9 to 3/25e6 to avoid numbers that don't fit in uint.
                // Add 1 to avoid division truncation.
                ((clk_base * 3) / 25_000_000) + 1
            };
            assert!(sda_tx_hold_count <= lcnt - 2);

            p.ic_fs_scl_hcnt().write(|w| w.set_ic_fs_scl_hcnt(hcnt as u16));
            p.ic_fs_scl_lcnt().write(|w| w.set_ic_fs_scl_lcnt(lcnt as u16));
            p.ic_fs_spklen()
                .write(|w| w.set_ic_fs_spklen(if lcnt < 16 { 1 } else { (lcnt / 16) as u8 }));
            p.ic_sda_hold()
                .write(|w| w.set_ic_sda_tx_hold(sda_tx_hold_count as u16));

            // Enable I2C block
            p.ic_enable().write(|w| w.set_enable(true));
        }

        Self { phantom: PhantomData }
    }
}

mod sealed {
    pub trait Instance {}
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

pub struct Master;
pub struct Slave;

impl_mode!(Master);
impl_mode!(Slave);

pub trait Instance: sealed::Instance {
    fn regs() -> pac::i2c::I2c;
}

macro_rules! impl_instance {
    ($type:ident, $irq:ident) => {
        impl sealed::Instance for peripherals::$type {}
        impl Instance for peripherals::$type {
            fn regs() -> pac::i2c::I2c {
                pac::$type
            }
        }
    };
}

impl_instance!(I2C0, I2c0);
impl_instance!(I2C1, I2c1);

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
