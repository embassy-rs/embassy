//! I2C slave driver.
use core::future;
use core::marker::PhantomData;
use core::task::Poll;

use pac::i2c;

use crate::i2c::{AbortReason, FIFO_SIZE, Info, Instance, InterruptHandler, SclPin, SdaPin, set_up_i2c_pin};
use crate::interrupt::InterruptExt;
use crate::interrupt::typelevel::Binding;
use crate::mode::{Async, Blocking, Mode};
use crate::{Peri, pac};

/// I2C error
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// I2C abort with error
    Abort(AbortReason),
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

/// Received command
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
    /// General Call
    GeneralCall(usize),
    /// Read
    Read,
    /// Write+read
    WriteRead(usize),
    /// Write
    Write(usize),
}

/// Possible responses to responding to a read
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReadStatus {
    /// Transaction Complete, controller naked our last byte
    Done,
    /// Transaction Incomplete, controller trying to read more bytes than were provided
    NeedMoreBytes,
    /// Transaction Complete, but controller stopped reading bytes before we ran out
    LeftoverBytes(u16),
}

/// Slave Configuration
#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Target Address
    pub addr: u16,
    /// Control if the peripheral should ack to and report general calls.
    pub general_call: bool,
    /// Enable internal pullup on SDA.
    ///
    /// Using external pullup resistors is recommended for I2C. If you do
    /// have external pullups you should not enable this.
    pub sda_pullup: bool,
    /// Enable internal pullup on SCL.
    ///
    /// Using external pullup resistors is recommended for I2C. If you do
    /// have external pullups you should not enable this.
    pub scl_pullup: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: 0x55,
            general_call: true,
            sda_pullup: true,
            scl_pullup: true,
        }
    }
}

/// I2CSlave driver.
pub struct I2cSlave<'d, M: Mode> {
    info: &'static Info,
    pending_byte: Option<u8>,
    config: Config,
    phantom: PhantomData<(&'d mut (), M)>,
}

impl<'d> I2cSlave<'d, Blocking> {
    /// Create a new instance in blocking mode.
    pub fn new_blocking<T: Instance>(
        _peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(T::info(), &scl, &sda, config, false)
    }
}

impl<'d> I2cSlave<'d, Async> {
    /// Create a new instance in async mode.
    pub fn new<T: Instance>(
        _peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(T::info(), &scl, &sda, config, true)
    }
}

