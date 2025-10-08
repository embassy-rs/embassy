#![macro_use]

use core::future;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;
use mspm0_metapac::i2c;

use crate::Peri;
use crate::gpio::{AnyPin, PfType, Pull, SealedPin};
use crate::interrupt::typelevel::Binding;
use crate::interrupt::{Interrupt, InterruptExt};
use crate::mode::{Async, Blocking, Mode};
use crate::pac::i2c::{I2c as Regs, vals};
use crate::pac::{self};

/// The clock source for the I2C.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockSel {
    /// Use the bus clock.
    ///
    /// Configurable clock.
    BusClk,

    /// Use the middle frequency clock.
    ///
    /// The MCLK runs at 4 MHz.
    MfClk,
}

/// The clock divider for the I2C.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockDiv {
    // "Do not divide clock source.
    DivBy1,
    // "Divide clock source by 2.
    DivBy2,
    // "Divide clock source by 3.
    DivBy3,
    // "Divide clock source by 4.
    DivBy4,
    // "Divide clock source by 5.
    DivBy5,
    // "Divide clock source by 6.
    DivBy6,
    // "Divide clock source by 7.
    DivBy7,
    // "Divide clock source by 8.
    DivBy8,
}

impl ClockDiv {
    fn into(self) -> vals::Ratio {
        match self {
            Self::DivBy1 => vals::Ratio::DIV_BY_1,
            Self::DivBy2 => vals::Ratio::DIV_BY_2,
            Self::DivBy3 => vals::Ratio::DIV_BY_3,
            Self::DivBy4 => vals::Ratio::DIV_BY_4,
            Self::DivBy5 => vals::Ratio::DIV_BY_5,
            Self::DivBy6 => vals::Ratio::DIV_BY_6,
            Self::DivBy7 => vals::Ratio::DIV_BY_7,
            Self::DivBy8 => vals::Ratio::DIV_BY_8,
        }
    }

    fn divider(self) -> u32 {
        match self {
            Self::DivBy1 => 1,
            Self::DivBy2 => 2,
            Self::DivBy3 => 3,
            Self::DivBy4 => 4,
            Self::DivBy5 => 5,
            Self::DivBy6 => 6,
            Self::DivBy7 => 7,
            Self::DivBy8 => 8,
        }
    }
}

/// The I2C mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BusSpeed {
    /// Standard mode.
    ///
    /// The Standard mode runs at 100 kHz.
    Standard,

    /// Fast mode.
    ///
    /// The fast mode runs at 400 kHz.
    FastMode,

    /// Fast mode plus.
    ///
    /// The fast mode plus runs at 1 MHz.
    FastModePlus,

    /// Custom mode.
    ///
    /// The custom mode frequency (in Hz) can be set manually.
    Custom(u32),
}

