use core::fmt::Debug;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_futures::select::{select, Either};
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use embedded_io::{self, ErrorKind};

use crate::dma::{AnyChannel, Channel};
use crate::gpio::{match_iocon, AnyPin, Bank, SealedPin};
use crate::interrupt::typelevel::{Binding, Interrupt as _};
use crate::interrupt::Interrupt;
use crate::pac::flexcomm::Flexcomm as FlexcommReg;
use crate::pac::iocon::vals::PioFunc;
use crate::pac::usart::Usart as UsartReg;
use crate::pac::*;
use crate::{Async, Blocking, Mode};

/// Serial error
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Triggered when the FIFO (or shift-register) is overflowed.
    Overrun,
    /// Triggered when there is a parity mismatch between what's received and
    /// our settings.
    Parity,
    /// Triggered when the received character didn't have a valid stop bit.
    Framing,
    /// Triggered when the receiver detects noise
    Noise,
    /// Triggered when the receiver gets a break
    Break,
}

impl embedded_io::Error for Error {
    fn kind(&self) -> ErrorKind {
        match self {
            Error::Overrun => ErrorKind::Other,
            Error::Parity => ErrorKind::InvalidData,
            Error::Framing => ErrorKind::InvalidData,
            Error::Noise => ErrorKind::Other,
            Error::Break => ErrorKind::Interrupted,
        }
    }
}
/// Word length.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DataBits {
    /// 7 bits data length.
    DataBits7,
    /// 8 bits data length.
    DataBits8,
    /// 9 bits data length. The 9th bit is commonly used for addressing in multidrop mode.
    DataBits9,
}

/// Parity bit.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Parity {
    /// No parity.
    ParityNone,
    /// Even parity.
    ParityEven,
    /// Odd parity.
    ParityOdd,
}

/// Stop bits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StopBits {
    /// 1 stop bit.
    Stop1,
    /// 2 stop bits. This setting should only be used for asynchronous communication.
    Stop2,
}

/// UART config.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Config {
    /// Baud rate.
    pub baudrate: u32,
    /// Word length.
    pub data_bits: DataBits,
    /// Stop bits.
    pub stop_bits: StopBits,
    /// Parity bit.
    pub parity: Parity,
    /// Invert the tx pin output
    pub invert_tx: bool,
    /// Invert the rx pin input
    pub invert_rx: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            baudrate: 115200,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::Stop1,
            parity: Parity::ParityNone,
            invert_rx: false,
            invert_tx: false,
        }
    }
}

/// Internal DMA state of UART RX.
pub struct DmaState {
    rx_err_waker: AtomicWaker,
    rx_err: AtomicBool,
}

/// # Type parameters
/// 'd: the lifetime marker ensuring correct borrow checking for peripherals used at compile time
/// T: the peripheral instance type allowing usage of peripheral specific registers
/// M: the operating mode of USART peripheral
pub struct Usart<'d, M: Mode> {
    tx: UsartTx<'d, M>,
    rx: UsartRx<'d, M>,
}

pub struct UsartTx<'d, M: Mode> {
    info: &'static Info,
    tx_dma: Option<Peri<'d, AnyChannel>>,
    phantom: PhantomData<M>,
}

pub struct UsartRx<'d, M: Mode> {
    info: &'static Info,
    dma_state: &'static DmaState,
    rx_dma: Option<Peri<'d, AnyChannel>>,
    phantom: PhantomData<M>,
}

impl<'d, M: Mode> UsartTx<'d, M> {
    pub fn new<T: Instance>(
        _usart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        tx_dma: Peri<'d, impl Channel>,
        config: Config,
    ) -> Self {
        Usart::<M>::init::<T>(Some(tx.into()), None, config);
        Self::new_inner(T::info(), Some(tx_dma.into()))
    }

