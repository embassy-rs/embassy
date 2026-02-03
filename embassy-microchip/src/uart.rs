//! UART driver.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU8, Ordering};
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use mec17xx_pac::uart0::regs::DataLsr;

use crate::gpio::{AnyPin, Pin, SealedPin};
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::pac::uart0::Uart0;
use crate::{interrupt, pac, peripherals};

// The datasheet does not appear to specify this, but the 16550 has a 16-byte FIFO
// and empirical testing confirms this is also the case for the MEC UART.
//
// This is useful to know since UART does not support DMA, allowing us to interrupt less frequently.
const FIFO_SZ: usize = 16;

// The HW gives us a convenient scratch register which we use to hold interrupt flags
mod int_flag {
    pub(crate) const RX_AVAILABLE: u8 = 1 << 0;
    pub(crate) const TIMEOUT: u8 = 1 << 1;
    pub(crate) const TX_EMPTY: u8 = 1 << 2;
}

/// UART interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let intid = InterruptType::try_from(T::info().reg.data().int_id().read().intid());
        match intid {
            // In case of RxAvailable, We need a way to know that the FIFO level is triggered in task,
            // but also need to disable the interrupt to clear it (which clears it in intid).
            //
            // In case of LineStatus, We don't use a separate flag from RxAvailable because
            // the RX task will keep reading bytes until it hits the byte that produced the error
            Ok(InterruptType::LineStatus) | Ok(InterruptType::RxAvailable) => {
                T::info().reg.data().scr().modify(|w| *w |= int_flag::RX_AVAILABLE);
                T::info().reg.data().ien().modify(|w| {
                    w.set_elsi(false);
                    w.set_erdai(false);
                });
                T::info().rx_waker.wake();
            }

            // There does not appear to be a way of disabling this interrupt, and it is always
            // enabled if RX available interrupt is enabled. The datasheet shows this as equal
            // priority to RxAvailable, but it seems to have higher priority because even if the
            // FIFO trigger level is reached, we will always see this interrupt ID until a byte
            // is read from FIFO.
            //
            // This is annoying because we want to be able to batch read to the set RX FIFO
            // level trigger, but if our input source is slow (such as person typing on keyboard),
            // this will almost always trigger after the first byte is received (since we want
            // to wait until the FIFO trigger is reached until reading bytes from FIFO).
            //
            // Alas we have to deal with this, which the RX task will do by falling back into
            // interrupting for every byte.
            Ok(InterruptType::CharacterTimeout) => {
                T::info().reg.data().scr().modify(|w| *w |= int_flag::TIMEOUT);
                T::info().reg.data().ien().modify(|w| {
                    w.set_elsi(false);
                    w.set_erdai(false);
                });
                T::info().rx_waker.wake();
            }

            // Note: We mark TX empty flag because although we could check LSR for tx empty
            // in the TX task, doing so will clear error bits in LSR.
            //
            // This is important because there is a potential race where even though Line Status
            // interrupt is higher priority, we disable it above so the interrupt could trigger
            // again, waking the TX task, which might go before the RX task, and it would end up
            // accidentally clearing LSR error bits before RX task can see them.
            Ok(InterruptType::TxEmpty) => {
                T::info().reg.data().scr().modify(|w| *w |= int_flag::TX_EMPTY);
                T::info().reg.data().ien().modify(|w| w.set_ethrei(false));
                T::info().tx_waker.wake();
            }

            // Unknown/unsupported interrupt type, so ignore
            _ => (),
        }
    }
}

enum InterruptType {
    LineStatus,
    RxAvailable,
    CharacterTimeout,
    TxEmpty,
    // MODEM is not currently supported
    _Modem,
}

impl TryFrom<u8> for InterruptType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        // Note: This always assumes we are in FIFO mode (the driver does not support non-FIFO mode)
        Ok(match value {
            0b011 => Self::LineStatus,
            0b010 => Self::RxAvailable,
            0b110 => Self::CharacterTimeout,
            0b001 => Self::TxEmpty,
            0b000 => Self::_Modem,
            _ => Err(())?,
        })
    }
}

enum RxFifoTrigger {
    _1,
    _4,
    _8,
    _14,
}