impl<'d, M: Mode> I2cSlave<'d, M> {
    fn new_inner<S, D>(info: &'static Info, scl: &S, sda: &D, config: Config, enable_irq: bool) -> Self
    where
        S: core::ops::Deref<Target: crate::gpio::Pin>,
        D: core::ops::Deref<Target: crate::gpio::Pin>,
    {
        assert!(config.addr != 0);

        // Configure SCL & SDA pins
        set_up_i2c_pin(scl, config.scl_pullup);
        set_up_i2c_pin(sda, config.sda_pullup);

        let mut ret = Self {
            info,
            pending_byte: None,
            config,
            phantom: PhantomData,
        };

        ret.reset_inner(enable_irq);

        ret
    }

    /// Reset the i2c peripheral. If you cancel a respond_to_read, you may stall the bus.
    /// You can recover the bus by calling this function, but doing so will almost certainly cause
    /// an i/o error in the master.
    pub fn reset(&mut self) {
        // Interrupts are only ever enabled for `Async` drivers, and `enable()` on an
        // already-enabled interrupt is harmless, so re-enabling matches the original state.
        let enable_irq = self.info.interrupt.is_enabled();
        self.reset_inner(enable_irq);
    }

    fn reset_inner(&mut self, enable_irq: bool) {
        let info = self.info;
        let p = info.regs;

        let reset = (info.reset)();
        crate::reset::reset(reset);
        crate::reset::unreset_wait(reset);

        p.ic_enable().write(|w| w.set_enable(false));

        p.ic_sar().write(|w| w.set_ic_sar(self.config.addr));
        p.ic_con().modify(|w| {
            w.set_master_mode(false);
            w.set_ic_slave_disable(false);
            w.set_tx_empty_ctrl(true);
            w.set_rx_fifo_full_hld_ctrl(true);

            // This typically makes no sense for a slave, but it is used to
            // tune spike suppression, according to the datasheet.
            w.set_speed(pac::i2c::vals::Speed::Fast);

            // Generate stop interrupts for general calls
            // This also causes stop interrupts for other devices on the bus but those will not be
            // propagated up to the application.
            w.set_stop_det_ifaddressed(!self.config.general_call);
        });
        p.ic_ack_general_call()
            .write(|w| w.set_ack_gen_call(self.config.general_call));

        // Set FIFO watermarks to 1 to make things simpler. This is encoded
        // by a register value of 0. Rx watermark should never change, but Tx watermark will be
        // adjusted in operation.
        p.ic_tx_tl().write(|w| w.set_tx_tl(0));
        p.ic_rx_tl().write(|w| w.set_rx_tl(0));

        // Clear interrupts
        p.ic_clr_intr().read();

        // Enable I2C block
        p.ic_enable().write(|w| w.set_enable(true));

        // mask everything initially
        p.ic_intr_mask().write_value(i2c::regs::IcIntrMask(0));
        info.interrupt.unpend();
        if enable_irq {
            unsafe { info.interrupt.enable() };
        }
    }

    /// Spins on `f` until it is ready. Used by the `Blocking` methods, where no
    /// interrupt is bound and so nothing would ever wake us.
    #[inline(always)]
    fn blocking_wait_on<F, U>(&mut self, mut f: F) -> U
    where
        F: FnMut(&mut Self) -> Poll<U>,
    {
        loop {
            if let Poll::Ready(r) = f(self) {
                return r;
            }
        }
    }

    #[inline(always)]
    fn drain_fifo(&mut self, buffer: &mut [u8], offset: &mut usize) {
        let p = self.info.regs;

        if let Some(pending) = self.pending_byte.take() {
            buffer[*offset] = pending;
            *offset += 1;
        }

        for b in &mut buffer[*offset..] {
            if !p.ic_status().read().rfne() {
                break;
            }

            let dat = p.ic_data_cmd().read();
            if *offset != 0 && dat.first_data_byte() {
                // The RP2040 state machine will keep placing bytes into the
                // FIFO, even if they are part of a subsequent write transaction.
                //
                // Unfortunately merely reading ic_data_cmd will consume that
                // byte, the first byte of the next transaction, so we need
                // to store it elsewhere
                self.pending_byte = Some(dat.dat());
                break;
            }

            *b = dat.dat();
            *offset += 1;
        }
    }

    /// Arm the interrupt sources `poll_listen` waits on.
    fn arm_listen(&mut self) {
        self.info.regs.ic_intr_mask().write(|w| {
            w.set_m_stop_det(true);
            w.set_m_restart_det(true);
            w.set_m_gen_call(true);
            w.set_m_rd_req(true);
            w.set_m_rx_full(true);
        });
    }

    /// Arm the interrupt sources `poll_respond_to_read` waits on.
    fn arm_respond_to_read(&mut self) {
        self.info.regs.ic_intr_mask().write(|w| {
            w.set_m_rx_done(true);
            w.set_m_tx_empty(true);
            w.set_m_tx_abrt(true);
        });
    }

    /// Prepare the hardware for a `listen`.
    fn start_listen(&mut self) {
        // set rx fifo watermark to 1 byte
        self.info.regs.ic_rx_tl().write(|w| w.set_rx_tl(0));
    }

    fn poll_listen(&mut self, buffer: &mut [u8], len: &mut usize) -> Poll<Result<Command, Error>> {
        let p = self.info.regs;
        let stat = p.ic_raw_intr_stat().read();
        trace!("ls:{:013b} len:{}", stat.0, *len);

        if p.ic_rxflr().read().rxflr() > 0 || self.pending_byte.is_some() {
            self.drain_fifo(buffer, len);
            // we're receiving data, set rx fifo watermark to 12 bytes (3/4 full) to reduce interrupt noise
            p.ic_rx_tl().write(|w| w.set_rx_tl(11));
        }

        if buffer.len() == *len {
            if stat.gen_call() {
                return Poll::Ready(Err(Error::PartialGeneralCall(buffer.len())));
            } else {
                return Poll::Ready(Err(Error::PartialWrite(buffer.len())));
            }
        }
        trace!("len:{}, pend:{:?}", *len, self.pending_byte);
        if self.pending_byte.is_some() {
            warn!("pending")
        }

        if stat.restart_det() && stat.rd_req() {
            p.ic_clr_restart_det().read();
            Poll::Ready(Ok(Command::WriteRead(*len)))
        } else if stat.gen_call() && stat.stop_det() && *len > 0 {
            p.ic_clr_gen_call().read();
            p.ic_clr_stop_det().read();
            Poll::Ready(Ok(Command::GeneralCall(*len)))
        } else if stat.stop_det() && *len > 0 {
            p.ic_clr_stop_det().read();
            Poll::Ready(Ok(Command::Write(*len)))
        } else if stat.rd_req() {
            p.ic_clr_stop_det().read();
            p.ic_clr_restart_det().read();
            p.ic_clr_gen_call().read();
            Poll::Ready(Ok(Command::Read))
        } else if stat.stop_det() {
            // clear stuck stop bit
            // This can happen if the SDA/SCL pullups are enabled after calling this func
            p.ic_clr_stop_det().read();
            Poll::Pending
        } else {
            Poll::Pending
        }
    }

    fn poll_respond_to_read(&mut self, buffer: &[u8], bytes_written: &mut usize) -> Poll<Result<ReadStatus, Error>> {
        let p = self.info.regs;
        let stat = p.ic_raw_intr_stat().read();
        trace!("rs:{:013b}", stat.0);

        if stat.tx_abrt() {
            if let Err(abort_reason) = self.read_and_clear_abort_reason() {
                if let Error::Abort(AbortReason::TxNotEmpty(bytes)) = abort_reason {
                    p.ic_clr_intr().read();
                    return Poll::Ready(Ok(ReadStatus::LeftoverBytes(bytes)));
                } else {
                    return Poll::Ready(Err(abort_reason));
                }
            }
        }

        if *bytes_written < buffer.len() {
            for _ in 0..((FIFO_SIZE - p.ic_txflr().read().txflr()) as usize).min(buffer.len() - *bytes_written) {
                p.ic_clr_rd_req().read();
                p.ic_data_cmd().write(|w| w.set_dat(buffer[*bytes_written]));
                *bytes_written += 1;
            }

            Poll::Pending
        } else if stat.rx_done() {
            p.ic_clr_rx_done().read();
            Poll::Ready(Ok(ReadStatus::Done))
        } else if stat.rd_req() && stat.tx_empty() {
            Poll::Ready(Ok(ReadStatus::NeedMoreBytes))
        } else {
            Poll::Pending
        }
    }

    /// Wait for a command from an I2C master, blocking the caller until one arrives.
    ///
    /// `buffer` is provided in case the controller does a 'write', 'write read', or 'general
    /// call', and is unused for 'read'.
    pub fn blocking_listen(&mut self, buffer: &mut [u8]) -> Result<Command, Error> {
        self.start_listen();
        let mut len = 0;
        self.blocking_wait_on(|me| me.poll_listen(buffer, &mut len))
    }

    /// Respond to an I2C master READ command, blocking the caller until done.
    pub fn blocking_respond_to_read(&mut self, buffer: &[u8]) -> Result<ReadStatus, Error> {
        if buffer.is_empty() {
            return Err(Error::InvalidResponseBufferLength);
        }
        let mut bytes_written = 0;
        self.blocking_wait_on(|me| me.poll_respond_to_read(buffer, &mut bytes_written))
    }

    /// Respond to reads with the fill byte until the controller stops asking, blocking the caller.
    pub fn blocking_respond_till_stop(&mut self, fill: u8) -> Result<(), Error> {
        let buff = [fill; FIFO_SIZE as usize];
        loop {
            match self.blocking_respond_to_read(&buff) {
                Ok(ReadStatus::NeedMoreBytes) => (),
                Ok(_) => break Ok(()),
                Err(e) => break Err(e),
            }
        }
    }

    /// Respond to a master read, then fill any remaining read bytes with `fill`, blocking the caller.
    pub fn blocking_respond_and_fill(&mut self, buffer: &[u8], fill: u8) -> Result<ReadStatus, Error> {
        let resp_stat = self.blocking_respond_to_read(buffer)?;

        if resp_stat == ReadStatus::NeedMoreBytes {
            self.blocking_respond_till_stop(fill)?;
            Ok(ReadStatus::Done)
        } else {
            Ok(resp_stat)
        }
    }

    #[inline(always)]
    fn read_and_clear_abort_reason(&mut self) -> Result<(), Error> {
        let p = self.info.regs;
        let abort_reason = p.ic_tx_abrt_source().read();

        if abort_reason.0 != 0 {
            // Note clearing the abort flag also clears the reason, and this
            // instance of flag is clear-on-read! Note also the
            // IC_CLR_TX_ABRT register always reads as 0.
            p.ic_clr_tx_abrt().read();

            let reason = if abort_reason.abrt_7b_addr_noack()
                | abort_reason.abrt_10addr1_noack()
                | abort_reason.abrt_10addr2_noack()
            {
                AbortReason::NoAcknowledge
            } else if abort_reason.arb_lost() {
                AbortReason::ArbitrationLoss
            } else if abort_reason.tx_flush_cnt() > 0 {
                AbortReason::TxNotEmpty(abort_reason.tx_flush_cnt())
            } else {
                AbortReason::Other(abort_reason.0)
            };

            Err(Error::Abort(reason))
        } else {
            Ok(())
        }
    }
}

impl<'d> I2cSlave<'d, Async> {
    /// Calls `f` to check if we are ready or not.
    /// If not, `g` is called once (to eg enable the required interrupts).
    /// The waker will always be registered prior to calling `f`.
    #[inline(always)]
    async fn wait_on<F, U, G>(&mut self, mut f: F, mut g: G) -> U
    where
        F: FnMut(&mut Self) -> Poll<U>,
        G: FnMut(&mut Self),
    {
        future::poll_fn(|cx| {
            // Register prior to checking the condition
            self.info.waker.register(cx.waker());
            let r = f(self);

            if r.is_pending() {
                g(self);
            }

            r
        })
        .await
    }

