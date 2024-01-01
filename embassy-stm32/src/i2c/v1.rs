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

use super::*;
use crate::dma::Transfer;
use crate::pac::i2c;
use crate::time::Hertz;

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
    let regs = T::regs();
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

impl<'d, T: Instance, TXDMA, RXDMA> I2c<'d, T, TXDMA, RXDMA> {
    pub(crate) fn init(&mut self, freq: Hertz, _config: Config) {
        T::regs().cr1().modify(|reg| {
            reg.set_pe(false);
            //reg.set_anfoff(false);
        });

        let timings = Timings::new(T::frequency(), freq);

        T::regs().cr2().modify(|reg| {
            reg.set_freq(timings.freq);
        });
        T::regs().ccr().modify(|reg| {
            reg.set_f_s(timings.mode.f_s());
            reg.set_duty(timings.duty.duty());
            reg.set_ccr(timings.ccr);
        });
        T::regs().trise().modify(|reg| {
            reg.set_trise(timings.trise);
        });

        T::regs().cr1().modify(|reg| {
            reg.set_pe(true);
        });
    }

    fn check_and_clear_error_flags() -> Result<i2c::regs::Sr1, Error> {
        // Note that flags should only be cleared once they have been registered. If flags are
        // cleared otherwise, there may be an inherent race condition and flags may be missed.
        let sr1 = T::regs().sr1().read();

        if sr1.timeout() {
            T::regs().sr1().write(|reg| {
                reg.0 = !0;
                reg.set_timeout(false);
            });
            return Err(Error::Timeout);
        }

        if sr1.pecerr() {
            T::regs().sr1().write(|reg| {
                reg.0 = !0;
                reg.set_pecerr(false);
            });
            return Err(Error::Crc);
        }

        if sr1.ovr() {
            T::regs().sr1().write(|reg| {
                reg.0 = !0;
                reg.set_ovr(false);
            });
            return Err(Error::Overrun);
        }

        if sr1.af() {
            T::regs().sr1().write(|reg| {
                reg.0 = !0;
                reg.set_af(false);
            });
            return Err(Error::Nack);
        }

        if sr1.arlo() {
            T::regs().sr1().write(|reg| {
                reg.0 = !0;
                reg.set_arlo(false);
            });
            return Err(Error::Arbitration);
        }

        // The errata indicates that BERR may be incorrectly detected. It recommends ignoring and
        // clearing the BERR bit instead.
        if sr1.berr() {
            T::regs().sr1().write(|reg| {
                reg.0 = !0;
                reg.set_berr(false);
            });
        }

        Ok(sr1)
    }

    fn write_bytes(&mut self, addr: u8, bytes: &[u8], timeout: Timeout) -> Result<(), Error> {
        // Send a START condition

        T::regs().cr1().modify(|reg| {
            reg.set_start(true);
        });

        // Wait until START condition was generated
        while !Self::check_and_clear_error_flags()?.start() {
            timeout.check()?;
        }

        // Also wait until signalled we're master and everything is waiting for us
        while {
            Self::check_and_clear_error_flags()?;

            let sr2 = T::regs().sr2().read();
            !sr2.msl() && !sr2.busy()
        } {
            timeout.check()?;
        }

        // Set up current address, we're trying to talk to
        T::regs().dr().write(|reg| reg.set_dr(addr << 1));

        // Wait until address was sent
        // Wait for the address to be acknowledged
        // Check for any I2C errors. If a NACK occurs, the ADDR bit will never be set.
        while !Self::check_and_clear_error_flags()?.addr() {
            timeout.check()?;
        }

        // Clear condition by reading SR2
        let _ = T::regs().sr2().read();

        // Send bytes
        for c in bytes {
            self.send_byte(*c, timeout)?;
        }

        // Fallthrough is success
        Ok(())
    }

    fn send_byte(&self, byte: u8, timeout: Timeout) -> Result<(), Error> {
        // Wait until we're ready for sending
        while {
            // Check for any I2C errors. If a NACK occurs, the ADDR bit will never be set.
            !Self::check_and_clear_error_flags()?.txe()
        } {
            timeout.check()?;
        }

        // Push out a byte of data
        T::regs().dr().write(|reg| reg.set_dr(byte));

        // Wait until byte is transferred
        while {
            // Check for any potential error conditions.
            !Self::check_and_clear_error_flags()?.btf()
        } {
            timeout.check()?;
        }

        Ok(())
    }

    fn recv_byte(&self, timeout: Timeout) -> Result<u8, Error> {
        while {
            // Check for any potential error conditions.
            Self::check_and_clear_error_flags()?;

            !T::regs().sr1().read().rxne()
        } {
            timeout.check()?;
        }

        let value = T::regs().dr().read().dr();
        Ok(value)
    }

