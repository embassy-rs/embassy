use core::future;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::into_ref;
use pac::i2c;
use pac::i2c::vals::Speed;

use crate::i2c::{
    i2c_reserved_addr, set_up_i2c_pin, AbortReason, Instance, InterruptHandler, SclPin, SdaPin, FIFO_SIZE,
};
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::{pac, Peripheral};

/// I2C error
#[derive(Debug, PartialEq, Eq)]
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
    /// Transaction Complere, but controller stopped reading bytes before we ran out
    LeftoverBytes(u16),
}

/// Slave Configuration
#[non_exhaustive]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Target Address
    pub addr: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self { addr: 0x55 }
    }
}

#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum State {
    #[default]
    Idle,
    Active,
    Read,
    Write,
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum I2cSlaveEvent {
    Start,
    Restart,
    DataRequested,
    DataTransmitted,
    Stop,
}

pub struct I2cSlave<'d, T: Instance> {
    state: State,
    pending_byte: Option<u8>,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> I2cSlave<'d, T> {
    pub fn new(
        _peri: impl Peripheral<P = T> + 'd,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        config: Config,
    ) -> Self {
        into_ref!(_peri, scl, sda);

        assert!(!i2c_reserved_addr(config.addr));
        assert!(config.addr != 0);

        let p = T::regs();

        let reset = T::reset();
        crate::reset::reset(reset);
        crate::reset::unreset_wait(reset);

        p.ic_enable().write(|w| w.set_enable(false));

        p.ic_sar().write(|w| w.set_ic_sar(config.addr));
        p.ic_con().modify(|w| {
            w.set_master_mode(false);
            w.set_ic_slave_disable(false);
            w.set_tx_empty_ctrl(true);
            w.set_rx_fifo_full_hld_ctrl(true);

            // This typically makes no sense for a slave, but it is used to
            // tune spike suppression, according to the datasheet.
            w.set_speed(Speed::FAST);

            // Generate stop interrupts for general calls (and other devices
            // on the bus).
            w.set_stop_det_ifaddressed(false);
        });

        // Set FIFO watermarks to 1 to make things simpler. This is encoded
        // by a register value of 0. Rx watermark should never change, but Tx watermark will be
        // adjusted in operation.
        p.ic_tx_tl().write(|w| w.set_tx_tl(0));
        p.ic_rx_tl().write(|w| w.set_rx_tl(0));

        // Configure SCL & SDA pins
        set_up_i2c_pin(&scl);
        set_up_i2c_pin(&sda);

        // Clear interrupts
        p.ic_clr_intr().read();

        // Enable I2C block
        p.ic_enable().write(|w| w.set_enable(true));

        // mask everything initially
        Self::set_intr_mask(|_| {});
        T::Interrupt::unpend();

        Self {
            state: State::Idle,
            pending_byte: None,
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    fn set_intr_mask(f: impl FnOnce(&mut i2c::regs::IcIntrMask)) {
        let mut value = i2c::regs::IcIntrMask(0);
        f(&mut value);
        T::regs().ic_intr_mask().write_value(value);
    }

    /// Calls `f` to check if we are ready or not.
    /// If not, `g` is called once the waker is set (to eg enable the required interrupts).
    #[inline(always)]
    async fn wait_on<F, U, G>(&mut self, mut f: F, mut g: G) -> U
    where
        F: FnMut(&mut Self) -> Poll<U>,
        G: FnMut(&mut Self),
    {
        future::poll_fn(|cx| {
            T::Interrupt::disable();

            let r = f(self);

            if r.is_pending() {
                T::waker().register(cx.waker());
                g(self);
                unsafe { T::Interrupt::enable() };
            }

            r
        })
        .await
    }

    #[inline(always)]
    fn drain_fifo(&mut self, buffer: &mut [u8], offset: &mut usize) {
        let p = T::regs();

        for b in &mut buffer[*offset..] {
            if let Some(pending) = self.pending_byte.take() {
                *b = pending;
                *offset += 1;
                continue;
            }

            let status = p.ic_status().read();
            if !status.rfne() {
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

    #[inline(always)]
    fn write_to_fifo(&mut self, buffer: &[u8]) {
        let p = T::regs();
        for byte in buffer {
            p.ic_data_cmd().write(|w| w.set_dat(*byte));
        }
    }

    #[inline(always)]
    fn rx_not_empty(&self) -> bool {
        let p = T::regs();
        self.pending_byte.is_some() || p.ic_status().read().rfne()
    }

    #[inline(always)]
    async fn next_event(&mut self) -> I2cSlaveEvent {
        self.wait_on(
            |me| {
                let p = T::regs();

                let i_stat = p.ic_raw_intr_stat().read();
                p.ic_clr_activity().read();

                match me.state {
                    State::Idle if me.pending_byte.is_some() => {
                        // 2. Continue emulating the end of a transaction, this
                        // is the start of the next transaction.
                        me.state = State::Active;
                        Poll::Ready(I2cSlaveEvent::Start)
                    }
                    State::Idle if i_stat.start_det() => {
                        p.ic_clr_start_det().read();
                        me.state = State::Active;
                        Poll::Ready(I2cSlaveEvent::Start)
                    }
                    State::Active if me.rx_not_empty() => {
                        // 3. Will catch normal starts as well as emulated
                        // transactions.

                        // Reduce interrupt noise.
                        p.ic_rx_tl().write(|w| w.set_rx_tl(11));
                        me.state = State::Write;
                        Poll::Ready(I2cSlaveEvent::DataTransmitted)
                    }
                    State::Active if i_stat.rd_req() => {
                        // We intentionally don't reset rd_req here, because
                        // resetting it will stop stretching the clock. Instead
                        // it gets reset in respond_to_read.
                        me.state = State::Read;
                        Poll::Ready(I2cSlaveEvent::DataRequested)
                    }
                    State::Read if i_stat.rd_req() => Poll::Ready(I2cSlaveEvent::DataRequested),
                    State::Read if i_stat.restart_det() => {
                        p.ic_clr_restart_det().read();
                        me.state = State::Active;
                        Poll::Ready(I2cSlaveEvent::Restart)
                    }
                    State::Write if me.pending_byte.is_some() => {
                        me.state = State::Idle;
                        // 1. Start emulating the end of a transaction.
                        // We know it is the end because it is not valid to
                        // issue a restart between transmissions in the same
                        // direction
                        Poll::Ready(I2cSlaveEvent::Stop)
                    }
                    State::Write if me.rx_not_empty() => Poll::Ready(I2cSlaveEvent::DataTransmitted),
                    State::Write if i_stat.restart_det() => {
                        p.ic_clr_restart_det().read();
                        me.state = State::Active;
                        Poll::Ready(I2cSlaveEvent::Restart)
                    }
                    State::Idle if i_stat.stop_det() => {
                        // Probably another device on the bus.
                        p.ic_clr_stop_det().read();
                        Poll::Pending
                    }
                    _ if i_stat.stop_det() => {
                        p.ic_clr_stop_det().read();

                        // The bus is going idle here, make sure the interrupt
                        // occurs as soon as possible if a write occurs.
                        p.ic_rx_tl().write(|w| w.set_rx_tl(0));

                        me.state = State::Idle;
                        Poll::Ready(I2cSlaveEvent::Stop)
                    }
                    _ => Poll::Pending,
                }
            },
            |_me| {
                Self::set_intr_mask(|w| {
                    w.set_m_start_det(true);
                    w.set_m_stop_det(true);
                    w.set_m_restart_det(true);
                    w.set_m_rd_req(true);
                    w.set_m_rx_full(true);
                })
            },
        )
        .await
    }

    /// Wait asynchronously for commands from an I2C master.
    /// `buffer` is provided in case master does a 'write' and is unused for 'read'.
    pub async fn listen(&mut self, buffer: &mut [u8]) -> Result<Command, Error> {
        let p = T::regs();

        let mut offset = 0;
        loop {
            let stat = p.ic_raw_intr_stat().read();
            let evt = self.next_event().await;
            match evt {
                I2cSlaveEvent::Start | I2cSlaveEvent::Restart => {}
                I2cSlaveEvent::DataRequested if offset == 0 => {
                    return Ok(Command::Read);
                }
                I2cSlaveEvent::DataRequested => {
                    return Ok(Command::WriteRead(offset));
                }
                I2cSlaveEvent::DataTransmitted if offset == buffer.len() => {
                    if stat.gen_call() {
                        p.ic_clr_gen_call().read();
                        return Err(Error::PartialGeneralCall(offset));
                    } else {
                        return Err(Error::PartialWrite(offset));
                    }
                }
                I2cSlaveEvent::DataTransmitted => {
                    self.drain_fifo(buffer, &mut offset);
                }
                I2cSlaveEvent::Stop if offset == 0 => {}
                I2cSlaveEvent::Stop => {
                    if stat.gen_call() {
                        p.ic_clr_gen_call().read();
                        return Ok(Command::GeneralCall(offset));
                    } else {
                        return Ok(Command::Write(offset));
                    }
                }
            }
        }
    }

    /// Respond to an I2C master READ command, asynchronously.
    pub async fn respond_to_read(&mut self, buffer: &[u8]) -> Result<ReadStatus, Error> {
        let p = T::regs();

        if buffer.len() == 0 {
            return Err(Error::InvalidResponseBufferLength);
        }

        let mut chunks = buffer.chunks(FIFO_SIZE as usize);

        self.wait_on(
            |me| {
                if let Err(abort_reason) = me.read_and_clear_abort_reason() {
                    if let Error::Abort(AbortReason::TxNotEmpty(bytes)) = abort_reason {
                        return Poll::Ready(Ok(ReadStatus::LeftoverBytes(bytes)));
                    } else {
                        return Poll::Ready(Err(abort_reason));
                    }
                }

                let stat = p.ic_raw_intr_stat().read();
                p.ic_clr_activity().read();

                if let Some(chunk) = chunks.next() {
                    me.write_to_fifo(chunk);

                    // Stop stretching the clk
                    p.ic_clr_rd_req().read();

                    Poll::Pending
                } else {
                    if stat.rx_done() && stat.stop_det() {
                        p.ic_clr_rx_done().read();
                        p.ic_clr_stop_det().read();
                        me.state = State::Idle;
                        Poll::Ready(Ok(ReadStatus::Done))
                    } else if stat.rd_req() {
                        Poll::Ready(Ok(ReadStatus::NeedMoreBytes))
                    } else {
                        Poll::Pending
                    }
                }
            },
            |_me| {
                p.ic_intr_mask().modify(|w| {
                    w.set_m_rd_req(true);
                    w.set_m_stop_det(true);
                    w.set_m_rx_done(true);
                    w.set_m_tx_empty(true);
                    w.set_m_tx_abrt(true);
                })
            },
        )
        .await
    }

    /// Respond to reads with the fill byte until the controller stops asking
    pub async fn respond_till_stop(&mut self, fill: u8) -> Result<(), Error> {
        loop {
            match self.respond_to_read(&[fill]).await {
                Ok(ReadStatus::NeedMoreBytes) => (),
                Ok(_) => break Ok(()),
                Err(e) => break Err(e),
            }
        }
    }

    /// Respond to a master read, then fill any remaining read bytes with `fill`
    pub async fn respond_and_fill(&mut self, buffer: &[u8], fill: u8) -> Result<ReadStatus, Error> {
        let resp_stat = self.respond_to_read(buffer).await?;

        if resp_stat == ReadStatus::NeedMoreBytes {
            self.respond_till_stop(fill).await?;
            Ok(ReadStatus::Done)
        } else {
            Ok(resp_stat)
        }
    }

    #[inline(always)]
    fn read_and_clear_abort_reason(&mut self) -> Result<(), Error> {
        let p = T::regs();
        let mut abort_reason = p.ic_tx_abrt_source().read();

        // Mask off fifo flush count
        let tx_flush_cnt = abort_reason.tx_flush_cnt();
        abort_reason.set_tx_flush_cnt(0);

        // Mask off master_dis
        abort_reason.set_abrt_master_dis(false);

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
            } else if abort_reason.abrt_slvflush_txfifo() {
                AbortReason::TxNotEmpty(tx_flush_cnt)
            } else {
                AbortReason::Other(abort_reason.0)
            };

            Err(Error::Abort(reason))
        } else {
            Ok(())
        }
    }
}
