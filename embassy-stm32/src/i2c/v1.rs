//! # I2Cv1
//!
//! This implementation is used for STM32F1, STM32F2, STM32F4, and STM32L1 devices.
//!
//! All other devices (as of 2023-12-28) use [`v2`](super::v2) instead.

use core::future::poll_fn;
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_futures::select::{select, Either};
use embassy_hal_internal::drop::OnDrop;
use embedded_hal_1::i2c::Operation;
use mode::Master;

use super::*;
use crate::mode::Mode;
use crate::pac::i2c;

use embassy_sync::waitqueue::AtomicWaker;

/// I2C v2 peripheral state  
pub(crate) struct State {
    pub(crate) waker: AtomicWaker,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

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
    trace!("i2c v1 interrupt triggered");
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

impl<'d, M: Mode, IM: MasterMode> I2c<'d, M, IM> {
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
            reg.set_f_s(timings.f_s);
            reg.set_duty(timings.duty);
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

    fn write_bytes(&mut self, address: u8, write_buffer: &[u8], timeout: Timeout, framing: OperationFraming) -> Result<(), Error> {
        if framing.send_start() {
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

        if framing.send_stop() {
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
        framing: OperationFraming,
    ) -> Result<(), Error> {
        let Some((last_byte, read_buffer)) = read_buffer.split_last_mut() else {
            return Err(Error::Overrun);
        };

        if framing.send_start() {
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
            if framing.send_nack() {
                reg.set_ack(false);
            }
            if framing.send_stop() {
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
        self.blocking_read_timeout(address, read_buffer, self.timeout(), OperationFraming::FirstAndLast)
    }

    /// Blocking write.
    pub fn blocking_write(&mut self, address: u8, write_buffer: &[u8]) -> Result<(), Error> {
        self.write_bytes(address, write_buffer, self.timeout(), OperationFraming::FirstAndLast)?;

        // Fallthrough is success
        Ok(())
    }

    /// Blocking write, restart, read.
    pub fn blocking_write_read(&mut self, address: u8, write_buffer: &[u8], read_buffer: &mut [u8]) -> Result<(), Error> {
        // Check empty read buffer before starting transaction. Otherwise, we would not generate the
        // stop condition below.
        if read_buffer.is_empty() {
            return Err(Error::Overrun);
        }

        let timeout = self.timeout();

        self.write_bytes(address, write_buffer, timeout, OperationFraming::First)?;
        self.blocking_read_timeout(address, read_buffer, timeout, OperationFraming::FirstAndLast)?;

        Ok(())
    }

    /// Blocking transaction with operations.
    ///
    /// Consecutive operations of same type are merged. See [transaction contract] for details.
    ///
    /// [transaction contract]: embedded_hal_1::i2c::I2c::transaction
    pub fn blocking_transaction(&mut self, address: u8, operations: &mut [Operation<'_>]) -> Result<(), Error> {
        let timeout = self.timeout();

        for (op, framing) in assign_operation_framing(operations)? {
            match op {
                Operation::Read(read_buffer) => self.blocking_read_timeout(address, read_buffer, timeout, framing)?,
                Operation::Write(write_buffer) => self.write_bytes(address, write_buffer, timeout, framing)?,
            }
        }

        Ok(())
    }

    // Async

    #[inline] // pretty sure this should always be inlined
    fn enable_interrupts(info: &'static Info) -> () {
        info.regs.cr2().modify(|w| {
            w.set_iterren(true);
            w.set_itevten(true);
        });
    }
}

impl<'d, IM: MasterMode> I2c<'d, Async, IM> {
    async fn write_with_framing(&mut self, address: u8, write_buffer: &[u8], framing: OperationFraming) -> Result<(), Error> {
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

        if framing.send_start() {
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
                    Err(e) => Poll::Ready(Err(e)),
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

            self.tx_dma.as_mut().unwrap().write(write_buffer, dst, Default::default())
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

        if framing.send_stop() {
            // The I2C transfer itself will take longer than the DMA transfer, so wait for that to finish too.

            // 18.3.8 “Master transmitter: In the interrupt routine after the EOT interrupt, disable DMA
            // requests then wait for a BTF event before programming the Stop condition.”
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
        self.write_with_framing(address, write_buffer, OperationFraming::FirstAndLast)
            .await?;

        Ok(())
    }

    /// Read.
    pub async fn read(&mut self, address: u8, read_buffer: &mut [u8]) -> Result<(), Error> {
        self.read_with_framing(address, read_buffer, OperationFraming::FirstAndLast)
            .await?;

        Ok(())
    }

    async fn read_with_framing(&mut self, address: u8, read_buffer: &mut [u8], framing: OperationFraming) -> Result<(), Error> {
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
            w.set_last(framing.send_nack() && !single_byte);
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

        if framing.send_start() {
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
                    Err(e) => Poll::Ready(Err(e)),
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
            if framing.send_nack() && single_byte {
                self.info.regs.cr1().modify(|w| {
                    w.set_ack(false);
                });
            }

            // Clear condition by reading SR2
            self.info.regs.sr2().read();
        } else {
            // Before starting reception of single byte (but without START condition, i.e. in case
            // of merged operations), program NACK to emit at end of this byte.
            if framing.send_nack() && single_byte {
                self.info.regs.cr1().modify(|w| {
                    w.set_ack(false);
                });
            }
        }

        // 18.3.8: When a single byte must be received: [snip] Then the user can program the STOP
        // condition either after clearing ADDR flag, or in the DMA Transfer Complete interrupt
        // routine.
        if framing.send_stop() && single_byte {
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

        if framing.send_stop() && !single_byte {
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
        // Check empty read buffer before starting transaction. Otherwise, we would not generate the
        // stop condition below.
        if read_buffer.is_empty() {
            return Err(Error::Overrun);
        }

        self.write_with_framing(address, write_buffer, OperationFraming::First).await?;
        self.read_with_framing(address, read_buffer, OperationFraming::FirstAndLast).await
    }

    /// Transaction with operations.
    ///
    /// Consecutive operations of same type are merged. See [transaction contract] for details.
    ///
    /// [transaction contract]: embedded_hal_1::i2c::I2c::transaction
    pub async fn transaction(&mut self, address: u8, operations: &mut [Operation<'_>]) -> Result<(), Error> {
        for (op, framing) in assign_operation_framing(operations)? {
            match op {
                Operation::Read(read_buffer) => self.read_with_framing(address, read_buffer, framing).await?,
                Operation::Write(write_buffer) => self.write_with_framing(address, write_buffer, framing).await?,
            }
        }

        Ok(())
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

impl<'d, M: Mode> I2c<'d, M, Master> {
    /// Configure the I2C driver for slave operations, allowing for the driver to be used as a slave and a master (multimaster)
    pub fn into_slave_multimaster(mut self, slave_addr_config: SlaveAddrConfig) -> I2c<'d, M, MultiMaster> {
        let mut slave = I2c {
            info: self.info,
            state: self.state,
            kernel_clock: self.kernel_clock,
            tx_dma: self.tx_dma.take(),  // Use take() to move ownership
            rx_dma: self.rx_dma.take(),  // Use take() to move ownership
            #[cfg(feature = "time")]
            timeout: self.timeout,
            _phantom: PhantomData,
            _phantom2: PhantomData,
            _drop_guard: self._drop_guard,  // Move the drop guard
        };
        slave.init_slave(slave_addr_config);
        slave
    }
}

impl<'d, M: Mode> I2c<'d, M, MultiMaster> {
    /// Listen for incoming I2C address match and return the command type
    /// 
    /// This method blocks until the slave address is matched by a master.
    /// Returns the command type (Read/Write) and the matched address.
    pub fn blocking_listen(&mut self) -> Result<SlaveCommand, Error> {
        trace!("I2C slave: listening for address match");
        let result = self.blocking_listen_with_timeout(self.timeout());
        trace!("I2C slave: listen result={:?}", result);
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
        trace!("I2C slave: responding to read, data_len={}", data.len());
        let result = self.transmit_to_master(data, self.timeout());
        trace!("I2C slave: read response complete, result={:?}", result);
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
        trace!("I2C slave: responding to write, buffer_len={}", buffer.len());
        let result = self.receive_from_master(buffer, self.timeout());
        trace!("I2C slave: write response complete, result={:?}", result);
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
                
                let matched_address = self.decode_matched_address(sr2)?;
                trace!("I2C slave: address matched, direction={:?}, addr={:?}", direction, matched_address);
                
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
        
        loop {
            // Determine next byte to send
            let byte_to_send = if bytes_transmitted < data.len() {
                data[bytes_transmitted]
            } else {
                0x00 // Send padding bytes when data is exhausted
            };
            
            // Attempt to send the byte
            match self.transmit_byte(byte_to_send, timeout)? {
                TransmitResult::Acknowledged => {
                    bytes_transmitted += 1;
                    // Continue transmission
                },
                TransmitResult::NotAcknowledged => {
                    bytes_transmitted += 1; // Count the NACKed byte
                    break; // Normal end of read transaction
                },
                TransmitResult::Stopped | TransmitResult::Restarted => {
                    break; // Transaction terminated by master
                }
            }
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
                },
                ReceiveResult::Stopped | ReceiveResult::Restarted => {
                    return Ok(bytes_stored);
                },
            }
        }
        
        // Handle buffer overflow by discarding excess bytes
        if bytes_stored == buffer.len() {
            trace!("I2C slave: buffer full, discarding excess bytes");
            self.discard_excess_bytes(timeout)?;
        }
        
        Ok(bytes_stored)
    }
    
    /// Discard excess bytes when buffer is full
    fn discard_excess_bytes(&mut self, timeout: Timeout) -> Result<(), Error> {
        loop {
            match self.receive_byte(timeout)? {
                ReceiveResult::Data(_) => {
                    // Byte received and ACKed, but discarded
                    continue;
                },
                ReceiveResult::Stopped | ReceiveResult::Restarted => {
                    break; // Transaction completed
                },
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
                self.clear_stop_flag();
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
                self.clear_stop_flag();
                return Ok(ReceiveResult::Stopped);
            }
            
            timeout.check()?;
        }
    }
    
    /// Determine which slave address was matched based on SR2 flags
    fn decode_matched_address(&self, sr2: i2c::regs::Sr2) -> Result<Address, Error> {
        if sr2.gencall() {
            Ok(Address::SevenBit(0x00)) // General call address
        } else if sr2.dualf() {
            // OA2 (secondary address) was matched
            let oar2 = self.info.regs.oar2().read();
            if oar2.endual() != i2c::vals::Endual::DUAL {
                return Err(Error::Bus); // Hardware inconsistency
            }
            Ok(Address::SevenBit(oar2.add2()))
        } else {
            // OA1 (primary address) was matched
            let oar1 = self.info.regs.oar1().read();
            match oar1.addmode() {
                i2c::vals::Addmode::BIT7 => {
                    let addr = (oar1.add() >> 1) as u8;
                    Ok(Address::SevenBit(addr))
                },
                i2c::vals::Addmode::BIT10 => {
                    Ok(Address::TenBit(oar1.add()))
                },
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
            },
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
                self.clear_stop_flag();
                Error::Bus // Unexpected STOP during setup
            },
            TransmitResult::Restarted => {
                Error::Bus // Unexpected RESTART during setup
            },
            TransmitResult::NotAcknowledged => {
                self.clear_acknowledge_failure();
                Error::Bus // Unexpected NACK during setup
            },
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
    
    /// Clear the stop condition flag  
    fn clear_stop_flag(&mut self) {
        self.info.regs.cr1().modify(|_w| {});
    }
}

// Address configuration methods
impl<'d, M: Mode, IM: MasterMode> I2c<'d, M, IM> {
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
        });
        
        trace!("I2C slave: initialization complete");
    }
    
    /// Apply the complete address configuration for slave mode
    fn apply_address_configuration(&mut self, config: SlaveAddrConfig) {
        match config.addr {
            OwnAddresses::OA1(addr) => {
                self.configure_primary_address(addr);
                self.disable_secondary_address();
            },
            OwnAddresses::OA2(oa2) => {
                self.configure_default_primary_address();
                self.configure_secondary_address(oa2.addr); // v1 ignores mask
            },
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
            },
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

/// Timing configuration for I2C v1 hardware
/// 
/// This struct encapsulates the complex timing calculations required for STM32 I2C v1 
/// peripherals, which use three separate registers (CR2.FREQ, CCR, TRISE) instead of 
/// the unified TIMINGR register found in v2 hardware.
struct Timings {
    freq: u8,                   // APB frequency in MHz for CR2.FREQ register
    f_s: i2c::vals::FS,         // Standard or Fast mode selection  
    trise: u8,                  // Rise time compensation value
    ccr: u16,                   // Clock control register value
    duty: i2c::vals::Duty,      // Fast mode duty cycle selection
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
        let f_s;

        // I2C clock control calculation
        if frequency <= 100_000 {
            duty = i2c::vals::Duty::DUTY2_1;
            f_s = i2c::vals::FS::STANDARD;
            ccr = {
                let ccr = clock / (frequency * 2);
                if ccr < 4 {
                    4
                } else {
                    ccr
                }
            };
        } else {
            const DUTYCYCLE: u8 = 0;
            f_s = i2c::vals::FS::FAST;
            if DUTYCYCLE == 0 {
                duty = i2c::vals::Duty::DUTY2_1;
                ccr = clock / (frequency * 3);
                ccr = if ccr < 1 { 1 } else { ccr };
            } else {
                duty = i2c::vals::Duty::DUTY16_9;
                ccr = clock / (frequency * 25);
                ccr = if ccr < 1 { 1 } else { ccr };
            }
        }

        Self {
            freq: freq as u8,
            f_s,
            trise: trise as u8,
            ccr: ccr as u16,
            duty,
        }
    }
}

impl<'d, M: Mode> SetConfig for I2c<'d, M, Master> {
    type Config = Hertz;
    type ConfigError = ();
    fn set_config(&mut self, config: &Self::Config) -> Result<(), ()> {
        let timings = Timings::new(self.kernel_clock, *config);
        self.info.regs.cr2().modify(|reg| {
            reg.set_freq(timings.freq);
        });
        self.info.regs.ccr().modify(|reg| {
            reg.set_f_s(timings.f_s);
            reg.set_duty(timings.duty);
            reg.set_ccr(timings.ccr);
        });
        self.info.regs.trise().modify(|reg| {
            reg.set_trise(timings.trise);
        });

        Ok(())
    }
}