    #[inline]
    fn new_inner(info: &'static Info, tx_dma: Option<Peri<'d, AnyChannel>>) -> Self {
        Self {
            info,
            tx_dma,
            phantom: PhantomData,
        }
    }

    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        for &b in buffer {
            while !(self.info.usart_reg.fifostat().read().txnotfull()) {}
            self.info.usart_reg.fifowr().write(|w| w.set_txdata(b as u16));
        }
        Ok(())
    }

    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        while !(self.info.usart_reg.fifostat().read().txempty()) {}
        Ok(())
    }

    pub fn tx_busy(&self) -> bool {
        !(self.info.usart_reg.fifostat().read().txempty())
    }
}

impl<'d> UsartTx<'d, Blocking> {
    pub fn new_blocking<T: Instance>(_usart: Peri<'d, T>, tx: Peri<'d, impl TxPin<T>>, config: Config) -> Self {
        Usart::<Blocking>::init::<T>(Some(tx.into()), None, config);
        Self::new_inner(T::info(), None)
    }
}

impl<'d> UsartTx<'d, Async> {
    /// Write to UART TX from the provided buffer using DMA.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        // Unwrap() can be used because UsartTx::new() in Async mode always sets it to Some
        let ch = self.tx_dma.as_mut().unwrap().reborrow();
        let transfer = unsafe {
            // Enable to pace DMA transfers.
            self.info.usart_reg.fifocfg().modify(|w| w.set_dmatx(true));
            // If future is not assigned to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            crate::dma::write(ch, buffer, self.info.usart_reg.fifowr().as_ptr() as *mut _)
        };
        transfer.await;
        Ok(())
    }
}
impl<'d, M: Mode> UsartRx<'d, M> {
    pub fn new<T: Instance>(
        _usart: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        has_irq: bool,
        rx_dma: Peri<'d, impl Channel>,
        config: Config,
    ) -> Self {
        Usart::<M>::init::<T>(None, Some(rx.into()), config);
        Self::new_inner(T::info(), T::dma_state(), has_irq, Some(rx_dma.into()))
    }

    fn new_inner(
        info: &'static Info,
        dma_state: &'static DmaState,
        has_irq: bool,
        rx_dma: Option<Peri<'d, AnyChannel>>,
    ) -> Self {
        core::debug_assert_eq!(has_irq, rx_dma.is_some());
        if has_irq {
            // Disable all the related interrupts for now.
            info.usart_reg.intenclr().write(|w| {
                w.set_framerrclr(true);
                w.set_parityerrclr(true);
                w.set_rxnoiseclr(true);
            });
            info.usart_reg.fifointenclr().modify(|w| {
                w.set_rxlvl(true);
                w.set_rxerr(true);
            });
            info.interrupt.unpend();
            unsafe {
                info.interrupt.enable();
            }
        }
        Self {
            info,
            dma_state,
            rx_dma,
            phantom: PhantomData,
        }
    }

    pub fn blocking_read(&mut self, mut buffer: &mut [u8]) -> Result<(), Error> {
        while !buffer.is_empty() {
            match Self::drain_fifo(self, buffer) {
                Ok(0) => continue, // Wait for more data
                Ok(n) => buffer = &mut buffer[n..],
                Err((_, err)) => return Err(err),
            }
        }
        Ok(())
    }

    /// Returns:
    /// - Ok(n) -> read n bytes
    /// - Err(n, Error) -> read n-1 bytes, but encountered an error while reading nth byte
    fn drain_fifo(&mut self, buffer: &mut [u8]) -> Result<usize, (usize, Error)> {
        for (i, b) in buffer.iter_mut().enumerate() {
            while !(self.info.usart_reg.fifostat().read().rxnotempty()) {}
            if self.info.usart_reg.fifostat().read().rxerr() {
                return Err((i, Error::Overrun));
            } else if self.info.usart_reg.fifordnopop().read().parityerr() {
                return Err((i, Error::Parity));
            } else if self.info.usart_reg.fifordnopop().read().framerr() {
                return Err((i, Error::Framing));
            } else if self.info.usart_reg.fifordnopop().read().rxnoise() {
                return Err((i, Error::Noise));
            } else if self.info.usart_reg.intstat().read().deltarxbrk() {
                return Err((i, Error::Break));
            }
            let dr = self.info.usart_reg.fiford().read().rxdata() as u8;
            *b = dr;
        }
        Ok(buffer.len())
    }
}