impl BusSpeed {
    fn hertz(self) -> u32 {
        match self {
            Self::Standard => 100_000,
            Self::FastMode => 400_000,
            Self::FastModePlus => 1_000_000,
            Self::Custom(s) => s,
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Config Error
pub enum ConfigError {
    /// Invalid clock rate.
    ///
    /// The clock rate could not be configured with the given conifguratoin.
    InvalidClockRate,

    /// Clock source not enabled.
    ///
    /// The clock soure is not enabled is SYSCTL.
    ClockSourceNotEnabled,
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Config
pub struct Config {
    /// I2C clock source.
    clock_source: ClockSel,

    /// I2C clock divider.
    pub clock_div: ClockDiv,

    /// If true: invert SDA pin signal values (V<sub>DD</sub> = 0/mark, Gnd = 1/idle).
    pub invert_sda: bool,

    /// If true: invert SCL pin signal values (V<sub>DD</sub> = 0/mark, Gnd = 1/idle).
    pub invert_scl: bool,

    /// Set the pull configuration for the SDA pin.
    pub sda_pull: Pull,

    /// Set the pull configuration for the SCL pin.
    pub scl_pull: Pull,

    /// Set the pull configuration for the SCL pin.
    pub bus_speed: BusSpeed,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clock_source: ClockSel::MfClk,
            clock_div: ClockDiv::DivBy1,
            invert_sda: false,
            invert_scl: false,
            sda_pull: Pull::None,
            scl_pull: Pull::None,
            bus_speed: BusSpeed::Standard,
        }
    }
}

impl Config {
    pub fn sda_pf(&self) -> PfType {
        PfType::input(self.sda_pull, self.invert_sda)
    }
    pub fn scl_pf(&self) -> PfType {
        PfType::input(self.scl_pull, self.invert_scl)
    }
    fn calculate_timer_period(&self) -> u8 {
        // Sets the timer period to bring the clock frequency to the selected I2C speed
        // From the documentation: TPR = (I2C_CLK / (I2C_FREQ * (SCL_LP + SCL_HP))) - 1 where:
        // - I2C_FREQ is desired I2C frequency (= I2C_BASE_FREQ divided by I2C_DIV)
        // - TPR is the Timer Period register value (range of 1 to 127)
        // - SCL_LP is the SCL Low period (fixed at 6)
        // - SCL_HP is the SCL High period (fixed at 4)
        // - I2C_CLK is functional clock frequency
        return ((self.calculate_clock_source() / (self.bus_speed.hertz() * 10u32)) - 1)
            .try_into()
            .unwrap();
    }

    #[cfg(any(mspm0c110x, mspm0c1105_c1106))]
    fn calculate_clock_source(&self) -> u32 {
        // Assume that BusClk has default value.
        // TODO: calculate BusClk more precisely.
        match self.clock_source {
            ClockSel::MfClk => 4_000_000 / self.clock_div.divider(),
            ClockSel::BusClk => 24_000_000 / self.clock_div.divider(),
        }
    }

    #[cfg(any(
        mspm0g110x, mspm0g150x, mspm0g151x, mspm0g310x, mspm0g350x, mspm0g351x, mspm0l110x, mspm0l122x, mspm0l130x,
        mspm0l134x, mspm0l222x
    ))]
    fn calculate_clock_source(&self) -> u32 {
        // Assume that BusClk has default value.
        // TODO: calculate BusClk more precisely.
        match self.clock_source {
            ClockSel::MfClk => 4_000_000 / self.clock_div.divider(),
            ClockSel::BusClk => 32_000_000 / self.clock_div.divider(),
        }
    }

    fn check_clock_i2c(&self) -> bool {
        // make sure source clock is ~20 faster than i2c clock
        let clk_ratio = 20;

        let i2c_clk = self.bus_speed.hertz() / self.clock_div.divider();
        let src_clk = self.calculate_clock_source();

        // check clock rate
        return src_clk >= i2c_clk * clk_ratio;
    }

    fn define_clock_source(&mut self) -> bool {
        // decide which clock source to choose based on i2c clock.
        // If i2c speed <= 200kHz, use MfClk, otherwise use BusClk
        if self.bus_speed.hertz() / self.clock_div.divider() > 200_000 {
            // TODO: check if BUSCLK enabled
            self.clock_source = ClockSel::BusClk;
        } else {
            // is MFCLK enabled
            if !pac::SYSCTL.mclkcfg().read().usemftick() {
                return false;
            }
            self.clock_source = ClockSel::MfClk;
        }
        return true;
    }

    /// Check the config.
    ///
    /// Make sure that configuration is valid and enabled by the system.
    pub fn check_config(&mut self) -> Result<(), ConfigError> {
        if !self.define_clock_source() {
            return Err(ConfigError::ClockSourceNotEnabled);
        }

        if !self.check_clock_i2c() {
            return Err(ConfigError::InvalidClockRate);
        }

        Ok(())
    }
}

/// Serial error
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Bus error
    Bus,

    /// Arbitration lost
    Arbitration,

    /// ACK not received (either to the address or to a data byte)
    Nack,

    /// Timeout
    Timeout,

    /// CRC error
    Crc,

    /// Overrun error
    Overrun,

    /// Zero-length transfers are not allowed.
    ZeroLengthTransfer,

    /// Transfer length is over limit.
    TransferLengthIsOverLimit,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::Bus => "Bus Error",
            Self::Arbitration => "Arbitration Lost",
            Self::Nack => "ACK Not Received",
            Self::Timeout => "Request Timed Out",
            Self::Crc => "CRC Mismatch",
            Self::Overrun => "Buffer Overrun",
            Self::ZeroLengthTransfer => "Zero-Length Transfers are not allowed",
            Self::TransferLengthIsOverLimit => "Transfer length is over limit",
        };

        write!(f, "{}", message)
    }
}

