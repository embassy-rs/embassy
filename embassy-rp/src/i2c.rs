//! I2C driver.
use core::future;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use pac::i2c;

use crate::gpio::AnyPin;
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::{interrupt, pac, peripherals, Peripheral};

/// I2C error abort reason
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AbortReason {
    /// A bus operation was not acknowledged, e.g. due to the addressed device
    /// not being available on the bus or the device not being ready to process
    /// requests at the moment
    NoAcknowledge,
    /// The arbitration was lost, e.g. electrical problems with the clock signal
    ArbitrationLoss,
    /// Transmit ended with data still in fifo
    TxNotEmpty(u16),
    /// Other reason.
    Other(u32),
}

/// I2C error
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// I2C abort with error
    Abort(AbortReason),
    /// User passed in a read buffer that was 0 length
    InvalidReadBufferLength,
    /// User passed in a write buffer that was 0 length
    InvalidWriteBufferLength,
    /// Target i2c address is out of range
    AddressOutOfRange(u16),
    /// Target i2c address is reserved
    AddressReserved(u16),
}

/// I2C config.
#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct Config {
    /// Frequency.
    pub frequency: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self { frequency: 100_000 }
    }
}

/// Size of I2C FIFO.
pub const FIFO_SIZE: u8 = 16;

/// I2C driver.
pub struct I2c<'d, T: Instance, M: Mode> {
    phantom: PhantomData<(&'d mut T, M)>,
}

impl<'d, T: Instance> I2c<'d, T, Blocking> {
    /// Create a new driver instance in blocking mode.
    pub fn new_blocking(
        peri: impl Peripheral<P = T> + 'd,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(scl, sda);
        Self::new_inner(peri, scl.map_into(), sda.map_into(), config)
    }
}

impl<'d, T: Instance> I2c<'d, T, Async> {
    /// Create a new driver instance in async mode.
    pub fn new_async(
        peri: impl Peripheral<P = T> + 'd,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        config: Config,
    ) -> Self {
        into_ref!(scl, sda);

        let i2c = Self::new_inner(peri, scl.map_into(), sda.map_into(), config);

        let r = T::regs();

        // mask everything initially
        r.ic_intr_mask().write_value(i2c::regs::IcIntrMask(0));
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        i2c
    }

    /// Calls `f` to check if we are ready or not.
    /// If not, `g` is called once the waker is set (to eg enable the required interrupts).
    async fn wait_on<F, U, G>(&mut self, mut f: F, mut g: G) -> U
    where
        F: FnMut(&mut Self) -> Poll<U>,
        G: FnMut(&mut Self),
    {
        future::poll_fn(|cx| {
            let r = f(self);

            if r.is_pending() {
                T::waker().register(cx.waker());
                g(self);
            }
            r
        })
        .await
    }

    async fn read_async_internal(&mut self, buffer: &mut [u8], restart: bool, send_stop: bool) -> Result<(), Error> {
        if buffer.is_empty() {
            return Err(Error::InvalidReadBufferLength);
        }

        let p = T::regs();

        let mut remaining = buffer.len();
        let mut remaining_queue = buffer.len();

        let mut abort_reason = Ok(());

        while remaining > 0 {
            // Waggle SCK - basically the same as write
            let tx_fifo_space = Self::tx_fifo_capacity();
            let mut batch = 0;

            debug_assert!(remaining_queue > 0);

            for _ in 0..remaining_queue.min(tx_fifo_space as usize) {
                remaining_queue -= 1;
                let last = remaining_queue == 0;
                batch += 1;

                p.ic_data_cmd().write(|w| {
                    w.set_restart(restart && remaining_queue == buffer.len() - 1);
                    w.set_stop(last && send_stop);
                    w.set_cmd(true);
                });
            }

            // We've either run out of txfifo or just plain finished setting up
            // the clocks for the message - either way we need to wait for rx
            // data.

            debug_assert!(batch > 0);
            let res = self
                .wait_on(
                    |me| {
                        let rxfifo = Self::rx_fifo_len();
                        if let Err(abort_reason) = me.read_and_clear_abort_reason() {
                            Poll::Ready(Err(abort_reason))
                        } else if rxfifo >= batch {
                            Poll::Ready(Ok(rxfifo))
                        } else {
                            Poll::Pending
                        }
                    },
                    |_me| {
                        // Set the read threshold to the number of bytes we're
                        // expecting so we don't get spurious interrupts.
                        p.ic_rx_tl().write(|w| w.set_rx_tl(batch - 1));

                        p.ic_intr_mask().modify(|w| {
                            w.set_m_rx_full(true);
                            w.set_m_tx_abrt(true);
                        });
                    },
                )
                .await;

            match res {
                Err(reason) => {
                    abort_reason = Err(reason);
                    break;
                }
                Ok(rxfifo) => {
                    // Fetch things from rx fifo. We're assuming we're the only
                    // rxfifo reader, so nothing else can take things from it.
                    let rxbytes = (rxfifo as usize).min(remaining);
                    let received = buffer.len() - remaining;
                    for b in &mut buffer[received..received + rxbytes] {
                        *b = p.ic_data_cmd().read().dat();
                    }
                    remaining -= rxbytes;
                }
            };
        }

        self.wait_stop_det(abort_reason, send_stop).await
    }

