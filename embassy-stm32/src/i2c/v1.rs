//! # I2Cv1
//!
//! This implementation is used for STM32F1, STM32F2, STM32F4, and STM32L1 devices.
//!
//! All other devices (as of 2023-12-28) use [`v2`](super::v2) instead.

use core::future::poll_fn;
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_futures::select::{Either, select};
use embassy_hal_internal::drop::OnDrop;
use embedded_hal_1::i2c::Operation;
use mode::Master;

use super::*;
use crate::mode::Mode as PeriMode;
use crate::pac::i2c;

// /!\                      /!\
// /!\ Implementation note! /!\
// /!\                      /!\
//
// It's somewhat unclear whether using interrupts here in a *strictly* one-shot style is actually
// what we want! If you are looking in this file because you are doing async I2C and your code is
// just totally hanging (sometimes), maybe swing by this issue:
// <https://github.com/embassy-rs/embassy/issues/2372>.
//
// There's some more details there, and we might have a fix for you. But please let us know if you
// hit a case like this!
pub unsafe fn on_interrupt<T: Instance>() {
    let regs = T::info().regs;
    trace!("I2C interrupt triggered");
    // i2c v2 only woke the task on transfer complete interrupts. v1 uses interrupts for a bunch of
    // other stuff, so we wake the task on every interrupt.
    T::state().waker.wake();
    critical_section::with(|_| {
        // Clear event interrupt flag.
        regs.cr2().modify(|w| {
            w.set_itevten(false);
            w.set_iterren(false);
        });
    });
}

impl<'d, M: PeriMode, IM: MasterMode> I2c<'d, M, IM> {
    pub(crate) fn init(&mut self, config: Config) {
        self.info.regs.cr1().modify(|reg| {
            reg.set_pe(false);
            //reg.set_anfoff(false);
        });

        // Errata: "Start cannot be generated after a misplaced Stop"
        //
        // > If a master generates a misplaced Stop on the bus (bus error)
        // > while the microcontroller I2C peripheral attempts to switch to
        // > Master mode by setting the START bit, the Start condition is
        // > not properly generated.
        //
        // This also can occur with falsely detected STOP events, for example
        // if the SDA line is shorted to low.
        //
        // The workaround for this is to trigger the SWRST line AFTER power is
        // enabled, AFTER PE is disabled and BEFORE making any other configuration.
        //
        // It COULD be possible to apply this workaround at runtime, instead of
        // only on initialization, however this would require detecting the timeout
        // or BUSY lockup condition, and re-configuring the peripheral after reset.
        //
        // This presents as an ~infinite hang on read or write, as the START condition
        // is never generated, meaning the start event is never generated.
        self.info.regs.cr1().modify(|reg| {
            reg.set_swrst(true);
        });
        self.info.regs.cr1().modify(|reg| {
            reg.set_swrst(false);
        });

        let timings = Timings::new(self.kernel_clock, config.frequency);

        self.info.regs.cr2().modify(|reg| {
            reg.set_freq(timings.freq);
        });
        self.info.regs.ccr().modify(|reg| {
            reg.set_f_s(timings.mode.f_s());
            reg.set_duty(timings.duty.duty());
            reg.set_ccr(timings.ccr);
        });
        self.info.regs.trise().modify(|reg| {
            reg.set_trise(timings.trise);
        });

        self.info.regs.cr1().modify(|reg| {
            reg.set_pe(true);
        });
        trace!("i2c v1 init complete");
    }

    fn check_and_clear_error_flags(info: &'static Info) -> Result<i2c::regs::Sr1, Error> {
        // Note that flags should only be cleared once they have been registered. If flags are
        // cleared otherwise, there may be an inherent race condition and flags may be missed.
        let sr1 = info.regs.sr1().read();

        if sr1.timeout() {
            info.regs.sr1().write(|reg| {
                reg.0 = !0;
                reg.set_timeout(false);
            });
            return Err(Error::Timeout);
        }

        if sr1.pecerr() {
            info.regs.sr1().write(|reg| {
                reg.0 = !0;
                reg.set_pecerr(false);
            });
            return Err(Error::Crc);
        }

        if sr1.ovr() {
            info.regs.sr1().write(|reg| {
                reg.0 = !0;
                reg.set_ovr(false);
            });
            return Err(Error::Overrun);
        }

        if sr1.af() {
            info.regs.sr1().write(|reg| {
                reg.0 = !0;
                reg.set_af(false);
            });
            return Err(Error::Nack);
        }

        if sr1.arlo() {
            info.regs.sr1().write(|reg| {
                reg.0 = !0;
                reg.set_arlo(false);
            });
            return Err(Error::Arbitration);
        }

        // The errata indicates that BERR may be incorrectly detected. It recommends ignoring and
        // clearing the BERR bit instead.
        if sr1.berr() {
            info.regs.sr1().write(|reg| {
                reg.0 = !0;
                reg.set_berr(false);
            });
        }

        Ok(sr1)
    }

    fn write_bytes(
        &mut self,
        address: u8,
        write_buffer: &[u8],
        timeout: Timeout,
        frame: FrameOptions,
    ) -> Result<(), Error> {
        if frame.send_start() {
            // Send a START condition

            self.info.regs.cr1().modify(|reg| {
                reg.set_start(true);
            });

            // Wait until START condition was generated
            while !Self::check_and_clear_error_flags(self.info)?.start() {
                timeout.check()?;
            }

            // Check if we were the ones to generate START
            if self.info.regs.cr1().read().start() || !self.info.regs.sr2().read().msl() {
                return Err(Error::Arbitration);
            }

            // Set up current address we're trying to talk to
            self.info.regs.dr().write(|reg| reg.set_dr(address << 1));

            // Wait until address was sent
            // Wait for the address to be acknowledged
            // Check for any I2C errors. If a NACK occurs, the ADDR bit will never be set.
            while !Self::check_and_clear_error_flags(self.info)?.addr() {
                timeout.check()?;
            }

            // Clear condition by reading SR2
            let _ = self.info.regs.sr2().read();
        }

        // Send bytes
        for c in write_buffer {
            self.send_byte(*c, timeout)?;
        }

        // Wait for the last byte to finish transmitting. This is essential even when not sending
        // STOP - if we're about to send a repeated START (for write_read), we must wait for the
        // last byte to finish transmitting, otherwise the repeated START will interrupt the
        // ongoing byte transmission and corrupt the transfer.
        while !Self::check_and_clear_error_flags(self.info)?.btf() {
            timeout.check()?;
        }

        if frame.send_stop() {
            // Send a STOP condition
            self.info.regs.cr1().modify(|reg| reg.set_stop(true));
        }

        // Fallthrough is success
        Ok(())
    }

    fn send_byte(&self, byte: u8, timeout: Timeout) -> Result<(), Error> {
        // Wait until we're ready for sending
        while {
            // Check for any I2C errors. If a NACK occurs, the ADDR bit will never be set.
            !Self::check_and_clear_error_flags(self.info)?.txe()
        } {
            timeout.check()?;
        }

        // Push out a byte of data
        self.info.regs.dr().write(|reg| reg.set_dr(byte));

        // Wait until byte is transferred
        while {
            // Check for any potential error conditions.
            !Self::check_and_clear_error_flags(self.info)?.btf()
        } {
            timeout.check()?;
        }

        Ok(())
    }

    fn recv_byte(&self, timeout: Timeout) -> Result<u8, Error> {
        while {
            // Check for any potential error conditions.
            Self::check_and_clear_error_flags(self.info)?;

            !self.info.regs.sr1().read().rxne()
        } {
            timeout.check()?;
        }

        let value = self.info.regs.dr().read().dr();
        Ok(value)
    }