    fn blocking_read_timeout(&mut self, addr: u8, buffer: &mut [u8], timeout: Timeout) -> Result<(), Error> {
        if let Some((last, buffer)) = buffer.split_last_mut() {
            // Send a START condition and set ACK bit
            T::regs().cr1().modify(|reg| {
                reg.set_start(true);
                reg.set_ack(true);
            });

            // Wait until START condition was generated
            while !Self::check_and_clear_error_flags()?.start() {
                timeout.check()?;
            }

            // Also wait until signalled we're master and everything is waiting for us
            while {
                let sr2 = T::regs().sr2().read();
                !sr2.msl() && !sr2.busy()
            } {
                timeout.check()?;
            }

            // Set up current address, we're trying to talk to
            T::regs().dr().write(|reg| reg.set_dr((addr << 1) + 1));

            // Wait until address was sent
            // Wait for the address to be acknowledged
            while !Self::check_and_clear_error_flags()?.addr() {
                timeout.check()?;
            }

            // Clear condition by reading SR2
            let _ = T::regs().sr2().read();

            // Receive bytes into buffer
            for c in buffer {
                *c = self.recv_byte(timeout)?;
            }

            // Prepare to send NACK then STOP after next byte
            T::regs().cr1().modify(|reg| {
                reg.set_ack(false);
                reg.set_stop(true);
            });

            // Receive last byte
            *last = self.recv_byte(timeout)?;

            // Wait for the STOP to be sent.
            while T::regs().cr1().read().stop() {
                timeout.check()?;
            }

            // Fallthrough is success
            Ok(())
        } else {
            Err(Error::Overrun)
        }
    }

    /// Blocking read.
    pub fn blocking_read(&mut self, addr: u8, read: &mut [u8]) -> Result<(), Error> {
        self.blocking_read_timeout(addr, read, self.timeout())
    }

    /// Blocking write.
    pub fn blocking_write(&mut self, addr: u8, write: &[u8]) -> Result<(), Error> {
        let timeout = self.timeout();

        self.write_bytes(addr, write, timeout)?;
        // Send a STOP condition
        T::regs().cr1().modify(|reg| reg.set_stop(true));
        // Wait for STOP condition to transmit.
        while T::regs().cr1().read().stop() {
            timeout.check()?;
        }

        // Fallthrough is success
        Ok(())
    }

    /// Blocking write, restart, read.
    pub fn blocking_write_read(&mut self, addr: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        let timeout = self.timeout();

        self.write_bytes(addr, write, timeout)?;
        self.blocking_read_timeout(addr, read, timeout)?;

        Ok(())
    }

    // Async

    #[inline] // pretty sure this should always be inlined
    fn enable_interrupts() -> () {
        T::regs().cr2().modify(|w| {
            w.set_iterren(true);
            w.set_itevten(true);
        });
    }