    async fn write_async_internal(
        &mut self,
        bytes: impl IntoIterator<Item = u8>,
        send_stop: bool,
    ) -> Result<(), Error> {
        let p = T::regs();

        let mut bytes = bytes.into_iter().peekable();

        let res = 'xmit: loop {
            let tx_fifo_space = Self::tx_fifo_capacity();

            for _ in 0..tx_fifo_space {
                if let Some(byte) = bytes.next() {
                    let last = bytes.peek().is_none();

                    p.ic_data_cmd().write(|w| {
                        w.set_stop(last && send_stop);
                        w.set_cmd(false);
                        w.set_dat(byte);
                    });
                } else {
                    break 'xmit Ok(());
                }
            }

            let res = self
                .wait_on(
                    |me| {
                        if let abort_reason @ Err(_) = me.read_and_clear_abort_reason() {
                            Poll::Ready(abort_reason)
                        } else if !Self::tx_fifo_full() {
                            // resume if there's any space free in the tx fifo
                            Poll::Ready(Ok(()))
                        } else {
                            Poll::Pending
                        }
                    },
                    |_me| {
                        // Set tx "free" threshold a little high so that we get
                        // woken before the fifo completely drains to minimize
                        // transfer stalls.
                        p.ic_tx_tl().write(|w| w.set_tx_tl(1));

                        p.ic_intr_mask().modify(|w| {
                            w.set_m_tx_empty(true);
                            w.set_m_tx_abrt(true);
                        })
                    },
                )
                .await;
            if res.is_err() {
                break res;
            }
        };

        self.wait_stop_det(res, send_stop).await
    }

    /// Helper to wait for a stop bit, for both tx and rx. If we had an abort,
    /// then we'll get a hardware-generated stop, otherwise wait for a stop if
    /// we're expecting it.
    ///
    /// Also handles an abort which arises while processing the tx fifo.
    async fn wait_stop_det(&mut self, had_abort: Result<(), Error>, do_stop: bool) -> Result<(), Error> {
        if had_abort.is_err() || do_stop {
            let p = T::regs();

            let had_abort2 = self
                .wait_on(
                    |me| {
                        // We could see an abort while processing fifo backlog,
                        // so handle it here.
                        let abort = me.read_and_clear_abort_reason();
                        if had_abort.is_ok() && abort.is_err() {
                            Poll::Ready(abort)
                        } else if p.ic_raw_intr_stat().read().stop_det() {
                            Poll::Ready(Ok(()))
                        } else {
                            Poll::Pending
                        }
                    },
                    |_me| {
                        p.ic_intr_mask().modify(|w| {
                            w.set_m_stop_det(true);
                            w.set_m_tx_abrt(true);
                        });
                    },
                )
                .await;
            p.ic_clr_stop_det().read();

            had_abort.and(had_abort2)
        } else {
            had_abort
        }
    }

    /// Read from address into buffer using DMA.
    pub async fn read_async(&mut self, addr: u16, buffer: &mut [u8]) -> Result<(), Error> {
        Self::setup(addr)?;
        self.read_async_internal(buffer, true, true).await
    }

    /// Write to address from buffer using DMA.
    pub async fn write_async(&mut self, addr: u16, bytes: impl IntoIterator<Item = u8>) -> Result<(), Error> {
        Self::setup(addr)?;
        self.write_async_internal(bytes, true).await
    }

    /// Write to address from bytes and read from address into buffer using DMA.
    pub async fn write_read_async(
        &mut self,
        addr: u16,
        bytes: impl IntoIterator<Item = u8>,
        buffer: &mut [u8],
    ) -> Result<(), Error> {
        Self::setup(addr)?;
        self.write_async_internal(bytes, false).await?;
        self.read_async_internal(buffer, true, true).await
    }
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _uart: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    // Mask interrupts and wake any task waiting for this interrupt
    unsafe fn on_interrupt() {
        let i2c = T::regs();
        i2c.ic_intr_mask().write_value(pac::i2c::regs::IcIntrMask::default());

        T::waker().wake();
    }
}

