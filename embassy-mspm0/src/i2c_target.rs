//! Inter-Integrated-Circuit (I2C) Target
// The following code is modified from embassy-stm32 and embassy-rp
// https://github.com/embassy-rs/embassy/tree/main/embassy-stm32
// https://github.com/embassy-rs/embassy/tree/main/embassy-rp

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::Ordering;
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use mspm0_metapac::i2c::vals::CpuIntIidxStat;

use crate::gpio::{AnyPin, SealedPin};
// Re-use I2c controller types
use crate::i2c::{ClockSel, ConfigError, Info, Instance, InterruptHandler, SclPin, SdaPin, State};
use crate::interrupt::InterruptExt;
use crate::mode::{Async, Blocking, Mode};
use crate::pac::i2c::vals;
use crate::pac::{self};
use crate::{Peri, i2c, i2c_target, interrupt};

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Config
pub struct Config {
    /// 7-bit Target Address
    pub target_addr: u8,

    /// Control if the target should ack to and report general calls.
    pub general_call: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target_addr: 0x48,
            general_call: false,
        }
    }
}

/// I2C error
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// User passed in a response buffer that was 0 length
    InvalidResponseBufferLength,
    /// The response buffer length was too short to contain the message
    ///
    /// The length parameter will always be the length of the buffer, and is
    /// provided as a convenience for matching alongside `Command::Write`.
    PartialWrite(usize),
    /// The response buffer length was too short to contain the message
    ///
    /// The length parameter will always be the length of the buffer, and is
    /// provided as a convenience for matching alongside `Command::GeneralCall`.
    PartialGeneralCall(usize),
}

/// Received command from the controller.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
    /// General Call Write: Controller sent the General Call address (0x00) followed by data.
    /// Contains the number of bytes written by the controller.
    GeneralCall(usize),
    /// Read: Controller wants to read data from the target.
    Read,
    /// Write: Controller sent the target's address followed by data.
    /// Contains the number of bytes written by the controller.
    Write(usize),
    /// Write followed by Read (Repeated Start): Controller wrote data, then issued a repeated
    /// start and wants to read data. Contains the number of bytes written before the read.
    WriteRead(usize),
}

/// Status after responding to a controller read request.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReadStatus {
    /// Transaction completed successfully. The controller either NACKed the last byte
    /// or sent a STOP condition.
    Done,
    /// Transaction incomplete, controller trying to read more bytes than were provided
    NeedMoreBytes,
    /// Transaction complete, but controller stopped reading bytes before we ran out
    LeftoverBytes(u16),
}

/// I2C Target driver.
// Use the same Instance, SclPin, SdaPin traits as the controller
pub struct I2cTarget<'d, M: Mode> {
    info: &'static Info,
    state: &'static State,
    scl: Option<Peri<'d, AnyPin>>,
    sda: Option<Peri<'d, AnyPin>>,
    config: i2c::Config,
    target_config: i2c_target::Config,
    _phantom: PhantomData<M>,
}

impl<'d> SetConfig for I2cTarget<'d, Async> {
    type Config = (i2c::Config, i2c_target::Config);
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.info.interrupt.disable();

        if let Some(ref sda) = self.sda {
            sda.update_pf(config.0.sda_pf());
        }

        if let Some(ref scl) = self.scl {
            scl.update_pf(config.0.scl_pf());
        }

        self.config = config.0.clone();
        self.target_config = config.1.clone();

        self.reset()
    }
}

impl<'d> SetConfig for I2cTarget<'d, Blocking> {
    type Config = (i2c::Config, i2c_target::Config);
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        if let Some(ref sda) = self.sda {
            sda.update_pf(config.0.sda_pf());
        }

        if let Some(ref scl) = self.scl {
            scl.update_pf(config.0.scl_pf());
        }

        self.config = config.0.clone();
        self.target_config = config.1.clone();

        self.reset()
    }
}