    fn blocking_read_timeout(
        &mut self,
        address: u8,
        read_buffer: &mut [u8],
        timeout: Timeout,
        frame: FrameOptions,
    ) -> Result<(), Error> {
        let Some((last_byte, read_buffer)) = read_buffer.split_last_mut() else {
            return Err(Error::Overrun);
        };

        if frame.send_start() {
            // Send a START condition and set ACK bit
            self.info.regs.cr1().modify(|reg| {
                reg.set_start(true);
                reg.set_ack(true);
            });

            // Wait until START condition was generated
            while !Self::check_and_clear_error_flags(self.info)?.start() {
                timeout.check()?;
            }

            // Check if we were the ones to generate START
            if self.info.regs.cr1().read().start() || !self.info.regs.sr2().read().msl() {
                return Err(Error::Arbitration);
            }

            // Set up current address we're trying to talk to
            self.info.regs.dr().write(|reg| reg.set_dr((address << 1) + 1));

            // Wait until address was sent
            // Wait for the address to be acknowledged
            while !Self::check_and_clear_error_flags(self.info)?.addr() {
                timeout.check()?;
            }

            // Clear condition by reading SR2
            let _ = self.info.regs.sr2().read();
        }

        // Receive bytes into buffer
        for c in read_buffer {
            *c = self.recv_byte(timeout)?;
        }

        // Prepare to send NACK then STOP after next byte
        self.info.regs.cr1().modify(|reg| {
            if frame.send_nack() {
                reg.set_ack(false);
            }
            if frame.send_stop() {
                reg.set_stop(true);
            }
        });

        // Receive last byte
        *last_byte = self.recv_byte(timeout)?;

        // Fallthrough is success
        Ok(())
    }

    /// Blocking read.
    pub fn blocking_read(&mut self, address: u8, read_buffer: &mut [u8]) -> Result<(), Error> {
        self.blocking_read_timeout(address, read_buffer, self.timeout(), FrameOptions::FirstAndLastFrame)
    }

    /// Blocking write.
    pub fn blocking_write(&mut self, address: u8, write_buffer: &[u8]) -> Result<(), Error> {
        self.write_bytes(address, write_buffer, self.timeout(), FrameOptions::FirstAndLastFrame)?;

        // Fallthrough is success
        Ok(())
    }

    /// Blocking write, restart, read.
    pub fn blocking_write_read(
        &mut self,
        address: u8,
        write_buffer: &[u8],
        read_buffer: &mut [u8],
    ) -> Result<(), Error> {
        // Check empty read buffer before starting transaction. Otherwise, we would not generate the
        // stop condition below.
        if read_buffer.is_empty() {
            return Err(Error::Overrun);
        }

        let timeout = self.timeout();

        self.write_bytes(address, write_buffer, timeout, FrameOptions::FirstFrame)?;
        self.blocking_read_timeout(address, read_buffer, timeout, FrameOptions::FirstAndLastFrame)?;

        Ok(())
    }

    /// Blocking transaction with operations.
    ///
    /// Consecutive operations of same type are merged. See [transaction contract] for details.
    ///
    /// [transaction contract]: embedded_hal_1::i2c::I2c::transaction
    pub fn blocking_transaction(&mut self, address: u8, operations: &mut [Operation<'_>]) -> Result<(), Error> {
        let timeout = self.timeout();

        for (op, frame) in operation_frames(operations)? {
            match op {
                Operation::Read(read_buffer) => self.blocking_read_timeout(address, read_buffer, timeout, frame)?,
                Operation::Write(write_buffer) => self.write_bytes(address, write_buffer, timeout, frame)?,
            }
        }

        Ok(())
    }

    /// Can be used by both blocking and async implementations  
    #[inline] // pretty sure this should always be inlined
    fn enable_interrupts(info: &'static Info) {
        // The interrupt handler disables interrupts globally, so we need to re-enable them
        // This must be done in a critical section to avoid races
        critical_section::with(|_| {
            info.regs.cr2().modify(|w| {
                w.set_iterren(true);
                w.set_itevten(true);
            });
        });
    }

    /// Can be used by both blocking and async implementations
    fn clear_stop_flag(info: &'static Info) {
        trace!("I2C slave: clearing STOPF flag (v1 sequence)");
        // v1 requires: READ SR1 then WRITE CR1 to clear STOPF
        let _ = info.regs.sr1().read();
        info.regs.cr1().modify(|_| {}); // Dummy write to clear STOPF
    }
}

impl<'d, IM: MasterMode> I2c<'d, Async, IM> {
    async fn write_frame(&mut self, address: u8, write_buffer: &[u8], frame: FrameOptions) -> Result<(), Error> {
        self.info.regs.cr2().modify(|w| {
            // Note: Do not enable the ITBUFEN bit in the I2C_CR2 register if DMA is used for
            // reception.
            w.set_itbufen(false);
            // DMA mode can be enabled for transmission by setting the DMAEN bit in the I2C_CR2
            // register.
            w.set_dmaen(true);
            // Sending NACK is not necessary (nor possible) for write transfer.
            w.set_last(false);
        });

        // Sentinel to disable transfer when an error occurs or future is canceled.
        // TODO: Generate STOP condition on cancel?
        let on_drop = OnDrop::new(|| {
            self.info.regs.cr2().modify(|w| {
                w.set_dmaen(false);
                w.set_iterren(false);
                w.set_itevten(false);
            })
        });

        if frame.send_start() {
            // Send a START condition
            self.info.regs.cr1().modify(|reg| {
                reg.set_start(true);
            });

            // Wait until START condition was generated
            poll_fn(|cx| {
                self.state.waker.register(cx.waker());

                match Self::check_and_clear_error_flags(self.info) {
                    Err(e) => Poll::Ready(Err(e)),
                    Ok(sr1) => {
                        if sr1.start() {
                            Poll::Ready(Ok(()))
                        } else {
                            // When pending, (re-)enable interrupts to wake us up.
                            Self::enable_interrupts(self.info);
                            Poll::Pending
                        }
                    }
                }
            })
            .await?;

            // Check if we were the ones to generate START
            if self.info.regs.cr1().read().start() || !self.info.regs.sr2().read().msl() {
                return Err(Error::Arbitration);
            }

            // Set up current address we're trying to talk to
            self.info.regs.dr().write(|reg| reg.set_dr(address << 1));

            // Wait for the address to be acknowledged
            poll_fn(|cx| {
                self.state.waker.register(cx.waker());

                match Self::check_and_clear_error_flags(self.info) {
                    Err(e) => {
                        // Send STOP condition, otherwise SCL will remain low forever.
                        trace!("I2C master: address not acknowledged, send stop");
                        self.info.regs.cr1().modify(|reg| reg.set_stop(true));

                        Poll::Ready(Err(e))
                    }
                    Ok(sr1) => {
                        if sr1.addr() {
                            Poll::Ready(Ok(()))
                        } else {
                            // When pending, (re-)enable interrupts to wake us up.
                            Self::enable_interrupts(self.info);
                            Poll::Pending
                        }
                    }
                }
            })
            .await?;

            // Clear condition by reading SR2
            self.info.regs.sr2().read();
        }

        let dma_transfer = unsafe {
            // Set the I2C_DR register address in the DMA_SxPAR register. The data will be moved to
            // this address from the memory after each TxE event.
            let dst = self.info.regs.dr().as_ptr() as *mut u8;

            self.tx_dma
                .as_mut()
                .unwrap()
                .write(write_buffer, dst, Default::default())
        };

        // Wait for bytes to be sent, or an error to occur.
        let poll_error = poll_fn(|cx| {
            self.state.waker.register(cx.waker());

            match Self::check_and_clear_error_flags(self.info) {
                Err(e) => Poll::Ready(Err::<(), Error>(e)),
                Ok(_) => {
                    // When pending, (re-)enable interrupts to wake us up.
                    Self::enable_interrupts(self.info);
                    Poll::Pending
                }
            }
        });

        // Wait for either the DMA transfer to successfully finish, or an I2C error to occur.
        match select(dma_transfer, poll_error).await {
            Either::Second(Err(e)) => Err(e),
            _ => Ok(()),
        }?;

        self.info.regs.cr2().modify(|w| {
            w.set_dmaen(false);
        });

        // The I2C transfer itself will take longer than the DMA transfer, so wait for that to finish too.
        // This is essential even when not sending STOP - if we're about to send a repeated START
        // (for write_read), we must wait for the last byte to finish transmitting, otherwise the
        // repeated START will interrupt the ongoing byte transmission and corrupt the transfer.
        //
        // 18.3.8 "Master transmitter: In the interrupt routine after the EOT interrupt, disable DMA
        // requests then wait for a BTF event before programming the Stop condition."
        poll_fn(|cx| {
            self.state.waker.register(cx.waker());

            match Self::check_and_clear_error_flags(self.info) {
                Err(e) => Poll::Ready(Err(e)),
                Ok(sr1) => {
                    if sr1.btf() {
                        Poll::Ready(Ok(()))
                    } else {
                        // When pending, (re-)enable interrupts to wake us up.
                        Self::enable_interrupts(self.info);
                        Poll::Pending
                    }
                }
            }
        })
        .await?;

        if frame.send_stop() {
            self.info.regs.cr1().modify(|w| {
                w.set_stop(true);
            });
        }

        drop(on_drop);

        // Fallthrough is success
        Ok(())
    }