impl<'d> UsartRx<'d, Blocking> {
    pub fn new_blocking<T: Instance>(_usart: Peri<'d, T>, rx: Peri<'d, impl RxPin<T>>, config: Config) -> Self {
        Usart::<Blocking>::init::<T>(None, Some(rx.into()), config);
        Self::new_inner(T::info(), T::dma_state(), false, None)
    }
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _uart: PhantomData<T>,
}

impl<T: Instance> crate::interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::info().usart_reg;
        if !regs.fifocfg().read().dmarx() {
            return;
        }
        let state = T::dma_state();
        state.rx_err.store(true, Ordering::Relaxed);
        state.rx_err_waker.wake();
        // Disable the error interrupts instead of clearing the flags. Clearing the
        // flags would allow the DMA transfer to continue, potentially signaling
        // completion before we can check for errors that happened *during* the transfer.
        regs.intenclr().write(|w| {
            w.set_framerrclr(true);
            w.set_rxnoiseclr(true);
            w.set_parityerrclr(true);
        });
        regs.fifointenclr().write(|w| w.set_rxerr(true));
    }
}

impl<'d> UsartRx<'d, Async> {
    /// Read from USART RX into the provided buffer.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        // Clear error flags before the FIFO is drained. Errors that have accumulated
        // in the flags will also be present in the FIFO.
        self.dma_state.rx_err.store(false, Ordering::Relaxed);
        self.info.usart_reg.intenclr().write(|w| {
            w.set_framerrclr(true);
            w.set_parityerrclr(true);
            w.set_rxnoiseclr(true);
        });
        self.info.usart_reg.fifointenclr().modify(|w| w.set_rxerr(true));
        // Then drain the fifo. It is necessary to read at most 16 bytes (the size of FIFO).
        // Errors that apply to FIFO bytes will be reported directly.
        let buffer = match {
            let limit = buffer.len().min(16);
            self.drain_fifo(&mut buffer[0..limit])
        } {
            Ok(len) if len < buffer.len() => &mut buffer[len..],
            Ok(_) => return Ok(()),
            Err((_i, e)) => return Err(e),
        };

        // Start a DMA transfer. If errors have happened in the interim some error
        // interrupt flags will have been raised, and those will be picked up immediately
        // by the interrupt handler.
        // Unwrap() can be used because UsartRx::new() in Async mode always sets it to Some
        let ch = self.rx_dma.as_mut().unwrap().reborrow();

        self.info.usart_reg.intenset().write(|w| {
            w.set_framerren(true);
            w.set_parityerren(true);
            w.set_rxnoiseen(true);
        });
        self.info.usart_reg.fifointenset().modify(|w| w.set_rxerr(true));
        self.info.usart_reg.fifocfg().modify(|w| w.set_dmarx(true));
        let transfer = unsafe {
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            crate::dma::read(ch, self.info.usart_reg.fiford().as_ptr() as *const _, buffer)
        };

        // wait for either the transfer to complete or an error to happen.
        let transfer_result = select(
            transfer,
            poll_fn(|cx| {
                self.dma_state.rx_err_waker.register(cx.waker());
                match self.dma_state.rx_err.swap(false, Ordering::Relaxed) {
                    false => Poll::Pending,
                    e => Poll::Ready(e),
                }
            }),
        )
        .await;

        let errors = match transfer_result {
            Either::First(()) => {
                // The DMA controller finished, BUT if an error occurred on the LAST
                // byte, then we may still need to grab the error state!
                self.dma_state.rx_err.swap(false, Ordering::Relaxed)
            }
            Either::Second(e) => {
                // There is an error, which means this is the error that
                // was problematic.
                e
            }
        };

        // If we got no error, just return at this point
        if !errors {
            return Ok(());
        }

