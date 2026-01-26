//! I3C Controller driver.

use core::marker::PhantomData;

use embassy_hal_internal::Peri;
use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::interrupt::InterruptExt;

use super::{Async, Blocking, Error, Info, InterruptHandler, Mode, SclPin, SdaPin};
use crate::clocks::periph_helpers::{Div4, I3cClockSel, I3cConfig};
use crate::clocks::{PoweredClock, WakeGuard, enable_and_reset};
use crate::gpio::{AnyPin, SealedPin};
pub use crate::i2c::controller::Speed;
use crate::interrupt::typelevel;
use crate::pac::i3c0::mctrl::{Dir as I3cDir, Type};
use crate::peripherals::I3C0;

const MAX_CHUNK_SIZE: usize = 255;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum SendStop {
    #[default]
    No,
    Yes,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(dead_code)]
pub enum BusType {
    /// I3C SDR
    #[default]
    I3cSdr,
    /// Legacy I2C
    I2c,
    /// I3C DDR
    I3cDdr,
}

impl From<BusType> for Type {
    fn from(value: BusType) -> Self {
        match value {
            BusType::I3cSdr => Self::I3c,
            BusType::I2c => Self::I2c,
            BusType::I3cDdr => Self::Ddr,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum Dir {
    #[default]
    Write,
    Read,
}

impl From<Dir> for I3cDir {
    fn from(value: Dir) -> Self {
        match value {
            Dir::Write => Self::Dirwrite,
            Dir::Read => Self::Dirread,
        }
    }
}

/// I3C controller configuration
#[non_exhaustive]
pub struct Config {
    /// I3C push-pull bus frequency in Hz.
    pub push_pull_freq: u32,

    /// I3C open-drain frequency in Hz.
    pub open_drain_freq: u32,

    /// I2C bus speed
    pub i2c_speed: Speed,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            push_pull_freq: 1_500_000,
            open_drain_freq: 750_000,
            i2c_speed: Speed::Fast,
        }
    }
}

fn calculate_error(cur_freq: u32, desired_freq: u32) -> u32 {
    let delta = cur_freq.abs_diff(desired_freq);
    delta * 100 / desired_freq
}

/// I3C controller driver.
pub struct I3c<'d, M: Mode> {
    info: &'static Info,
    _scl: Peri<'d, AnyPin>,
    _sda: Peri<'d, AnyPin>,
    fclk: u32,
    _wg: Option<WakeGuard>,
    _phantom: PhantomData<M>,
}

impl<'d, M: Mode> I3c<'d, M> {
    fn new_inner(
        _peri: Peri<'d, I3C0>,
        scl: Peri<'d, impl SclPin<I3C0>>,
        sda: Peri<'d, impl SdaPin<I3C0>>,
        config: Config,
    ) -> Result<Self, Error> {
        let (power, source, div) = Self::clock_config();

        // Enable clocks
        let conf = I3cConfig { power, source, div };

        let parts = unsafe { enable_and_reset::<I3C0>(&conf).map_err(Error::ClockSetup)? };

        scl.mux();
        sda.mux();

        let _scl = scl.into();
        let _sda = sda.into();

        let inst = Self {
            info: super::info(),
            _scl,
            _sda,
            fclk: parts.freq,
            _wg: parts.wake_guard,
            _phantom: PhantomData,
        };

        inst.set_configuration(&config)?;

        Ok(inst)
    }

    // REVISIT: turn this into a function of the speed parameter
    fn clock_config() -> (PoweredClock, I3cClockSel, Div4) {
        (
            PoweredClock::NormalEnabledDeepSleepDisabled,
            I3cClockSel::FroLfDiv,
            const { Div4::no_div() },
        )
    }

