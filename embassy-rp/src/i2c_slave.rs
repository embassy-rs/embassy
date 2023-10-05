use core::future;
use core::marker::PhantomData;
use core::ops::Deref;
use core::task::Poll;

use embassy_hal_internal::into_ref;
use pac::i2c;

use crate::i2c::{i2c_reserved_addr, AbortReason, Instance, InterruptHandler, SclPin, SdaPin, FIFO_SIZE};
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
}

/// Slave Configuration
#[non_exhaustive]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Target Address
    pub addr: u16,
}

struct InternalConfig {
    restart_en: bool,
    full_hld_control: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { addr: 0x55 }
    }
}

struct I2cSlaveCommon<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> I2cSlaveCommon<'d, T> {
    fn new(
        _peri: impl Peripheral<P = T> + 'd,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        config: Config,
        internal_config: InternalConfig,
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
        p.ic_con().modify(move |w| {
            w.set_master_mode(false);
            w.set_ic_slave_disable(false);
            w.set_rx_fifo_full_hld_ctrl(internal_config.full_hld_control);
            w.set_ic_restart_en(internal_config.restart_en);
            w.set_tx_empty_ctrl(true);
        });

        // Set FIFO watermarks to 1 to make things simpler. This is encoded
        // by a register value of 0. Rx watermark should never change, but Tx watermark will be
        // adjusted in operation.
        p.ic_tx_tl().write(|w| w.set_tx_tl(0));
        p.ic_rx_tl().write(|w| w.set_rx_tl(0));

        crate::i2c::set_up_i2c_pin(scl.deref());
        crate::i2c::set_up_i2c_pin(sda.deref());

        // Clear interrupts
        p.ic_clr_intr().read();

        // Enable I2C block
        p.ic_enable().write(|w| w.set_enable(true));

        // mask everything initially
        p.ic_clr_intr().read();
        p.ic_intr_mask().write_value(i2c::regs::IcIntrMask(0));
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self { phantom: PhantomData }
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

    #[inline(always)]
    fn rx_fifo_empty(&self) -> bool {
        let p = T::regs();
        p.ic_rxflr().read().rxflr() == 0
    }

    #[inline(always)]
    fn tx_fifo_full(&self) -> bool {
        let p = T::regs();
        p.ic_txflr().read().txflr() == FIFO_SIZE
    }
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
    /// Transaction Complete, controller NAKed our last byte
    Done,
    /// Transaction Incomplete, controller trying to read more bytes than were provided
    NeedMoreBytes,
    /// Transaction Complete, but controller stopped reading bytes before we ran out
    LeftoverBytes(u16),
}

pub struct I2cSlave<'d, T: Instance> {
    common: I2cSlaveCommon<'d, T>,
}

impl<'d, T: Instance> I2cSlave<'d, T> {
    pub fn new(
        _peri: impl Peripheral<P = T> + 'd,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        config: Config,
    ) -> Self {
        Self {
            common: I2cSlaveCommon::new(
                _peri,
                scl,
                sda,
                config,
                InternalConfig {
                    restart_en: false,
                    full_hld_control: false,
                },
            ),
        }
    }

    #[inline(always)]
    fn drain_fifo(&mut self, buffer: &mut [u8], offset: usize) -> usize {
        let p = T::regs();
        let len = p.ic_rxflr().read().rxflr() as usize;
        let end = buffer.len().min(offset + len);
        for i in offset..end {
            buffer[i] = p.ic_data_cmd().read().dat();
        }
        end
    }

    #[inline(always)]
    fn write_to_fifo(&mut self, buffer: &[u8]) {
        let p = T::regs();
        for byte in buffer {
            p.ic_data_cmd().write(|w| w.set_dat(*byte));
        }
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
            let r = f(self);

            trace!("intr p: {:013b}", T::regs().ic_raw_intr_stat().read().0);

            if r.is_pending() {
                T::waker().register(cx.waker());
                g(self);
            }

            r
        })
        .await
    }

    /// Wait asynchronously for commands from an I2C master.
    /// `buffer` is provided in case master does a 'write' and is unused for 'read'.
    pub async fn listen(&mut self, buffer: &mut [u8]) -> Result<Command, Error> {
        let p = T::regs();

        p.ic_clr_intr().read();
        // set rx fifo watermark to 1 byte
        p.ic_rx_tl().write(|w| w.set_rx_tl(0));

        let mut len = 0;
        let ret = self
            .wait_on(
                |me| {
                    let stat = p.ic_raw_intr_stat().read();
                    if !me.common.rx_fifo_empty() {
                        len = me.drain_fifo(buffer, len);
                        // we're recieving data, set rx fifo watermark to 12 bytes to reduce interrupt noise
                        p.ic_rx_tl().write(|w| w.set_rx_tl(11));
                    }

                    if stat.restart_det() && stat.rd_req() {
                        Poll::Ready(Ok(Command::WriteRead(len)))
                    } else if stat.gen_call() && stat.stop_det() && len > 0 {
                        Poll::Ready(Ok(Command::GeneralCall(len)))
                    } else if stat.stop_det() {
                        Poll::Ready(Ok(Command::Write(len)))
                    } else if stat.rd_req() {
                        Poll::Ready(Ok(Command::Read))
                    } else {
                        Poll::Pending
                    }
                },
                |_me| {
                    p.ic_intr_mask().modify(|w| {
                        w.set_m_stop_det(true);
                        w.set_m_restart_det(true);
                        w.set_m_gen_call(true);
                        w.set_m_rd_req(true);
                        w.set_m_rx_full(true);
                    });
                },
            )
            .await;

        p.ic_clr_intr().read();

        ret
    }

    /// Respond to an I2C master READ command, asynchronously.
    pub async fn respond_to_read(&mut self, buffer: &[u8]) -> Result<ReadStatus, Error> {
        let p = T::regs();

        if buffer.len() == 0 {
            return Err(Error::InvalidResponseBufferLength);
        }

        let mut chunks = buffer.chunks(FIFO_SIZE as usize);

        let ret = self
            .wait_on(
                |me| {
                    if let Err(abort_reason) = me.common.read_and_clear_abort_reason() {
                        if let Error::Abort(AbortReason::TxNotEmpty(bytes)) = abort_reason {
                            return Poll::Ready(Ok(ReadStatus::LeftoverBytes(bytes)));
                        } else {
                            return Poll::Ready(Err(abort_reason));
                        }
                    }

                    if let Some(chunk) = chunks.next() {
                        me.write_to_fifo(chunk);

                        Poll::Pending
                    } else {
                        let stat = p.ic_raw_intr_stat().read();

                        if stat.rx_done() && stat.stop_det() {
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
                        w.set_m_stop_det(true);
                        w.set_m_rx_done(true);
                        w.set_m_tx_empty(true);
                        w.set_m_tx_abrt(true);
                    })
                },
            )
            .await;

        p.ic_clr_intr().read();

        ret
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
}