impl<'d> I2cTarget<'d, Async> {
    /// Create a new asynchronous I2C target driver using interrupts
    /// The `config` reuses the i2c controller config to setup the clock while `target_config`
    /// configures i2c target specific parameters.
    pub fn new<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: i2c::Config,
        target_config: i2c_target::Config,
    ) -> Result<Self, ConfigError> {
        let mut this = Self::new_inner(
            peri,
            new_pin!(scl, config.scl_pf()),
            new_pin!(sda, config.sda_pf()),
            config,
            target_config,
        );
        this.reset()?;
        Ok(this)
    }

    /// Reset the i2c peripheral. If you cancel a respond_to_read, you may stall the bus.
    /// You can recover the bus by calling this function, but doing so will almost certainly cause
    /// an i/o error in the controller.
    pub fn reset(&mut self) -> Result<(), ConfigError> {
        self.init()?;
        unsafe { self.info.interrupt.enable() };
        Ok(())
    }
}

impl<'d> I2cTarget<'d, Blocking> {
    /// Create a new blocking I2C target driver.
    /// The `config` reuses the i2c controller config to setup the clock while `target_config`
    /// configures i2c target specific parameters.
    pub fn new_blocking<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        config: i2c::Config,
        target_config: i2c_target::Config,
    ) -> Result<Self, ConfigError> {
        let mut this = Self::new_inner(
            peri,
            new_pin!(scl, config.scl_pf()),
            new_pin!(sda, config.sda_pf()),
            config,
            target_config,
        );
        this.reset()?;
        Ok(this)
    }

    /// Reset the i2c peripheral. If you cancel a respond_to_read, you may stall the bus.
    /// You can recover the bus by calling this function, but doing so will almost certainly cause
    /// an i/o error in the controller.
    pub fn reset(&mut self) -> Result<(), ConfigError> {
        self.init()?;
        Ok(())
    }
}

impl<'d, M: Mode> I2cTarget<'d, M> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        scl: Option<Peri<'d, AnyPin>>,
        sda: Option<Peri<'d, AnyPin>>,
        config: i2c::Config,
        target_config: i2c_target::Config,
    ) -> Self {
        if let Some(ref scl) = scl {
            let pincm = pac::IOMUX.pincm(scl._pin_cm() as usize);
            pincm.modify(|w| {
                w.set_hiz1(true);
            });
        }
        if let Some(ref sda) = sda {
            let pincm = pac::IOMUX.pincm(sda._pin_cm() as usize);
            pincm.modify(|w| {
                w.set_hiz1(true);
            });
        }

        Self {
            info: T::info(),
            state: T::state(),
            scl,
            sda,
            config,
            target_config,
            _phantom: PhantomData,
        }
    }

    fn init(&mut self) -> Result<(), ConfigError> {
        let mut config = self.config;
        let target_config = self.target_config;
        let regs = self.info.regs;

        config.check_config()?;
        // Target address must be 7-bit
        if !(target_config.target_addr < 0x80) {
            return Err(ConfigError::InvalidTargetAddress);
        }

        regs.target(0).tctr().modify(|w| {
            w.set_active(false);
        });

        // Init power for I2C
        regs.gprcm(0).rstctl().write(|w| {
            w.set_resetstkyclr(true);
            w.set_resetassert(true);
            w.set_key(vals::ResetKey::KEY);
        });

        regs.gprcm(0).pwren().write(|w| {
            w.set_enable(true);
            w.set_key(vals::PwrenKey::KEY);
        });

        self.info.interrupt.disable();

        // Init delay from the M0 examples by TI in CCStudio (16 cycles)
        cortex_m::asm::delay(16);

        // Select and configure the I2C clock using the CLKSEL and CLKDIV registers
        regs.clksel().write(|w| match config.clock_source {
            ClockSel::BusClk => {
                w.set_mfclk_sel(false);
                w.set_busclk_sel(true);
            }
            ClockSel::MfClk => {
                w.set_mfclk_sel(true);
                w.set_busclk_sel(false);
            }
        });
        regs.clkdiv().write(|w| w.set_ratio(config.clock_div.into()));

        // Configure at least one target address by writing the 7-bit address to I2Cx.SOAR register. The additional
        // target address can be enabled and configured by using I2Cx.TOAR2 register.
        regs.target(0).toar().modify(|w| {
            w.set_oaren(true);
            w.set_oar(target_config.target_addr as u16);
        });

        self.state
            .clock
            .store(config.calculate_clock_source(), Ordering::Relaxed);

        regs.target(0).tctr().modify(|w| {
            w.set_gencall(target_config.general_call);
            w.set_tclkstretch(true);
            // Disable target wakeup, follow TI example. (TI note: Workaround for errata I2C_ERR_04.)
            w.set_twuen(false);
            w.set_txempty_on_treq(true);
        });

        // Enable the I2C target mode by setting the ACTIVE bit in I2Cx.TCTR register.
        regs.target(0).tctr().modify(|w| {
            w.set_active(true);
        });

        Ok(())
    }

    #[inline(always)]
    fn drain_fifo(&mut self, buffer: &mut [u8], offset: &mut usize) {
        let regs = self.info.regs;

        for b in &mut buffer[*offset..] {
            if regs.target(0).tfifosr().read().rxfifocnt() == 0 {
                break;
            }

            *b = regs.target(0).trxdata().read().value();
            *offset += 1;
        }
    }

    /// Blocking function to empty the tx fifo
    ///
    /// This function can be used to empty the transmit FIFO if data remains after handling a 'read' command (LeftoverBytes).
    pub fn flush_tx_fifo(&mut self) {
        self.info.regs.target(0).tfifoctl().modify(|w| {
            w.set_txflush(true);
        });
        while self.info.regs.target(0).tfifosr().read().txfifocnt() as usize != self.info.fifo_size {}
        self.info.regs.target(0).tfifoctl().modify(|w| {
            w.set_txflush(false);
        });
    }
}