        // If we DID get an error, we need to figure out which one it was.
        if self.info.usart_reg.intstat().read().framerrint() {
            return Err(Error::Framing);
        } else if self.info.usart_reg.intstat().read().parityerrint() {
            return Err(Error::Parity);
        } else if self.info.usart_reg.intstat().read().rxnoiseint() {
            return Err(Error::Noise);
        } else if self.info.usart_reg.fifointstat().read().rxerr() {
            return Err(Error::Overrun);
        }
        unreachable!("unrecognized rx error");
    }
}

impl<'d> Usart<'d, Blocking> {
    pub fn new_blocking<T: Instance>(
        usart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(usart, tx.into(), rx.into(), false, None, None, config)
    }
}

impl<'d> Usart<'d, Async> {
    pub fn new<T: Instance>(
        uart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        tx_dma: Peri<'d, impl TxChannel<T>>,
        rx_dma: Peri<'d, impl RxChannel<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            uart,
            tx.into(),
            rx.into(),
            true,
            Some(tx_dma.into()),
            Some(rx_dma.into()),
            config,
        )
    }
}

impl<'d, M: Mode> Usart<'d, M> {
    fn new_inner<T: Instance>(
        _usart: Peri<'d, T>,
        mut tx: Peri<'d, AnyPin>,
        mut rx: Peri<'d, AnyPin>,
        has_irq: bool,
        tx_dma: Option<Peri<'d, AnyChannel>>,
        rx_dma: Option<Peri<'d, AnyChannel>>,
        config: Config,
    ) -> Self {
        Self::init::<T>(Some(tx.reborrow()), Some(rx.reborrow()), config);
        Self {
            tx: UsartTx::new_inner(T::info(), tx_dma),
            rx: UsartRx::new_inner(T::info(), T::dma_state(), has_irq, rx_dma),
        }
    }

    fn init<T: Instance>(tx: Option<Peri<'_, AnyPin>>, rx: Option<Peri<'_, AnyPin>>, config: Config) {
        Self::configure_flexcomm(T::info().fc_reg, T::instance_number());
        Self::configure_clock::<T>(&config);
        Self::pin_config::<T>(tx, rx);
        Self::configure_usart(T::info(), &config);
    }