    /// Write.
    pub async fn write(&mut self, address: u8, write_buffer: &[u8]) -> Result<(), Error> {
        let _scoped_wake_guard = self.info.rcc.wake_guard();
        self.write_frame(address, write_buffer, FrameOptions::FirstAndLastFrame)
            .await?;

        Ok(())
    }

    /// Read.
    pub async fn read(&mut self, address: u8, read_buffer: &mut [u8]) -> Result<(), Error> {
        let _scoped_wake_guard = self.info.rcc.wake_guard();
        self.read_frame(address, read_buffer, FrameOptions::FirstAndLastFrame)
            .await?;

        Ok(())
    }

    async fn read_frame(&mut self, address: u8, read_buffer: &mut [u8], frame: FrameOptions) -> Result<(), Error> {
        if read_buffer.is_empty() {
            return Err(Error::Overrun);
        }

        // Some branches below depend on whether the buffer contains only a single byte.
        let single_byte = read_buffer.len() == 1;

        self.info.regs.cr2().modify(|w| {
            // Note: Do not enable the ITBUFEN bit in the I2C_CR2 register if DMA is used for
            // reception.
            w.set_itbufen(false);
            // DMA mode can be enabled for transmission by setting the DMAEN bit in the I2C_CR2
            // register.
            w.set_dmaen(true);
            // If, in the I2C_CR2 register, the LAST bit is set, I2C automatically sends a NACK
            // after the next byte following EOT_1. The user can generate a Stop condition in
            // the DMA Transfer Complete interrupt routine if enabled.
            w.set_last(frame.send_nack() && !single_byte);
        });

        // Sentinel to disable transfer when an error occurs or future is canceled.
        // TODO: Generate STOP condition on cancel?
        let on_drop = OnDrop::new(|| {
            self.info.regs.cr2().modify(|w| {
                w.set_dmaen(false);
                w.set_iterren(false);
                w.set_itevten(false);
            })
        });

        if frame.send_start() {
            // Send a START condition and set ACK bit
            self.info.regs.cr1().modify(|reg| {
                reg.set_start(true);
                reg.set_ack(true);
            });

            // Wait until START condition was generated
            poll_fn(|cx| {
                self.state.waker.register(cx.waker());

                match Self::check_and_clear_error_flags(self.info) {
                    Err(e) => Poll::Ready(Err(e)),
                    Ok(sr1) => {
                        if sr1.start() {
                            Poll::Ready(Ok(()))
                        } else {
                            // When pending, (re-)enable interrupts to wake us up.
                            Self::enable_interrupts(self.info);
                            Poll::Pending
                        }
                    }
                }
            })
            .await?;

            // Check if we were the ones to generate START
            if self.info.regs.cr1().read().start() || !self.info.regs.sr2().read().msl() {
                return Err(Error::Arbitration);
            }

            // Set up current address we're trying to talk to
            self.info.regs.dr().write(|reg| reg.set_dr((address << 1) + 1));

            // Wait for the address to be acknowledged
            poll_fn(|cx| {
                self.state.waker.register(cx.waker());

                match Self::check_and_clear_error_flags(self.info) {
                    Err(e) => {
                        // Send STOP condition, otherwise SCL will remain low forever.
                        trace!("I2C master: address not acknowledged, send stop");
                        self.info.regs.cr1().modify(|reg| reg.set_stop(true));

                        Poll::Ready(Err(e))
                    }
                    Ok(sr1) => {
                        if sr1.addr() {
                            Poll::Ready(Ok(()))
                        } else {
                            // When pending, (re-)enable interrupts to wake us up.
                            Self::enable_interrupts(self.info);
                            Poll::Pending
                        }
                    }
                }
            })
            .await?;

            // 18.3.8: When a single byte must be received: the NACK must be programmed during EV6
            // event, i.e. program ACK=0 when ADDR=1, before clearing ADDR flag.
            if frame.send_nack() && single_byte {
                self.info.regs.cr1().modify(|w| {
                    w.set_ack(false);
                });
            }

            // Clear condition by reading SR2
            self.info.regs.sr2().read();
        } else {
            // Before starting reception of single byte (but without START condition, i.e. in case
            // of merged operations), program NACK to emit at end of this byte.
            if frame.send_nack() && single_byte {
                self.info.regs.cr1().modify(|w| {
                    w.set_ack(false);
                });
            }
        }

        // 18.3.8: When a single byte must be received: [snip] Then the user can program the STOP
        // condition either after clearing ADDR flag, or in the DMA Transfer Complete interrupt
        // routine.
        if frame.send_stop() && single_byte {
            self.info.regs.cr1().modify(|w| {
                w.set_stop(true);
            });
        }

        let dma_transfer = unsafe {
            // Set the I2C_DR register address in the DMA_SxPAR register. The data will be moved
            // from this address from the memory after each RxE event.
            let src = self.info.regs.dr().as_ptr() as *mut u8;

            self.rx_dma.as_mut().unwrap().read(src, read_buffer, Default::default())
        };

        // Wait for bytes to be received, or an error to occur.
        let poll_error = poll_fn(|cx| {
            self.state.waker.register(cx.waker());

            match Self::check_and_clear_error_flags(self.info) {
                Err(e) => Poll::Ready(Err::<(), Error>(e)),
                _ => {
                    // When pending, (re-)enable interrupts to wake us up.
                    Self::enable_interrupts(self.info);
                    Poll::Pending
                }
            }
        });

        match select(dma_transfer, poll_error).await {
            Either::Second(Err(e)) => Err(e),
            _ => Ok(()),
        }?;

        self.info.regs.cr2().modify(|w| {
            w.set_dmaen(false);
        });

        if frame.send_stop() && !single_byte {
            self.info.regs.cr1().modify(|w| {
                w.set_stop(true);
            });
        }

        drop(on_drop);

        // Fallthrough is success
        Ok(())
    }

    /// Write, restart, read.
    pub async fn write_read(&mut self, address: u8, write_buffer: &[u8], read_buffer: &mut [u8]) -> Result<(), Error> {
        let _scoped_wake_guard = self.info.rcc.wake_guard();
        // Check empty read buffer before starting transaction. Otherwise, we would not generate the
        // stop condition below.
        if read_buffer.is_empty() {
            return Err(Error::Overrun);
        }

        self.write_frame(address, write_buffer, FrameOptions::FirstFrame)
            .await?;
        self.read_frame(address, read_buffer, FrameOptions::FirstAndLastFrame)
            .await
    }

