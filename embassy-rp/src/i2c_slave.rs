//! I2C slave driver.
use core::future;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::into_ref;
use pac::i2c;

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

/// I2CSlave driver.
pub struct I2cSlave<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> I2cSlave<'d, T> {
    /// Create a new instance.
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
        p.ic_intr_mask().write_value(i2c::regs::IcIntrMask(0));
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self { phantom: PhantomData }
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

    #[inline(always)]
    fn drain_fifo(&mut self, buffer: &mut [u8], offset: usize) -> usize {
        let p = T::regs();
        let len = p.ic_rxflr().read().rxflr() as usize;
        let end = offset + len;
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
                    if p.ic_rxflr().read().rxflr() > 0 {
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
                    if let Err(abort_reason) = me.read_and_clear_abort_reason() {
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
