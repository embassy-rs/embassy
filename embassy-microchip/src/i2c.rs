//! I2C driver.

use core::future;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use pac::smb0;

use crate::gpio::{AnyPin, SealedPin};
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::{interrupt, pac, peripherals};

/// I2c error abort reason
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AbortReason {
    /// A bus operation was not acknowledged.
    NoAcknowledge,

    /// Lost arbitration
    ArbitrationLoss,

    /// Other reason
    Other,
}

/// I2C error bus timeout reason
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BusTimeoutReason {
    /// Clock high, data high timeout
    ClockHighDataHigh,

    /// Clock high, data low timeout
    ClockHighDataLow,

    /// Slave cumulative timeout
    SlaveCumulativeTimeout,

    /// Master cumulative timeout
    MasterCumulativeTimeout,

    /// Device timeout
    DeviceTimeout,
}

/// I2C error
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// I2C abort with error
    Abort(AbortReason),

    /// Bus timeout
    BusTimeout(BusTimeoutReason),

    /// User passed in a read buffer with 0 length
    InvalidReadBufferLength,

    /// User passed in a write buffer with 0 length
    InvalidWriteBufferLength,

    /// Target I2C address is out of range
    AddressOutOfRange(u8),

    /// Target I2C address is reserved
    AddressReserved(u8),
}

/// I2C config error
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConfigError {
    /// Max I2C frequency is 1MHz
    FrequencyTooHigh,

    /// The baud rate clock is too slow to support the requested
    /// frequency
    ClockTooSlow,

    /// The baud rate clock is too fast to support the requested
    /// frequency
    ClockTooFast,
}

/// I2C bus speeds
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BusSpeed {
    /// 100kHz
    Standard,

    /// 400kHz,
    Fast,

    /// 1MHz,
    FastPlus,
}

/// I2C config
#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct Config {
    /// Bus speed.
    pub speed: BusSpeed,

    /// Own address 1 in 7-bit format.
    ///
    /// The internal master should not attempt transactions to a slave
    /// with the same address as `addr1`. This represents an illegal
    /// operation.
    ///
    /// Passing `None` results in the value 0 being written to the Own
    /// Address register and General Call Match to be disabled.
    pub addr1: Option<u8>,

    /// Own address 2 in 7-bit format.
    ///
    /// The internal master should not attempt transactions to a slave
    /// with the same address as `addr2`. This represents an illegal
    /// operation.
    ///
    /// Passing `None` results in the value 0 being written to the Own
    /// Address register and General Call Match to be disabled.
    pub addr2: Option<u8>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            speed: BusSpeed::Standard,
            addr1: None,
            addr2: None,
        }
    }
}

macro_rules! write_register_with_delay {
    ($reg_path:expr, |$w:ident| $body:block) => {{
        $reg_path.write(|$w| $body);
        // TODO - empirically determined delay
        cortex_m::asm::delay(20_000);
    }};
}

/// I2C driver.
pub struct I2c<'d, T: Instance, M: Mode> {
    _peri: Peri<'d, T>,
    _sda: Peri<'d, AnyPin>,
    _scl: Peri<'d, AnyPin>,
    config: Config,
    port: u8,
    phantom: PhantomData<M>,
}

impl<'d, T: Instance> I2c<'d, T, Blocking> {
    /// Create a new driver instance in blocking mode.
    pub fn new_blocking<SCL: SclPin, SDA: SdaPin>(
        _peri: Peri<'d, T>,
        _scl: Peri<'d, SCL>,
        _sda: Peri<'d, SDA>,
        config: Config,
    ) -> Self
    where
        (T, SCL, SDA): ValidI2cConfig,
    {
        _scl.setup();
        _sda.setup();

        let port = <(T, SCL, SDA) as SealedValidI2cConfig>::port();
        Self::new_inner(_peri, _scl.into(), _sda.into(), config, port)
    }
}