    /// Transaction with operations.
    ///
    /// Consecutive operations of same type are merged. See [transaction contract] for details.
    ///
    /// [transaction contract]: embedded_hal_1::i2c::I2c::transaction
    pub async fn transaction(&mut self, address: u8, operations: &mut [Operation<'_>]) -> Result<(), Error> {
        let _scoped_wake_guard = self.info.rcc.wake_guard();
        for (op, frame) in operation_frames(operations)? {
            match op {
                Operation::Read(read_buffer) => self.read_frame(address, read_buffer, frame).await?,
                Operation::Write(write_buffer) => self.write_frame(address, write_buffer, frame).await?,
            }
        }

        Ok(())
    }
}

enum Mode {
    Fast,
    Standard,
}

impl Mode {
    fn f_s(&self) -> i2c::vals::FS {
        match self {
            Mode::Fast => i2c::vals::FS::FAST,
            Mode::Standard => i2c::vals::FS::STANDARD,
        }
    }
}

enum Duty {
    Duty2_1,
    Duty16_9,
}

impl Duty {
    fn duty(&self) -> i2c::vals::Duty {
        match self {
            Duty::Duty2_1 => i2c::vals::Duty::DUTY2_1,
            Duty::Duty16_9 => i2c::vals::Duty::DUTY16_9,
        }
    }
}

/// Result of attempting to send a byte in slave transmitter mode
#[derive(Debug, PartialEq)]
enum TransmitResult {
    /// Byte sent and ACKed by master - continue transmission
    Acknowledged,
    /// Byte sent but NACKed by master - normal end of read transaction
    NotAcknowledged,
    /// STOP condition detected - master terminated transaction
    Stopped,
    /// RESTART condition detected - master starting new transaction
    Restarted,
}

/// Result of attempting to receive a byte in slave receiver mode
#[derive(Debug, PartialEq)]
enum ReceiveResult {
    /// Data byte successfully received
    Data(u8),
    /// STOP condition detected - end of write transaction
    Stopped,
    /// RESTART condition detected - master starting new transaction
    Restarted,
}

/// Enumeration of slave transaction termination conditions
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum SlaveTermination {
    /// STOP condition received - normal end of transaction
    Stop,
    /// RESTART condition received - master starting new transaction  
    Restart,
    /// NACK received - normal end of read transaction
    Nack,
}

impl<'d, M: PeriMode> I2c<'d, M, Master> {
    /// Configure the I2C driver for slave operations, allowing for the driver to be used as a slave and a master (multimaster)
    pub fn into_slave_multimaster(mut self, slave_addr_config: SlaveAddrConfig) -> I2c<'d, M, MultiMaster> {
        let mut slave = I2c {
            info: self.info,
            state: self.state,
            kernel_clock: self.kernel_clock,
            tx_dma: self.tx_dma.take(), // Use take() to move ownership
            rx_dma: self.rx_dma.take(), // Use take() to move ownership
            #[cfg(feature = "time")]
            timeout: self.timeout,
            _phantom: PhantomData,
            _phantom2: PhantomData,
            _drop_guard: self._drop_guard, // Move the drop guard
        };
        slave.init_slave(slave_addr_config);
        slave
    }
}

// Address configuration methods
impl<'d, M: PeriMode, IM: MasterMode> I2c<'d, M, IM> {
    /// Initialize slave mode with address configuration
    pub(crate) fn init_slave(&mut self, config: SlaveAddrConfig) {
        trace!("I2C slave: initializing with config={:?}", config);

        // Disable peripheral for configuration
        self.info.regs.cr1().modify(|reg| reg.set_pe(false));

        // Configure slave addresses
        self.apply_address_configuration(config);

        // Enable peripheral with slave settings
        self.info.regs.cr1().modify(|reg| {
            reg.set_pe(true);
            reg.set_ack(true); // Enable acknowledgment for slave mode
            reg.set_nostretch(false); // Allow clock stretching for processing time
        });

        trace!("I2C slave: initialization complete");
    }

    /// Apply the complete address configuration for slave mode
    fn apply_address_configuration(&mut self, config: SlaveAddrConfig) {
        match config.addr {
            OwnAddresses::OA1(addr) => {
                self.configure_primary_address(addr);
                self.disable_secondary_address();
            }
            OwnAddresses::OA2(oa2) => {
                self.configure_default_primary_address();
                self.configure_secondary_address(oa2.addr); // v1 ignores mask
            }
            OwnAddresses::Both { oa1, oa2 } => {
                self.configure_primary_address(oa1);
                self.configure_secondary_address(oa2.addr); // v1 ignores mask
            }
        }

        // Configure general call detection
        if config.general_call {
            self.info.regs.cr1().modify(|w| w.set_engc(true));
        }
    }

    /// Configure the primary address (OA1) register
    fn configure_primary_address(&mut self, addr: Address) {
        match addr {
            Address::SevenBit(addr) => {
                self.info.regs.oar1().write(|reg| {
                    let hw_addr = (addr as u16) << 1; // Address in bits [7:1]
                    reg.set_add(hw_addr);
                    reg.set_addmode(i2c::vals::Addmode::BIT7);
                });
            }
            Address::TenBit(addr) => {
                self.info.regs.oar1().write(|reg| {
                    reg.set_add(addr);
                    reg.set_addmode(i2c::vals::Addmode::BIT10);
                });
            }
        }

        // Set required bit 14 as per reference manual
        self.info.regs.oar1().modify(|reg| reg.0 |= 1 << 14);
    }

    /// Configure the secondary address (OA2) register  
    fn configure_secondary_address(&mut self, addr: u8) {
        self.info.regs.oar2().write(|reg| {
            reg.set_add2(addr);
            reg.set_endual(i2c::vals::Endual::DUAL);
        });
    }

    /// Set a default primary address when using OA2-only mode
    fn configure_default_primary_address(&mut self) {
        self.info.regs.oar1().write(|reg| {
            reg.set_add(0); // Reserved address, safe to use
            reg.set_addmode(i2c::vals::Addmode::BIT7);
        });
        self.info.regs.oar1().modify(|reg| reg.0 |= 1 << 14);
    }

    /// Disable secondary address when not needed
    fn disable_secondary_address(&mut self) {
        self.info.regs.oar2().write(|reg| {
            reg.set_endual(i2c::vals::Endual::SINGLE);
        });
    }
}

impl<'d, M: PeriMode> I2c<'d, M, MultiMaster> {
    /// Listen for incoming I2C address match and return the command type
    ///
    /// This method blocks until the slave address is matched by a master.
    /// Returns the command type (Read/Write) and the matched address.
    pub fn blocking_listen(&mut self) -> Result<SlaveCommand, Error> {
        trace!("I2C slave: starting blocking listen for address match");
        let result = self.blocking_listen_with_timeout(self.timeout());
        trace!("I2C slave: blocking listen complete, result={:?}", result);
        result
    }

    /// Respond to a master read request by transmitting data
    ///
    /// Sends the provided data to the master. If the master requests more bytes
    /// than available, padding bytes (0x00) are sent until the master terminates
    /// the transaction with NACK.
    ///
    /// Returns the total number of bytes transmitted (including padding).
    pub fn blocking_respond_to_read(&mut self, data: &[u8]) -> Result<usize, Error> {
        trace!("I2C slave: starting blocking respond_to_read, data_len={}", data.len());

        if let Some(zero_length_result) = self.detect_zero_length_read(self.timeout())? {
            trace!("I2C slave: zero-length read detected");
            return Ok(zero_length_result);
        }

        let result = self.transmit_to_master(data, self.timeout());
        trace!("I2C slave: blocking respond_to_read complete, result={:?}", result);
        result
    }