impl core::error::Error for Error {}

/// I2C Driver.
pub struct I2c<'d, M: Mode> {
    info: &'static Info,
    state: &'static State,
    scl: Option<Peri<'d, AnyPin>>,
    sda: Option<Peri<'d, AnyPin>>,
    _phantom: PhantomData<M>,
}

impl<'d, M: Mode> SetConfig for I2c<'d, M> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(*config)
    }
}

impl<'d> I2c<'d, Blocking> {
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        mut config: Config,
    ) -> Result<Self, ConfigError> {
        if let Err(err) = config.check_config() {
            return Err(err);
        }

        Self::new_inner(peri, scl, sda, config)
    }
}

impl<'d> I2c<'d, Async> {
    pub fn new_async<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        mut config: Config,
    ) -> Result<Self, ConfigError> {
        if let Err(err) = config.check_config() {
            return Err(err);
        }

        let i2c = Self::new_inner(peri, scl, sda, config);

        T::info().interrupt.unpend();
        unsafe { T::info().interrupt.enable() };

        i2c
    }
}

impl<'d, M: Mode> I2c<'d, M> {
    /// Reconfigure the driver
    pub fn set_config(&mut self, mut config: Config) -> Result<(), ConfigError> {
        if let Err(err) = config.check_config() {
            return Err(err);
        }

        self.info.interrupt.disable();

        if let Some(ref sda) = self.sda {
            sda.update_pf(config.sda_pf());
        }

        if let Some(ref scl) = self.scl {
            scl.update_pf(config.scl_pf());
        }

        self.init(&config)
    }

    fn init(&mut self, config: &Config) -> Result<(), ConfigError> {
        // Init I2C
        self.info.regs.clksel().write(|w| match config.clock_source {
            ClockSel::BusClk => {
                w.set_mfclk_sel(false);
                w.set_busclk_sel(true);
            }
            ClockSel::MfClk => {
                w.set_mfclk_sel(true);
                w.set_busclk_sel(false);
            }
        });
        self.info.regs.clkdiv().write(|w| w.set_ratio(config.clock_div.into()));

        // set up glitch filter
        self.info.regs.gfctl().modify(|w| {
            w.set_agfen(false);
            w.set_agfsel(vals::Agfsel::AGLIT_50);
            w.set_chain(true);
        });

        // Reset controller transfer, follow TI example
        self.info.regs.controller(0).cctr().modify(|w| {
            w.set_burstrun(false);
            w.set_start(false);
            w.set_stop(false);
            w.set_ack(false);
            w.set_cackoen(false);
            w.set_rd_on_txempty(false);
            w.set_cblen(0);
        });

        self.state
            .clock
            .store(config.calculate_clock_source(), Ordering::Relaxed);

        self.info
            .regs
            .controller(0)
            .ctpr()
            .write(|w| w.set_tpr(config.calculate_timer_period()));

        // Set Tx Fifo threshold, follow TI example
        self.info
            .regs
            .controller(0)
            .cfifoctl()
            .write(|w| w.set_txtrig(vals::CfifoctlTxtrig::EMPTY));
        // Set Rx Fifo threshold, follow TI example
        self.info
            .regs
            .controller(0)
            .cfifoctl()
            .write(|w| w.set_rxtrig(vals::CfifoctlRxtrig::LEVEL_1));
        // Enable controller clock stretching, follow TI example

        self.info.regs.controller(0).ccr().modify(|w| {
            w.set_clkstretch(true);
            w.set_active(true);
        });

        Ok(())
    }

    fn master_stop(&mut self) {
        // not the first transaction, delay 1000 cycles
        cortex_m::asm::delay(1000);

        // Stop transaction
        self.info.regs.controller(0).cctr().modify(|w| {
            w.set_cblen(0);
            w.set_stop(true);
            w.set_start(false);
        });
    }

    fn master_continue(&mut self, length: usize, send_ack_nack: bool, send_stop: bool) -> Result<(), Error> {
        // delay between ongoing transactions, 1000 cycles
        cortex_m::asm::delay(1000);

        // Update transaction to length amount of bytes
        self.info.regs.controller(0).cctr().modify(|w| {
            w.set_cblen(length as u16);
            w.set_start(false);
            w.set_ack(send_ack_nack);
            if send_stop {
                w.set_stop(true);
            }
        });

        Ok(())
    }