impl<'d, T: Instance> I2c<'d, T, Async> {
    /// Create a new driver instance in async mode.
    pub fn new_async<SCL: SclPin, SDA: SdaPin>(
        _peri: Peri<'d, T>,
        _scl: Peri<'d, SCL>,
        _sda: Peri<'d, SDA>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        config: Config,
    ) -> Self
    where
        (T, SCL, SDA): ValidI2cConfig,
    {
        // mask all interrupts
        pac::ECIA.src13().write_value(1 << T::irq_bit());
        pac::ECIA.en_clr13().write_value(1 << T::irq_bit());

        _scl.setup();
        _sda.setup();

        let port = <(T, SCL, SDA) as SealedValidI2cConfig>::port();
        let i2c = Self::new_inner(_peri, _scl.into(), _sda.into(), config, port);

        // unmask interrupt
        pac::ECIA.en_set13().write_value(1 << T::irq_bit());

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        i2c
    }

    /// Calls `f` to check if we are ready or not.
    /// If not, `g` is called once(to eg enable the required interrupts).
    /// The waker will always be registered prior to calling `f`.
    async fn wait_on<F, U, G>(&mut self, mut f: F, mut g: G) -> U
    where
        F: FnMut(&mut Self) -> Poll<U>,
        G: FnMut(&mut Self),
    {
        future::poll_fn(|cx| {
            // Register prior to checking the condition
            T::waker().register(cx.waker());
            let r = f(self);

            if r.is_pending() {
                g(self);
            }
            r
        })
        .await
    }

    async fn wait_for_bus_free_async(&mut self) -> Result<(), Error> {
        self.wait_on(
            |_| {
                let compl = T::regs().compl().read();
                let sts = T::regs().rsts().read();

                if sts.nbb() {
                    T::regs().compl().write(|w| w.set_idle(true));
                    Poll::Ready(Ok(()))
                } else if compl.lab() {
                    T::regs().compl().write(|w| w.set_lab(true));
                    Poll::Ready(Err(Error::Abort(AbortReason::ArbitrationLoss)))
                } else if compl.ber() && compl.dto() {
                    T::regs().compl().write(|w| w.set_dto(true));
                    Poll::Ready(Err(Error::BusTimeout(BusTimeoutReason::DeviceTimeout)))
                } else if compl.ber() && compl.mcto() {
                    T::regs().compl().write(|w| w.set_mcto(true));
                    Poll::Ready(Err(Error::BusTimeout(BusTimeoutReason::MasterCumulativeTimeout)))
                } else if compl.ber() && compl.chdl() {
                    T::regs().compl().write(|w| w.set_chdl(true));
                    Poll::Ready(Err(Error::BusTimeout(BusTimeoutReason::ClockHighDataLow)))
                } else if compl.ber() && compl.chdh() {
                    T::regs().compl().write(|w| w.set_chdh(true));
                    Poll::Ready(Err(Error::BusTimeout(BusTimeoutReason::ClockHighDataHigh)))
                } else if compl.ber() {
                    T::regs().compl().write(|w| w.set_ber(true));
                    Poll::Ready(Err(Error::Abort(AbortReason::Other)))
                } else {
                    Poll::Pending
                }
            },
            |_| {
                T::regs().cfg().modify(|w| w.set_enidi(true));
            },
        )
        .await
    }

    async fn wait_for_completion_async(&mut self) -> Result<(), Error> {
        self.wait_on(
            |_| {
                let compl = T::regs().compl().read();
                let sts = T::regs().rsts().read();

                if sts.pin() {
                    Poll::Pending
                } else if !sts.lrb_ad0() {
                    Poll::Ready(Ok(()))
                } else if compl.lab() {
                    T::regs().compl().write(|w| w.set_lab(true));
                    Poll::Ready(Err(Error::Abort(AbortReason::ArbitrationLoss)))
                } else if compl.ber() && compl.dto() {
                    T::regs().compl().write(|w| w.set_dto(true));
                    Poll::Ready(Err(Error::BusTimeout(BusTimeoutReason::DeviceTimeout)))
                } else if compl.ber() && compl.mcto() {
                    T::regs().compl().write(|w| w.set_mcto(true));
                    Poll::Ready(Err(Error::BusTimeout(BusTimeoutReason::MasterCumulativeTimeout)))
                } else if compl.ber() && compl.chdl() {
                    T::regs().compl().write(|w| w.set_chdl(true));
                    Poll::Ready(Err(Error::BusTimeout(BusTimeoutReason::ClockHighDataLow)))
                } else if compl.ber() && compl.chdh() {
                    T::regs().compl().write(|w| w.set_chdh(true));
                    Poll::Ready(Err(Error::BusTimeout(BusTimeoutReason::ClockHighDataHigh)))
                } else if compl.ber() {
                    T::regs().compl().write(|w| w.set_ber(true));
                    Poll::Ready(Err(Error::Abort(AbortReason::Other)))
                } else {
                    Poll::Pending
                }
            },
            |_| {
                T::regs().cfg().modify(|w| {
                    w.set_enidi(true);
                    w.set_enmi(true);
                    w.set_ensi(true);
                });
            },
        )
        .await
    }