    /// Respond to a master write request by receiving data
    ///
    /// Receives data from the master into the provided buffer. If the master
    /// sends more bytes than the buffer can hold, excess bytes are acknowledged
    /// but discarded.
    ///
    /// Returns the number of bytes stored in the buffer (not total received).
    pub fn blocking_respond_to_write(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        trace!(
            "I2C slave: starting blocking respond_to_write, buffer_len={}",
            buffer.len()
        );
        let result = self.receive_from_master(buffer, self.timeout());
        trace!("I2C slave: blocking respond_to_write complete, result={:?}", result);
        result
    }

    // Private implementation methods

    /// Wait for address match and determine transaction type
    fn blocking_listen_with_timeout(&mut self, timeout: Timeout) -> Result<SlaveCommand, Error> {
        // Ensure interrupts are disabled for blocking operation
        self.disable_i2c_interrupts();

        // Wait for address match (ADDR flag)
        loop {
            let sr1 = Self::read_status_and_handle_errors(self.info)?;

            if sr1.addr() {
                // Address matched - read SR2 to get direction and clear ADDR flag
                let sr2 = self.info.regs.sr2().read();
                let direction = if sr2.tra() {
                    SlaveCommandKind::Read
                } else {
                    SlaveCommandKind::Write
                };

                // Use the static method instead of the instance method
                let matched_address = Self::decode_matched_address(sr2, self.info)?;
                trace!(
                    "I2C slave: address matched, direction={:?}, addr={:?}",
                    direction, matched_address
                );

                return Ok(SlaveCommand {
                    kind: direction,
                    address: matched_address,
                });
            }

            timeout.check()?;
        }
    }

    /// Transmit data to master in response to read request
    fn transmit_to_master(&mut self, data: &[u8], timeout: Timeout) -> Result<usize, Error> {
        let mut bytes_transmitted = 0;
        let mut padding_count = 0;

        loop {
            let byte_to_send = if bytes_transmitted < data.len() {
                data[bytes_transmitted]
            } else {
                padding_count += 1;
                0x00 // Send padding bytes when data is exhausted
            };

            match self.transmit_byte(byte_to_send, timeout)? {
                TransmitResult::Acknowledged => {
                    bytes_transmitted += 1;
                }
                TransmitResult::NotAcknowledged => {
                    bytes_transmitted += 1; // Count the NACKed byte
                    break;
                }
                TransmitResult::Stopped | TransmitResult::Restarted => {
                    break;
                }
            }
        }

        if padding_count > 0 {
            trace!(
                "I2C slave: sent {} data bytes + {} padding bytes = {} total",
                data.len(),
                padding_count,
                bytes_transmitted
            );
        }

        Ok(bytes_transmitted)
    }

    /// Receive data from master during write request
    fn receive_from_master(&mut self, buffer: &mut [u8], timeout: Timeout) -> Result<usize, Error> {
        let mut bytes_stored = 0;

        // Receive bytes that fit in buffer
        while bytes_stored < buffer.len() {
            match self.receive_byte(timeout)? {
                ReceiveResult::Data(byte) => {
                    buffer[bytes_stored] = byte;
                    bytes_stored += 1;
                }
                ReceiveResult::Stopped | ReceiveResult::Restarted => {
                    return Ok(bytes_stored);
                }
            }
        }

        // Handle buffer overflow by discarding excess bytes
        if bytes_stored == buffer.len() {
            trace!("I2C slave: buffer full, discarding excess bytes");
            self.discard_excess_bytes(timeout)?;
        }

        Ok(bytes_stored)
    }

    /// Detect zero-length read pattern early
    ///
    /// Zero-length reads occur when a master sends START+ADDR+R followed immediately
    /// by NACK+STOP without wanting any data. This must be detected before attempting
    /// to transmit any bytes to avoid SDA line issues.
    fn detect_zero_length_read(&mut self, _timeout: Timeout) -> Result<Option<usize>, Error> {
        // Quick check for immediate termination signals
        let sr1 = self.info.regs.sr1().read();

        // Check for immediate NACK (fastest zero-length pattern)
        if sr1.af() {
            self.clear_acknowledge_failure();
            return Ok(Some(0));
        }

        // Check for immediate STOP (alternative zero-length pattern)
        if sr1.stopf() {
            Self::clear_stop_flag(self.info);
            return Ok(Some(0));
        }

        // Give a brief window for master to send termination signals
        // This handles masters that have slight delays between address ACK and NACK
        const ZERO_LENGTH_DETECTION_CYCLES: u32 = 20; // ~5-10µs window

        for _ in 0..ZERO_LENGTH_DETECTION_CYCLES {
            let sr1 = self.info.regs.sr1().read();

            // Immediate NACK indicates zero-length read
            if sr1.af() {
                self.clear_acknowledge_failure();
                return Ok(Some(0));
            }

            // Immediate STOP indicates zero-length read
            if sr1.stopf() {
                Self::clear_stop_flag(self.info);
                return Ok(Some(0));
            }

            // If TXE becomes ready, master is waiting for data - not zero-length
            if sr1.txe() {
                return Ok(None); // Proceed with normal transmission
            }

            // If RESTART detected, handle as zero-length
            if sr1.addr() {
                return Ok(Some(0));
            }
        }

        // No zero-length pattern detected within the window
        Ok(None)
    }

    /// Discard excess bytes when buffer is full
    fn discard_excess_bytes(&mut self, timeout: Timeout) -> Result<(), Error> {
        let mut discarded_count = 0;

        loop {
            match self.receive_byte(timeout)? {
                ReceiveResult::Data(_) => {
                    discarded_count += 1;
                    continue;
                }
                ReceiveResult::Stopped | ReceiveResult::Restarted => {
                    if discarded_count > 0 {
                        trace!("I2C slave: discarded {} excess bytes", discarded_count);
                    }
                    break;
                }
            }
        }
        Ok(())
    }

    /// Send a single byte and wait for master's response
    fn transmit_byte(&mut self, byte: u8, timeout: Timeout) -> Result<TransmitResult, Error> {
        // Wait for transmit buffer ready
        self.wait_for_transmit_ready(timeout)?;

        // Send the byte
        self.info.regs.dr().write(|w| w.set_dr(byte));

        // Wait for transmission completion or master response
        self.wait_for_transmit_completion(timeout)
    }

    /// Wait until transmit buffer is ready (TXE flag set)
    fn wait_for_transmit_ready(&mut self, timeout: Timeout) -> Result<(), Error> {
        loop {
            let sr1 = Self::read_status_and_handle_errors(self.info)?;

            // Check for early termination conditions
            if let Some(result) = Self::check_early_termination(sr1) {
                return Err(self.handle_early_termination(result));
            }

            if sr1.txe() {
                return Ok(()); // Ready to transmit
            }

            timeout.check()?;
        }
    }

    /// Wait for byte transmission completion or master response
    fn wait_for_transmit_completion(&mut self, timeout: Timeout) -> Result<TransmitResult, Error> {
        loop {
            let sr1 = self.info.regs.sr1().read();

            // Check flags in priority order
            if sr1.af() {
                self.clear_acknowledge_failure();
                return Ok(TransmitResult::NotAcknowledged);
            }

            if sr1.btf() {
                return Ok(TransmitResult::Acknowledged);
            }

            if sr1.stopf() {
                Self::clear_stop_flag(self.info);
                return Ok(TransmitResult::Stopped);
            }

            if sr1.addr() {
                return Ok(TransmitResult::Restarted);
            }

            // Check for other error conditions
            self.check_for_hardware_errors(sr1)?;

            timeout.check()?;
        }
    }