impl From<RxFifoTrigger> for u8 {
    // Note: These are the bit representations in the register
    fn from(trigger: RxFifoTrigger) -> Self {
        match trigger {
            RxFifoTrigger::_1 => 0b00,
            RxFifoTrigger::_4 => 0b01,
            RxFifoTrigger::_8 => 0b10,
            RxFifoTrigger::_14 => 0b11,
        }
    }
}

/// Data word length.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WordLen {
    /// 5 bits.
    _5,
    /// 6 bits.
    _6,
    /// 7 bits.
    _7,
    /// 8 bits.
    _8,
}

impl WordLen {
    fn as_bits(&self) -> u8 {
        match self {
            Self::_5 => 0b00,
            Self::_6 => 0b01,
            Self::_7 => 0b10,
            Self::_8 => 0b11,
        }
    }
}

/// Parity selection.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Parity {
    /// No parity bit.
    None,
    /// Odd parity.
    Odd,
    /// Even parity.
    Even,
    /// Mark parity.
    Mark,
    /// Space parity.
    Space,
}

impl Parity {
    fn as_par_sel(&self) -> bool {
        matches!(*self, Self::Even | Self::Space)
    }

    fn as_stick_par(&self) -> bool {
        matches!(*self, Self::Mark | Self::Space)
    }
}

/// Baud clock source.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClkSrc {
    /// External clock (in Hz).
    External(u32),
    /// Internal 1.8432 MHz clock.
    _1_8432MHz,
    /// Internal 48 MHz clock.
    _48MHz,
}

impl ClkSrc {
    fn as_bits(&self) -> bool {
        matches!(self, ClkSrc::External(_))
    }

    fn as_baud_clk_sel_bits(&self) -> u8 {
        // If using external clock source this is just a "don't care" bit
        (*self == Self::_48MHz) as u8
    }
}

impl From<ClkSrc> for u32 {
    fn from(clk_src: ClkSrc) -> Self {
        match clk_src {
            ClkSrc::External(f) => f,
            ClkSrc::_1_8432MHz => 1_843_200,
            ClkSrc::_48MHz => 48_000_000,
        }
    }
}

/// UART configuration.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// The clock source which the baud clock is derived from.
    ///
    /// This will affect which baud rates can be accurately represented.
    pub clk_src: ClkSrc,
    /// Baudrate in bits per second (BPS).
    pub baudrate: u32,
    /// Data word length.
    ///
    /// Note: If multi stop bits is enabled, the number of stop bits corresponds to chosen word length.
    pub word_len: WordLen,
    /// Enable multiple stop bits.
    ///
    /// If enabled, the number of stop bits will be dependent on chosen word length.
    ///
    /// If disabled, the number of stop bits will always be 1.
    pub use_multi_stop: bool,
    /// Parity selection.
    pub parity: Parity,
    /// Enable inverted polarity on RX and TX pins.
    pub invert_polarity: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clk_src: ClkSrc::_1_8432MHz,
            baudrate: 115_200,
            word_len: WordLen::_8,
            use_multi_stop: false,
            parity: Parity::None,
            invert_polarity: false,
        }
    }
}

/// UART error.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// A baud rate was supplied which can not be represenetd based on chosen clock source.
    InvalidBaud,
    /// RX FIFO overrun occurred.
    Overrun,
    /// RX parity error occurred.
    Parity,
    /// RX frame error occurred.
    Frame,
    /// RX break interrupt occurred.
    Break,
}

/// UART RX error.
///
/// Contains the [`Error`] along with number of valid bytes read before error occurred.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RxError {
    /// Bytes read before error occurred.
    pub bytes_read: usize,
    /// The actual error encountered.
    pub err: Error,
}

impl core::fmt::Display for RxError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.err {
            Error::InvalidBaud => write!(
                f,
                "A baud rate was supplied which can not be represenetd based on chosen clock source."
            ),
            Error::Overrun => write!(f, "RX FIFO overrun occurred."),
            Error::Parity => write!(f, "RX parity error occurred."),
            Error::Frame => write!(f, "RX frame error occurred."),
            Error::Break => write!(f, "RX break interrupt occurred."),
        }
    }
}