    async fn start_async(&mut self, address: u8, rw: bool, repeated: bool) -> Result<(), Error> {
        if address >= 0x80 {
            return Err(Error::AddressOutOfRange(address));
        }

        let r = T::regs();
        let own_addr = r.own_addr().read();
        let (addr1, addr2) = (own_addr.addr1(), own_addr.addr2());

        if address == addr1 || address == addr2 {
            return Err(Error::AddressReserved(address));
        }

        if repeated {
            write_register_with_delay!(T::regs().wctrl(), |w| {
                w.set_eso(true);
                w.set_sta(true);
                w.set_ack(true);
            });
            T::regs().i2cdata().write_value(address << 1 | u8::from(rw));
        } else {
            if rw {
                T::regs().i2cdata().write_value(address << 1 | 1);
                self.wait_for_bus_free_async().await?;
            } else {
                self.wait_for_bus_free_async().await?;
                T::regs().i2cdata().write_value(address << 1 | 0);
            }

            write_register_with_delay!(T::regs().wctrl(), |w| {
                w.set_pin(true);
                w.set_eso(true);
                w.set_sta(true);
                w.set_ack(true);
            });

            self.wait_for_completion_async().await?;
        }

        Ok(())
    }

    async fn read_byte_async(&mut self) -> Result<u8, Error> {
        self.wait_for_completion_async().await?;
        let byte = T::regs().i2cdata().read();
        Ok(byte)
    }

    async fn write_byte_async(&mut self, byte: u8) -> Result<(), Error> {
        self.wait_for_completion_async().await?;
        T::regs().i2cdata().write_value(byte);
        Ok(())
    }

    async fn read_async_internal(
        &mut self,
        address: u8,
        read: &mut [u8],
        repeated: bool,
        send_stop: bool,
    ) -> Result<(), Error> {
        if read.is_empty() {
            return Err(Error::InvalidReadBufferLength);
        }

        self.start_async(address, true, repeated).await?;

        // First byte in the FIFO is the slave address. Ignore it.
        let _ = self.read_byte_async().await?;

        let last = read.len() - 1;
        for (i, byte) in read.iter_mut().enumerate() {
            if i == last {
                write_register_with_delay!(T::regs().wctrl(), |w| {
                    w.set_eso(true);
                });
            }

            let b = self.read_byte_async().await?;
            *byte = b;
        }

        if send_stop {
            Self::stop();
        }

        Ok(())
    }

    async fn write_async_internal(&mut self, address: u8, write: &[u8], send_stop: bool) -> Result<(), Error> {
        if write.is_empty() {
            return Err(Error::InvalidWriteBufferLength);
        }

        self.start_async(address, false, false).await?;

        for byte in write.iter() {
            self.write_byte_async(*byte).await?;
        }

        if send_stop {
            Self::stop();
        }

        Ok(())
    }

    /// Read from address into read asynchronously.
    pub async fn read_async(&mut self, address: u8, read: &mut [u8]) -> Result<(), Error> {
        self.read_async_internal(address, read, false, true).await
    }

    /// Write to address from write asynchronously.
    pub async fn write_async(&mut self, address: u8, write: &[u8]) -> Result<(), Error> {
        self.write_async_internal(address, write, true).await
    }

    /// Write to address from write and read from address into read asynchronously.
    pub async fn write_read_async(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        self.write_async_internal(address, write, false).await?;
        self.read_async_internal(address, read, true, true).await
    }
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        pac::ECIA.src13().write_value(1 << T::irq_bit());

        T::regs().cfg().modify(|w| {
            w.set_enmi(false);
            w.set_enidi(false);
            w.set_en_aas(false);
        });

