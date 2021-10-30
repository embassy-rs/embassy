use core::cmp;
use core::future::Future;
use core::marker::PhantomData;
use core::task::Poll;

use atomic_polyfill::{AtomicUsize, Ordering};
use embassy::interrupt::InterruptExt;
use embassy::traits::i2c::I2c as I2cTrait;
use embassy::util::Unborrow;
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::unborrow;
use embedded_hal::blocking::i2c::Read;
use embedded_hal::blocking::i2c::Write;
use embedded_hal::blocking::i2c::WriteRead;
use futures::future::poll_fn;

use crate::dma::NoDma;
use crate::i2c::{Error, Instance, SclPin, SdaPin};
use crate::pac;
use crate::pac::gpio::vals::{Afr, Moder, Ot};
use crate::pac::gpio::Gpio;
use crate::pac::i2c;
use crate::time::Hertz;

const I2C_COUNT: usize = pac::peripheral_count!(i2c);

pub struct State {
    waker: [AtomicWaker; I2C_COUNT],
    chunks_transferred: [AtomicUsize; I2C_COUNT],
}

impl State {
    const fn new() -> Self {
        const AW: AtomicWaker = AtomicWaker::new();
        const CT: AtomicUsize = AtomicUsize::new(0);

        Self {
            waker: [AW; I2C_COUNT],
            chunks_transferred: [CT; I2C_COUNT],
        }
    }
}

static STATE: State = State::new();

pub struct I2c<'d, T: Instance, TXDMA = NoDma, RXDMA = NoDma> {
    phantom: PhantomData<&'d mut T>,
    tx_dma: TXDMA,
    #[allow(dead_code)]
    rx_dma: RXDMA,
}

impl<'d, T: Instance, TXDMA, RXDMA> I2c<'d, T, TXDMA, RXDMA> {
    pub fn new<F>(
        _peri: impl Unborrow<Target = T> + 'd,
        scl: impl Unborrow<Target = impl SclPin<T>> + 'd,
        sda: impl Unborrow<Target = impl SdaPin<T>> + 'd,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
        tx_dma: impl Unborrow<Target = TXDMA> + 'd,
        rx_dma: impl Unborrow<Target = RXDMA> + 'd,
        freq: F,
    ) -> Self
    where
        F: Into<Hertz>,
    {
        unborrow!(irq, scl, sda, tx_dma, rx_dma);

        T::enable();

        unsafe {
            Self::configure_pin(scl.block(), scl.pin() as _, scl.af_num());
            Self::configure_pin(sda.block(), sda.pin() as _, sda.af_num());
        }

        unsafe {
            T::regs().cr1().modify(|reg| {
                reg.set_pe(false);
                reg.set_anfoff(false);
            });
        }

        let timings = Timings::new(T::frequency(), freq.into());

        unsafe {
            T::regs().timingr().write(|reg| {
                reg.set_presc(timings.prescale);
                reg.set_scll(timings.scll);
                reg.set_sclh(timings.sclh);
                reg.set_sdadel(timings.sdadel);
                reg.set_scldel(timings.scldel);
            });
        }

        unsafe {
            T::regs().cr1().modify(|reg| {
                reg.set_pe(true);
            });
        }

        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        Self {
            phantom: PhantomData,
            tx_dma,
            rx_dma,
        }
    }

    unsafe fn on_interrupt(_: *mut ()) {
        let regs = T::regs();
        let isr = regs.isr().read();

        if isr.tcr() || isr.tc() {
            let n = T::state_number();
            STATE.chunks_transferred[n].fetch_add(1, Ordering::Relaxed);
            STATE.waker[n].wake();
        }
        // The flag can only be cleared by writting to nbytes, we won't do that here, so disable
        // the interrupt
        critical_section::with(|_| {
            regs.cr1().modify(|w| w.set_tcie(false));
        });
    }

    unsafe fn configure_pin(block: Gpio, pin: usize, af_num: u8) {
        let (afr, n_af) = if pin < 8 { (0, pin) } else { (1, pin - 8) };
        block.moder().modify(|w| w.set_moder(pin, Moder::ALTERNATE));
        block.afr(afr).modify(|w| w.set_afr(n_af, Afr(af_num)));
        block.otyper().modify(|w| w.set_ot(pin, Ot::OPENDRAIN));
        //block
        //.ospeedr()
        //.modify(|w| w.set_ospeedr(pin, crate::pac::gpio::vals::Ospeedr::VERYHIGHSPEED));
    }