impl core::error::Error for RxError {}

fn init<'d, T: Instance>(
    _rx_pin: Option<&Peri<'d, AnyPin>>,
    _tx_pin: Option<&Peri<'d, AnyPin>>,
    config: Config,
) -> Result<(), Error> {
    // Ensure baudrate is nonzero
    let divisor = u32::from(config.clk_src)
        .checked_div(16 * config.baudrate)
        .ok_or(Error::InvalidBaud)? as u16;

    // Divisor can be 15 bits max (the 16th bit is used to select clock source)
    if divisor > 0x7FFF {
        return Err(Error::InvalidBaud);
    };

    // Configure pins
    critical_section::with(|_| {
        if let Some(rx) = _rx_pin {
            rx.regs().ctrl1.modify(|w| {
                w.set_mux_ctrl(pac::Function::F1);
                w.set_dir(pac::Dir::INPUT);
                w.set_inp_dis(false);
                w.set_pu_pd(pac::Pull::NONE);
            })
        }
        if let Some(tx) = _tx_pin {
            tx.regs().ctrl1.modify(|w| {
                w.set_mux_ctrl(pac::Function::F1);
                w.set_dir(pac::Dir::OUTPUT);
                w.set_inp_dis(true);
                w.set_pu_pd(pac::Pull::NONE);
            })
        }
    });

    // Set config
    T::info().reg.data().cfg_sel().write(|w| {
        w.set_clk_src(config.clk_src.as_bits());
        w.set_polar(config.invert_polarity);
    });

    // Set line control with latched DLAB since we modify baud registers next
    T::info().reg.data().lcr().write(|w| {
        w.set_word_len(config.word_len.as_bits());
        w.set_stop_bits(config.use_multi_stop);
        w.set_en_par(config.parity != Parity::None);
        if config.parity != Parity::None {
            w.set_par_sel(config.parity.as_par_sel());
            w.set_stick_par(config.parity.as_stick_par());
        }
        w.set_dlab(true);
    });

    // Set baud rate divisor MSB
    // Note: bit 7 determines the baud clock source,
    // but the PAC doesn't appear to have a type with fields for this reg
    T::info()
        .reg
        .dlab()
        .baudrt_msb()
        .write(|w| *w = (config.clk_src.as_baud_clk_sel_bits() << 7) | (divisor >> 8) as u8);
    // Set baud rate divisor LSB
    T::info().reg.dlab().baudrt_lsb().write(|w| *w = divisor as u8);

    // Unlatch DLAB, remaining unlatched for rest of instance's lifetime
    T::info().reg.dlab().lcr().modify(|w| w.set_dlab(false));

    // Enable and clear FIFO
    T::info().reg.data().fifo_cr().write(|w| {
        w.set_exrf(true);
        w.set_clr_recv_fifo(true);
        w.set_clr_xmit_fifo(true);
    });

    // Ensure scratch is cleared which we use for interrupt flags
    T::info().reg.data().scr().write(|w| *w = 0);

    // Enable UART
    T::info().reg.data().activate().write(|w| *w = 1);

    Ok(())
}

fn interrupt_en<T: Instance>() {
    T::info().reg.data().mcr().modify(|w| w.set_out2(true));

    // Unmask interrupt
    pac::ECIA.en_set15().write_value(1 << T::irq_bit());
    T::Interrupt::unpend();
    // SAFETY: We have sole control of UART interrupts so this is safe to do
    unsafe { T::Interrupt::enable() };
}

fn drop_rx_tx(info: &'static Info) {
    // Only disable UART once both UartRx and UartTx have been dropped
    if info.rx_tx_refcount.fetch_sub(1, Ordering::AcqRel) == 1 {
        info.reg.data().mcr().modify(|w| w.set_out2(false));
        info.reg.data().activate().write(|w| *w = 0);
    }
}

/// UART driver.
pub struct Uart<'d, M: Mode> {
    rx: UartRx<'d, M>,
    tx: UartTx<'d, M>,
}