pub(crate) fn set_up_i2c_pin<'d, P, T>(pin: &P)
where
    P: core::ops::Deref<Target = T>,
    T: crate::gpio::Pin,
{
    pin.gpio().ctrl().write(|w| w.set_funcsel(3));
    pin.pad_ctrl().write(|w| {
        w.set_schmitt(true);
        w.set_slewfast(false);
        w.set_ie(true);
        w.set_od(false);
        w.set_pue(true);
        w.set_pde(false);
    });
}

impl<'d, T: Instance + 'd, M: Mode> I2c<'d, T, M> {
    fn new_inner(
        _peri: impl Peripheral<P = T> + 'd,
        scl: PeripheralRef<'d, AnyPin>,
        sda: PeripheralRef<'d, AnyPin>,
        config: Config,
    ) -> Self {
        into_ref!(_peri);

        assert!(config.frequency <= 1_000_000);
        assert!(config.frequency > 0);

        let p = T::regs();

        let reset = T::reset();
        crate::reset::reset(reset);
        crate::reset::unreset_wait(reset);

        p.ic_enable().write(|w| w.set_enable(false));

        // Select controller mode & speed
        p.ic_con().modify(|w| {
            // Always use "fast" mode (<= 400 kHz, works fine for standard
            // mode too)
            w.set_speed(i2c::vals::Speed::FAST);
            w.set_master_mode(true);
            w.set_ic_slave_disable(true);
            w.set_ic_restart_en(true);
            w.set_tx_empty_ctrl(true);
        });

        // Set FIFO watermarks to 1 to make things simpler. This is encoded
        // by a register value of 0.
        p.ic_tx_tl().write(|w| w.set_tx_tl(0));
        p.ic_rx_tl().write(|w| w.set_rx_tl(0));

        // Configure SCL & SDA pins
        set_up_i2c_pin(&scl);
        set_up_i2c_pin(&sda);

        // Configure baudrate

        // There are some subtleties to I2C timing which we are completely
        // ignoring here See:
        // https://github.com/raspberrypi/pico-sdk/blob/bfcbefafc5d2a210551a4d9d80b4303d4ae0adf7/src/rp2_common/hardware_i2c/i2c.c#L69
        let clk_base = crate::clocks::clk_peri_freq();

        let period = (clk_base + config.frequency / 2) / config.frequency;
        let lcnt = period * 3 / 5; // spend 3/5 (60%) of the period low
        let hcnt = period - lcnt; // and 2/5 (40%) of the period high

        // Check for out-of-range divisors:
        assert!(hcnt <= 0xffff);
        assert!(lcnt <= 0xffff);
        assert!(hcnt >= 8);
        assert!(lcnt >= 8);

        // Per I2C-bus specification a device in standard or fast mode must
        // internally provide a hold time of at least 300ns for the SDA
        // signal to bridge the undefined region of the falling edge of SCL.
        // A smaller hold time of 120ns is used for fast mode plus.
        let sda_tx_hold_count = if config.frequency < 1_000_000 {
            // sda_tx_hold_count = clk_base [cycles/s] * 300ns * (1s /
            // 1e9ns) Reduce 300/1e9 to 3/1e7 to avoid numbers that don't
            // fit in uint. Add 1 to avoid division truncation.
            ((clk_base * 3) / 10_000_000) + 1
        } else {
            // fast mode plus requires a clk_base > 32MHz
            assert!(clk_base >= 32_000_000);

            // sda_tx_hold_count = clk_base [cycles/s] * 120ns * (1s /
            // 1e9ns) Reduce 120/1e9 to 3/25e6 to avoid numbers that don't
            // fit in uint. Add 1 to avoid division truncation.
            ((clk_base * 3) / 25_000_000) + 1
        };
        assert!(sda_tx_hold_count <= lcnt - 2);

        p.ic_fs_scl_hcnt().write(|w| w.set_ic_fs_scl_hcnt(hcnt as u16));
        p.ic_fs_scl_lcnt().write(|w| w.set_ic_fs_scl_lcnt(lcnt as u16));
        p.ic_fs_spklen()
            .write(|w| w.set_ic_fs_spklen(if lcnt < 16 { 1 } else { (lcnt / 16) as u8 }));
        p.ic_sda_hold()
            .modify(|w| w.set_ic_sda_tx_hold(sda_tx_hold_count as u16));

        // Enable I2C block
        p.ic_enable().write(|w| w.set_enable(true));

        Self { phantom: PhantomData }
    }

    fn setup(addr: u16) -> Result<(), Error> {
        if addr >= 0x80 {
            return Err(Error::AddressOutOfRange(addr));
        }

        if i2c_reserved_addr(addr) {
            return Err(Error::AddressReserved(addr));
        }

        let p = T::regs();
        p.ic_enable().write(|w| w.set_enable(false));
        p.ic_tar().write(|w| w.set_ic_tar(addr));
        p.ic_enable().write(|w| w.set_enable(true));
        Ok(())
    }

    #[inline]
    fn tx_fifo_full() -> bool {
        Self::tx_fifo_capacity() == 0
    }

    #[inline]
    fn tx_fifo_capacity() -> u8 {
        let p = T::regs();
        FIFO_SIZE - p.ic_txflr().read().txflr()
    }

    #[inline]
    fn rx_fifo_len() -> u8 {
        let p = T::regs();
        p.ic_rxflr().read().rxflr()
    }

    fn read_and_clear_abort_reason(&mut self) -> Result<(), Error> {
        let p = T::regs();
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
            } else {
                AbortReason::Other(abort_reason.0)
            };

            Err(Error::Abort(reason))
        } else {
            Ok(())
        }
    }

    fn read_blocking_internal(&mut self, read: &mut [u8], restart: bool, send_stop: bool) -> Result<(), Error> {
        if read.is_empty() {
            return Err(Error::InvalidReadBufferLength);
        }

        let p = T::regs();
        let lastindex = read.len() - 1;
        for (i, byte) in read.iter_mut().enumerate() {
            let first = i == 0;
            let last = i == lastindex;

            // wait until there is space in the FIFO to write the next byte
            while Self::tx_fifo_full() {}

            p.ic_data_cmd().write(|w| {
                w.set_restart(restart && first);
                w.set_stop(send_stop && last);

                w.set_cmd(true);
            });

            while Self::rx_fifo_len() == 0 {
                self.read_and_clear_abort_reason()?;
            }

            *byte = p.ic_data_cmd().read().dat();
        }

        Ok(())
    }

    fn write_blocking_internal(&mut self, write: &[u8], send_stop: bool) -> Result<(), Error> {
        if write.is_empty() {
            return Err(Error::InvalidWriteBufferLength);
        }

        let p = T::regs();

        for (i, byte) in write.iter().enumerate() {
            let last = i == write.len() - 1;

            p.ic_data_cmd().write(|w| {
                w.set_stop(send_stop && last);
                w.set_dat(*byte);
            });

            // Wait until the transmission of the address/data from the
            // internal shift register has completed. For this to function
            // correctly, the TX_EMPTY_CTRL flag in IC_CON must be set. The
            // TX_EMPTY_CTRL flag was set in i2c_init.
            while !p.ic_raw_intr_stat().read().tx_empty() {}

            let abort_reason = self.read_and_clear_abort_reason();

            if abort_reason.is_err() || (send_stop && last) {
                // If the transaction was aborted or if it completed
                // successfully wait until the STOP condition has occurred.

                while !p.ic_raw_intr_stat().read().stop_det() {}

                p.ic_clr_stop_det().read().clr_stop_det();
            }

            // Note the hardware issues a STOP automatically on an abort
            // condition. Note also the hardware clears RX FIFO as well as
            // TX on abort, ecause we set hwparam
            // IC_AVOID_RX_FIFO_FLUSH_ON_TX_ABRT to 0.
            abort_reason?;
        }
        Ok(())
    }

    // =========================
    // Blocking public API
    // =========================

    /// Read from address into buffer blocking caller until done.
    pub fn blocking_read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Error> {
        Self::setup(address.into())?;
        self.read_blocking_internal(read, true, true)
        // Automatic Stop
    }

    /// Write to address from buffer blocking caller until done.
    pub fn blocking_write(&mut self, address: u8, write: &[u8]) -> Result<(), Error> {
        Self::setup(address.into())?;
        self.write_blocking_internal(write, true)
    }

    /// Write to address from bytes and read from address into buffer blocking caller until done.
    pub fn blocking_write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        Self::setup(address.into())?;
        self.write_blocking_internal(write, false)?;
        self.read_blocking_internal(read, true, true)
        // Automatic Stop
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::i2c::Read for I2c<'d, T, M> {
    type Error = Error;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(address, buffer)
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::i2c::Write for I2c<'d, T, M> {
    type Error = Error;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(address, bytes)
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::i2c::WriteRead for I2c<'d, T, M> {
    type Error = Error;

    fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_write_read(address, bytes, buffer)
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::i2c::Transactional for I2c<'d, T, M> {
    type Error = Error;

    fn exec(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_02::blocking::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        Self::setup(address.into())?;
        for i in 0..operations.len() {
            let last = i == operations.len() - 1;
            match &mut operations[i] {
                embedded_hal_02::blocking::i2c::Operation::Read(buf) => {
                    self.read_blocking_internal(buf, false, last)?
                }
                embedded_hal_02::blocking::i2c::Operation::Write(buf) => self.write_blocking_internal(buf, last)?,
            }
        }
        Ok(())
    }
}