    fn master_read(
        &mut self,
        address: u8,
        length: usize,
        restart: bool,
        send_ack_nack: bool,
        send_stop: bool,
    ) -> Result<(), Error> {
        if restart {
            // not the first transaction, delay 1000 cycles
            cortex_m::asm::delay(1000);
        }

        // Set START and prepare to receive bytes into
        // `buffer`. The START bit can be set even if the bus
        // is BUSY or I2C is in slave mode.
        self.info.regs.controller(0).csa().modify(|w| {
            w.set_taddr(address as u16);
            w.set_cmode(vals::Mode::MODE7);
            w.set_dir(vals::Dir::RECEIVE);
        });

        self.info.regs.controller(0).cctr().modify(|w| {
            w.set_cblen(length as u16);
            w.set_burstrun(true);
            w.set_ack(send_ack_nack);
            w.set_start(true);
            if send_stop {
                w.set_stop(true);
            }
        });

        Ok(())
    }

    fn master_write(&mut self, address: u8, length: usize, send_stop: bool) -> Result<(), Error> {
        // Start transfer of length amount of bytes
        self.info.regs.controller(0).csa().modify(|w| {
            w.set_taddr(address as u16);
            w.set_cmode(vals::Mode::MODE7);
            w.set_dir(vals::Dir::TRANSMIT);
        });
        self.info.regs.controller(0).cctr().modify(|w| {
            w.set_cblen(length as u16);
            w.set_burstrun(true);
            w.set_start(true);
            if send_stop {
                w.set_stop(true);
            }
        });

        Ok(())
    }

    fn check_error(&self) -> Result<(), Error> {
        let csr = self.info.regs.controller(0).csr().read();
        if csr.err() {
            return Err(Error::Nack);
        } else if csr.arblst() {
            return Err(Error::Arbitration);
        }
        Ok(())
    }
}

impl<'d> I2c<'d, Blocking> {
    fn master_blocking_continue(&mut self, length: usize, send_ack_nack: bool, send_stop: bool) -> Result<(), Error> {
        // Perform transaction
        self.master_continue(length, send_ack_nack, send_stop)?;

        // Poll until the Controller process all bytes or NACK
        while self.info.regs.controller(0).csr().read().busy() {}

        Ok(())
    }

    fn master_blocking_read(
        &mut self,
        address: u8,
        length: usize,
        restart: bool,
        send_ack_nack: bool,
        send_stop: bool,
    ) -> Result<(), Error> {
        // unless restart, Wait for the controller to be idle,
        if !restart {
            while !self.info.regs.controller(0).csr().read().idle() {}
        }

        self.master_read(address, length, restart, send_ack_nack, send_stop)?;

        // Poll until the Controller process all bytes or NACK
        while self.info.regs.controller(0).csr().read().busy() {}

        Ok(())
    }

    fn master_blocking_write(&mut self, address: u8, length: usize, send_stop: bool) -> Result<(), Error> {
        // Wait for the controller to be idle
        while !self.info.regs.controller(0).csr().read().idle() {}

        // Perform writing
        self.master_write(address, length, send_stop)?;

        // Poll until the Controller writes all bytes or NACK
        while self.info.regs.controller(0).csr().read().busy() {}

        Ok(())
    }

    fn read_blocking_internal(
        &mut self,
        address: u8,
        read: &mut [u8],
        restart: bool,
        end_w_stop: bool,
    ) -> Result<(), Error> {
        if read.is_empty() {
            return Err(Error::ZeroLengthTransfer);
        }
        if read.len() > self.info.fifo_size {
            return Err(Error::TransferLengthIsOverLimit);
        }

        let read_len = read.len();
        let mut bytes_to_read = read_len;
        for (number, chunk) in read.chunks_mut(self.info.fifo_size).enumerate() {
            bytes_to_read -= chunk.len();
            // if the current transaction is the last & end_w_stop, send stop
            let send_stop = bytes_to_read == 0 && end_w_stop;
            // if there are still bytes to read, send ACK
            let send_ack_nack = bytes_to_read != 0;

            if number == 0 {
                self.master_blocking_read(
                    address,
                    chunk.len().min(self.info.fifo_size),
                    restart,
                    send_ack_nack,
                    send_stop,
                )?
            } else {
                self.master_blocking_continue(chunk.len(), send_ack_nack, send_stop)?;
            }

            // check errors
            if let Err(err) = self.check_error() {
                self.master_stop();
                return Err(err);
            }

            for byte in chunk {
                *byte = self.info.regs.controller(0).crxdata().read().value();
            }
        }
        Ok(())
    }