    fn master_stop(&mut self) {
        unsafe {
            T::regs().cr2().write(|w| w.set_stop(i2c::vals::Stop::STOP));
        }
    }

    unsafe fn master_read(address: u8, length: usize, stop: Stop, reload: bool, restart: bool) {
        assert!(length < 256 && length > 0);

        if !restart {
            // Wait for any previous address sequence to end
            // automatically. This could be up to 50% of a bus
            // cycle (ie. up to 0.5/freq)
            while T::regs().cr2().read().start() == i2c::vals::Start::START {}
        }

        // Set START and prepare to receive bytes into
        // `buffer`. The START bit can be set even if the bus
        // is BUSY or I2C is in slave mode.

        let reload = if reload {
            i2c::vals::Reload::NOTCOMPLETED
        } else {
            i2c::vals::Reload::COMPLETED
        };

        T::regs().cr2().modify(|w| {
            w.set_sadd((address << 1 | 0) as u16);
            w.set_add10(i2c::vals::Add::BIT7);
            w.set_rd_wrn(i2c::vals::RdWrn::READ);
            w.set_nbytes(length as u8);
            w.set_start(i2c::vals::Start::START);
            w.set_autoend(stop.autoend());
            w.set_reload(reload);
        });
    }

    unsafe fn master_write(address: u8, length: usize, stop: Stop, reload: bool) {
        assert!(length < 256 && length > 0);

        // Wait for any previous address sequence to end
        // automatically. This could be up to 50% of a bus
        // cycle (ie. up to 0.5/freq)
        while T::regs().cr2().read().start() == i2c::vals::Start::START {}

        let reload = if reload {
            i2c::vals::Reload::NOTCOMPLETED
        } else {
            i2c::vals::Reload::COMPLETED
        };

        // Set START and prepare to send `bytes`. The
        // START bit can be set even if the bus is BUSY or
        // I2C is in slave mode.
        T::regs().cr2().modify(|w| {
            w.set_sadd((address << 1 | 0) as u16);
            w.set_add10(i2c::vals::Add::BIT7);
            w.set_rd_wrn(i2c::vals::RdWrn::WRITE);
            w.set_nbytes(length as u8);
            w.set_start(i2c::vals::Start::START);
            w.set_autoend(stop.autoend());
            w.set_reload(reload);
        });
    }

    unsafe fn master_continue(length: usize, reload: bool) {
        assert!(length < 256 && length > 0);

        while !T::regs().isr().read().tcr() {}

        let reload = if reload {
            i2c::vals::Reload::NOTCOMPLETED
        } else {
            i2c::vals::Reload::COMPLETED
        };

        T::regs().cr2().modify(|w| {
            w.set_nbytes(length as u8);
            w.set_reload(reload);
        });
    }

    fn flush_txdr(&self) {
        //if $i2c.isr.read().txis().bit_is_set() {
        //$i2c.txdr.write(|w| w.txdata().bits(0));
        //}

        unsafe {
            if T::regs().isr().read().txis() {
                T::regs().txdr().write(|w| w.set_txdata(0));
            }
            if T::regs().isr().read().txe() {
                T::regs().isr().modify(|w| w.set_txe(true))
            }
        }

        // If TXDR is not flagged as empty, write 1 to flush it
        //if $i2c.isr.read().txe().is_not_empty() {
        //$i2c.isr.write(|w| w.txe().set_bit());
        //}
    }

    fn wait_txe(&self) -> Result<(), Error> {
        loop {
            unsafe {
                let isr = T::regs().isr().read();
                if isr.txe() {
                    return Ok(());
                } else if isr.berr() {
                    T::regs().icr().write(|reg| reg.set_berrcf(true));
                    return Err(Error::Bus);
                } else if isr.arlo() {
                    T::regs().icr().write(|reg| reg.set_arlocf(true));
                    return Err(Error::Arbitration);
                } else if isr.nackf() {
                    T::regs().icr().write(|reg| reg.set_nackcf(true));
                    self.flush_txdr();
                    return Err(Error::Nack);
                }
            }
        }
    }