    fn set_configuration(&self, config: &Config) -> Result<(), Error> {
        self.clear_flags();

        self.info.regs().mdatactrl().modify(|_, w| {
            w.flushtb()
                .flush()
                .flushfb()
                .flush()
                .unlock()
                .set_bit()
                .txtrig()
                .full_or_less()
                .rxtrig()
                .not_empty()
        });

        let (ppbaud, odbaud, i2cbaud) = self.calculate_baud_rate_params(config)?;

        self.info.regs().mconfig().write(|w| {
            unsafe {
                w.ppbaud()
                    .bits(ppbaud as u8)
                    .odbaud()
                    .bits(odbaud as u8)
                    .i2cbaud()
                    .bits(i2cbaud as u8)
            }
            .mstena()
            .master_on()
            .disto()
            .clear_bit()
            .hkeep()
            .none()
            .odstop()
            .disable()
            .odhpp()
            .enable()
        });

        Ok(())
    }

    // REVISIT: not very readable
    fn calculate_baud_rate_params(&self, config: &Config) -> Result<(u32, u32, u32), Error> {
        const NSEC_PER_SEC: u32 = 1_000_000_000;

        let fclk = self.fclk;

        let target_pp_hz = config.push_pull_freq;

        if target_pp_hz == 0 {
            return Err(Error::InvalidConfiguration);
        }

        let max_pp_hz = target_pp_hz + target_pp_hz / 10;

        let target_od_hz = config.open_drain_freq;
        let max_od_hz = target_od_hz + target_od_hz / 10;

        let target_i2c_hz = <Speed as Into<u32>>::into(config.i2c_speed);

        /* -------------------------------------------------------------
         * 1) Push‑Pull baud (PPBAUD)
         *    Generated from fclk / 2
         * ------------------------------------------------------------- */

        let mut pp_src_hz = fclk / 2;

        let mut pp_div = (pp_src_hz / target_pp_hz).max(1);
        if pp_src_hz / pp_div > max_pp_hz {
            pp_div += 1;
        }

        let pp_baud = pp_div - 1;
        pp_src_hz /= pp_div;

        let pp_low_ns = NSEC_PER_SEC / (2 * pp_src_hz);

        /* -------------------------------------------------------------
         * 2) Open‑Drain baud (ODBAUD)
         *    Depends on ODHPP mode
         * ------------------------------------------------------------- */

        let odhpp_enabled = self.info.regs().mconfig().read().odhpp().bit_is_set();

        let (od_baud, _od_src_hz) = if odhpp_enabled {
            // OD rate derived from 2×PP clock
            let mut div = ((2 * pp_src_hz) / target_od_hz).max(2);
            if (2 * pp_src_hz) / div > max_od_hz {
                div += 1;
            }

            (div - 2, (2 * pp_src_hz) / div)
        } else {
            // OD rate derived directly
            let mut div = (pp_src_hz / target_od_hz).max(1);
            if pp_src_hz / div > max_od_hz {
                div += 1;
            }

            (div - 1, pp_src_hz / div)
        };

        let od_low_ns = (od_baud + 1) * pp_low_ns;

        /* -------------------------------------------------------------
         * 3) I²C baud selection
         *    Choose even/odd divider with lowest error
         * ------------------------------------------------------------- */

        let even_div = ((fclk / target_i2c_hz) / (2 * (pp_baud + 1) * (od_baud + 1))).max(1);
        let even_rate = NSEC_PER_SEC / (2 * even_div * od_low_ns);
        let even_error = calculate_error(even_rate, target_i2c_hz);

        let odd_div = (((fclk / target_i2c_hz) / ((pp_baud + 1) * (od_baud + 1) - 1)) / 2).max(1);
        let odd_rate = NSEC_PER_SEC / ((2 * odd_div + 1) * od_low_ns);
        let odd_error = calculate_error(odd_rate, target_i2c_hz);

        let i2c_baud = if even_error < 10 || odd_error < 10 {
            if even_error < odd_error {
                (even_div - 1) * 2
            } else {
                (odd_div - 1) * 2 + 1
            }
        } else {
            if pp_src_hz / even_div < target_i2c_hz {
                (even_div - 1) * 2
            } else {
                even_div * 2
            }
        };

        Ok((pp_baud, od_baud, i2c_baud))
    }

    fn blocking_remediation(&self, bus_type: BusType) {
        // if the FIFO is not empty, drop its contents.
        if self.info.regs().mdatactrl().read().txcount() != 0 {
            self.info
                .regs()
                .mdatactrl()
                .modify(|_, w| w.flushtb().flush().flushfb().flush());
        }

        // send a stop command
        let _ = self.blocking_stop(bus_type);
    }