impl<'d, M: Mode> Uart<'d, M> {
    fn new_inner<T: Instance>(
        _rx_pin: Peri<'d, impl RxPin<T>>,
        _tx_pin: Peri<'d, impl TxPin<T>>,
        config: Config,
    ) -> Result<Self, Error> {
        let rx_pin = _rx_pin.into();
        let tx_pin = _tx_pin.into();
        init::<T>(Some(&rx_pin), Some(&tx_pin), config)?;
        let rx = UartRx::new_inner::<T>(rx_pin);
        let tx = UartTx::new_inner::<T>(tx_pin);
        Ok(Self { rx, tx })
    }

    /// Reads bytes from RX FIFO until buffer is full, blocking if empty.
    ///
    /// # Errors
    ///
    /// Returns [`RxError`] if error occurred during read.
    pub fn blocking_read(&mut self, buf: &mut [u8]) -> Result<(), RxError> {
        self.rx.blocking_read(buf)
    }

    /// Writes bytes to TX FIFO, blocking if full.
    pub fn blocking_write(&mut self, bytes: &[u8]) {
        self.tx.blocking_write(bytes)
    }

    /// Blocks until both the transmit holding and shift registers are empty.
    pub fn blocking_flush(&mut self) {
        self.tx.blocking_flush()
    }

    /// Splits the UART driver into separate [`UartRx`] and [`UartTx`] drivers.
    ///
    /// Helpful for sharing the UART among receiver/transmitter tasks.
    pub fn split(self) -> (UartRx<'d, M>, UartTx<'d, M>) {
        (self.rx, self.tx)
    }

    /// Splits the UART driver into separate [`UartRx`] and [`UartTx`] drivers by mutable reference.
    ///
    /// Helpful for sharing the UART among receiver/transmitter tasks without destroying the original [`Uart`] instance.
    pub fn split_ref(&mut self) -> (&mut UartRx<'d, M>, &mut UartTx<'d, M>) {
        (&mut self.rx, &mut self.tx)
    }
}

impl<'d> Uart<'d, Blocking> {
    /// Create a new blocking UART driver instance with given configuration.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidBaud`] if the supplied baud rate can not be represented by
    /// given clock source.
    pub fn new_blocking<T: Instance>(
        _peri: Peri<'d, T>,
        _rx_pin: Peri<'d, impl RxPin<T>>,
        _tx_pin: Peri<'d, impl TxPin<T>>,
        config: Config,
    ) -> Result<Self, Error> {
        Self::new_inner(_rx_pin, _tx_pin, config)
    }
}

impl<'d> Uart<'d, Async> {
    /// Create a new async UART driver instance with given configuration.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidBaud`] if the supplied baud rate can not be represented by
    /// given clock source.
    pub fn new_async<T: Instance>(
        _peri: Peri<'d, T>,
        _rx_pin: Peri<'d, impl RxPin<T>>,
        _tx_pin: Peri<'d, impl TxPin<T>>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        config: Config,
    ) -> Result<Self, Error> {
        let uart = Self::new_inner(_rx_pin, _tx_pin, config)?;
        interrupt_en::<T>();
        Ok(uart)
    }

    /// Reads bytes from RX FIFO until buffer is full.
    ///
    /// # Errors
    ///
    /// Returns [`RxError`] if error occurred during read.
    pub fn read(&mut self, buf: &mut [u8]) -> impl Future<Output = Result<(), RxError>> {
        self.rx.read(buf)
    }

    /// Writes bytes to TX FIFO.
    pub fn write(&mut self, bytes: &[u8]) -> impl Future<Output = ()> {
        self.tx.write(bytes)
    }

    /// Waits until both the transmit holding and shift registers are empty.
    pub fn flush(&mut self) -> impl Future<Output = ()> {
        self.tx.flush()
    }
}

/// RX-only UART driver.
pub struct UartRx<'d, M: Mode> {
    info: &'static Info,
    _rx_pin: Peri<'d, AnyPin>,
    _phantom: PhantomData<&'d M>,
}

impl<'d, M: Mode> UartRx<'d, M> {
    fn new_inner<T: Instance>(_rx_pin: Peri<'d, AnyPin>) -> Self {
        T::info().rx_tx_refcount.fetch_add(1, Ordering::AcqRel);

        Self {
            info: T::info(),
            _rx_pin,
            _phantom: PhantomData,
        }
    }