    /// Receive a single byte or detect transaction termination
    fn receive_byte(&mut self, timeout: Timeout) -> Result<ReceiveResult, Error> {
        loop {
            let sr1 = Self::read_status_and_handle_errors(self.info)?;

            // Check for received data first (prioritize data over control signals)
            if sr1.rxne() {
                let byte = self.info.regs.dr().read().dr();
                return Ok(ReceiveResult::Data(byte));
            }

            // Check for transaction termination
            if sr1.addr() {
                return Ok(ReceiveResult::Restarted);
            }

            if sr1.stopf() {
                Self::clear_stop_flag(self.info);
                return Ok(ReceiveResult::Stopped);
            }

            timeout.check()?;
        }
    }

    /// Determine which slave address was matched based on SR2 flags
    fn decode_matched_address(sr2: i2c::regs::Sr2, info: &'static Info) -> Result<Address, Error> {
        if sr2.gencall() {
            Ok(Address::SevenBit(0x00)) // General call address
        } else if sr2.dualf() {
            // OA2 (secondary address) was matched
            let oar2 = info.regs.oar2().read();
            if oar2.endual() != i2c::vals::Endual::DUAL {
                return Err(Error::Bus); // Hardware inconsistency
            }
            Ok(Address::SevenBit(oar2.add2()))
        } else {
            // OA1 (primary address) was matched
            let oar1 = info.regs.oar1().read();
            match oar1.addmode() {
                i2c::vals::Addmode::BIT7 => {
                    let addr = (oar1.add() >> 1) as u8;
                    Ok(Address::SevenBit(addr))
                }
                i2c::vals::Addmode::BIT10 => Ok(Address::TenBit(oar1.add())),
            }
        }
    }

    // Helper methods for hardware interaction

    /// Read status register and handle I2C errors (except NACK in slave mode)
    fn read_status_and_handle_errors(info: &'static Info) -> Result<i2c::regs::Sr1, Error> {
        match Self::check_and_clear_error_flags(info) {
            Ok(sr1) => Ok(sr1),
            Err(Error::Nack) => {
                // In slave mode, NACK is normal protocol behavior, not an error
                Ok(info.regs.sr1().read())
            }
            Err(other_error) => Err(other_error),
        }
    }

    /// Check for conditions that cause early termination of operations
    fn check_early_termination(sr1: i2c::regs::Sr1) -> Option<TransmitResult> {
        if sr1.stopf() {
            Some(TransmitResult::Stopped)
        } else if sr1.addr() {
            Some(TransmitResult::Restarted)
        } else if sr1.af() {
            Some(TransmitResult::NotAcknowledged)
        } else {
            None
        }
    }

    /// Convert early termination to appropriate error
    fn handle_early_termination(&mut self, result: TransmitResult) -> Error {
        match result {
            TransmitResult::Stopped => {
                Self::clear_stop_flag(self.info);
                Error::Bus // Unexpected STOP during setup
            }
            TransmitResult::Restarted => {
                Error::Bus // Unexpected RESTART during setup
            }
            TransmitResult::NotAcknowledged => {
                self.clear_acknowledge_failure();
                Error::Bus // Unexpected NACK during setup
            }
            TransmitResult::Acknowledged => {
                unreachable!() // This should never be passed to this function
            }
        }
    }

    /// Check for hardware-level I2C errors during transmission
    fn check_for_hardware_errors(&self, sr1: i2c::regs::Sr1) -> Result<(), Error> {
        if sr1.timeout() || sr1.ovr() || sr1.arlo() || sr1.berr() {
            // Delegate to existing error handling
            Self::check_and_clear_error_flags(self.info)?;
        }
        Ok(())
    }

    /// Disable I2C event and error interrupts for blocking operations
    fn disable_i2c_interrupts(&mut self) {
        self.info.regs.cr2().modify(|w| {
            w.set_itevten(false);
            w.set_iterren(false);
        });
    }

    /// Clear the acknowledge failure flag
    fn clear_acknowledge_failure(&mut self) {
        self.info.regs.sr1().write(|reg| {
            reg.0 = !0;
            reg.set_af(false);
        });
    }

    /// Configure DMA settings for slave operations (shared between read/write)
    fn setup_slave_dma_base(&mut self) {
        self.info.regs.cr2().modify(|w| {
            w.set_itbufen(false); // Always disable buffer interrupts when using DMA
            w.set_dmaen(true); // Enable DMA requests
            w.set_last(false); // LAST bit not used in slave mode for v1 hardware
        });
    }

    /// Disable DMA and interrupts in a critical section
    fn disable_dma_and_interrupts(info: &'static Info) {
        critical_section::with(|_| {
            info.regs.cr2().modify(|w| {
                w.set_dmaen(false);
                w.set_iterren(false);
                w.set_itevten(false);
            });
        });
    }

    /// Check for early termination conditions during slave operations
    /// Returns Some(result) if termination detected, None to continue
    fn check_slave_termination_conditions(sr1: i2c::regs::Sr1) -> Option<SlaveTermination> {
        if sr1.stopf() {
            Some(SlaveTermination::Stop)
        } else if sr1.addr() {
            Some(SlaveTermination::Restart)
        } else if sr1.af() {
            Some(SlaveTermination::Nack)
        } else {
            None
        }
    }
}

impl<'d> I2c<'d, Async, MultiMaster> {
    /// Async listen for incoming I2C messages using interrupts
    ///
    /// Waits for a master to address this slave and returns the command type
    /// (Read/Write) and the matched address. This method will suspend until
    /// an address match occurs.
    pub async fn listen(&mut self) -> Result<SlaveCommand, Error> {
        let _scoped_wake_guard = self.info.rcc.wake_guard();
        trace!("I2C slave: starting async listen for address match");
        let state = self.state;
        let info = self.info;

        Self::enable_interrupts(info);

        let on_drop = OnDrop::new(|| {
            Self::disable_dma_and_interrupts(info);
        });

        let result = poll_fn(|cx| {
            state.waker.register(cx.waker());

            match Self::check_and_clear_error_flags(info) {
                Err(e) => {
                    error!("I2C slave: error during listen: {:?}", e);
                    Poll::Ready(Err(e))
                }
                Ok(sr1) => {
                    if sr1.addr() {
                        let sr2 = info.regs.sr2().read();
                        let direction = if sr2.tra() {
                            SlaveCommandKind::Read
                        } else {
                            SlaveCommandKind::Write
                        };

                        let matched_address = match Self::decode_matched_address(sr2, info) {
                            Ok(addr) => {
                                trace!("I2C slave: address matched, direction={:?}, addr={:?}", direction, addr);
                                addr
                            }
                            Err(e) => {
                                error!("I2C slave: failed to decode matched address: {:?}", e);
                                return Poll::Ready(Err(e));
                            }
                        };

                        Poll::Ready(Ok(SlaveCommand {
                            kind: direction,
                            address: matched_address,
                        }))
                    } else {
                        Self::enable_interrupts(info);
                        Poll::Pending
                    }
                }
            }
        })
        .await;

        drop(on_drop);
        trace!("I2C slave: listen complete, result={:?}", result);
        result
    }

    /// Async respond to write command using RX DMA
    ///
    /// Receives data from the master into the provided buffer using DMA.
    /// If the master sends more bytes than the buffer can hold, excess bytes
    /// are acknowledged but discarded to prevent interrupt flooding.
    ///
    /// Returns the number of bytes stored in the buffer (not total received).
    pub async fn respond_to_write(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        let _scoped_wake_guard = self.info.rcc.wake_guard();
        trace!("I2C slave: starting respond_to_write, buffer_len={}", buffer.len());

        if buffer.is_empty() {
            warn!("I2C slave: respond_to_write called with empty buffer");
            return Err(Error::Overrun);
        }

        let state = self.state;
        let info = self.info;

        self.setup_slave_dma_base();

        let on_drop = OnDrop::new(|| {
            Self::disable_dma_and_interrupts(info);
        });

        info.regs.sr2().read();

        let result = self.execute_slave_receive_transfer(buffer, state, info).await;

        drop(on_drop);
        trace!("I2C slave: respond_to_write complete, result={:?}", result);
        result
    }