    fn wait_rxne(&self) -> Result<(), Error> {
        loop {
            unsafe {
                let isr = T::regs().isr().read();
                if isr.rxne() {
                    return Ok(());
                } else if isr.berr() {
                    T::regs().icr().write(|reg| reg.set_berrcf(true));
                    return Err(Error::Bus);
                } else if isr.arlo() {
                    T::regs().icr().write(|reg| reg.set_arlocf(true));
                    return Err(Error::Arbitration);
                } else if isr.nackf() {
                    T::regs().icr().write(|reg| reg.set_nackcf(true));
                    self.flush_txdr();
                    return Err(Error::Nack);
                }
            }
        }
    }

    fn wait_tc(&self) -> Result<(), Error> {
        loop {
            unsafe {
                let isr = T::regs().isr().read();
                if isr.tc() {
                    return Ok(());
                } else if isr.berr() {
                    T::regs().icr().write(|reg| reg.set_berrcf(true));
                    return Err(Error::Bus);
                } else if isr.arlo() {
                    T::regs().icr().write(|reg| reg.set_arlocf(true));
                    return Err(Error::Arbitration);
                } else if isr.nackf() {
                    T::regs().icr().write(|reg| reg.set_nackcf(true));
                    self.flush_txdr();
                    return Err(Error::Nack);
                }
            }
        }
    }

    fn read(&mut self, address: u8, buffer: &mut [u8], restart: bool) -> Result<(), Error> {
        let completed_chunks = buffer.len() / 255;
        let total_chunks = if completed_chunks * 255 == buffer.len() {
            completed_chunks
        } else {
            completed_chunks + 1
        };
        let last_chunk_idx = total_chunks.saturating_sub(1);

        unsafe {
            Self::master_read(
                address,
                buffer.len().min(255),
                Stop::Automatic,
                last_chunk_idx != 0,
                restart,
            );
        }

        for (number, chunk) in buffer.chunks_mut(255).enumerate() {
            if number != 0 {
                // NOTE(unsafe) We have &mut self
                unsafe {
                    Self::master_continue(chunk.len(), number != last_chunk_idx);
                }
            }

            for byte in chunk {
                // Wait until we have received something
                self.wait_rxne()?;

                unsafe {
                    *byte = T::regs().rxdr().read().rxdata();
                }
            }
        }
        Ok(())
    }

    fn write(&mut self, address: u8, bytes: &[u8], send_stop: bool) -> Result<(), Error> {
        let completed_chunks = bytes.len() / 255;
        let total_chunks = if completed_chunks * 255 == bytes.len() {
            completed_chunks
        } else {
            completed_chunks + 1
        };
        let last_chunk_idx = total_chunks.saturating_sub(1);

        // I2C start
        //
        // ST SAD+W
        // NOTE(unsafe) We have &mut self
        unsafe {
            Self::master_write(
                address,
                bytes.len().min(255),
                Stop::Software,
                last_chunk_idx != 0,
            );
        }

        for (number, chunk) in bytes.chunks(255).enumerate() {
            if number != 0 {
                // NOTE(unsafe) We have &mut self
                unsafe {
                    Self::master_continue(chunk.len(), number != last_chunk_idx);
                }
            }

            for byte in chunk {
                // Wait until we are allowed to send data
                // (START has been ACKed or last byte when
                // through)
                self.wait_txe()?;

                unsafe {
                    T::regs().txdr().write(|w| w.set_txdata(*byte));
                }
            }
        }
        // Wait until the write finishes
        self.wait_tc()?;

        if send_stop {
            self.master_stop();
        }
        Ok(())
    }