    fn read_inner(&mut self, lsr: DataLsr) -> Result<u8, Error> {
        // An overrun error is not associated with particular character like others
        // We just bail out early, and it might be hard to recover correctly from this
        //
        // The onus will likely need to be on caller to handle an overrun how they see fit
        if lsr.overrun() {
            return Err(Error::Overrun);
        }

        // Read byte first (because even if the byte produces an error we still want to drain it)
        let byte = self.info.reg.data().rx_dat().read();

        // Note: We don't make use of bit 7 (FIFO_ERROR) because it just tells us if one of the
        // below errors is somewhere in the FIFO, but we are checking the specific byte at top of
        // the FIFO for an error

        // And only return it if there is no error
        if lsr.pe() {
            Err(Error::Parity)
        } else if lsr.frame_err() {
            Err(Error::Frame)
        } else if lsr.brk_intr() {
            Err(Error::Break)
        } else {
            Ok(byte)
        }
    }

    fn nb_read_byte(&mut self) -> Result<u8, Error> {
        let lsr = self.info.reg.data().lsr().read();
        self.read_inner(lsr)
    }

    fn blocking_read_byte(&mut self) -> Result<u8, Error> {
        // LSR clears error bits on read so want to make sure only read it once after data ready
        let lsr = loop {
            let lsr = self.info.reg.data().lsr().read();
            if lsr.data_ready() {
                break lsr;
            }
        };

        self.read_inner(lsr)
    }

    /// Reads bytes from RX FIFO until buffer is full, blocking if empty.
    ///
    /// # Errors
    ///
    /// Returns [`RxError`] if error occurred during read.
    pub fn blocking_read(&mut self, buf: &mut [u8]) -> Result<(), RxError> {
        // If we encounter an error, return the number of valid bytes read up until error occurred
        for (bytes_read, byte) in buf.iter_mut().enumerate() {
            *byte = self.blocking_read_byte().map_err(|err| RxError { bytes_read, err })?;
        }

        Ok(())
    }
}

impl<'d> UartRx<'d, Blocking> {
    /// Create a new blocking RX-only UART driver instance with given configuration.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidBaud`] if the supplied baud rate can not be represented by
    /// given clock source.
    pub fn new_blocking<T: Instance>(
        _peri: Peri<'d, T>,
        _rx_pin: Peri<'d, impl RxPin<T>>,
        config: Config,
    ) -> Result<Self, Error> {
        let rx_pin = _rx_pin.into();
        init::<T>(Some(&rx_pin), None, config)?;
        Ok(Self::new_inner::<T>(rx_pin))
    }
}

impl<'d> UartRx<'d, Async> {
    async fn wait_rx_ready(&mut self) -> bool {
        poll_fn(|cx| {
            self.info.rx_waker.register(cx.waker());
            let int_flags = self.info.reg.data().scr().read();

            if int_flags & int_flag::TIMEOUT != 0 {
                critical_section::with(|_| self.info.reg.data().scr().write(|w| *w &= !int_flag::TIMEOUT));
                // Indicates a byte is ready to read, but we didn't trigger the FIFO level before timeout
                Poll::Ready(false)
            } else if int_flags & int_flag::RX_AVAILABLE != 0 {
                critical_section::with(|_| self.info.reg.data().scr().write(|w| *w &= !int_flag::RX_AVAILABLE));
                Poll::Ready(true)
            } else {
                critical_section::with(|_| {
                    self.info.reg.data().ien().modify(|w| {
                        w.set_elsi(true);
                        w.set_erdai(true);
                    })
                });
                Poll::Pending
            }
        })
        .await
    }

    fn set_fifo_trigger(&mut self, trigger: RxFifoTrigger) {
        self.info.reg.data().fifo_cr().write(|w| {
            // Must always set EXRF when setting other bits in the reg, even if previously set
            w.set_exrf(true);
            w.set_recv_fifo_trig_lvl(trigger.into());
        });
    }