impl<'d> I2cTarget<'d, Async> {
    /// Wait asynchronously for commands from an I2C controller.
    /// `buffer` is provided in case controller does a 'write', 'write read', or 'general call' and is unused for 'read'.
    pub async fn listen(&mut self, buffer: &mut [u8]) -> Result<Command, Error> {
        let regs = self.info.regs;

        let mut len = 0;

        // Set the rx fifo interrupt to avoid a fifo overflow
        regs.target(0).tfifoctl().modify(|r| {
            r.set_rxtrig(vals::TfifoctlRxtrig::LEVEL_6);
        });

        self.wait_on(
            |me| {
                // Check if address matches the General Call address (0x00)
                let is_gencall = regs.target(0).tsr().read().addrmatch() == 0;

                if regs.target(0).tfifosr().read().rxfifocnt() > 0 {
                    me.drain_fifo(buffer, &mut len);
                }

                if buffer.len() == len && regs.target(0).tfifosr().read().rxfifocnt() > 0 {
                    if is_gencall {
                        return Poll::Ready(Err(Error::PartialGeneralCall(buffer.len())));
                    } else {
                        return Poll::Ready(Err(Error::PartialWrite(buffer.len())));
                    }
                }

                let iidx = regs.cpu_int(0).iidx().read().stat();
                trace!("ls:{} len:{}", iidx as u8, len);
                let result = match iidx {
                    CpuIntIidxStat::TTXEMPTY => match len {
                        0 => Poll::Ready(Ok(Command::Read)),
                        w => Poll::Ready(Ok(Command::WriteRead(w))),
                    },
                    CpuIntIidxStat::TSTOPFG => match (is_gencall, len) {
                        (_, 0) => Poll::Pending,
                        (true, w) => Poll::Ready(Ok(Command::GeneralCall(w))),
                        (false, w) => Poll::Ready(Ok(Command::Write(w))),
                    },
                    _ => Poll::Pending,
                };
                if !result.is_pending() {
                    regs.cpu_int(0).imask().write(|_| {});
                }
                result
            },
            |_me| {
                regs.cpu_int(0).imask().write(|_| {});
                regs.cpu_int(0).imask().modify(|w| {
                    w.set_tgencall(true);
                    w.set_trxfifotrg(true);
                    w.set_tstop(true);
                    w.set_ttxempty(true);
                });
            },
        )
        .await
    }