    /// Async respond to read command using TX DMA
    ///
    /// Transmits data to the master using DMA. If the master requests more bytes
    /// than available in the data buffer, padding bytes (0x00) are sent until
    /// the master terminates the transaction with NACK, STOP, or RESTART.
    ///
    /// Returns the total number of bytes transmitted (data + padding).
    pub async fn respond_to_read(&mut self, data: &[u8]) -> Result<usize, Error> {
        let _scoped_wake_guard = self.info.rcc.wake_guard();
        trace!("I2C slave: starting respond_to_read, data_len={}", data.len());

        if data.is_empty() {
            warn!("I2C slave: respond_to_read called with empty data");
            return Err(Error::Overrun);
        }

        let state = self.state;
        let info = self.info;

        self.setup_slave_dma_base();

        let on_drop = OnDrop::new(|| {
            Self::disable_dma_and_interrupts(info);
        });

        info.regs.sr2().read();

        let result = self.execute_slave_transmit_transfer(data, state, info).await;

        drop(on_drop);
        trace!("I2C slave: respond_to_read complete, result={:?}", result);
        result
    }

    // === Private Transfer Execution Methods ===

    /// Execute complete slave receive transfer with excess byte handling
    async fn execute_slave_receive_transfer(
        &mut self,
        buffer: &mut [u8],
        state: &'static State,
        info: &'static Info,
    ) -> Result<usize, Error> {
        let total_len = buffer.len();

        let mut dma_transfer = unsafe {
            let src = info.regs.dr().as_ptr() as *mut u8;
            self.rx_dma.as_mut().unwrap().read(src, buffer, Default::default())
        };

        // Track whether transfer was terminated by I2C event (STOP/RESTART) vs DMA completion.
        // We only need to handle excess bytes if DMA completed (buffer full) and master
        // is still sending. If STOP/RESTART terminated the transfer, there are no excess bytes.
        let mut terminated_by_i2c_event = false;

        // Use poll_fn to monitor both DMA and I2C events, allowing us to get the
        // remaining DMA transfer count when STOP/RESTART is detected.
        // Returns Ok(remaining) where remaining is the DMA NDTR value when termination was detected.
        let result = poll_fn(|cx| {
            state.waker.register(cx.waker());

            // Check for I2C errors first
            match Self::check_and_clear_error_flags(info) {
                Err(e) => {
                    error!("I2C slave: error during receive transfer: {:?}", e);
                    return Poll::Ready(Err(e));
                }
                Ok(sr1) => {
                    // Check for STOP or RESTART conditions
                    if let Some(termination) = Self::check_slave_termination_conditions(sr1) {
                        match termination {
                            SlaveTermination::Stop | SlaveTermination::Restart => {
                                // Get DMA remaining count
                                let remaining = dma_transfer.get_remaining_transfers() as usize;

                                // Clear the termination flag
                                match termination {
                                    SlaveTermination::Stop => Self::clear_stop_flag(info),
                                    SlaveTermination::Restart => {
                                        // ADDR flag will be handled by next listen() call
                                    }
                                    SlaveTermination::Nack => unreachable!(),
                                }

                                terminated_by_i2c_event = true;
                                trace!(
                                    "I2C slave: receive terminated by {:?}, received {} bytes (remaining {})",
                                    termination,
                                    total_len.saturating_sub(remaining),
                                    remaining
                                );
                                return Poll::Ready(Ok(remaining));
                            }
                            SlaveTermination::Nack => {
                                // Unexpected NACK during receive
                                return Poll::Ready(Err(Error::Bus));
                            }
                        }
                    }

                    // Check if DMA transfer completed (buffer full)
                    if !dma_transfer.is_running() {
                        trace!("I2C slave: DMA receive completed (buffer full)");
                        return Poll::Ready(Ok(0_usize)); // remaining = 0
                    }

                    // Neither condition met, enable interrupts and wait
                    Self::enable_interrupts(info);
                    Poll::Pending
                }
            }
        })
        .await;

        // Stop the DMA transfer - this releases the borrow on buffer
        drop(dma_transfer);
        Self::disable_dma_and_interrupts(info);

        // Calculate actual bytes received
        let result = match result {
            Ok(remaining) => {
                let received = total_len.saturating_sub(remaining);
                trace!("I2C slave: receive complete, received {} bytes", received);
                Ok(received)
            }
            Err(e) => Err(e),
        };

        // Only handle excess bytes if DMA completed (buffer full) AND transfer was NOT
        // terminated by STOP/RESTART. If STOP/RESTART occurred, there are no more bytes coming.
        if let Ok(received) = result {
            if received == total_len && !terminated_by_i2c_event {
                self.handle_excess_bytes(state, info).await?;
            }
        }

        result
    }

    /// Execute complete slave transmit transfer with padding byte handling
    async fn execute_slave_transmit_transfer(
        &mut self,
        data: &[u8],
        state: &'static State,
        info: &'static Info,
    ) -> Result<usize, Error> {
        let total_len = data.len();

        let mut dma_transfer = unsafe {
            let dst = info.regs.dr().as_ptr() as *mut u8;
            self.tx_dma.as_mut().unwrap().write(data, dst, Default::default())
        };

        // Track whether transfer was terminated by I2C event (STOP/RESTART/NACK) vs DMA completion.
        // We only need to handle padding bytes if DMA completed and master wants more data.
        let mut terminated_by_i2c_event = false;

        // Use poll_fn to monitor both DMA and I2C events, allowing us to get the
        // remaining DMA transfer count when STOP/RESTART/NACK is detected
        // Helper to calculate actual bytes transmitted.
        // DMA pre-loads the next byte into DR before it's clocked out. When termination
        // occurs, if TXE is clear, there's a byte in DR that was loaded but never sent.
        let calc_bytes_sent = |remaining: usize, sr1: crate::pac::i2c::regs::Sr1| {
            let dma_sent = total_len.saturating_sub(remaining);
            // If TXE is clear, DR contains a byte that DMA loaded but master never clocked out
            if !sr1.txe() && dma_sent > 0 {
                dma_sent - 1
            } else {
                dma_sent
            }
        };

        let result = poll_fn(|cx| {
            state.waker.register(cx.waker());

            // Check for I2C errors first (NACK is expected for read termination)
            match Self::check_and_clear_error_flags(info) {
                Err(Error::Nack) => {
                    // NACK is normal - master doesn't want more data
                    let sr1 = info.regs.sr1().read();
                    let remaining = dma_transfer.get_remaining_transfers() as usize;
                    let sent = calc_bytes_sent(remaining, sr1);
                    terminated_by_i2c_event = true;
                    trace!(
                        "I2C slave: transmit terminated by Nack, sent {} bytes (remaining {}, txe={})",
                        sent,
                        remaining,
                        sr1.txe()
                    );
                    return Poll::Ready(Ok(sent));
                }
                Err(e) => {
                    error!("I2C slave: error during transmit transfer: {:?}", e);
                    return Poll::Ready(Err(e));
                }
                Ok(sr1) => {
                    // Check for STOP or RESTART conditions
                    if let Some(termination) = Self::check_slave_termination_conditions(sr1) {
                        match termination {
                            SlaveTermination::Stop | SlaveTermination::Restart => {
                                let remaining = dma_transfer.get_remaining_transfers() as usize;
                                let sent = calc_bytes_sent(remaining, sr1);

                                match termination {
                                    SlaveTermination::Stop => Self::clear_stop_flag(info),
                                    SlaveTermination::Restart => {
                                        // ADDR flag will be handled by next listen() call
                                    }
                                    SlaveTermination::Nack => unreachable!(),
                                }

                                terminated_by_i2c_event = true;
                                trace!(
                                    "I2C slave: transmit terminated by {:?}, sent {} bytes (remaining {}, txe={})",
                                    termination,
                                    sent,
                                    remaining,
                                    sr1.txe()
                                );
                                return Poll::Ready(Ok(sent));
                            }
                            SlaveTermination::Nack => {
                                // Handled above via check_and_clear_error_flags
                                let remaining = dma_transfer.get_remaining_transfers() as usize;
                                let sent = calc_bytes_sent(remaining, sr1);
                                terminated_by_i2c_event = true;
                                info.regs.sr1().write(|reg| {
                                    reg.0 = !0;
                                    reg.set_af(false);
                                });
                                trace!(
                                    "I2C slave: transmit terminated by Nack, sent {} bytes (remaining {}, txe={})",
                                    sent,
                                    remaining,
                                    sr1.txe()
                                );
                                return Poll::Ready(Ok(sent));
                            }
                        }
                    }

                    // Check if DMA transfer completed (all data sent)
                    if !dma_transfer.is_running() {
                        trace!("I2C slave: DMA transmit completed (all data sent)");
                        return Poll::Ready(Ok(total_len));
                    }

                    // Neither condition met, enable interrupts and wait
                    Self::enable_interrupts(info);
                    Poll::Pending
                }
            }
        })
        .await;

        // Stop the DMA transfer
        drop(dma_transfer);
        Self::disable_dma_and_interrupts(info);

        // Only handle padding bytes if DMA completed (all data sent) AND transfer was NOT
        // terminated by STOP/RESTART/NACK. If terminated early, master doesn't want more data.
        if let Ok(sent) = result {
            if sent == total_len && !terminated_by_i2c_event {
                let padding_count = self.handle_padding_bytes(state, info).await?;
                let total_bytes = sent + padding_count;
                trace!(
                    "I2C slave: sent {} data bytes + {} padding bytes = {} total",
                    sent, padding_count, total_bytes
                );
                return Ok(total_bytes);
            }
        }

        result
    }