    async fn read_byte(&mut self) -> Result<u8, Error> {
        // LSR clears error bits on read so want to make sure only read it once when data ready then check bits
        let lsr = {
            let lsr = self.info.reg.data().lsr().read();
            if !lsr.data_ready() {
                let _ = self.wait_rx_ready().await;
                self.info.reg.data().lsr().read()
            } else {
                lsr
            }
        };

        self.read_inner(lsr)
    }

    async fn read_chunk(&mut self, chunk: &mut [u8], bytes_read_start: usize) -> Result<(), RxError> {
        self.set_fifo_trigger(RxFifoTrigger::_1);
        for (bytes_read, byte) in chunk.iter_mut().enumerate() {
            *byte = self.read_byte().await.map_err(|err| RxError {
                bytes_read: bytes_read_start + bytes_read,
                err,
            })?;
        }
        Ok(())
    }

    async fn read_chunk_batched(&mut self, chunk: &mut [u8], bytes_read_start: usize) -> Result<(), RxError> {
        // If our FIFO level was reached without timeout, we can read all the bytes in one go,
        // but still need to check each byte for an error
        if self.wait_rx_ready().await {
            for (bytes_read, byte) in chunk.iter_mut().enumerate() {
                *byte = self.nb_read_byte().map_err(|err| RxError {
                    bytes_read: bytes_read_start + bytes_read,
                    err,
                })?;
            }

        // However, if a timeout occured, our assumptions about the number of bytes in the FIFO
        // no longer holds (since we have to read a byte to clear the timeout interrupt), meaning
        // we have no choice but to unfortunately fall back on byte-by-byte interrupts
        } else {
            self.read_chunk(chunk, bytes_read_start).await?;
        }

        Ok(())
    }

    async fn read_chunks<const N: usize>(
        &mut self,
        chunks: &mut [[u8; N]],
        trigger: RxFifoTrigger,
        bytes_read_start: usize,
    ) -> Result<(), RxError> {
        self.set_fifo_trigger(trigger);
        for (i, chunk) in chunks.iter_mut().enumerate() {
            self.read_chunk_batched(chunk, bytes_read_start + (N * i)).await?;
        }
        Ok(())
    }

    /// Create a new async RX-only UART driver instance with given configuration.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidBaud`] if the supplied baud rate can not be represented by
    /// given clock source.
    pub fn new_async<T: Instance>(
        _peri: Peri<'d, T>,
        _rx_pin: Peri<'d, impl RxPin<T>>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        config: Config,
    ) -> Result<Self, Error> {
        let rx_pin = _rx_pin.into();
        init::<T>(Some(&rx_pin), None, config)?;
        interrupt_en::<T>();
        Ok(Self::new_inner::<T>(rx_pin))
    }

    /// Reads bytes from RX FIFO until buffer is full.
    ///
    /// # Errors
    ///
    /// Returns [`RxError`] if error occurred during read.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<(), RxError> {
        // The idea here is that the HW provides us 4 FIFO level triggers (14, 8, 4, 1),
        // so to minimize the number of interrupts, we split the buffer up greedily into chunks
        // of the highest trigger level we can, and then know when the interrupt is triggered,
        // we can read that number of bytes in one shot.
        const TRIG_14: usize = 14;
        const TRIG_8: usize = 8;
        const TRIG_4: usize = 4;

        let (c14, rem) = buf.as_chunks_mut::<TRIG_14>();
        let (c8, rem) = rem.as_chunks_mut::<TRIG_8>();
        let (c4, c1) = rem.as_chunks_mut::<TRIG_4>();

        // We keep track of total number of valid bytes read, so in case of error,
        // we can return this number to the caller
        let mut bytes_read = 0;

        self.read_chunks(c14, RxFifoTrigger::_14, bytes_read).await?;
        bytes_read += c14.len() * TRIG_14;

        self.read_chunks(c8, RxFifoTrigger::_8, bytes_read).await?;
        bytes_read += c8.len() * TRIG_8;

        self.read_chunks(c4, RxFifoTrigger::_4, bytes_read).await?;
        bytes_read += c4.len() * TRIG_4;

        // The last chunk is smaller than 4, so we have no choice but to interrupt for each byte
        self.read_chunk(c1, bytes_read).await?;

        Ok(())
    }
}