    /// Respond to an I2C controller 'read' command, asynchronously.
    pub async fn respond_to_read(&mut self, buffer: &[u8]) -> Result<ReadStatus, Error> {
        if buffer.is_empty() {
            return Err(Error::InvalidResponseBufferLength);
        }

        let regs = self.info.regs;
        let fifo_size = self.info.fifo_size;
        let mut chunks = buffer.chunks(self.info.fifo_size);

        self.wait_on(
            |_me| {
                if let Some(chunk) = chunks.next() {
                    for byte in chunk {
                        regs.target(0).ttxdata().write(|w| w.set_value(*byte));
                    }

                    return Poll::Pending;
                }

                let iidx = regs.cpu_int(0).iidx().read().stat();
                let fifo_bytes = fifo_size - regs.target(0).tfifosr().read().txfifocnt() as usize;
                trace!("rs:{}, fifo:{}", iidx as u8, fifo_bytes);

                let result = match iidx {
                    CpuIntIidxStat::TTXEMPTY => Poll::Ready(Ok(ReadStatus::NeedMoreBytes)),
                    CpuIntIidxStat::TSTOPFG => match fifo_bytes {
                        0 => Poll::Ready(Ok(ReadStatus::Done)),
                        w => Poll::Ready(Ok(ReadStatus::LeftoverBytes(w as u16))),
                    },
                    _ => Poll::Pending,
                };
                if !result.is_pending() {
                    regs.cpu_int(0).imask().write(|_| {});
                }
                result
            },
            |_me| {
                regs.cpu_int(0).imask().write(|_| {});
                regs.cpu_int(0).imask().modify(|w| {
                    w.set_ttxempty(true);
                    w.set_tstop(true);
                });
            },
        )
        .await
    }

    /// Respond to reads with the fill byte until the controller stops asking
    pub async fn respond_till_stop(&mut self, fill: u8) -> Result<(), Error> {
        // The buffer size could be increased to reduce interrupt noise but has higher probability
        // of LeftoverBytes
        let buff = [fill];
        loop {
            match self.respond_to_read(&buff).await {
                Ok(ReadStatus::NeedMoreBytes) => (),
                Ok(_) => break Ok(()),
                Err(e) => break Err(e),
            }
        }
    }

    /// Respond to a controller read, then fill any remaining read bytes with `fill`
    pub async fn respond_and_fill(&mut self, buffer: &[u8], fill: u8) -> Result<ReadStatus, Error> {
        let resp_stat = self.respond_to_read(buffer).await?;

        if resp_stat == ReadStatus::NeedMoreBytes {
            self.respond_till_stop(fill).await?;
            Ok(ReadStatus::Done)
        } else {
            Ok(resp_stat)
        }
    }

    /// Calls `f` to check if we are ready or not.
    /// If not, `g` is called once(to eg enable the required interrupts).
    /// The waker will always be registered prior to calling `f`.
    #[inline(always)]
    async fn wait_on<F, U, G>(&mut self, mut f: F, mut g: G) -> U
    where
        F: FnMut(&mut Self) -> Poll<U>,
        G: FnMut(&mut Self),
    {
        poll_fn(|cx| {
            // Register prior to checking the condition
            self.state.waker.register(cx.waker());
            let r = f(self);

            if r.is_pending() {
                g(self);
            }

            r
        })
        .await
    }
}

impl<'d, M: Mode> Drop for I2cTarget<'d, M> {
    fn drop(&mut self) {
        // Ensure peripheral is disabled and pins are reset
        self.info.regs.target(0).tctr().modify(|w| w.set_active(false));

        self.scl.as_ref().map(|x| x.set_as_disconnected());
        self.sda.as_ref().map(|x| x.set_as_disconnected());
    }
}