    fn clear_flags(&self) {
        self.info.regs().mstatus().write(|w| {
            w.slvstart()
                .clear_bit_by_one()
                .mctrldone()
                .clear_bit_by_one()
                .complete()
                .clear_bit_by_one()
                .ibiwon()
                .clear_bit_by_one()
                .nowmaster()
                .clear_bit_by_one()
        });
    }

    fn blocking_wait_for_ctrldone(&self) {
        while self.info.regs().mstatus().read().mctrldone().is_not_done() {}
    }

    fn blocking_wait_for_complete(&self) {
        while self.info.regs().mstatus().read().complete().is_not_complete() {}
    }

    fn blocking_wait_for_tx_fifo(&self) {
        while self.info.regs().mdatactrl().read().txfull().is_full() {}
    }

    fn blocking_wait_for_rx_fifo(&self) {
        while self.info.regs().mdatactrl().read().rxempty().is_empty() {}
    }

    fn status(&self) -> Result<(), Error> {
        if self.info.regs().mstatus().read().errwarn().is_error() {
            let merrwarn = self.info.regs().merrwarn().read();

            if merrwarn.urun().is_error() {
                Err(Error::Underrun)
            } else if merrwarn.nack().is_error() {
                Err(Error::Nack)
            } else if merrwarn.wrabt().is_error() {
                Err(Error::WriteAbort)
            } else if merrwarn.term().is_error() {
                Err(Error::Terminate)
            } else if merrwarn.hpar().is_error() {
                Err(Error::HighDataRateParity)
            } else if merrwarn.hcrc().is_error() {
                Err(Error::HighDataRateCrc)
            } else if merrwarn.oread().is_error() {
                Err(Error::Overread)
            } else if merrwarn.owrite().is_error() {
                Err(Error::Overwrite)
            } else if merrwarn.msgerr().is_error() {
                Err(Error::Message)
            } else if merrwarn.invreq().is_error() {
                Err(Error::InvalidRequest)
            } else if merrwarn.timeout().is_error() {
                Err(Error::Timeout)
            } else {
                // should never happen
                Err(Error::Other)
            }
        } else {
            Ok(())
        }
    }

    /// Prepares an appropriate Start condition on bus by issuing a
    /// `Start` request together with the device address, bus type
    /// (i3c sdr, i3c ddr, or i2c), and R/w bit.
    fn blocking_start(&self, address: u8, bus_type: BusType, dir: Dir, len: u8) -> Result<(), Error> {
        self.clear_flags();

        self.info.regs().mctrl().write(|w| {
            unsafe { w.addr().bits(address).rdterm().bits(len) }
                .type_()
                .variant(bus_type.into())
                .request()
                .emitstartaddr()
                .dir()
                .variant(dir.into())
                .ibiresp()
                .ack()
        });

        self.blocking_wait_for_ctrldone();
        self.status()
    }

    /// Prepares a Stop condition on the bus.
    ///
    /// Analogous to `start`, this blocks waiting for space in the
    /// FIFO to become available, then sends the command and blocks
    /// waiting for the FIFO to become empty ensuring the command was
    /// sent.
    fn blocking_stop(&self, bus_type: BusType) -> Result<(), Error> {
        if !self.info.regs().mstatus().read().state().is_normact() {
            Err(Error::InvalidRequest)
        } else {
            // NOTE: Section 41.3.2.1 states that "when sending STOP
            // in I2C mode, MCONFIG[ODSTOP] and MCTRL[TYPE] must be
            // 1".
            if bus_type == BusType::I2c {
                self.info.regs().mconfig().modify(|_, w| w.odstop().enable());
                self.info
                    .regs()
                    .mctrl()
                    .write(|w| w.request().emitstop().type_().variant(bus_type.into()));
            } else {
                self.info.regs().mconfig().modify(|_, w| w.odstop().disable());
                self.info
                    .regs()
                    .mctrl()
                    .write(|w| w.request().emitstop().type_().variant(bus_type.into()));
            }
            self.blocking_wait_for_ctrldone();
            self.status()
        }
    }