impl<'d, M: Mode> Drop for UartRx<'d, M> {
    fn drop(&mut self) {
        // Revisit: Add API for disabling pins
        drop_rx_tx(self.info);
    }
}

/// TX-only UART driver.
pub struct UartTx<'d, M: Mode> {
    info: &'static Info,
    _tx_pin: Peri<'d, AnyPin>,
    _phantom: PhantomData<&'d M>,
}

impl<'d, M: Mode> UartTx<'d, M> {
    fn new_inner<T: Instance>(_tx_pin: Peri<'d, AnyPin>) -> Self {
        T::info().rx_tx_refcount.fetch_add(1, Ordering::AcqRel);

        Self {
            info: T::info(),
            _tx_pin,
            _phantom: PhantomData,
        }
    }

    fn write_chunk(&mut self, chunk: &[u8]) {
        for byte in chunk {
            self.info.reg.data().tx_dat().write(|w| *w = *byte);
        }
    }

    /// Writes bytes to TX FIFO, blocking if full.
    pub fn blocking_write(&mut self, buf: &[u8]) {
        for chunk in buf.chunks(FIFO_SZ) {
            while !self.info.reg.data().lsr().read().trans_empty() {}
            self.write_chunk(chunk);
        }
    }

    /// Blocks until both the transmit holding and shift registers are empty.
    pub fn blocking_flush(&mut self) {
        // Note: The register for some reason is named "Transmit Error" but it really
        // reflects the status of both the transmit and shift registers aka "Busy" status
        while !self.info.reg.data().lsr().read().trans_err() {}
    }
}

impl<'d> UartTx<'d, Blocking> {
    /// Create a new blocking TX-only UART driver instance with given configuration.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidBaud`] if the supplied baud rate can not be represented by
    /// given clock source.
    pub fn new_blocking<T: Instance>(
        _peri: Peri<'d, T>,
        _tx_pin: Peri<'d, impl TxPin<T>>,
        config: Config,
    ) -> Result<Self, Error> {
        let tx_pin = _tx_pin.into();
        init::<T>(None, Some(&tx_pin), config)?;
        Ok(Self::new_inner::<T>(tx_pin))
    }
}

impl<'d> UartTx<'d, Async> {
    async fn wait_tx_empty(&mut self) {
        poll_fn(|cx| {
            self.info.tx_waker.register(cx.waker());
            if self.info.reg.data().scr().read() & int_flag::TX_EMPTY != 0 {
                critical_section::with(|_| self.info.reg.data().scr().write(|w| *w &= !int_flag::TX_EMPTY));
                Poll::Ready(())
            } else {
                critical_section::with(|_| self.info.reg.data().ien().modify(|w| w.set_ethrei(true)));
                Poll::Pending
            }
        })
        .await
    }

    /// Create a new async TX-only UART driver instance with given configuration.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidBaud`] if the supplied baud rate can not be represented by
    /// given clock source.
    pub fn new_async<T: Instance>(
        _peri: Peri<'d, T>,
        _tx_pin: Peri<'d, impl TxPin<T>>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        config: Config,
    ) -> Result<Self, Error> {
        let tx_pin = _tx_pin.into();
        init::<T>(None, Some(&tx_pin), config)?;
        interrupt_en::<T>();
        Ok(Self::new_inner::<T>(tx_pin))
    }

    /// Writes bytes to TX FIFO.
    pub async fn write(&mut self, buf: &[u8]) {
        for chunk in buf.chunks(FIFO_SZ) {
            self.wait_tx_empty().await;
            self.write_chunk(chunk);
        }
    }

    /// Waits until both the transmit holding and shift registers are empty.
    pub async fn flush(&mut self) {
        // We can wait for an interrupt to know when the TX FIFO is empty,
        // but there does not appear to be an interrupt for when the TX shift reg is empty,
        // so have to block
        self.wait_tx_empty().await;
        self.blocking_flush();
    }
}

impl<'d, M: Mode> Drop for UartTx<'d, M> {
    fn drop(&mut self) {
        // Revisit: Add API for disabling pins
        drop_rx_tx(self.info);
    }
}