    fn configure_clock<T: Instance>(config: &Config) {
        // Select source clock

        // Adaptive clock choice based on baud rate
        // To get the desired baud rate, it is essential to choose the clock bigger than baud rate so that it can be 'chiseled'
        // There are two types of dividers: integer divider (baud rate generator register and oversample selection value)
        // and fractional divider (fractional rate divider).

        // By default, oversampling rate is 16 which is an industry standard.
        // That means 16 clocks are used to deliver the byte to recipient.
        // In this way the probability of getting correct bytes instead of noise directly increases as oversampling increases as well.

        // Minimum and maximum values were computed taking these formulas into account:
        // For minimum value, MULT = 0, BRGVAL = 0
        // For maximum value, MULT = 255, BRGVAL = 255
        // Flexcomm Interface function clock = (clock selected via FCCLKSEL) / (1 + MULT / DIV)
        // By default, OSRVAL = 15 (see above)
        // Baud rate = [FCLK / (OSRVAL+1)] / (BRGVAL + 1)
        let source_clock = match config.baudrate {
            750_001..=6_000_000 => {
                SYSCON
                    .fcclksel(T::instance_number())
                    .modify(|w| w.set_sel(syscon::vals::FcclkselSel::ENUM_0X3)); // 96 MHz
                96_000_000
            }
            1501..=750_000 => {
                SYSCON
                    .fcclksel(T::instance_number())
                    .modify(|w| w.set_sel(syscon::vals::FcclkselSel::ENUM_0X2)); // 12 MHz
                12_000_000
            }
            121..=1500 => {
                SYSCON
                    .fcclksel(T::instance_number())
                    .modify(|w| w.set_sel(syscon::vals::FcclkselSel::ENUM_0X4)); // 1 MHz
                1_000_000
            }
            _ => {
                panic!("{} baudrate is not permitted in this mode", config.baudrate);
            }
        };
        // Calculate MULT and BRG values based on baudrate

        // There are two types of dividers: integer divider (baud rate generator register and oversample selection value)
        // and fractional divider (fractional rate divider).
        // For oversampling, the default is industry standard 16x oversampling, i.e. OSRVAL = 15

        // The formulas are:

        // FLCK = (clock selected via FCCLKSEL) / (1 + MULT / DIV)
        // DIV is always 256, then:
        // FLCK = (clock selected via FCCLKSEL) / (1 + MULT / 256)

        // Baud rate = [FCLK / (OSRVAL+1)] / (BRGVAL + 1) =>
        // Baud rate = [FCLK / 16] / (BRGVAL + 1)

        // There are 2 unknowns: MULT and BRGVAL.
        // MULT is responsible for fractional division
        // BRGVAL is responsible for integer division

        //  The Fractional Rate Generator can be used to obtain more precise baud rates when the
        //  function clock is not a good multiple of standard (or otherwise desirable) baud rates.
        //  The FRG is typically set up to produce an integer multiple of the highest required baud
        //  rate, or a very close approximation. The BRG is then used to obtain the actual baud rate
        //  needed.

        // Firstly, BRGVAL is calculated to get the raw clock which is a rough approximation that has to be adjusted
        // so that the desired baud rate is obtained.
        // Secondly, MULT is calculated to ultimately 'chisel' the clock to get the baud rate.
        // The deduced formulas are written below.

        let brg_value = (source_clock / (16 * config.baudrate)).min(255);
        let raw_clock = source_clock / (16 * brg_value);
        let mult_value = ((raw_clock * 256 / config.baudrate) - 256).min(255);

        // Write values to the registers

        // FCLK =  (clock selected via FCCLKSEL) / (1+ MULT / DIV)
        // Remark: To use the fractional baud rate generator, 0xFF must be wirtten to the DIV value
        // to yield a denominator vale of 256. All other values are not supported
        SYSCON.flexfrgctrl(T::instance_number()).modify(|w| {
            w.set_div(0xFF);
            w.set_mult(mult_value as u8);
        });

        // Baud rate = [FCLK / (OSRVAL+1)] / (BRGVAL + 1)
        // By default, oversampling is 16x, i.e. OSRVAL = 15

        // Typical industry standard USARTs use a 16x oversample clock to transmit and receive
        // asynchronous data. This is the number of BRG clocks used for one data bit. The
        // Oversample Select Register (OSR) allows this USART to use a 16x down to a 5x
        // oversample clock. There is no oversampling in synchronous modes.
        T::info()
            .usart_reg
            .brg()
            .modify(|w| w.set_brgval((brg_value - 1) as u16));
    }

    fn pin_config<T: Instance>(tx: Option<Peri<'_, AnyPin>>, rx: Option<Peri<'_, AnyPin>>) {
        if let Some(tx_pin) = tx {
            match_iocon!(register, tx_pin.pin_bank(), tx_pin.pin_number(), {
                register.modify(|w| {
                    w.set_func(T::tx_pin_func());
                    w.set_mode(iocon::vals::PioMode::INACTIVE);
                    w.set_slew(iocon::vals::PioSlew::STANDARD);
                    w.set_invert(false);
                    w.set_digimode(iocon::vals::PioDigimode::DIGITAL);
                    w.set_od(iocon::vals::PioOd::NORMAL);
                });
            })
        }

        if let Some(rx_pin) = rx {
            match_iocon!(register, rx_pin.pin_bank(), rx_pin.pin_number(), {
                register.modify(|w| {
                    w.set_func(T::rx_pin_func());
                    w.set_mode(iocon::vals::PioMode::INACTIVE);
                    w.set_slew(iocon::vals::PioSlew::STANDARD);
                    w.set_invert(false);
                    w.set_digimode(iocon::vals::PioDigimode::DIGITAL);
                    w.set_od(iocon::vals::PioOd::NORMAL);
                });
            })
        };
    }