    async fn write_with_stop(&mut self, address: u8, write: &[u8], send_stop: bool) -> Result<(), Error>
    where
        TXDMA: crate::i2c::TxDma<T>,
    {
        let dma_transfer = unsafe {
            let regs = T::regs();
            regs.cr2().modify(|w| {
                // DMA mode can be enabled for transmission by setting the DMAEN bit in the I2C_CR2 register.
                w.set_dmaen(true);
                w.set_itbufen(false);
            });
            // Set the I2C_DR register address in the DMA_SxPAR register. The data will be moved to this address from the memory after each TxE event.
            let dst = regs.dr().as_ptr() as *mut u8;

            let ch = &mut self.tx_dma;
            let request = ch.request();
            Transfer::new_write(ch, request, write, dst, Default::default())
        };

        let on_drop = OnDrop::new(|| {
            let regs = T::regs();
            regs.cr2().modify(|w| {
                w.set_dmaen(false);
                w.set_iterren(false);
                w.set_itevten(false);
            })
        });

        Self::enable_interrupts();

        // Send a START condition
        T::regs().cr1().modify(|reg| {
            reg.set_start(true);
        });

        let state = T::state();

        // Wait until START condition was generated
        poll_fn(|cx| {
            state.waker.register(cx.waker());

            match Self::check_and_clear_error_flags() {
                Err(e) => Poll::Ready(Err(e)),
                Ok(sr1) => {
                    if sr1.start() {
                        Poll::Ready(Ok(()))
                    } else {
                        Poll::Pending
                    }
                }
            }
        })
        .await?;

        // Also wait until signalled we're master and everything is waiting for us
        Self::enable_interrupts();
        poll_fn(|cx| {
            state.waker.register(cx.waker());

            match Self::check_and_clear_error_flags() {
                Err(e) => Poll::Ready(Err(e)),
                Ok(_) => {
                    let sr2 = T::regs().sr2().read();
                    if !sr2.msl() && !sr2.busy() {
                        Poll::Pending
                    } else {
                        Poll::Ready(Ok(()))
                    }
                }
            }
        })
        .await?;

        // Set up current address, we're trying to talk to
        Self::enable_interrupts();
        T::regs().dr().write(|reg| reg.set_dr(address << 1));

        poll_fn(|cx| {
            state.waker.register(cx.waker());
            match Self::check_and_clear_error_flags() {
                Err(e) => Poll::Ready(Err(e)),
                Ok(sr1) => {
                    if sr1.addr() {
                        // Clear the ADDR condition by reading SR2.
                        T::regs().sr2().read();
                        Poll::Ready(Ok(()))
                    } else {
                        // If we need to go around, then re-enable the interrupts, otherwise nothing
                        // can wake us up and we'll hang.
                        Self::enable_interrupts();
                        Poll::Pending
                    }
                }
            }
        })
        .await?;
        Self::enable_interrupts();
        let poll_error = poll_fn(|cx| {
            state.waker.register(cx.waker());

            match Self::check_and_clear_error_flags() {
                // Unclear why the Err turbofish is necessary here? The compiler didn’t require it in the other
                // identical poll_fn check_and_clear matches.
                Err(e) => Poll::Ready(Err::<T, Error>(e)),
                Ok(_) => Poll::Pending,
            }
        });

        // Wait for either the DMA transfer to successfully finish, or an I2C error to occur.
        match select(dma_transfer, poll_error).await {
            Either::Second(Err(e)) => Err(e),
            _ => Ok(()),
        }?;

        // The I2C transfer itself will take longer than the DMA transfer, so wait for that to finish too.

        // 18.3.8 “Master transmitter: In the interrupt routine after the EOT interrupt, disable DMA
        // requests then wait for a BTF event before programming the Stop condition.”

        // TODO: If this has to be done “in the interrupt routine after the EOT interrupt”, where to put it?
        T::regs().cr2().modify(|w| {
            w.set_dmaen(false);
        });

        Self::enable_interrupts();
        poll_fn(|cx| {
            state.waker.register(cx.waker());

            match Self::check_and_clear_error_flags() {
                Err(e) => Poll::Ready(Err(e)),
                Ok(sr1) => {
                    if sr1.btf() {
                        if send_stop {
                            T::regs().cr1().modify(|w| {
                                w.set_stop(true);
                            });
                        }

                        Poll::Ready(Ok(()))
                    } else {
                        Poll::Pending
                    }
                }
            }
        })
        .await?;

        drop(on_drop);

        // Fallthrough is success
        Ok(())
    }

    /// Write.
    pub async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Error>
    where
        TXDMA: crate::i2c::TxDma<T>,
    {
        self.write_with_stop(address, write, true).await?;

        // Wait for STOP condition to transmit.
        Self::enable_interrupts();
        poll_fn(|cx| {
            T::state().waker.register(cx.waker());
            // TODO: error interrupts are enabled here, should we additional check for and return errors?
            if T::regs().cr1().read().stop() {
                Poll::Pending
            } else {
                Poll::Ready(Ok(()))
            }
        })
        .await?;

        Ok(())
    }