impl embedded_hal_1::i2c::Error for Error {
    fn kind(&self) -> embedded_hal_1::i2c::ErrorKind {
        match *self {
            Self::Abort(AbortReason::ArbitrationLoss) => embedded_hal_1::i2c::ErrorKind::ArbitrationLoss,
            Self::Abort(AbortReason::NoAcknowledge) => {
                embedded_hal_1::i2c::ErrorKind::NoAcknowledge(embedded_hal_1::i2c::NoAcknowledgeSource::Address)
            }
            Self::Abort(AbortReason::TxNotEmpty(_)) => embedded_hal_1::i2c::ErrorKind::Other,
            Self::Abort(AbortReason::Other(_)) => embedded_hal_1::i2c::ErrorKind::Other,
            Self::InvalidReadBufferLength => embedded_hal_1::i2c::ErrorKind::Other,
            Self::InvalidWriteBufferLength => embedded_hal_1::i2c::ErrorKind::Other,
            Self::AddressOutOfRange(_) => embedded_hal_1::i2c::ErrorKind::Other,
            Self::AddressReserved(_) => embedded_hal_1::i2c::ErrorKind::Other,
        }
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_1::i2c::ErrorType for I2c<'d, T, M> {
    type Error = Error;
}

impl<'d, T: Instance, M: Mode> embedded_hal_1::i2c::I2c for I2c<'d, T, M> {
    fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(address, read)
    }

    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(address, write)
    }

    fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_write_read(address, write, read)
    }

    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        Self::setup(address.into())?;
        for i in 0..operations.len() {
            let last = i == operations.len() - 1;
            match &mut operations[i] {
                embedded_hal_1::i2c::Operation::Read(buf) => self.read_blocking_internal(buf, false, last)?,
                embedded_hal_1::i2c::Operation::Write(buf) => self.write_blocking_internal(buf, last)?,
            }
        }
        Ok(())
    }
}