    async fn write_dma_internal(
        &mut self,
        address: u8,
        bytes: &[u8],
        first_slice: bool,
        last_slice: bool,
    ) -> Result<(), Error>
    where
        TXDMA: crate::i2c::TxDma<T>,
    {
        let total_len = bytes.len();
        let completed_chunks = total_len / 255;
        let total_chunks = if completed_chunks * 255 == total_len {
            completed_chunks
        } else {
            completed_chunks + 1
        };

        let dma_transfer = unsafe {
            let regs = T::regs();
            regs.cr1().modify(|w| {
                w.set_txdmaen(true);
                if first_slice {
                    w.set_tcie(true);
                }
            });
            let dst = regs.txdr().ptr() as *mut u8;

            let ch = &mut self.tx_dma;
            ch.write(ch.request(), bytes, dst)
        };

        let state_number = T::state_number();
        STATE.chunks_transferred[state_number].store(0, Ordering::Relaxed);
        let mut remaining_len = total_len;

        let _on_drop = OnDrop::new(|| {
            let regs = T::regs();
            unsafe {
                regs.cr1().modify(|w| {
                    if last_slice {
                        w.set_txdmaen(false);
                    }
                    w.set_tcie(false);
                })
            }
        });

        // NOTE(unsafe) self.tx_dma does not fiddle with the i2c registers
        if first_slice {
            unsafe {
                Self::master_write(
                    address,
                    total_len.min(255),
                    Stop::Software,
                    (total_chunks != 1) || !last_slice,
                );
            }
        } else {
            unsafe {
                Self::master_continue(total_len.min(255), (total_chunks != 1) || !last_slice);
                T::regs().cr1().modify(|w| w.set_tcie(true));
            }
        }

        poll_fn(|cx| {
            STATE.waker[state_number].register(cx.waker());
            let chunks_transferred = STATE.chunks_transferred[state_number].load(Ordering::Relaxed);

            if chunks_transferred == total_chunks {
                return Poll::Ready(());
            } else if chunks_transferred != 0 {
                remaining_len = remaining_len.saturating_sub(255);
                let last_piece = (chunks_transferred + 1 == total_chunks) && last_slice;

                // NOTE(unsafe) self.tx_dma does not fiddle with the i2c registers
                unsafe {
                    Self::master_continue(remaining_len.min(255), !last_piece);
                    T::regs().cr1().modify(|w| w.set_tcie(true));
                }
            }
            Poll::Pending
        })
        .await;

        dma_transfer.await;

        if last_slice {
            // This should be done already
            self.wait_tc()?;
            self.master_stop();
        }
        Ok(())
    }

    async fn read_dma_internal(
        &mut self,
        address: u8,
        buffer: &mut [u8],
        restart: bool,
    ) -> Result<(), Error>
    where
        RXDMA: crate::i2c::RxDma<T>,
    {
        let total_len = buffer.len();
        let completed_chunks = total_len / 255;
        let total_chunks = if completed_chunks * 255 == total_len {
            completed_chunks
        } else {
            completed_chunks + 1
        };

        let dma_transfer = unsafe {
            let regs = T::regs();
            regs.cr1().modify(|w| {
                w.set_rxdmaen(true);
                w.set_tcie(true);
            });
            let src = regs.rxdr().ptr() as *mut u8;

            let ch = &mut self.rx_dma;
            ch.read(ch.request(), src, buffer)
        };

        let state_number = T::state_number();
        STATE.chunks_transferred[state_number].store(0, Ordering::Relaxed);
        let mut remaining_len = total_len;

        let _on_drop = OnDrop::new(|| {
            let regs = T::regs();
            unsafe {
                regs.cr1().modify(|w| {
                    w.set_rxdmaen(false);
                    w.set_tcie(false);
                })
            }
        });

        // NOTE(unsafe) self.rx_dma does not fiddle with the i2c registers
        unsafe {
            Self::master_read(
                address,
                total_len.min(255),
                Stop::Software,
                total_chunks != 1,
                restart,
            );
        }

        poll_fn(|cx| {
            STATE.waker[state_number].register(cx.waker());
            let chunks_transferred = STATE.chunks_transferred[state_number].load(Ordering::Relaxed);

            if chunks_transferred == total_chunks {
                return Poll::Ready(());
            } else if chunks_transferred != 0 {
                remaining_len = remaining_len.saturating_sub(255);
                let last_piece = chunks_transferred + 1 == total_chunks;

                // NOTE(unsafe) self.rx_dma does not fiddle with the i2c registers
                unsafe {
                    Self::master_continue(remaining_len.min(255), !last_piece);
                    T::regs().cr1().modify(|w| w.set_tcie(true));
                }
            }
            Poll::Pending
        })
        .await;

        dma_transfer.await;

        // This should be done already
        self.wait_tc()?;
        self.master_stop();
        Ok(())
    }

    pub async fn write_dma(&mut self, address: u8, bytes: &[u8]) -> Result<(), Error>
    where
        TXDMA: crate::i2c::TxDma<T>,
    {
        self.write_dma_internal(address, bytes, true, true).await
    }