    fn blocking_read_internal(
        &self,
        address: u8,
        read: &mut [u8],
        bus_type: BusType,
        send_stop: SendStop,
    ) -> Result<(), Error> {
        if read.is_empty() {
            return Err(Error::InvalidReadBufferLength);
        }

        for chunk in read.chunks_mut(MAX_CHUNK_SIZE) {
            match self.blocking_start(address, bus_type, Dir::Read, chunk.len() as u8) {
                Err(e) => {
                    self.blocking_remediation(bus_type);
                    return Err(e);
                }
                _ => {}
            };

            for byte in chunk.iter_mut() {
                self.blocking_wait_for_rx_fifo();
                *byte = self.info.regs().mrdatab().read().value().bits();
            }
        }

        if send_stop == SendStop::Yes {
            self.blocking_stop(bus_type)?;
        }

        Ok(())
    }

    fn blocking_write_internal(
        &self,
        address: u8,
        write: &[u8],
        bus_type: BusType,
        send_stop: SendStop,
    ) -> Result<(), Error> {
        match self.blocking_start(address, bus_type, Dir::Write, 0) {
            Err(e) => {
                self.blocking_remediation(bus_type);
                return Err(e);
            }
            _ => {}
        };

        // Usually, embassy HALs error out with an empty write,
        // however empty writes are useful for writing I2C scanning
        // logic through write probing. That is, we send a start with
        // R/w bit cleared, but instead of writing any data, just send
        // the stop onto the bus. This has the effect of checking if
        // the resulting address got an ACK but causing no
        // side-effects to the device on the other end.
        //
        // Because of this, we are not going to error out in case of
        // empty writes.
        #[cfg(feature = "defmt")]
        if write.is_empty() {
            defmt::trace!("Empty write, write probing?");
        }

        let Some((last, rest)) = write.split_last() else {
            return Err(Error::InvalidWriteBufferLength);
        };

        for byte in rest {
            // Wait until we have space in the TX FIFO.
            self.blocking_wait_for_tx_fifo();
            self.info.regs().mwdatab().write(|w| unsafe { w.value().bits(*byte) });
        }

        self.blocking_wait_for_tx_fifo();
        self.info.regs().mwdatabe().write(|w| unsafe { w.value().bits(*last) });
        self.blocking_wait_for_complete();

        if send_stop == SendStop::Yes {
            self.blocking_stop(bus_type)?;
        }

        Ok(())
    }

    // Public API: Blocking

    /// Read from address into buffer blocking caller until done.
    pub fn blocking_read(&mut self, address: u8, read: &mut [u8], bus_type: BusType) -> Result<(), Error> {
        self.blocking_read_internal(address, read, bus_type, SendStop::Yes)
    }

    /// Write to address from buffer blocking caller until done.
    pub fn blocking_write(&mut self, address: u8, write: &[u8], bus_type: BusType) -> Result<(), Error> {
        self.blocking_write_internal(address, write, bus_type, SendStop::Yes)
    }

    /// Write to address from bytes and read from address into buffer blocking caller until done.
    pub fn blocking_write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
        bus_type: BusType,
    ) -> Result<(), Error> {
        self.blocking_write_internal(address, write, bus_type, SendStop::No)?;
        self.blocking_read_internal(address, read, bus_type, SendStop::Yes)
    }
}

impl<'d> I3c<'d, Blocking> {
    /// Create a new blocking instance of the I3C controller bus driver.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_blocking(
        peri: Peri<'d, I3C0>,
        scl: Peri<'d, impl SclPin<I3C0>>,
        sda: Peri<'d, impl SdaPin<I3C0>>,
        config: Config,
    ) -> Result<Self, Error> {
        Self::new_inner(peri, scl, sda, config)
    }
}

impl<'d> I3c<'d, Async> {
    /// Create a new asynchronous instance of the I3C controller bus driver.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_async(
        peri: Peri<'d, I3C0>,
        scl: Peri<'d, impl SclPin<I3C0>>,
        sda: Peri<'d, impl SdaPin<I3C0>>,
        _irq: impl typelevel::Binding<typelevel::I3C0, InterruptHandler> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        let inst = Self::new_inner(peri, scl, sda, config);