        T::waker().wake();
    }
}

impl<'d, T: Instance + 'd, M: Mode> I2c<'d, T, M> {
    fn new_inner(_peri: Peri<'d, T>, _scl: Peri<'d, AnyPin>, _sda: Peri<'d, AnyPin>, config: Config, port: u8) -> Self {
        let mut i2c = Self {
            _peri,
            _scl,
            _sda,
            config,
            port,
            phantom: PhantomData,
        };
        i2c.reset_reconfigure();
        i2c
    }

    fn reset_reconfigure(&mut self) {
        let r = T::regs();

        let addr1 = self.config.addr1.unwrap_or(0);
        let addr2 = self.config.addr2.unwrap_or(0);

        critical_section::with(|_| {
            // Reset the controller first.
            T::reset();

            r.cfg().write(|w| {
                w.set_flush_sxbuf(true);
                w.set_flush_srbuf(true);
                w.set_flush_mxbuf(true);
                w.set_flush_mrbuf(true);
            });

            r.wctrl().write(|w| w.set_pin(true));
            r.own_addr().write(|w| {
                w.set_addr1(addr1);
                w.set_addr2(addr2);
            });

            r.cfg().write(|w| {
                if addr1 == 0 || addr2 == 0 {
                    w.set_gc_dis(false);
                }
                w.set_port_sel(self.port);
                w.set_fen(true);
            });

            match self.config.speed {
                BusSpeed::Standard => {
                    r.busclk().write(|w| {
                        w.set_low_per(0x4f);
                        w.set_high_per(0x4f);
                    });
                    r.datatm().write(|w| {
                        w.set_data_hold(0x06);
                        w.set_restart_setup(0x50);
                        w.set_stop_setup(0x4d);
                        w.set_first_start_hold(0x0c);
                    });

                    r.rshtm().write(|w| w.set_rshtm(0x4d));
                    r.idlsc().write(|w| {
                        w.set_fair_bus_idl_min(0x01ed);
                        w.set_fair_idl_dly(0x01fc);
                    });
                    r.tmoutsc().write(|w| {
                        w.set_clk_high_tim_out(0xc7);
                        w.set_slv_cum_tim_out(0xc2);
                        w.set_mast_cum_tim_out(0x9c);
                        w.set_bus_idle_min(0x4b);
                    });
                }
                BusSpeed::Fast => {
                    r.busclk().write(|w| {
                        w.set_low_per(0x17);
                        w.set_high_per(0x0f);
                    });
                    r.datatm().write(|w| {
                        w.set_data_hold(0x06);
                        w.set_restart_setup(0x0a);
                        w.set_stop_setup(0x0a);
                        w.set_first_start_hold(0x04);
                    });

                    r.rshtm().write(|w| w.set_rshtm(0x0a));
                    r.idlsc().write(|w| {
                        w.set_fair_bus_idl_min(0x0050);
                        w.set_fair_idl_dly(0x0100);
                    });
                    r.tmoutsc().write(|w| {
                        w.set_clk_high_tim_out(0xc7);
                        w.set_slv_cum_tim_out(0xc2);
                        w.set_mast_cum_tim_out(0x9c);
                        w.set_bus_idle_min(0x15);
                    });
                }
                BusSpeed::FastPlus => {
                    r.busclk().write(|w| {
                        w.set_low_per(0x09);
                        w.set_high_per(0x05);
                    });
                    r.datatm().write(|w| {
                        w.set_data_hold(0x01);
                        w.set_restart_setup(0x06);
                        w.set_stop_setup(0x06);
                        w.set_first_start_hold(0x04);
                    });

                    r.rshtm().write(|w| w.set_rshtm(0x06));
                    r.idlsc().write(|w| {
                        w.set_fair_bus_idl_min(0x0050);
                        w.set_fair_idl_dly(0x0100);
                    });
                    r.tmoutsc().write(|w| {
                        w.set_clk_high_tim_out(0xc7);
                        w.set_slv_cum_tim_out(0xc2);
                        w.set_mast_cum_tim_out(0x9c);
                        w.set_bus_idle_min(0x08);
                    });
                }
            }

            r.wctrl().write(|w| {
                w.set_pin(true);
                w.set_eso(true);
                w.set_ack(true);
            });

            r.cfg().modify(|w| w.set_en(true));
        });

        while !r.rsts().read().nbb() {}

        // 6. Delay.
        //
        // Documentation states: Wait a time equal to the longest i2c
        // message to synchronize the NBB bit in multi-master systems
        // only.
        //
        // We're assuming that we're not in a multi-master scenario,
        // therefore skipping the delay.
    }

    fn wait_for_bus_free() {
        while !T::regs().rsts().read().nbb() {}
    }

    fn start(address: u8, rw: bool, repeated: bool) -> Result<(), Error> {
        if address >= 0x80 {
            return Err(Error::AddressOutOfRange(address));
        }

        let r = T::regs();
        let own_addr = r.own_addr().read();
        let (addr1, addr2) = (own_addr.addr1(), own_addr.addr2());

        if address == addr1 || address == addr2 {
            return Err(Error::AddressReserved(address));
        }

        if repeated {
            write_register_with_delay!(T::regs().wctrl(), |w| {
                w.set_eso(true);
                w.set_sta(true);
                w.set_ack(true);
            });
            T::regs().i2cdata().write_value(address << 1 | u8::from(rw));
        } else {
            if rw {
                T::regs().i2cdata().write_value(address << 1 | 1);
                Self::wait_for_bus_free();
            } else {
                Self::wait_for_bus_free();
                T::regs().i2cdata().write_value(address << 1 | 0);
            }

            write_register_with_delay!(T::regs().wctrl(), |w| {
                w.set_pin(true);
                w.set_eso(true);
                w.set_sta(true);
                w.set_ack(true);
            });
        }

        Ok(())
    }

    fn stop() {
        write_register_with_delay!(T::regs().wctrl(), |w| {
            w.set_pin(true);
            w.set_eso(true);
            w.set_sto(true);
            w.set_sta(false);
            w.set_ack(true);
        });
    }

    fn check_status() -> Result<(), Error> {
        while T::regs().rsts().read().pin() {}

        let status = T::regs().rsts().read();

        if status.lrb_ad0() {
            Self::stop();
            Err(Error::Abort(AbortReason::NoAcknowledge))
        } else if status.lab() {
            Err(Error::Abort(AbortReason::ArbitrationLoss))
        } else if status.ber() {
            let completion = T::regs().compl().read();

            if completion.dto() {
                Err(Error::BusTimeout(BusTimeoutReason::DeviceTimeout))
            } else if completion.mcto() {
                Err(Error::BusTimeout(BusTimeoutReason::MasterCumulativeTimeout))
            } else if completion.chdl() {
                Err(Error::BusTimeout(BusTimeoutReason::ClockHighDataLow))
            } else if completion.chdh() {
                Err(Error::BusTimeout(BusTimeoutReason::ClockHighDataHigh))
            } else {
                Err(Error::Abort(AbortReason::Other))
            }
        } else {
            Ok(())
        }
    }

    fn read_byte(last: bool) -> Result<u8, Error> {
        if !last {
            Self::check_status()?;
        } else {
            write_register_with_delay!(T::regs().wctrl(), |w| {
                w.set_eso(true);
            });
        }

        let byte = T::regs().i2cdata().read();

        if last {
            Self::check_status()?;
        }

        Ok(byte)
    }

    fn write_byte(byte: u8) -> Result<(), Error> {
        Self::check_status()?;
        T::regs().i2cdata().write_value(byte);
        Ok(())
    }

    fn read_blocking_internal(
        &mut self,
        address: u8,
        read: &mut [u8],
        repeated: bool,
        send_stop: bool,
    ) -> Result<(), Error> {
        if read.is_empty() {
            return Err(Error::InvalidReadBufferLength);
        }

        Self::start(address, true, repeated)?;

        // First byte in the FIFO is the slave address. Ignore it.
        let _ = Self::read_byte(false)?;

        let last = read.len() - 1;
        for (i, byte) in read.iter_mut().enumerate() {
            *byte = Self::read_byte(i == last)?;
        }

        if send_stop {
            Self::stop();
        }

        Ok(())
    }

    fn write_blocking_internal(&mut self, address: u8, write: &[u8], send_stop: bool) -> Result<(), Error> {
        if write.is_empty() {
            return Err(Error::InvalidWriteBufferLength);
        }

        Self::start(address, false, false)?;

        for byte in write.iter() {
            Self::write_byte(*byte)?;
        }

        if send_stop {
            Self::stop();
        }

        Ok(())
    }

    /// Read from address into read blocking caller until done.
    pub fn blocking_read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Error> {
        // TODO - empirically determined delay
        cortex_m::asm::delay(20_000);
        let retval = self.read_blocking_internal(address, read, false, true);
        retval
    }

    /// Write to address from write blocking caller until done.
    pub fn blocking_write(&mut self, address: u8, write: &[u8]) -> Result<(), Error> {
        // TODO - empirically determined delay
        cortex_m::asm::delay(20_000);
        let retval = self.write_blocking_internal(address, write, true);
        retval
    }

    /// Write to address from write and read from address into read blocking caller until done.
    pub fn blocking_write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        // TODO - empirically determined delay
        cortex_m::asm::delay(20_000);
        self.write_blocking_internal(address, write, false)?;
        // TODO - empirically determined delay
        cortex_m::asm::delay(20_000);
        self.read_blocking_internal(address, read, true, true)
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::i2c::Read for I2c<'d, T, M> {
    type Error = Error;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(address, buffer)
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::i2c::Write for I2c<'d, T, M> {
    type Error = Error;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(address, bytes)
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::i2c::WriteRead for I2c<'d, T, M> {
    type Error = Error;

    fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_write_read(address, bytes, buffer)
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::i2c::Transactional for I2c<'d, T, M> {
    type Error = Error;

    fn exec(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_02::blocking::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        for i in 0..operations.len() {
            let last = i == operations.len() - 1;
            match &mut operations[i] {
                embedded_hal_02::blocking::i2c::Operation::Read(buf) => {
                    self.read_blocking_internal(address, buf, false, last)?
                }
                embedded_hal_02::blocking::i2c::Operation::Write(buf) => {
                    self.write_blocking_internal(address, buf, last)?
                }
            }
        }
        Ok(())
    }
}