    fn configure_flexcomm(flexcomm_register: crate::pac::flexcomm::Flexcomm, instance_number: usize) {
        critical_section::with(|_cs| {
            if !(SYSCON.ahbclkctrl0().read().iocon()) {
                SYSCON.ahbclkctrl0().modify(|w| w.set_iocon(true));
            }
        });
        critical_section::with(|_cs| {
            if !(SYSCON.ahbclkctrl1().read().fc(instance_number)) {
                SYSCON.ahbclkctrl1().modify(|w| w.set_fc(instance_number, true));
            }
        });
        SYSCON
            .presetctrl1()
            .modify(|w| w.set_fc_rst(instance_number, syscon::vals::FcRst::ASSERTED));
        SYSCON
            .presetctrl1()
            .modify(|w| w.set_fc_rst(instance_number, syscon::vals::FcRst::RELEASED));
        flexcomm_register.pselid().modify(|w| {
            w.set_persel(flexcomm::vals::Persel::USART);
            // This will lock the peripheral PERSEL and will not allow any changes until the board is reset.
            w.set_lock(true);
        });
    }

    fn configure_usart(info: &'static Info, config: &Config) {
        let registers = info.usart_reg;
        // See section 34.6.1
        registers.cfg().modify(|w| {
            // LIN break mode enable
            // Disabled. Break detect and generate is configured for normal operation.
            w.set_linmode(false);
            //CTS Enable. Determines whether CTS is used for flow control. CTS can be from the
            //input pin, or from the USART’s own RTS if loopback mode is enabled.
            // No flow control. The transmitter does not receive any automatic flow control signal.
            w.set_ctsen(false);
            // Selects synchronous or asynchronous operation.
            w.set_syncen(usart::vals::Syncen::ASYNCHRONOUS_MODE);
            // Selects the clock polarity and sampling edge of received data in synchronous mode.
            w.set_clkpol(usart::vals::Clkpol::RISING_EDGE);
            // Synchronous mode Master select.
            // When synchronous mode is enabled, the USART is a master.
            w.set_syncmst(usart::vals::Syncmst::MASTER);
            // Selects data loopback mode
            w.set_loop_(usart::vals::Loop::NORMAL);
            // Output Enable Turnaround time enable for RS-485 operation.
            // Disabled. If selected by OESEL, the Output Enable signal deasserted at the end of
            // the last stop bit of a transmission.
            w.set_oeta(false);
            // Output enable select.
            // Standard. The RTS signal is used as the standard flow control function.
            w.set_oesel(usart::vals::Oesel::STANDARD);
            // Automatic address matching enable.
            // Disabled. When addressing is enabled by ADDRDET, address matching is done by
            // software. This provides the possibility of versatile addressing (e.g. respond to more
            // than one address)
            w.set_autoaddr(false);
            // Output enable polarity.
            // Low. If selected by OESEL, the output enable is active low.
            w.set_oepol(usart::vals::Oepol::LOW);
        });

        // Configurations based on the config written by a user
        registers.cfg().modify(|w| {
            w.set_datalen(match config.data_bits {
                DataBits::DataBits7 => usart::vals::Datalen::BIT_7,
                DataBits::DataBits8 => usart::vals::Datalen::BIT_8,
                DataBits::DataBits9 => usart::vals::Datalen::BIT_9,
            });
            w.set_paritysel(match config.parity {
                Parity::ParityNone => usart::vals::Paritysel::NO_PARITY,
                Parity::ParityEven => usart::vals::Paritysel::EVEN_PARITY,
                Parity::ParityOdd => usart::vals::Paritysel::ODD_PARITY,
            });
            w.set_stoplen(match config.stop_bits {
                StopBits::Stop1 => usart::vals::Stoplen::BIT_1,
                StopBits::Stop2 => usart::vals::Stoplen::BITS_2,
            });
            w.set_rxpol(match config.invert_rx {
                false => usart::vals::Rxpol::STANDARD,
                true => usart::vals::Rxpol::INVERTED,
            });
            w.set_txpol(match config.invert_tx {
                false => usart::vals::Txpol::STANDARD,
                true => usart::vals::Txpol::INVERTED,
            });
        });

        // DMA-related settings
        registers.fifocfg().modify(|w| {
            w.set_dmatx(false);
            w.set_dmatx(false);
        });

        // Enabling USART
        registers.fifocfg().modify(|w| {
            w.set_enabletx(true);
            w.set_enablerx(true);
        });
        registers.cfg().modify(|w| w.set_enable(true));

        registers.fifointenset().modify(|w| w.set_rxerr(true));

        // Drain RX FIFO in case it still has some unrelevant data
        while registers.fifostat().read().rxnotempty() {
            let _ = registers.fiford().read().0;
        }
    }
}