impl<'d, A, T> embedded_hal_async::i2c::I2c<A> for I2c<'d, T, Async>
where
    A: embedded_hal_async::i2c::AddressMode + Into<u16> + 'static,
    T: Instance + 'd,
{
    async fn read(&mut self, address: A, read: &mut [u8]) -> Result<(), Self::Error> {
        let addr: u16 = address.into();

        Self::setup(addr)?;
        self.read_async_internal(read, false, true).await
    }

    async fn write(&mut self, address: A, write: &[u8]) -> Result<(), Self::Error> {
        let addr: u16 = address.into();

        Self::setup(addr)?;
        self.write_async_internal(write.iter().copied(), true).await
    }

    async fn write_read(&mut self, address: A, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
        let addr: u16 = address.into();

        Self::setup(addr)?;
        self.write_async_internal(write.iter().cloned(), false).await?;
        self.read_async_internal(read, true, true).await
    }

    async fn transaction(
        &mut self,
        address: A,
        operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        use embedded_hal_1::i2c::Operation;

        let addr: u16 = address.into();

        if operations.len() > 0 {
            Self::setup(addr)?;
        }
        let mut iterator = operations.iter_mut();

        while let Some(op) = iterator.next() {
            let last = iterator.len() == 0;

            match op {
                Operation::Read(buffer) => {
                    self.read_async_internal(buffer, false, last).await?;
                }
                Operation::Write(buffer) => {
                    self.write_async_internal(buffer.into_iter().cloned(), last).await?;
                }
            }
        }
        Ok(())
    }
}

/// Check if address is reserved.
pub fn i2c_reserved_addr(addr: u16) -> bool {
    ((addr & 0x78) == 0 || (addr & 0x78) == 0x78) && addr != 0
}