impl embedded_hal_1::i2c::Error for Error {
    fn kind(&self) -> embedded_hal_1::i2c::ErrorKind {
        match *self {
            Self::Abort(AbortReason::ArbitrationLoss) => embedded_hal_1::i2c::ErrorKind::ArbitrationLoss,
            Self::Abort(AbortReason::NoAcknowledge) => {
                embedded_hal_1::i2c::ErrorKind::NoAcknowledge(embedded_hal_1::i2c::NoAcknowledgeSource::Address)
            }
            Self::Abort(AbortReason::Other) => embedded_hal_1::i2c::ErrorKind::Other,
            Self::BusTimeout(_) => embedded_hal_1::i2c::ErrorKind::Bus,
            Self::InvalidReadBufferLength => embedded_hal_1::i2c::ErrorKind::Other,
            Self::InvalidWriteBufferLength => embedded_hal_1::i2c::ErrorKind::Other,
            Self::AddressOutOfRange(_) => embedded_hal_1::i2c::ErrorKind::Other,
            Self::AddressReserved(_) => embedded_hal_1::i2c::ErrorKind::Other,
        }
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_1::i2c::ErrorType for I2c<'d, T, M> {
    type Error = Error;
}

impl<'d, T: Instance, M: Mode> embedded_hal_1::i2c::I2c for I2c<'d, T, M> {
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
        operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        for i in 0..operations.len() {
            let last = i == operations.len() - 1;
            match &mut operations[i] {
                embedded_hal_1::i2c::Operation::Read(buf) => self.write_blocking_internal(address, buf, last)?,
                embedded_hal_1::i2c::Operation::Write(buf) => self.write_blocking_internal(address, buf, last)?,
            }
        }
        Ok(())
    }
}