    fn write_blocking_internal(&mut self, address: u8, write: &[u8], end_w_stop: bool) -> Result<(), Error> {
        if write.is_empty() {
            return Err(Error::ZeroLengthTransfer);
        }
        if write.len() > self.info.fifo_size {
            return Err(Error::TransferLengthIsOverLimit);
        }

        let mut bytes_to_send = write.len();
        for (number, chunk) in write.chunks(self.info.fifo_size).enumerate() {
            for byte in chunk {
                let ctrl0 = self.info.regs.controller(0).ctxdata();
                ctrl0.write(|w| w.set_value(*byte));
            }

            // if the current transaction is the last & end_w_stop, send stop
            bytes_to_send -= chunk.len();
            let send_stop = end_w_stop && bytes_to_send == 0;

            if number == 0 {
                self.master_blocking_write(address, chunk.len(), send_stop)?;
            } else {
                self.master_blocking_continue(chunk.len(), false, send_stop)?;
            }

            // check errors
            if let Err(err) = self.check_error() {
                self.master_stop();
                return Err(err);
            }
        }
        Ok(())
    }

    // =========================
    //  Blocking public API

    /// Blocking read.
    pub fn blocking_read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Error> {
        // wait until bus is free
        while self.info.regs.controller(0).csr().read().busbsy() {}
        self.read_blocking_internal(address, read, false, true)
    }

    /// Blocking write.
    pub fn blocking_write(&mut self, address: u8, write: &[u8]) -> Result<(), Error> {
        // wait until bus is free
        while self.info.regs.controller(0).csr().read().busbsy() {}
        self.write_blocking_internal(address, write, true)
    }

    /// Blocking write, restart, read.
    pub fn blocking_write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        // wait until bus is free
        while self.info.regs.controller(0).csr().read().busbsy() {}
        let err = self.write_blocking_internal(address, write, false);
        if err != Ok(()) {
            return err;
        }
        self.read_blocking_internal(address, read, true, true)
    }
}

impl<'d> I2c<'d, Async> {
    async fn write_async_internal(&mut self, addr: u8, write: &[u8], end_w_stop: bool) -> Result<(), Error> {
        let ctrl = self.info.regs.controller(0);

        let mut bytes_to_send = write.len();
        for (number, chunk) in write.chunks(self.info.fifo_size).enumerate() {
            self.info.regs.cpu_int(0).imask().modify(|w| {
                w.set_carblost(true);
                w.set_cnack(true);
                w.set_ctxdone(true);
            });

            for byte in chunk {
                ctrl.ctxdata().write(|w| w.set_value(*byte));
            }

            // if the current transaction is the last & end_w_stop, send stop
            bytes_to_send -= chunk.len();
            let send_stop = end_w_stop && bytes_to_send == 0;

            if number == 0 {
                self.master_write(addr, chunk.len(), send_stop)?;
            } else {
                self.master_continue(chunk.len(), false, send_stop)?;
            }

            let res: Result<(), Error> = future::poll_fn(|cx| {
                use crate::i2c::vals::CpuIntIidxStat;
                // Register prior to checking the condition
                self.state.waker.register(cx.waker());

                let result = match self.info.regs.cpu_int(0).iidx().read().stat() {
                    CpuIntIidxStat::NO_INTR => Poll::Pending,
                    CpuIntIidxStat::CNACKFG => Poll::Ready(Err(Error::Nack)),
                    CpuIntIidxStat::CARBLOSTFG => Poll::Ready(Err(Error::Arbitration)),
                    CpuIntIidxStat::CTXDONEFG => Poll::Ready(Ok(())),
                    _ => Poll::Pending,
                };

                if !result.is_pending() {
                    self.info
                        .regs
                        .cpu_int(0)
                        .imask()
                        .write_value(i2c::regs::CpuInt::default());
                }
                return result;
            })
            .await;

            if res.is_err() {
                self.master_stop();
                return res;
            }
        }
        Ok(())
    }