struct Info {
    reg: Uart0,
    rx_tx_refcount: AtomicU8,
    rx_waker: AtomicWaker,
    tx_waker: AtomicWaker,
}

trait SealedMode {}

/// Blocking mode.
pub struct Blocking;
impl SealedMode for Blocking {}
impl Mode for Blocking {}

/// Async mode.
pub struct Async;
impl SealedMode for Async {}
impl Mode for Async {}

/// Driver mode.
#[allow(private_bounds)]
pub trait Mode: SealedMode {}

trait SealedInstance {
    fn info() -> &'static Info;
    fn irq_bit() -> usize;
}

/// UART instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_instance {
    ($peri:ident, $bit:expr) => {
        impl SealedInstance for peripherals::$peri {
            #[inline(always)]
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    reg: pac::$peri,
                    rx_tx_refcount: AtomicU8::new(0),
                    rx_waker: AtomicWaker::new(),
                    tx_waker: AtomicWaker::new(),
                };
                &INFO
            }

            #[inline(always)]
            fn irq_bit() -> usize {
                $bit
            }
        }

        impl Instance for peripherals::$peri {
            type Interrupt = crate::interrupt::typelevel::$peri;
        }
    };
}

impl_instance!(UART0, 0);
impl_instance!(UART1, 1);

/// A pin that can be configured as a UART RX pin.
pub trait RxPin<T: Instance>: Pin + PeripheralType {}

/// A pin that can be configured as a UART TX pin.
pub trait TxPin<T: Instance>: Pin + PeripheralType {}

macro_rules! impl_pin {
    ($function:ident, $peri:ident, $($pin:ident),*) => {
        $(
            impl $function<peripherals::$peri> for peripherals::$pin {}
        )*
    }
}

impl_pin!(RxPin, UART0, GPIO105);
impl_pin!(TxPin, UART0, GPIO104);
impl_pin!(RxPin, UART1, GPIO171, GPIO255);
impl_pin!(TxPin, UART1, GPIO170);

impl<'d, M: Mode> embedded_hal_02::blocking::serial::Write<u8> for Uart<'d, M> {
    type Error = core::convert::Infallible;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(buffer);
        Ok(())
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush();
        Ok(())
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::serial::Write<u8> for UartTx<'d, M> {
    type Error = core::convert::Infallible;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(buffer);
        Ok(())
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush();
        Ok(())
    }
}

impl embedded_io::Error for RxError {
    fn kind(&self) -> embedded_io::ErrorKind {
        // Overrun error may need to be handled differently than other errors,
        // but there isn't a perfect `ErrorKind` match for it
        if self.err == Error::Overrun {
            embedded_io::ErrorKind::Other
        } else {
            embedded_io::ErrorKind::InvalidData
        }
    }
}

impl<'d, M: Mode> embedded_io::ErrorType for Uart<'d, M> {
    type Error = RxError;
}

impl<'d, M: Mode> embedded_io::Read for Uart<'d, M> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.blocking_read(buf).map(|_| buf.len())
    }
}

impl<'d> embedded_io_async::Read for Uart<'d, Async> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read(buf).await.map(|_| buf.len())
    }
}

impl<'d, M: Mode> embedded_io::Write for Uart<'d, M> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush();
        Ok(())
    }
}

impl<'d> embedded_io_async::Write for Uart<'d, Async> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.write(buf).await;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush().await;
        Ok(())
    }
}

impl<'d, M: Mode> embedded_io::ErrorType for UartTx<'d, M> {
    type Error = core::convert::Infallible;
}

impl<'d, M: Mode> embedded_io::Write for UartTx<'d, M> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush();
        Ok(())
    }
}

impl<'d> embedded_io_async::Write for UartTx<'d, Async> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.write(buf).await;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush().await;
        Ok(())
    }
}

impl<'d, M: Mode> embedded_io::ErrorType for UartRx<'d, M> {
    type Error = RxError;
}

impl<'d, M: Mode> embedded_io::Read for UartRx<'d, M> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.blocking_read(buf).map(|_| buf.len())
    }
}

impl<'d> embedded_io_async::Read for UartRx<'d, Async> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read(buf).await.map(|_| buf.len())
    }
}