/// Events
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Event {
    Start,
    Restart,
    Read,
    Write,
    Stop,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum State {
    Idle,
    Active,
    Read,
    Write,
}

/// Possible responses to responding to a read
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WriteStatus {
    /// Transaction Complete, controller sent last byte
    Done(usize),
    /// Transaction Incomplete, controller trying to send more bytes than the size of the buffer
    HasMoreBytes,
}

pub struct I2cSlaveEventIterator<'d, T: Instance> {
    common: I2cSlaveCommon<'d, T>,
    state: State,
}

impl<'d, T: Instance> I2cSlaveEventIterator<'d, T> {
    pub fn new(
        _peri: impl Peripheral<P = T> + 'd,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        config: Config,
    ) -> Self {
        Self {
            common: I2cSlaveCommon::new(
                _peri,
                scl,
                sda,
                config,
                InternalConfig {
                    restart_en: true,
                    full_hld_control: true,
                },
            ),
            state: State::Idle,
        }
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
            let r = f(self);

            trace!("intr p: {:013b}", T::regs().ic_raw_intr_stat().read().0);

            if r.is_pending() {
                T::waker().register(cx.waker());
                g(self);
            }

            r
        })
        .await
    }

    /// Wait asynchronously for events from an I2C master.
    pub async fn next_event(&mut self) -> Event {
        let p = T::regs();

        // set rx fifo watermark to 1 byte
        p.ic_intr_mask().write_value(i2c::regs::IcIntrMask(0));
        p.ic_rx_tl().write(|w| w.set_rx_tl(0));

        let ret = self
            .wait_on(
                |me| {
                    let stat = p.ic_raw_intr_stat().read();
                    p.ic_clr_activity().read();

                    match me.state {
                        State::Idle if stat.start_det() => {
                            p.ic_clr_start_det().read();
                            me.state = State::Active;
                            Poll::Ready(Event::Start)
                        }
                        State::Active if !me.common.rx_fifo_empty() => {
                            me.state = State::Write;
                            Poll::Ready(Event::Write)
                        }
                        State::Active if stat.rd_req() => {
                            // Clearing `rd_req` is used by the hardware to detect when the I2C block can stop
                            // stretching the clock and start process the data pushed to the FIFO (if any).
                            // This is done in `Self::respond_to_read`.
                            me.state = State::Read;
                            Poll::Ready(Event::Read)
                        }
                        State::Read if stat.rd_req() => Poll::Ready(Event::Read),
                        State::Read if stat.restart_det() => {
                            p.ic_clr_restart_det().read();
                            me.state = State::Active;
                            Poll::Ready(Event::Restart)
                        }
                        State::Write if !me.common.rx_fifo_empty() => Poll::Ready(Event::Write),
                        State::Write if stat.restart_det() => {
                            p.ic_clr_restart_det().read();
                            me.state = State::Active;
                            Poll::Ready(Event::Restart)
                        }
                        _ if stat.stop_det() => {
                            p.ic_clr_stop_det().read();
                            me.state = State::Idle;
                            Poll::Ready(Event::Stop)
                        }
                        _ => Poll::Pending,
                    }
                },
                |_me| {
                    p.ic_intr_mask().modify(|w| {
                        w.set_m_activity(true);
                    });
                },
            )
            .await;

        p.ic_intr_mask().write_value(i2c::regs::IcIntrMask(0));
        ret
    }

    /// Respond to an I2C master READ command, asynchronously.
    pub fn respond_to_read(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        let p = T::regs();

        if buffer.len() == 0 {
            return Err(Error::InvalidResponseBufferLength);
        }

        if let Err(abort_reason) = self.common.read_and_clear_abort_reason() {
            return Err(abort_reason);
        }

        let mut sent = 0;
        for &b in buffer.iter() {
            if self.common.tx_fifo_full() {
                break;
            }
            p.ic_data_cmd().write(|w| w.set_dat(b));
            sent += 1;
        }
        p.ic_clr_rd_req().read();

        Ok(sent)
    }

    /// Accept data from an I2C master WRITE command, asynchronously.
    pub fn accept_write(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        let p = T::regs();

        if buffer.len() == 0 {
            return Err(Error::InvalidResponseBufferLength);
        }

        if let Err(abort_reason) = self.common.read_and_clear_abort_reason() {
            return Err(abort_reason);
        }

        let mut read = 0;
        for b in buffer.iter_mut() {
            if self.common.rx_fifo_empty() {
                break;
            }
            *b = p.ic_data_cmd().read().dat();
            read += 1;
        }

        Ok(read)
    }
}