    /// Handle excess bytes after DMA buffer is full
    ///
    /// Reads and discards bytes until transaction termination to prevent interrupt flooding
    async fn handle_excess_bytes(&mut self, state: &'static State, info: &'static Info) -> Result<(), Error> {
        let mut discarded_count = 0;

        poll_fn(|cx| {
            state.waker.register(cx.waker());

            match Self::check_and_clear_error_flags(info) {
                Err(e) => {
                    error!("I2C slave: error while discarding excess bytes: {:?}", e);
                    Poll::Ready(Err(e))
                }
                Ok(sr1) => {
                    // Drain any pending data BEFORE checking for termination.
                    // This ensures we count all excess bytes even if STOP arrives
                    // at the same time as the last data byte.
                    if sr1.rxne() {
                        let _discarded_byte = info.regs.dr().read().dr();
                        discarded_count += 1;
                        Self::enable_interrupts(info);
                        return Poll::Pending;
                    }

                    if let Some(termination) = Self::check_slave_termination_conditions(sr1) {
                        match termination {
                            SlaveTermination::Stop => Self::clear_stop_flag(info),
                            SlaveTermination::Restart => {}
                            SlaveTermination::Nack => unreachable!("NACK not expected during receive"),
                        }
                        if discarded_count > 0 {
                            trace!("I2C slave: discarded {} excess bytes", discarded_count);
                        }
                        return Poll::Ready(Ok(()));
                    }

                    Self::enable_interrupts(info);
                    Poll::Pending
                }
            }
        })
        .await
    }

    /// Handle padding bytes after DMA data is exhausted
    ///
    /// Sends 0x00 bytes until transaction termination to prevent interrupt flooding
    async fn handle_padding_bytes(&mut self, state: &'static State, info: &'static Info) -> Result<usize, Error> {
        let mut padding_count = 0;

        poll_fn(|cx| {
            state.waker.register(cx.waker());

            match Self::check_and_clear_error_flags(info) {
                Err(Error::Nack) => Poll::Ready(Ok(padding_count)),
                Err(e) => {
                    error!("I2C slave: error while sending padding bytes: {:?}", e);
                    Poll::Ready(Err(e))
                }
                Ok(sr1) => {
                    if let Some(termination) = Self::check_slave_termination_conditions(sr1) {
                        match termination {
                            SlaveTermination::Stop => Self::clear_stop_flag(info),
                            SlaveTermination::Restart => {}
                            SlaveTermination::Nack => {
                                info.regs.sr1().write(|reg| {
                                    reg.0 = !0;
                                    reg.set_af(false);
                                });
                            }
                        }
                        return Poll::Ready(Ok(padding_count));
                    }

                    if sr1.txe() {
                        info.regs.dr().write(|w| w.set_dr(0x00));
                        padding_count += 1;
                        Self::enable_interrupts(info);
                        return Poll::Pending;
                    }

                    Self::enable_interrupts(info);
                    Poll::Pending
                }
            }
        })
        .await
    }
}

/// Timing configuration for I2C v1 hardware
///
/// This struct encapsulates the complex timing calculations required for STM32 I2C v1
/// peripherals, which use three separate registers (CR2.FREQ, CCR, TRISE) instead of
/// the unified TIMINGR register found in v2 hardware.
struct Timings {
    freq: u8,   // APB frequency in MHz for CR2.FREQ register
    mode: Mode, // Standard or Fast mode selection
    trise: u8,  // Rise time compensation value
    ccr: u16,   // Clock control register value
    duty: Duty, // Fast mode duty cycle selection
}

impl Timings {
    fn new(i2cclk: Hertz, frequency: Hertz) -> Self {
        // Calculate settings for I2C speed modes
        let frequency = frequency.0;
        let clock = i2cclk.0;
        let freq = clock / 1_000_000;
        assert!((2..=50).contains(&freq));

        // Configure bus frequency into I2C peripheral
        let trise = if frequency <= 100_000 {
            freq + 1
        } else {
            (freq * 300) / 1000 + 1
        };

        let mut ccr;
        let duty;
        let mode;

        // I2C clock control calculation
        if frequency <= 100_000 {
            duty = Duty::Duty2_1;
            mode = Mode::Standard;
            ccr = {
                let ccr = clock / (frequency * 2);
                if ccr < 4 { 4 } else { ccr }
            };
        } else {
            const DUTYCYCLE: u8 = 0;
            mode = Mode::Fast;
            if DUTYCYCLE == 0 {
                duty = Duty::Duty2_1;
                ccr = clock / (frequency * 3);
                ccr = if ccr < 1 { 1 } else { ccr };
            } else {
                duty = Duty::Duty16_9;
                ccr = clock / (frequency * 25);
                ccr = if ccr < 1 { 1 } else { ccr };
            }
        }

        Self {
            freq: freq as u8,
            trise: trise as u8,
            ccr: ccr as u16,
            duty,
            mode,
        }
    }
}

impl<'d, M: PeriMode> SetConfig for I2c<'d, M, Master> {
    type Config = Hertz;
    type ConfigError = ();
    fn set_config(&mut self, config: &Self::Config) -> Result<(), ()> {
        let timings = Timings::new(self.kernel_clock, *config);
        self.info.regs.cr2().modify(|reg| {
            reg.set_freq(timings.freq);
        });
        self.info.regs.ccr().modify(|reg| {
            reg.set_f_s(timings.mode.f_s());
            reg.set_duty(timings.duty.duty());
            reg.set_ccr(timings.ccr);
        });
        self.info.regs.trise().modify(|reg| {
            reg.set_trise(timings.trise);
        });

        Ok(())
    }
}