    /// Wait for a command from an I2C master.
    ///
    /// `buffer` is provided in case the controller does a 'write', 'write read', or 'general
    /// call', and is unused for 'read'.
    pub async fn listen(&mut self, buffer: &mut [u8]) -> Result<Command, Error> {
        self.start_listen();
        let mut len = 0;
        self.wait_on(|me| me.poll_listen(buffer, &mut len), |me| me.arm_listen())
            .await
    }

    /// Respond to an I2C master READ command.
    pub async fn respond_to_read(&mut self, buffer: &[u8]) -> Result<ReadStatus, Error> {
        if buffer.is_empty() {
            return Err(Error::InvalidResponseBufferLength);
        }

        let mut bytes_written = 0;
        self.wait_on(
            |me| me.poll_respond_to_read(buffer, &mut bytes_written),
            |me| me.arm_respond_to_read(),
        )
        .await
    }

    /// Respond to reads with the fill byte until the controller stops asking.
    pub async fn respond_till_stop(&mut self, fill: u8) -> Result<(), Error> {
        // Send fill bytes a full fifo at a time, to reduce interrupt noise.
        // This does mean we'll almost certainly abort the write, but since these are fill bytes,
        // we don't care.
        let buff = [fill; FIFO_SIZE as usize];
        loop {
            match self.respond_to_read(&buff).await {
                Ok(ReadStatus::NeedMoreBytes) => (),
                Ok(_) => break Ok(()),
                Err(e) => break Err(e),
            }
        }
    }

    /// Respond to a master read, then fill any remaining read bytes with `fill`.
    pub async fn respond_and_fill(&mut self, buffer: &[u8], fill: u8) -> Result<ReadStatus, Error> {
        let resp_stat = self.respond_to_read(buffer).await?;

        if resp_stat == ReadStatus::NeedMoreBytes {
            self.respond_till_stop(fill).await?;
            Ok(ReadStatus::Done)
        } else {
            Ok(resp_stat)
        }
    }
}