mod sealed {
    use embassy_sync::waitqueue::AtomicWaker;

    use crate::interrupt;

    pub trait Instance {
        const TX_DREQ: u8;
        const RX_DREQ: u8;

        type Interrupt: interrupt::typelevel::Interrupt;

        fn regs() -> crate::pac::i2c::I2c;
        fn reset() -> crate::pac::resets::regs::Peripherals;
        fn waker() -> &'static AtomicWaker;
    }

    pub trait Mode {}

    pub trait SdaPin<T: Instance> {}
    pub trait SclPin<T: Instance> {}
}

/// Driver mode.
pub trait Mode: sealed::Mode {}

macro_rules! impl_mode {
    ($name:ident) => {
        impl sealed::Mode for $name {}
        impl Mode for $name {}
    };
}

/// Blocking mode.
pub struct Blocking;
/// Async mode.
pub struct Async;

impl_mode!(Blocking);
impl_mode!(Async);

/// I2C instance.
pub trait Instance: sealed::Instance {}

macro_rules! impl_instance {
    ($type:ident, $irq:ident, $reset:ident, $tx_dreq:expr, $rx_dreq:expr) => {
        impl sealed::Instance for peripherals::$type {
            const TX_DREQ: u8 = $tx_dreq;
            const RX_DREQ: u8 = $rx_dreq;

            type Interrupt = crate::interrupt::typelevel::$irq;

            #[inline]
            fn regs() -> pac::i2c::I2c {
                pac::$type
            }

            #[inline]
            fn reset() -> pac::resets::regs::Peripherals {
                let mut ret = pac::resets::regs::Peripherals::default();
                ret.$reset(true);
                ret
            }

            #[inline]
            fn waker() -> &'static AtomicWaker {
                static WAKER: AtomicWaker = AtomicWaker::new();

                &WAKER
            }
        }
        impl Instance for peripherals::$type {}
    };
}

impl_instance!(I2C0, I2C0_IRQ, set_i2c0, 32, 33);
impl_instance!(I2C1, I2C1_IRQ, set_i2c1, 34, 35);

/// SDA pin.
pub trait SdaPin<T: Instance>: sealed::SdaPin<T> + crate::gpio::Pin {}
/// SCL pin.
pub trait SclPin<T: Instance>: sealed::SclPin<T> + crate::gpio::Pin {}

macro_rules! impl_pin {
    ($pin:ident, $instance:ident, $function:ident) => {
        impl sealed::$function<peripherals::$instance> for peripherals::$pin {}
        impl $function<peripherals::$instance> for peripherals::$pin {}
    };
}

impl_pin!(PIN_0, I2C0, SdaPin);
impl_pin!(PIN_1, I2C0, SclPin);
impl_pin!(PIN_2, I2C1, SdaPin);
impl_pin!(PIN_3, I2C1, SclPin);
impl_pin!(PIN_4, I2C0, SdaPin);
impl_pin!(PIN_5, I2C0, SclPin);
impl_pin!(PIN_6, I2C1, SdaPin);
impl_pin!(PIN_7, I2C1, SclPin);
impl_pin!(PIN_8, I2C0, SdaPin);
impl_pin!(PIN_9, I2C0, SclPin);
impl_pin!(PIN_10, I2C1, SdaPin);
impl_pin!(PIN_11, I2C1, SclPin);
impl_pin!(PIN_12, I2C0, SdaPin);
impl_pin!(PIN_13, I2C0, SclPin);
impl_pin!(PIN_14, I2C1, SdaPin);
impl_pin!(PIN_15, I2C1, SclPin);
impl_pin!(PIN_16, I2C0, SdaPin);
impl_pin!(PIN_17, I2C0, SclPin);
impl_pin!(PIN_18, I2C1, SdaPin);
impl_pin!(PIN_19, I2C1, SclPin);
impl_pin!(PIN_20, I2C0, SdaPin);
impl_pin!(PIN_21, I2C0, SclPin);
impl_pin!(PIN_22, I2C1, SdaPin);
impl_pin!(PIN_23, I2C1, SclPin);
impl_pin!(PIN_24, I2C0, SdaPin);
impl_pin!(PIN_25, I2C0, SclPin);
impl_pin!(PIN_26, I2C1, SdaPin);
impl_pin!(PIN_27, I2C1, SclPin);
impl_pin!(PIN_28, I2C0, SdaPin);
impl_pin!(PIN_29, I2C0, SclPin);