    /// Read.
    pub async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error>
    where
        RXDMA: crate::i2c::RxDma<T>,
    {
        let state = T::state();
        let buffer_len = buffer.len();

        let dma_transfer = unsafe {
            let regs = T::regs();
            regs.cr2().modify(|w| {
                // DMA mode can be enabled for transmission by setting the DMAEN bit in the I2C_CR2 register.
                w.set_itbufen(false);
                w.set_dmaen(true);
            });
            // Set the I2C_DR register address in the DMA_SxPAR register. The data will be moved to this address from the memory after each TxE event.
            let src = regs.dr().as_ptr() as *mut u8;

            let ch = &mut self.rx_dma;
            let request = ch.request();
            Transfer::new_read(ch, request, src, buffer, Default::default())
        };

        let on_drop = OnDrop::new(|| {
            let regs = T::regs();
            regs.cr2().modify(|w| {
                w.set_dmaen(false);
                w.set_iterren(false);
                w.set_itevten(false);
            })
        });

        Self::enable_interrupts();

        // Send a START condition and set ACK bit
        T::regs().cr1().modify(|reg| {
            reg.set_start(true);
            reg.set_ack(true);
        });

        // Wait until START condition was generated
        poll_fn(|cx| {
            state.waker.register(cx.waker());

            match Self::check_and_clear_error_flags() {
                Err(e) => Poll::Ready(Err(e)),
                Ok(sr1) => {
                    if sr1.start() {
                        Poll::Ready(Ok(()))
                    } else {
                        Poll::Pending
                    }
                }
            }
        })
        .await?;

        // Also wait until signalled we're master and everything is waiting for us
        Self::enable_interrupts();
        poll_fn(|cx| {
            state.waker.register(cx.waker());

            // blocking read didn’t have a check_and_clear call here, but blocking write did so
            // I’m adding it here in case that was an oversight.
            match Self::check_and_clear_error_flags() {
                Err(e) => Poll::Ready(Err(e)),
                Ok(_) => {
                    let sr2 = T::regs().sr2().read();
                    if !sr2.msl() && !sr2.busy() {
                        Poll::Pending
                    } else {
                        Poll::Ready(Ok(()))
                    }
                }
            }
        })
        .await?;

        // Set up current address, we're trying to talk to
        T::regs().dr().write(|reg| reg.set_dr((address << 1) + 1));

        // Wait for the address to be acknowledged

        Self::enable_interrupts();
        poll_fn(|cx| {
            state.waker.register(cx.waker());

            match Self::check_and_clear_error_flags() {
                Err(e) => Poll::Ready(Err(e)),
                Ok(sr1) => {
                    if sr1.addr() {
                        // 18.3.8: When a single byte must be received: the NACK must be programmed during EV6
                        // event, i.e. program ACK=0 when ADDR=1, before clearing ADDR flag.
                        if buffer_len == 1 {
                            T::regs().cr1().modify(|w| {
                                w.set_ack(false);
                            });
                        }
                        Poll::Ready(Ok(()))
                    } else {
                        Poll::Pending
                    }
                }
            }
        })
        .await?;

        // Clear ADDR condition by reading SR2
        T::regs().sr2().read();

        // 18.3.8: When a single byte must be received: [snip] Then the
        // user can program the STOP condition either after clearing ADDR flag, or in the
        // DMA Transfer Complete interrupt routine.
        if buffer_len == 1 {
            T::regs().cr1().modify(|w| {
                w.set_stop(true);
            });
        } else {
            // If, in the I2C_CR2 register, the LAST bit is set, I2C
            // automatically sends a NACK after the next byte following EOT_1. The user can
            // generate a Stop condition in the DMA Transfer Complete interrupt routine if enabled.
            T::regs().cr2().modify(|w| {
                w.set_last(true);
            })
        }

        // Wait for bytes to be received, or an error to occur.
        Self::enable_interrupts();
        let poll_error = poll_fn(|cx| {
            state.waker.register(cx.waker());

            match Self::check_and_clear_error_flags() {
                Err(e) => Poll::Ready(Err::<T, Error>(e)),
                _ => Poll::Pending,
            }
        });

        match select(dma_transfer, poll_error).await {
            Either::Second(Err(e)) => Err(e),
            _ => Ok(()),
        }?;

        // Wait for the STOP to be sent (STOP bit cleared).
        Self::enable_interrupts();
        poll_fn(|cx| {
            state.waker.register(cx.waker());
            // TODO: error interrupts are enabled here, should we additional check for and return errors?
            if T::regs().cr1().read().stop() {
                Poll::Pending
            } else {
                Poll::Ready(Ok(()))
            }
        })
        .await?;
        drop(on_drop);

        // Fallthrough is success
        Ok(())
    }

    /// Write, restart, read.
    pub async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error>
    where
        RXDMA: crate::i2c::RxDma<T>,
        TXDMA: crate::i2c::TxDma<T>,
    {
        self.write_with_stop(address, write, false).await?;
        self.read(address, read).await
    }
}

impl<'d, T: Instance, TXDMA, RXDMA> Drop for I2c<'d, T, TXDMA, RXDMA> {
    fn drop(&mut self) {
        T::disable();
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

impl<'d, T: Instance> SetConfig for I2c<'d, T> {
    type Config = Hertz;
    type ConfigError = ();
    fn set_config(&mut self, config: &Self::Config) -> Result<(), ()> {
        let timings = Timings::new(T::frequency(), *config);
        T::regs().cr2().modify(|reg| {
            reg.set_freq(timings.freq);
        });
        T::regs().ccr().modify(|reg| {
            reg.set_f_s(timings.mode.f_s());
            reg.set_duty(timings.duty.duty());
            reg.set_ccr(timings.ccr);
        });
        T::regs().trise().modify(|reg| {
            reg.set_trise(timings.trise);
        });

        Ok(())
    }
}