        crate::pac::Interrupt::I3C0.unpend();
        unsafe {
            crate::pac::Interrupt::I3C0.enable();
        }

        inst
    }

    async fn async_wait_for_ctrldone(&self) -> Result<(), Error> {
        self.info
            .wait_cell()
            .wait_for(|| {
                // enable control done interrupt
                self.info
                    .regs()
                    .mintset()
                    .write(|w| w.mctrldone().enable().errwarn().enable());
                // check control done status
                self.info.regs().mstatus().read().mctrldone().is_done()
                    || self.info.regs().mstatus().read().errwarn().is_error()
            })
            .await
            .map_err(|_| Error::Other)
    }

    async fn async_wait_for_complete(&self) -> Result<(), Error> {
        self.info
            .wait_cell()
            .wait_for(|| {
                // enable control done interrupt
                self.info
                    .regs()
                    .mintset()
                    .write(|w| w.complete().enable().errwarn().enable());
                // check control done status
                self.info.regs().mstatus().read().complete().is_complete()
                    || self.info.regs().mstatus().read().errwarn().is_error()
            })
            .await
            .map_err(|_| Error::Other)
    }

    async fn async_wait_for_tx_fifo(&self) -> Result<(), Error> {
        // Wait until we have space in the TX FIFO.
        self.info
            .wait_cell()
            .wait_for(|| {
                // enable TXNOTFULL interrupt
                self.info
                    .regs()
                    .mintset()
                    .write(|w| w.txnotfull().set_bit().errwarn().enable());
                // if the TX FIFO isn't full, we can write bytes
                self.info.regs().mstatus().read().txnotfull().is_notfull()
                    || self.info.regs().mstatus().read().errwarn().is_error()
            })
            .await
            .map_err(|_| Error::Overwrite)
    }

    async fn async_wait_for_rx_fifo(&self) -> Result<(), Error> {
        self.info
            .wait_cell()
            .wait_for(|| {
                // enable RXPEND interrupt
                self.info
                    .regs()
                    .mintset()
                    .write(|w| w.rxpend().set_bit().errwarn().enable());
                // if the rx FIFO is pending, we need to read bytes
                self.info.regs().mstatus().read().rxpend().is_pending()
                    || self.info.regs().mstatus().read().errwarn().is_error()
            })
            .await
            .map_err(|_| Error::Overread)
    }

    /// Prepares an appropriate Start condition on bus by issuing a
    /// `Start` request together with the device address, bus type
    /// (i3c sdr, i3c ddr, or i2c), and R/w bit.
    async fn async_start(&self, address: u8, bus_type: BusType, dir: Dir, len: u8) -> Result<(), Error> {
        self.clear_flags();

        self.info.regs().mctrl().write(|w| {
            unsafe { w.addr().bits(address).rdterm().bits(len) }
                .type_()
                .variant(bus_type.into())
                .request()
                .emitstartaddr()
                .dir()
                .variant(dir.into())
                .ibiresp()
                .ack()
        });

        self.async_wait_for_ctrldone().await?;
        self.status()
    }

    /// Prepares a Stop condition on the bus.
    ///
    /// Analogous to `start`, this blocks waiting for space in the
    /// FIFO to become available, then sends the command and blocks
    /// waiting for the FIFO to become empty ensuring the command was
    /// sent.
    async fn async_stop(&self, bus_type: BusType) -> Result<(), Error> {
        if !self.info.regs().mstatus().read().state().is_normact() {
            Err(Error::InvalidRequest)
        } else {
            // NOTE: Section 41.3.2.1 states that "when sending STOP
            // in I2C mode, MCONFIG[ODSTOP] and MCTRL[TYPE] must be
            // 1".
            if bus_type == BusType::I2c {
                self.info.regs().mconfig().modify(|_, w| w.odstop().enable());
            } else {
                self.info.regs().mconfig().modify(|_, w| w.odstop().disable());
            }

            self.info
                .regs()
                .mctrl()
                .write(|w| w.request().emitstop().type_().variant(bus_type.into()));
            self.async_wait_for_ctrldone().await?;
            self.status()
        }
    }

    async fn async_read_internal(
        &self,
        address: u8,
        read: &mut [u8],
        bus_type: BusType,
        send_stop: SendStop,
    ) -> Result<(), Error> {
        if read.is_empty() {
            return Err(Error::InvalidReadBufferLength);
        }

        // perform corrective action if the future is dropped
        let on_drop = OnDrop::new(|| {
            self.blocking_remediation(bus_type);
        });

        for chunk in read.chunks_mut(MAX_CHUNK_SIZE) {
            self.async_start(address, bus_type, Dir::Read, chunk.len() as u8)
                .await?;

            for byte in chunk.iter_mut() {
                self.async_wait_for_rx_fifo().await?;
                *byte = self.info.regs().mrdatab().read().value().bits();
            }
        }

        if send_stop == SendStop::Yes {
            self.async_stop(bus_type).await?;
        }

        // defuse it if the future is not dropped
        on_drop.defuse();

        Ok(())
    }

    async fn async_write_internal(
        &self,
        address: u8,
        write: &[u8],
        bus_type: BusType,
        send_stop: SendStop,
    ) -> Result<(), Error> {
        // perform corrective action if the future is dropped
        let on_drop = OnDrop::new(|| {
            self.blocking_remediation(bus_type);
        });

        self.async_start(address, bus_type, Dir::Write, 0).await?;

        // Usually, embassy HALs error out with an empty write,
        // however empty writes are useful for writing I2C scanning
        // logic through write probing. That is, we send a start with
        // R/w bit cleared, but instead of writing any data, just send
        // the stop onto the bus. This has the effect of checking if
        // the resulting address got an ACK but causing no
        // side-effects to the device on the other end.
        //
        // Because of this, we are not going to error out in case of
        // empty writes.
        #[cfg(feature = "defmt")]
        if write.is_empty() {
            defmt::trace!("Empty write, write probing?");
        }

        let Some((last, rest)) = write.split_last() else {
            return Err(Error::InvalidWriteBufferLength);
        };

        for byte in rest {
            self.async_wait_for_tx_fifo().await?;
            self.info.regs().mwdatab().write(|w| unsafe { w.value().bits(*byte) });
        }

        // Wait until we have space in the TX FIFO.
        self.async_wait_for_tx_fifo().await?;
        self.info.regs().mwdatabe().write(|w| unsafe { w.value().bits(*last) });

        // Wait for complete
        self.async_wait_for_complete().await?;

        if send_stop == SendStop::Yes {
            self.async_stop(bus_type).await?;
        }

        // defuse it if the future is not dropped
        on_drop.defuse();

        Ok(())
    }

    // Public API: Async

    /// Read from address into buffer asynchronously.
    pub fn async_read<'a>(
        &mut self,
        address: u8,
        read: &'a mut [u8],
        bus_type: BusType,
    ) -> impl Future<Output = Result<(), Error>> + use<'_, 'a, 'd> {
        self.async_read_internal(address, read, bus_type, SendStop::Yes)
    }

    /// Write to address from buffer asynchronously.
    pub fn async_write<'a>(
        &mut self,
        address: u8,
        write: &'a [u8],
        bus_type: BusType,
    ) -> impl Future<Output = Result<(), Error>> + use<'_, 'a, 'd> {
        self.async_write_internal(address, write, bus_type, SendStop::Yes)
    }

    /// Write to address from bytes and then read from address into buffer asynchronously.
    pub async fn async_write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
        bus_type: BusType,
    ) -> Result<(), Error> {
        self.async_write_internal(address, write, bus_type, SendStop::No)
            .await?;
        self.async_read_internal(address, read, bus_type, SendStop::Yes).await
    }
}