impl<'d, M: Mode> Usart<'d, M> {
    /// Transmit the provided buffer blocking execution until done.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buffer)
    }

    /// Flush USART TX blocking execution until done.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.tx.blocking_flush()
    }

    /// Read from USART RX blocking execution until done.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }

    /// Check if UART is busy transmitting.
    pub fn tx_busy(&self) -> bool {
        self.tx.tx_busy()
    }

    /// Split the Usart into a transmitter and receiver, which is particularly
    /// useful when having two tasks correlating to transmitting and receiving.
    pub fn split(self) -> (UsartTx<'d, M>, UsartRx<'d, M>) {
        (self.tx, self.rx)
    }

    /// Split the Usart into a transmitter and receiver by mutable reference,
    /// which is particularly useful when having two tasks correlating to
    /// transmitting and receiving.
    pub fn split_ref(&mut self) -> (&mut UsartTx<'d, M>, &mut UsartRx<'d, M>) {
        (&mut self.tx, &mut self.rx)
    }
}

impl<'d> Usart<'d, Async> {
    /// Write to UART TX from the provided buffer.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write(buffer).await
    }

    /// Read from UART RX into the provided buffer.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.read(buffer).await
    }
}
impl<'d, M: Mode> embedded_hal_02::blocking::serial::Write<u8> for UsartTx<'d, M> {
    type Error = Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(buffer)
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::serial::Write<u8> for Usart<'d, M> {
    type Error = Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(buffer)
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d> embedded_io::ErrorType for UsartTx<'d, Blocking> {
    type Error = Error;
}

impl<'d> embedded_io::Write for UsartTx<'d, Blocking> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf).map(|_| buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d> embedded_io::ErrorType for UsartRx<'d, Blocking> {
    type Error = Error;
}

impl<'d> embedded_io::Read for UsartRx<'d, Blocking> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.blocking_read(buf).map(|_| buf.len())
    }
}

impl<'d> embedded_io::ErrorType for Usart<'d, Blocking> {
    type Error = Error;
}

impl<'d> embedded_io::Write for Usart<'d, Blocking> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf).map(|_| buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d> embedded_io::Read for Usart<'d, Blocking> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.blocking_read(buf).map(|_| buf.len())
    }
}

struct Info {
    usart_reg: UsartReg,
    fc_reg: FlexcommReg,
    interrupt: Interrupt,
}

trait SealedInstance {
    fn info() -> &'static Info;
    fn dma_state() -> &'static DmaState;
    fn instance_number() -> usize;
    fn tx_pin_func() -> PioFunc;
    fn rx_pin_func() -> PioFunc;
}

/// UART instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {
    /// Interrupt for this instance.
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}

macro_rules! impl_instance {
    ($inst:ident, $fc:ident, $tx_pin:ident, $rx_pin:ident, $fc_num:expr) => {
        impl $crate::usart::inner::SealedInstance for $crate::peripherals::$inst {
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    usart_reg: crate::pac::$inst,
                    fc_reg: crate::pac::$fc,
                    interrupt: crate::interrupt::typelevel::$fc::IRQ,
                };
                &INFO
            }

            fn dma_state() -> &'static DmaState {
                static STATE: DmaState = DmaState {
                    rx_err_waker: AtomicWaker::new(),
                    rx_err: AtomicBool::new(false),
                };
                &STATE
            }
            #[inline]
            fn instance_number() -> usize {
                $fc_num
            }
            #[inline]
            fn tx_pin_func() -> PioFunc {
                PioFunc::$tx_pin
            }
            #[inline]
            fn rx_pin_func() -> PioFunc {
                PioFunc::$rx_pin
            }
        }
        impl $crate::usart::Instance for $crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$fc;
        }
    };
}