    async fn read_async_internal(
        &mut self,
        addr: u8,
        read: &mut [u8],
        restart: bool,
        end_w_stop: bool,
    ) -> Result<(), Error> {
        let read_len = read.len();

        let mut bytes_to_read = read_len;
        for (number, chunk) in read.chunks_mut(self.info.fifo_size).enumerate() {
            bytes_to_read -= chunk.len();
            // if the current transaction is the last & end_w_stop, send stop
            let send_stop = bytes_to_read == 0 && end_w_stop;
            // if there are still bytes to read, send ACK
            let send_ack_nack = bytes_to_read != 0;

            self.info.regs.cpu_int(0).imask().modify(|w| {
                w.set_carblost(true);
                w.set_cnack(true);
                w.set_crxdone(true);
            });

            if number == 0 {
                self.master_read(addr, chunk.len(), restart, send_ack_nack, send_stop)?
            } else {
                self.master_continue(chunk.len(), send_ack_nack, send_stop)?;
            }

            let res: Result<(), Error> = future::poll_fn(|cx| {
                use crate::i2c::vals::CpuIntIidxStat;
                // Register prior to checking the condition
                self.state.waker.register(cx.waker());

                let result = match self.info.regs.cpu_int(0).iidx().read().stat() {
                    CpuIntIidxStat::NO_INTR => Poll::Pending,
                    CpuIntIidxStat::CNACKFG => Poll::Ready(Err(Error::Nack)),
                    CpuIntIidxStat::CARBLOSTFG => Poll::Ready(Err(Error::Arbitration)),
                    CpuIntIidxStat::CRXDONEFG => Poll::Ready(Ok(())),
                    _ => Poll::Pending,
                };

                if !result.is_pending() {
                    self.info
                        .regs
                        .cpu_int(0)
                        .imask()
                        .write_value(i2c::regs::CpuInt::default());
                }
                return result;
            })
            .await;

            if res.is_err() {
                self.master_stop();
                return res;
            }

            for byte in chunk {
                *byte = self.info.regs.controller(0).crxdata().read().value();
            }
        }
        Ok(())
    }

    // =========================
    //  Async public API

    pub async fn async_write(&mut self, address: u8, write: &[u8]) -> Result<(), Error> {
        // wait until bus is free
        while self.info.regs.controller(0).csr().read().busbsy() {}
        self.write_async_internal(address, write, true).await
    }

    pub async fn async_read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Error> {
        // wait until bus is free
        while self.info.regs.controller(0).csr().read().busbsy() {}
        self.read_async_internal(address, read, false, true).await
    }

    pub async fn async_write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        // wait until bus is free
        while self.info.regs.controller(0).csr().read().busbsy() {}

        let err = self.write_async_internal(address, write, false).await;
        if err != Ok(()) {
            return err;
        }
        self.read_async_internal(address, read, true, true).await
    }
}

impl<'d> embedded_hal_02::blocking::i2c::Read for I2c<'d, Blocking> {
    type Error = Error;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(address, buffer)
    }
}

impl<'d> embedded_hal_02::blocking::i2c::Write for I2c<'d, Blocking> {
    type Error = Error;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(address, bytes)
    }
}

impl<'d> embedded_hal_02::blocking::i2c::WriteRead for I2c<'d, Blocking> {
    type Error = Error;

    fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_write_read(address, bytes, buffer)
    }
}

impl<'d> embedded_hal_02::blocking::i2c::Transactional for I2c<'d, Blocking> {
    type Error = Error;

    fn exec(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_02::blocking::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        // wait until bus is free
        while self.info.regs.controller(0).csr().read().busbsy() {}
        for i in 0..operations.len() {
            match &mut operations[i] {
                embedded_hal_02::blocking::i2c::Operation::Read(buf) => {
                    self.read_blocking_internal(address, buf, false, false)?
                }
                embedded_hal_02::blocking::i2c::Operation::Write(buf) => {
                    self.write_blocking_internal(address, buf, false)?
                }
            }
        }
        self.master_stop();
        Ok(())
    }
}