impl<'d, M: Mode> Drop for I3c<'d, M> {
    fn drop(&mut self) {
        self._scl.set_as_disabled();
        self._sda.set_as_disabled();
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::i2c::Read for I3c<'d, M> {
    type Error = Error;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        self.blocking_read(address, buffer, BusType::I2c)
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::i2c::Write for I3c<'d, M> {
    type Error = Error;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Error> {
        self.blocking_write(address, bytes, BusType::I2c)
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::i2c::WriteRead for I3c<'d, M> {
    type Error = Error;

    fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Error> {
        self.blocking_write_read(address, bytes, buffer, BusType::I2c)
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::i2c::Transactional for I3c<'d, M> {
    type Error = Error;

    fn exec(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_02::blocking::i2c::Operation<'_>],
    ) -> Result<(), Error> {
        let Some((last, rest)) = operations.split_last_mut() else {
            return Ok(());
        };

        for op in rest {
            match op {
                embedded_hal_02::blocking::i2c::Operation::Read(buf) => {
                    self.blocking_read_internal(address, buf, BusType::I2c, SendStop::No)?
                }
                embedded_hal_02::blocking::i2c::Operation::Write(buf) => {
                    self.blocking_write_internal(address, buf, BusType::I2c, SendStop::No)?
                }
            }
        }

        match last {
            embedded_hal_02::blocking::i2c::Operation::Read(buf) => {
                self.blocking_read_internal(address, buf, BusType::I2c, SendStop::Yes)
            }
            embedded_hal_02::blocking::i2c::Operation::Write(buf) => {
                self.blocking_write_internal(address, buf, BusType::I2c, SendStop::Yes)
            }
        }
    }
}

impl embedded_hal_1::i2c::Error for Error {
    fn kind(&self) -> embedded_hal_1::i2c::ErrorKind {
        match *self {
            Self::Nack => {
                embedded_hal_1::i2c::ErrorKind::NoAcknowledge(embedded_hal_1::i2c::NoAcknowledgeSource::Unknown)
            }
            _ => embedded_hal_1::i2c::ErrorKind::Other,
        }
    }
}

impl<'d, M: Mode> embedded_hal_1::i2c::ErrorType for I3c<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_hal_1::i2c::I2c for I3c<'d, M> {
    fn transaction(&mut self, address: u8, operations: &mut [embedded_hal_1::i2c::Operation<'_>]) -> Result<(), Error> {
        let Some((last, rest)) = operations.split_last_mut() else {
            return Ok(());
        };

        for op in rest {
            match op {
                embedded_hal_1::i2c::Operation::Read(buf) => {
                    self.blocking_read_internal(address, buf, BusType::I2c, SendStop::No)?
                }
                embedded_hal_1::i2c::Operation::Write(buf) => {
                    self.blocking_write_internal(address, buf, BusType::I2c, SendStop::No)?
                }
            }
        }

        match last {
            embedded_hal_1::i2c::Operation::Read(buf) => {
                self.blocking_read_internal(address, buf, BusType::I2c, SendStop::Yes)
            }
            embedded_hal_1::i2c::Operation::Write(buf) => {
                self.blocking_write_internal(address, buf, BusType::I2c, SendStop::Yes)
            }
        }
    }
}

impl<'d> embedded_hal_async::i2c::I2c for I3c<'d, Async> {
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_async::i2c::Operation<'_>],
    ) -> Result<(), Error> {
        let Some((last, rest)) = operations.split_last_mut() else {
            return Ok(());
        };

        for op in rest {
            match op {
                embedded_hal_async::i2c::Operation::Read(buf) => {
                    self.async_read_internal(address, buf, BusType::I2c, SendStop::No)
                        .await?
                }
                embedded_hal_async::i2c::Operation::Write(buf) => {
                    self.async_write_internal(address, buf, BusType::I2c, SendStop::No)
                        .await?
                }
            }
        }

        match last {
            embedded_hal_async::i2c::Operation::Read(buf) => {
                self.async_read_internal(address, buf, BusType::I2c, SendStop::Yes)
                    .await
            }
            embedded_hal_async::i2c::Operation::Write(buf) => {
                self.async_write_internal(address, buf, BusType::I2c, SendStop::Yes)
                    .await
            }
        }
    }
}

impl<'d, M: Mode> embassy_embedded_hal::SetConfig for I3c<'d, M> {
    type Config = Config;
    type ConfigError = Error;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Error> {
        self.set_configuration(config)
    }
}