    pub async fn write_dma_vectored(&mut self, address: u8, bytes: &[&[u8]]) -> Result<(), Error>
    where
        TXDMA: crate::i2c::TxDma<T>,
    {
        if bytes.is_empty() {
            return Err(Error::ZeroLengthTransfer);
        }
        let mut iter = bytes.iter();

        let mut first = true;
        let mut current = iter.next();
        while let Some(c) = current {
            let next = iter.next();
            let is_last = next.is_none();

            self.write_dma_internal(address, c, first, is_last).await?;
            first = false;
            current = next;
        }
        Ok(())
    }

    pub async fn read_dma(
        &mut self,
        address: u8,
        buffer: &mut [u8],
        restart: bool,
    ) -> Result<(), Error>
    where
        RXDMA: crate::i2c::RxDma<T>,
    {
        self.read_dma_internal(address, buffer, restart).await
    }

    pub fn write_vectored(&mut self, address: u8, bytes: &[&[u8]]) -> Result<(), Error> {
        if bytes.is_empty() {
            return Err(Error::ZeroLengthTransfer);
        }
        let first_length = bytes[0].len();
        let last_slice_index = bytes.len() - 1;

        // NOTE(unsafe) We have &mut self
        unsafe {
            Self::master_write(
                address,
                first_length.min(255),
                Stop::Software,
                (first_length > 255) || (last_slice_index != 0),
            );
        }

        for (idx, slice) in bytes.iter().enumerate() {
            let slice_len = slice.len();
            let completed_chunks = slice_len / 255;
            let total_chunks = if completed_chunks * 255 == slice_len {
                completed_chunks
            } else {
                completed_chunks + 1
            };
            let last_chunk_idx = total_chunks.saturating_sub(1);

            if idx != 0 {
                // NOTE(unsafe) We have &mut self
                unsafe {
                    Self::master_continue(
                        slice_len.min(255),
                        (idx != last_slice_index) || (slice_len > 255),
                    );
                }
            }

            for (number, chunk) in slice.chunks(255).enumerate() {
                if number != 0 {
                    // NOTE(unsafe) We have &mut self
                    unsafe {
                        Self::master_continue(
                            chunk.len(),
                            (number != last_chunk_idx) || (idx != last_slice_index),
                        );
                    }
                }

                for byte in chunk {
                    // Wait until we are allowed to send data
                    // (START has been ACKed or last byte when
                    // through)
                    self.wait_txe()?;

                    // Put byte on the wire
                    //self.i2c.txdr.write(|w| w.txdata().bits(*byte));
                    unsafe {
                        T::regs().txdr().write(|w| w.set_txdata(*byte));
                    }
                }
            }
        }
        // Wait until the write finishes
        self.wait_tc()?;
        self.master_stop();

        Ok(())
    }
}

impl<'d, T: Instance> Read for I2c<'d, T> {
    type Error = Error;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.read(address, buffer, false)
        // Automatic Stop
    }
}

impl<'d, T: Instance> Write for I2c<'d, T> {
    type Error = Error;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write(address, bytes, true)
    }
}

impl<'d, T: Instance> WriteRead for I2c<'d, T> {
    type Error = Error;

    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.write(address, bytes, false)?;
        self.read(address, buffer, true)
        // Automatic Stop
    }
}

/// I2C Stop Configuration
///
/// Peripheral options for generating the STOP condition
#[derive(Copy, Clone, PartialEq)]
pub enum Stop {
    /// Software end mode: Must write register to generate STOP condition
    Software,
    /// Automatic end mode: A STOP condition is automatically generated once the
    /// configured number of bytes have been transferred
    Automatic,
}

impl Stop {
    fn autoend(&self) -> i2c::vals::Autoend {
        match self {
            Stop::Software => i2c::vals::Autoend::SOFTWARE,
            Stop::Automatic => i2c::vals::Autoend::AUTOMATIC,
        }
    }
}

struct Timings {
    prescale: u8,
    scll: u8,
    sclh: u8,
    sdadel: u8,
    scldel: u8,
}