impl embedded_hal::i2c::Error for Error {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        match *self {
            Self::Bus => embedded_hal::i2c::ErrorKind::Bus,
            Self::Arbitration => embedded_hal::i2c::ErrorKind::ArbitrationLoss,
            Self::Nack => embedded_hal::i2c::ErrorKind::NoAcknowledge(embedded_hal::i2c::NoAcknowledgeSource::Unknown),
            Self::Timeout => embedded_hal::i2c::ErrorKind::Other,
            Self::Crc => embedded_hal::i2c::ErrorKind::Other,
            Self::Overrun => embedded_hal::i2c::ErrorKind::Overrun,
            Self::ZeroLengthTransfer => embedded_hal::i2c::ErrorKind::Other,
            Self::TransferLengthIsOverLimit => embedded_hal::i2c::ErrorKind::Other,
        }
    }
}

impl<'d, M: Mode> embedded_hal::i2c::ErrorType for I2c<'d, M> {
    type Error = Error;
}

impl<'d> embedded_hal::i2c::I2c for I2c<'d, Blocking> {
    fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(address, read)
    }

    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(address, write)
    }

    fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_write_read(address, write, read)
    }

    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        // wait until bus is free
        while self.info.regs.controller(0).csr().read().busbsy() {}
        for i in 0..operations.len() {
            match &mut operations[i] {
                embedded_hal::i2c::Operation::Read(buf) => self.read_blocking_internal(address, buf, false, false)?,
                embedded_hal::i2c::Operation::Write(buf) => self.write_blocking_internal(address, buf, false)?,
            }
        }
        self.master_stop();
        Ok(())
    }
}

impl<'d> embedded_hal_async::i2c::I2c for I2c<'d, Async> {
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        self.async_read(address, read).await
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.async_write(address, write).await
    }

    async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        self.async_write_read(address, write, read).await
    }

    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        // wait until bus is free
        while self.info.regs.controller(0).csr().read().busbsy() {}
        for i in 0..operations.len() {
            match &mut operations[i] {
                embedded_hal::i2c::Operation::Read(buf) => self.read_async_internal(address, buf, false, false).await?,
                embedded_hal::i2c::Operation::Write(buf) => self.write_async_internal(address, buf, false).await?,
            }
        }
        self.master_stop();
        Ok(())
    }
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _i2c: PhantomData<T>,
}

impl<T: Instance> crate::interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    // Mask interrupts and wake any task waiting for this interrupt
    unsafe fn on_interrupt() {
        T::state().waker.wake();
    }
}

/// Peripheral instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}

/// I2C `SDA` pin trait
pub trait SdaPin<T: Instance>: crate::gpio::Pin {
    /// Get the PF number needed to use this pin as `SDA`.
    fn pf_num(&self) -> u8;
}

/// I2C `SCL` pin trait
pub trait SclPin<T: Instance>: crate::gpio::Pin {
    /// Get the PF number needed to use this pin as `SCL`.
    fn pf_num(&self) -> u8;
}

// ==== IMPL types ====

pub(crate) struct Info {
    pub(crate) regs: Regs,
    pub(crate) interrupt: Interrupt,
    pub fifo_size: usize,
}

pub(crate) struct State {
    /// The clock rate of the I2C. This might be configured.
    pub(crate) clock: AtomicU32,
    pub(crate) waker: AtomicWaker,
}

impl<'d, M: Mode> I2c<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        // Init power for I2C
        T::info().regs.gprcm(0).rstctl().write(|w| {
            w.set_resetstkyclr(true);
            w.set_resetassert(true);
            w.set_key(vals::ResetKey::KEY);
        });

        T::info().regs.gprcm(0).pwren().write(|w| {
            w.set_enable(true);
            w.set_key(vals::PwrenKey::KEY);
        });

        // init delay, 16 cycles
        cortex_m::asm::delay(16);

        // Init GPIO
        let scl_inner = new_pin!(scl, config.scl_pf());
        let sda_inner = new_pin!(sda, config.sda_pf());

        if let Some(ref scl) = scl_inner {
            let pincm = pac::IOMUX.pincm(scl._pin_cm() as usize);
            pincm.modify(|w| {
                w.set_hiz1(true);
            });
        }

        if let Some(ref sda) = sda_inner {
            let pincm = pac::IOMUX.pincm(sda._pin_cm() as usize);
            pincm.modify(|w| {
                w.set_hiz1(true);
            });
        }

        let mut this = Self {
            info: T::info(),
            state: T::state(),
            scl: scl_inner,
            sda: sda_inner,
            _phantom: PhantomData,
        };
        this.init(&config)?;

        Ok(this)
    }
}

