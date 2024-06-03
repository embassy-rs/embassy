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

impl<'d, M: PeriMode> I2c<'d, M> {
    pub(crate) fn init(&mut self, freq: Hertz, _config: Config) {
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

        let timings = Timings::new(self.kernel_clock, freq);

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

    fn write_bytes(&mut self, addr: u8, bytes: &[u8], timeout: Timeout, frame: FrameOptions) -> Result<(), Error> {
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
            self.info.regs.dr().write(|reg| reg.set_dr(addr << 1));

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
        for c in bytes {
            self.send_byte(*c, timeout)?;
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
        addr: u8,
        buffer: &mut [u8],
        timeout: Timeout,
        frame: FrameOptions,
    ) -> Result<(), Error> {
        let Some((last, buffer)) = buffer.split_last_mut() else {
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
            self.info.regs.dr().write(|reg| reg.set_dr((addr << 1) + 1));

            // Wait until address was sent
            // Wait for the address to be acknowledged
            while !Self::check_and_clear_error_flags(self.info)?.addr() {
                timeout.check()?;
            }

            // Clear condition by reading SR2
            let _ = self.info.regs.sr2().read();
        }

        // Receive bytes into buffer
        for c in buffer {
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
        *last = self.recv_byte(timeout)?;

        // Fallthrough is success
        Ok(())
    }

    /// Blocking read.
    pub fn blocking_read(&mut self, addr: u8, read: &mut [u8]) -> Result<(), Error> {
        self.blocking_read_timeout(addr, read, self.timeout(), FrameOptions::FirstAndLastFrame)
    }

    /// Blocking write.
    pub fn blocking_write(&mut self, addr: u8, write: &[u8]) -> Result<(), Error> {
        self.write_bytes(addr, write, self.timeout(), FrameOptions::FirstAndLastFrame)?;

        // Fallthrough is success
        Ok(())
    }

    /// Blocking write, restart, read.
    pub fn blocking_write_read(&mut self, addr: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        // Check empty read buffer before starting transaction. Otherwise, we would not generate the
        // stop condition below.
        if read.is_empty() {
            return Err(Error::Overrun);
        }

        let timeout = self.timeout();

        self.write_bytes(addr, write, timeout, FrameOptions::FirstFrame)?;
        self.blocking_read_timeout(addr, read, timeout, FrameOptions::FirstAndLastFrame)?;

        Ok(())
    }

    /// Blocking transaction with operations.
    ///
    /// Consecutive operations of same type are merged. See [transaction contract] for details.
    ///
    /// [transaction contract]: embedded_hal_1::i2c::I2c::transaction
    pub fn blocking_transaction(&mut self, addr: u8, operations: &mut [Operation<'_>]) -> Result<(), Error> {
        let timeout = self.timeout();

        for (op, frame) in operation_frames(operations)? {
            match op {
                Operation::Read(read) => self.blocking_read_timeout(addr, read, timeout, frame)?,
                Operation::Write(write) => self.write_bytes(addr, write, timeout, frame)?,
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

impl<'d> I2c<'d, Async> {
    async fn write_frame(&mut self, address: u8, write: &[u8], frame: FrameOptions) -> Result<(), Error> {
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

            self.tx_dma.as_mut().unwrap().write(write, dst, Default::default())
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

        if frame.send_stop() {
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
    pub async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Error> {
        self.write_frame(address, write, FrameOptions::FirstAndLastFrame)
            .await?;

        Ok(())
    }

    /// Read.
    pub async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        self.read_frame(address, buffer, FrameOptions::FirstAndLastFrame)
            .await?;

        Ok(())
    }

    async fn read_frame(&mut self, address: u8, buffer: &mut [u8], frame: FrameOptions) -> Result<(), Error> {
        if buffer.is_empty() {
            return Err(Error::Overrun);
        }

        // Some branches below depend on whether the buffer contains only a single byte.
        let single_byte = buffer.len() == 1;

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
            if frame.send_nack() && single_byte {
                self.info.regs.cr1().modify(|w| {
                    w.set_ack(false);
                });
            }

            // Clear condition by reading SR2
            self.info.regs.sr2().read();
        } else {
            // Before starting reception of single byte (but without START condition, i.e. in case
            // of continued frame), program NACK to emit at end of this byte.
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

            self.rx_dma.as_mut().unwrap().read(src, buffer, Default::default())
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
    pub async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        // Check empty read buffer before starting transaction. Otherwise, we would not generate the
        // stop condition below.
        if read.is_empty() {
            return Err(Error::Overrun);
        }

        self.write_frame(address, write, FrameOptions::FirstFrame).await?;
        self.read_frame(address, read, FrameOptions::FirstAndLastFrame).await
    }

    /// Transaction with operations.
    ///
    /// Consecutive operations of same type are merged. See [transaction contract] for details.
    ///
    /// [transaction contract]: embedded_hal_1::i2c::I2c::transaction
    pub async fn transaction(&mut self, addr: u8, operations: &mut [Operation<'_>]) -> Result<(), Error> {
        for (op, frame) in operation_frames(operations)? {
            match op {
                Operation::Read(read) => self.read_frame(addr, read, frame).await?,
                Operation::Write(write) => self.write_frame(addr, write, frame).await?,
            }
        }

        Ok(())
    }
}

impl<'d, M: PeriMode> Drop for I2c<'d, M> {
    fn drop(&mut self) {
        self.info.rcc.disable()
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

struct Timings {
    freq: u8,
    mode: Mode,
    trise: u8,
    ccr: u16,
    duty: Duty,
}

impl Timings {
    fn new(i2cclk: Hertz, speed: Hertz) -> Self {
        // Calculate settings for I2C speed modes
        let speed = speed.0;
        let clock = i2cclk.0;
        let freq = clock / 1_000_000;
        assert!((2..=50).contains(&freq));

        // Configure bus frequency into I2C peripheral
        let trise = if speed <= 100_000 {
            freq + 1
        } else {
            (freq * 300) / 1000 + 1
        };

        let mut ccr;
        let duty;
        let mode;

        // I2C clock control calculation
        if speed <= 100_000 {
            duty = Duty::Duty2_1;
            mode = Mode::Standard;
            ccr = {
                let ccr = clock / (speed * 2);
                if ccr < 4 {
                    4
                } else {
                    ccr
                }
            };
        } else {
            const DUTYCYCLE: u8 = 0;
            mode = Mode::Fast;
            if DUTYCYCLE == 0 {
                duty = Duty::Duty2_1;
                ccr = clock / (speed * 3);
                ccr = if ccr < 1 { 1 } else { ccr };

                // Set clock to fast mode with appropriate parameters for selected speed (2:1 duty cycle)
            } else {
                duty = Duty::Duty16_9;
                ccr = clock / (speed * 25);
                ccr = if ccr < 1 { 1 } else { ccr };

                // Set clock to fast mode with appropriate parameters for selected speed (16:9 duty cycle)
            }
        }

        Self {
            freq: freq as u8,
            trise: trise as u8,
            ccr: ccr as u16,
            duty,
            mode,
            //prescale: presc_reg,
            //scll,
            //sclh,
            //sdadel,
            //scldel,
        }
    }
}

impl<'d, M: PeriMode> SetConfig for I2c<'d, M> {
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