impl_instance!(USART0, FLEXCOMM0, ALT1, ALT1, 0);
impl_instance!(USART1, FLEXCOMM1, ALT2, ALT2, 1);
impl_instance!(USART2, FLEXCOMM2, ALT1, ALT1, 2);
impl_instance!(USART3, FLEXCOMM3, ALT1, ALT1, 3);
impl_instance!(USART4, FLEXCOMM4, ALT1, ALT2, 4);
impl_instance!(USART5, FLEXCOMM5, ALT3, ALT3, 5);
impl_instance!(USART6, FLEXCOMM6, ALT2, ALT2, 6);
impl_instance!(USART7, FLEXCOMM7, ALT7, ALT7, 7);

/// Trait for TX pins.
pub trait TxPin<T: Instance>: crate::gpio::Pin {}
/// Trait for RX pins.
pub trait RxPin<T: Instance>: crate::gpio::Pin {}

macro_rules! impl_pin {
    ($pin:ident, $instance:ident, Tx) => {
        impl TxPin<crate::peripherals::$instance> for crate::peripherals::$pin {}
    };
    ($pin:ident, $instance:ident, Rx) => {
        impl RxPin<crate::peripherals::$instance> for crate::peripherals::$pin {}
    };
}

impl_pin!(PIO1_6, USART0, Tx);
impl_pin!(PIO1_5, USART0, Rx);
impl_pin!(PIO1_11, USART1, Tx);
impl_pin!(PIO1_10, USART1, Rx);
impl_pin!(PIO0_27, USART2, Tx);
impl_pin!(PIO1_24, USART2, Rx);
impl_pin!(PIO0_2, USART3, Tx);
impl_pin!(PIO0_3, USART3, Rx);
impl_pin!(PIO0_16, USART4, Tx);
impl_pin!(PIO0_5, USART4, Rx);
impl_pin!(PIO0_9, USART5, Tx);
impl_pin!(PIO0_8, USART5, Rx);
impl_pin!(PIO1_16, USART6, Tx);
impl_pin!(PIO1_13, USART6, Rx);
impl_pin!(PIO0_19, USART7, Tx);
impl_pin!(PIO0_20, USART7, Rx);

/// Trait for TX DMA channels.
pub trait TxChannel<T: Instance>: crate::dma::Channel {}
/// Trait for RX DMA channels.
pub trait RxChannel<T: Instance>: crate::dma::Channel {}

macro_rules! impl_channel {
    ($dma:ident, $instance:ident, Tx) => {
        impl TxChannel<crate::peripherals::$instance> for crate::peripherals::$dma {}
    };
    ($dma:ident, $instance:ident, Rx) => {
        impl RxChannel<crate::peripherals::$instance> for crate::peripherals::$dma {}
    };
}

impl_channel!(DMA_CH4, USART0, Rx);
impl_channel!(DMA_CH5, USART0, Tx);
impl_channel!(DMA_CH6, USART1, Rx);
impl_channel!(DMA_CH7, USART1, Tx);
impl_channel!(DMA_CH10, USART2, Rx);
impl_channel!(DMA_CH11, USART2, Tx);
impl_channel!(DMA_CH8, USART3, Rx);
impl_channel!(DMA_CH9, USART3, Tx);
impl_channel!(DMA_CH12, USART4, Rx);
impl_channel!(DMA_CH13, USART4, Tx);
impl_channel!(DMA_CH14, USART5, Rx);
impl_channel!(DMA_CH15, USART5, Tx);
impl_channel!(DMA_CH16, USART6, Rx);
impl_channel!(DMA_CH17, USART6, Tx);
impl_channel!(DMA_CH18, USART7, Rx);
impl_channel!(DMA_CH19, USART7, Tx);