pub(crate) trait SealedInstance {
    fn info() -> &'static Info;
    fn state() -> &'static State;
}

macro_rules! impl_i2c_instance {
    ($instance: ident, $fifo_size: expr) => {
        impl crate::i2c::SealedInstance for crate::peripherals::$instance {
            fn info() -> &'static crate::i2c::Info {
                use crate::i2c::Info;
                use crate::interrupt::typelevel::Interrupt;

                const INFO: Info = Info {
                    regs: crate::pac::$instance,
                    interrupt: crate::interrupt::typelevel::$instance::IRQ,
                    fifo_size: $fifo_size,
                };
                &INFO
            }

            fn state() -> &'static crate::i2c::State {
                use crate::i2c::State;
                use crate::interrupt::typelevel::Interrupt;

                static STATE: State = State {
                    clock: core::sync::atomic::AtomicU32::new(0),
                    waker: embassy_sync::waitqueue::AtomicWaker::new(),
                };
                &STATE
            }
        }

        impl crate::i2c::Instance for crate::peripherals::$instance {
            type Interrupt = crate::interrupt::typelevel::$instance;
        }
    };
}

macro_rules! impl_i2c_sda_pin {
    ($instance: ident, $pin: ident, $pf: expr) => {
        impl crate::i2c::SdaPin<crate::peripherals::$instance> for crate::peripherals::$pin {
            fn pf_num(&self) -> u8 {
                $pf
            }
        }
    };
}

macro_rules! impl_i2c_scl_pin {
    ($instance: ident, $pin: ident, $pf: expr) => {
        impl crate::i2c::SclPin<crate::peripherals::$instance> for crate::peripherals::$pin {
            fn pf_num(&self) -> u8 {
                $pf
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::i2c::{BusSpeed, ClockDiv, ClockSel, Config};

    /// These tests are based on TI's reference caluclation.
    #[test]
    fn ti_calculate_timer_period() {
        let mut config = Config::default();
        config.clock_div = ClockDiv::DivBy1;
        config.bus_speed = BusSpeed::FastMode;
        config.clock_source = ClockSel::BusClk;
        assert_eq!(config.calculate_timer_period(), 7u8);
    }

    #[test]
    fn ti_calculate_timer_period_2() {
        let mut config = Config::default();
        config.clock_div = ClockDiv::DivBy2;
        config.bus_speed = BusSpeed::FastMode;
        config.clock_source = ClockSel::BusClk;
        assert_eq!(config.calculate_timer_period(), 3u8);
    }

    #[test]
    fn ti_calculate_timer_period_3() {
        let mut config = Config::default();
        config.clock_div = ClockDiv::DivBy2;
        config.bus_speed = BusSpeed::Standard;
        config.clock_source = ClockSel::BusClk;
        assert_eq!(config.calculate_timer_period(), 15u8);
    }

    #[test]
    fn ti_calculate_timer_period_4() {
        let mut config = Config::default();
        config.clock_div = ClockDiv::DivBy2;
        config.bus_speed = BusSpeed::Custom(100_000);
        config.clock_source = ClockSel::BusClk;
        assert_eq!(config.calculate_timer_period(), 15u8);
    }

    #[test]
    fn clock_check_fastmodeplus_rate_with_busclk() {
        let mut config = Config::default();
        config.clock_source = ClockSel::BusClk;
        config.bus_speed = BusSpeed::FastModePlus;
        assert!(config.check_clock_i2c());
    }

    #[test]
    fn clock_check_fastmode_rate_with_busclk() {
        let mut config = Config::default();
        config.clock_source = ClockSel::BusClk;
        config.bus_speed = BusSpeed::FastMode;
        assert!(config.check_clock_i2c());
    }

    #[test]
    fn clock_check_fastmodeplus_rate_with_mfclk() {
        let mut config = Config::default();
        config.clock_source = ClockSel::MfClk;
        config.bus_speed = BusSpeed::FastModePlus;
        assert!(!config.check_clock_i2c());
    }

    #[test]
    fn clock_check_fastmode_rate_with_mfclk() {
        let mut config = Config::default();
        config.clock_source = ClockSel::MfClk;
        config.bus_speed = BusSpeed::FastMode;
        assert!(!config.check_clock_i2c());
    }
}