impl Timings {
    fn new(i2cclk: Hertz, freq: Hertz) -> Self {
        let i2cclk = i2cclk.0;
        let freq = freq.0;
        // Refer to RM0433 Rev 7 Figure 539 for setup and hold timing:
        //
        // t_I2CCLK = 1 / PCLK1
        // t_PRESC  = (PRESC + 1) * t_I2CCLK
        // t_SCLL   = (SCLL + 1) * t_PRESC
        // t_SCLH   = (SCLH + 1) * t_PRESC
        //
        // t_SYNC1 + t_SYNC2 > 4 * t_I2CCLK
        // t_SCL ~= t_SYNC1 + t_SYNC2 + t_SCLL + t_SCLH
        let ratio = i2cclk / freq;

        // For the standard-mode configuration method, we must have a ratio of 4
        // or higher
        assert!(
            ratio >= 4,
            "The I2C PCLK must be at least 4 times the bus frequency!"
        );

        let (presc_reg, scll, sclh, sdadel, scldel) = if freq > 100_000 {
            // Fast-mode (Fm) or Fast-mode Plus (Fm+)
            // here we pick SCLL + 1 = 2 * (SCLH + 1)

            // Prescaler, 384 ticks for sclh/scll. Round up then subtract 1
            let presc_reg = ((ratio - 1) / 384) as u8;
            // ratio < 1200 by pclk 120MHz max., therefore presc < 16

            // Actual precale value selected
            let presc = (presc_reg + 1) as u32;

            let sclh = ((ratio / presc) - 3) / 3;
            let scll = (2 * (sclh + 1)) - 1;

            let (sdadel, scldel) = if freq > 400_000 {
                // Fast-mode Plus (Fm+)
                assert!(i2cclk >= 17_000_000); // See table in datsheet

                let sdadel = i2cclk / 8_000_000 / presc;
                let scldel = i2cclk / 4_000_000 / presc - 1;

                (sdadel, scldel)
            } else {
                // Fast-mode (Fm)
                assert!(i2cclk >= 8_000_000); // See table in datsheet

                let sdadel = i2cclk / 4_000_000 / presc;
                let scldel = i2cclk / 2_000_000 / presc - 1;

                (sdadel, scldel)
            };

            (
                presc_reg,
                scll as u8,
                sclh as u8,
                sdadel as u8,
                scldel as u8,
            )
        } else {
            // Standard-mode (Sm)
            // here we pick SCLL = SCLH
            assert!(i2cclk >= 2_000_000); // See table in datsheet

            // Prescaler, 512 ticks for sclh/scll. Round up then
            // subtract 1
            let presc = (ratio - 1) / 512;
            let presc_reg = cmp::min(presc, 15) as u8;

            // Actual prescale value selected
            let presc = (presc_reg + 1) as u32;

            let sclh = ((ratio / presc) - 2) / 2;
            let scll = sclh;

            // Speed check
            assert!(
                sclh < 256,
                "The I2C PCLK is too fast for this bus frequency!"
            );

            let sdadel = i2cclk / 2_000_000 / presc;
            let scldel = i2cclk / 500_000 / presc - 1;

            (
                presc_reg,
                scll as u8,
                sclh as u8,
                sdadel as u8,
                scldel as u8,
            )
        };

        // Sanity check
        assert!(presc_reg < 16);

        // Keep values within reasonable limits for fast per_ck
        let sdadel = cmp::max(sdadel, 2);
        let scldel = cmp::max(scldel, 4);

        //(presc_reg, scll, sclh, sdadel, scldel)
        Self {
            prescale: presc_reg,
            scll,
            sclh,
            sdadel,
            scldel,
        }
    }
}

impl<'d, T: Instance, TXDMA: super::TxDma<T>, RXDMA: super::RxDma<T>> I2cTrait<u8>
    for I2c<'d, T, TXDMA, RXDMA>
{
    type Error = super::Error;

    #[rustfmt::skip]
    type WriteFuture<'a> where 'd: 'a, T: 'a, TXDMA: 'a, RXDMA: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;
    #[rustfmt::skip]
    type ReadFuture<'a> where 'd: 'a, T: 'a, TXDMA: 'a, RXDMA: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;
    #[rustfmt::skip]
    type WriteReadFuture<'a> where 'd: 'a, T: 'a, TXDMA: 'a, RXDMA: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn read<'a>(&'a mut self, address: u8, buffer: &'a mut [u8]) -> Self::ReadFuture<'a> {
        self.read_dma(address, buffer, false)
    }

    fn write<'a>(&'a mut self, address: u8, bytes: &'a [u8]) -> Self::WriteFuture<'a> {
        self.write_dma(address, bytes)
    }

    fn write_read<'a>(
        &'a mut self,
        address: u8,
        bytes: &'a [u8],
        buffer: &'a mut [u8],
    ) -> Self::WriteReadFuture<'a> {
        async move {
            self.write_dma(address, bytes).await?;
            self.read_dma(address, buffer, true).await
        }
    }
}