impl<'d, A, T> embedded_hal_async::i2c::I2c<A> for I2c<'d, T, Async>
where
    A: embedded_hal_async::i2c::AddressMode + Into<u8> + 'static,
    T: Instance + 'd,
{
    async fn read(&mut self, address: A, read: &mut [u8]) -> Result<(), Self::Error> {
        self.read_async(address.into(), read).await
    }

    async fn write(&mut self, address: A, write: &[u8]) -> Result<(), Self::Error> {
        self.write_async(address.into(), write).await
    }

    async fn write_read(&mut self, address: A, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        self.write_read_async(address.into(), write, read).await
    }

    async fn transaction(
        &mut self,
        address: A,
        operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        use embedded_hal_1::i2c::Operation;

        let addr: u8 = address.into();
        let mut iterator = operations.iter_mut();

        while let Some(op) = iterator.next() {
            let last = iterator.len() == 0;

            match op {
                Operation::Read(buffer) => {
                    self.read_async_internal(addr, buffer, false, last).await?;
                }
                Operation::Write(buffer) => {
                    self.write_async_internal(addr, buffer, last).await?;
                }
            }
        }
        Ok(())
    }
}

trait SealedInstance {
    fn regs() -> smb0::Smb0;
    fn waker() -> &'static AtomicWaker;
    fn irq_bit() -> usize;
    fn reset();
}

trait SealedMode {}

/// Drivermode.
#[allow(private_bounds)]
pub trait Mode: SealedMode {}

macro_rules! impl_mode {
    ($mode:ident) => {
        impl SealedMode for $mode {}
        impl Mode for $mode {}
    };
}

/// Blocking mode.
pub struct Blocking;

/// Async mode.
pub struct Async;

impl_mode!(Blocking);
impl_mode!(Async);

/// I2C instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

// Ideally, these should be part of a dedicated PCR driver
const LOCK: u32 = 0xa638_2d4d;
const UNLOCK: u32 = 0xa638_2d4c;

macro_rules! impl_instance {
    ($peri:ident, $bit:expr, $irq:ident, $reset:expr) => {
        impl SealedInstance for peripherals::$peri {
            #[inline(always)]
            fn regs() -> smb0::Smb0 {
                pac::$peri
            }

            #[inline(always)]
            fn waker() -> &'static AtomicWaker {
                static WAKER: AtomicWaker = AtomicWaker::new();
                &WAKER
            }

            #[inline(always)]
            fn irq_bit() -> usize {
                $bit
            }

            #[inline(always)]
            fn reset() {
                fn internal_reset(f: impl FnOnce(pac::pcr::Pcr)) {
                    pac::PCR.lock_reg().write(|w| w.set_pcr_rst_en_lock(UNLOCK));
                    f(pac::PCR);
                    pac::PCR.lock_reg().write(|w| w.set_pcr_rst_en_lock(LOCK));
                }

                internal_reset($reset);
            }
        }

        impl Instance for peripherals::$peri {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}

impl_instance!(SMB0, 0, I2CSMB0, |pcr| pcr
    .rst_en_1()
    .write(|w| w.set_smb0_rst_en(true)));
impl_instance!(SMB1, 1, I2CSMB1, |pcr| pcr
    .rst_en_3()
    .write(|w| w.set_smb1_rst_en(true)));
impl_instance!(SMB2, 2, I2CSMB2, |pcr| pcr
    .rst_en_3()
    .write(|w| w.set_smb2_rst_en(true)));
impl_instance!(SMB3, 3, I2CSMB3, |pcr| pcr
    .rst_en_3()
    .write(|w| w.set_smb3_rst_en(true)));
impl_instance!(SMB4, 4, I2CSMB4, |pcr| pcr
    .rst_en_3()
    .write(|w| w.set_smb_4_rst_en(true)));

/// SDA pin.
pub trait SdaPin: crate::gpio::Pin {
    fn setup(&self);
}

/// SCL pin.
pub trait SclPin: crate::gpio::Pin {
    fn setup(&self);
}

macro_rules! impl_pin {
    ($function:ident, $($pin:ident, $mux:ident),*) => {
	$(
            impl $function for peripherals::$pin {
		#[inline(always)]
		fn setup(&self) {
		    critical_section::with(|_| {
			self.regs().ctrl1.modify(|w| {
			    w.set_out_buff_type(crate::pac::BufferType::OPEN_DRAIN);
                            w.set_inp_dis(false);
			    w.set_alt_data(true);
			    w.set_mux_ctrl(crate::pac::Function::$mux);
			})
		    });
		}
	    }
	)*
    };
}

#[rustfmt::skip]
impl_pin!(
    SdaPin,
    GPIO0, F3,
    GPIO3, F1,
    GPIO5, F1,
    GPIO7, F1,
    GPIO12, F1,
    GPIO26, F3,
    GPIO30, F2,
    GPIO66, F2,
    GPIO70, F2,
    GPIO72, F2,
    GPIO130, F1,
    GPIO132, F1,
    GPIO141, F1,
    GPIO143, F1,
    GPIO145, F1,
    GPIO147, F1,
    GPIO152, F3,
    GPIO154, F1,
    GPIO231, F1
);

#[rustfmt::skip]
impl_pin!(
    SclPin,
    GPIO4, F1,
    GPIO6, F1,
    GPIO10, F1,
    GPIO13, F1,
    GPIO24, F3,
    GPIO27, F3,
    GPIO62, F2,
    GPIO65, F2,
    GPIO71, F2,
    GPIO73, F2,
    GPIO107, F4,
    GPIO131, F1,
    GPIO140, F1,
    GPIO142, F1,
    GPIO144, F4,
    GPIO146, F1,
    GPIO150, F1,
    GPIO155, F1,
    GPIO230, F1
);

trait SealedValidI2cConfig {
    fn port() -> u8;
}

/// A marker trait implemented for valid configurations
#[allow(private_bounds)]
pub trait ValidI2cConfig: SealedValidI2cConfig {}

macro_rules! impl_config {
    ($scl:ident, $sda:ident, $port:expr) => {
        impl SealedValidI2cConfig for (peripherals::SMB0, peripherals::$scl, peripherals::$sda) {
            #[inline(always)]
            fn port() -> u8 {
                $port
            }
        }
        impl SealedValidI2cConfig for (peripherals::SMB1, peripherals::$scl, peripherals::$sda) {
            #[inline(always)]
            fn port() -> u8 {
                $port
            }
        }
        impl SealedValidI2cConfig for (peripherals::SMB2, peripherals::$scl, peripherals::$sda) {
            #[inline(always)]
            fn port() -> u8 {
                $port
            }
        }
        impl SealedValidI2cConfig for (peripherals::SMB3, peripherals::$scl, peripherals::$sda) {
            #[inline(always)]
            fn port() -> u8 {
                $port
            }
        }
        impl SealedValidI2cConfig for (peripherals::SMB4, peripherals::$scl, peripherals::$sda) {
            #[inline(always)]
            fn port() -> u8 {
                $port
            }
        }

        impl ValidI2cConfig for (peripherals::SMB0, peripherals::$scl, peripherals::$sda) {}
        impl ValidI2cConfig for (peripherals::SMB1, peripherals::$scl, peripherals::$sda) {}
        impl ValidI2cConfig for (peripherals::SMB2, peripherals::$scl, peripherals::$sda) {}
        impl ValidI2cConfig for (peripherals::SMB3, peripherals::$scl, peripherals::$sda) {}
        impl ValidI2cConfig for (peripherals::SMB4, peripherals::$scl, peripherals::$sda) {}
    };
}

// I2C00
impl_config!(GPIO4, GPIO3, 0);

// I2C01
impl_config!(GPIO73, GPIO72, 1);
impl_config!(GPIO131, GPIO130, 1);

// I2C02
impl_config!(GPIO155, GPIO154, 2);

// I2C03
impl_config!(GPIO10, GPIO7, 3);

// I2C04
impl_config!(GPIO144, GPIO143, 4);

// I2C05
impl_config!(GPIO142, GPIO141, 5);

// I2C06
impl_config!(GPIO140, GPIO132, 6);

// I2C07
impl_config!(GPIO13, GPIO12, 7);
impl_config!(GPIO24, GPIO152, 7);

// I2C08
impl_config!(GPIO230, GPIO231, 8);

// I2C09
impl_config!(GPIO146, GPIO145, 9);

// I2C10
impl_config!(GPIO107, GPIO30, 10);

// I2C11
impl_config!(GPIO62, GPIO0, 11);
impl_config!(GPIO6, GPIO5, 11);

// I2C12
impl_config!(GPIO27, GPIO26, 12);

// I2C13
impl_config!(GPIO65, GPIO66, 13);

// I2C14
impl_config!(GPIO71, GPIO70, 14);

// I2C15
impl_config!(GPIO150, GPIO147, 15);
