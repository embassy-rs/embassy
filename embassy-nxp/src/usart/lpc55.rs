use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use embedded_io::{self, ErrorKind};
pub use sealed::SealedInstance;

use crate::gpio::AnyPin;
use crate::{Blocking, Mode};

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

impl DataBits {
    fn bits(&self) -> u8 {
        match self {
            Self::DataBits7 => 0b00,
            Self::DataBits8 => 0b01,
            Self::DataBits9 => 0b10,
        }
    }
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

impl Parity {
    fn bits(&self) -> u8 {
        match self {
            Self::ParityNone => 0b00,
            Self::ParityEven => 0b10,
            Self::ParityOdd => 0b11,
        }
    }
}

/// Stop bits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StopBits {
    /// 1 stop bit.
    Stop1,
    /// 2 stop bits. This setting should only be used for asynchronous communication.
    Stop2,
}

impl StopBits {
    fn bits(&self) -> bool {
        return match self {
            Self::Stop1 => false,
            Self::Stop2 => true,
        };
    }
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
            baudrate: 9600,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::Stop1,
            parity: Parity::ParityNone,
            invert_rx: false,
            invert_tx: false,
        }
    }
}

/// # Type parameters
/// 'd: the lifetime marker ensuring correct borrow checking for peripherals used at compile time
/// T: the peripheral instance type allowing usage of peripheral specific registers
/// M: the operating mode of USART peripheral
pub struct Usart<'d, T: Instance, M: Mode> {
    tx: UsartTx<'d, T, M>,
    rx: UsartRx<'d, T, M>,
}

pub struct UsartTx<'d, T: Instance, M: Mode> {
    phantom: PhantomData<(&'d (), T, M)>,
}

pub struct UsartRx<'d, T: Instance, M: Mode> {
    phantom: PhantomData<(&'d (), T, M)>,
}

impl<'d, T: Instance, M: Mode> UsartTx<'d, T, M> {
    pub fn new(_usart: Peri<'d, T>, tx: Peri<'d, impl TxPin<T>>, config: Config) -> Self {
        Usart::<T, M>::init(Some(tx.into()), None, config);
        Self::new_inner()
    }

    #[inline]
    fn new_inner() -> Self {
        Self { phantom: PhantomData }
    }

    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        T::blocking_write(buffer)
    }

    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        T::blocking_flush()
    }

    pub fn tx_busy(&self) -> bool {
        T::tx_busy()
    }
}

impl<'d, T: Instance> UsartTx<'d, T, Blocking> {
    pub fn new_blocking(_usart: Peri<'d, T>, tx: Peri<'d, impl TxPin<T>>, config: Config) -> Self {
        Usart::<T, Blocking>::init(Some(tx.into()), None, config);
        Self::new_inner()
    }
}

impl<'d, T: Instance, M: Mode> UsartRx<'d, T, M> {
    pub fn new(_usart: Peri<'d, T>, rx: Peri<'d, impl RxPin<T>>, config: Config) -> Self {
        Usart::<T, M>::init(None, Some(rx.into()), config);
        Self::new_inner()
    }

    #[inline]
    fn new_inner() -> Self {
        Self { phantom: PhantomData }
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
        T::drain_fifo(buffer)
    }
}

impl<'d, T: Instance> UsartRx<'d, T, Blocking> {
    pub fn new_blocking(_usart: Peri<'d, T>, rx: Peri<'d, impl RxPin<T>>, config: Config) -> Self {
        Usart::<T, Blocking>::init(None, Some(rx.into()), config);
        Self::new_inner()
    }
}

impl<'d, T: Instance> Usart<'d, T, Blocking> {
    pub fn new_blocking(
        usart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(usart, tx.into(), rx.into(), config)
    }
}

impl<'d, T: Instance, M: Mode> Usart<'d, T, M> {
    fn new_inner(_usart: Peri<'d, T>, mut tx: Peri<'d, AnyPin>, mut rx: Peri<'d, AnyPin>, config: Config) -> Self {
        Self::init(Some(tx.reborrow()), Some(rx.reborrow()), config);
        Self {
            tx: UsartTx::new_inner(),
            rx: UsartRx::new_inner(),
        }
    }

    fn init(_tx: Option<Peri<'_, AnyPin>>, _rx: Option<Peri<'_, AnyPin>>, config: Config) {
        T::enable_clock();
        T::reset_flexcomm();
        let source_clock: u32 = T::select_clock(config.baudrate);
        T::configure_flexcomm();
        T::tx_pin_config();
        T::rx_pin_config();
        Self::set_baudrate(source_clock, config.baudrate);
        T::configure_usart(config);
        T::disable_dma();
        T::enable_usart();
    }

    fn set_baudrate(source_clock: u32, baudrate: u32) {
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

        let brg_value = (source_clock / (16 * baudrate)).min(255);
        let raw_clock = source_clock / (16 * brg_value);
        let mult_value = ((raw_clock * 256 / baudrate) - 256).min(255);
        T::set_baudrate(mult_value as u8, brg_value as u8);
    }
}

impl<'d, T: Instance, M: Mode> Usart<'d, T, M> {
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
    pub fn split(self) -> (UsartTx<'d, T, M>, UsartRx<'d, T, M>) {
        (self.tx, self.rx)
    }

    /// Split the Usart into a transmitter and receiver by mutable reference,
    /// which is particularly useful when having two tasks correlating to
    /// transmitting and receiving.
    pub fn split_ref(&mut self) -> (&mut UsartTx<'d, T, M>, &mut UsartRx<'d, T, M>) {
        (&mut self.tx, &mut self.rx)
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::serial::Write<u8> for UsartTx<'d, T, M> {
    type Error = Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(buffer)
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::serial::Write<u8> for Usart<'d, T, M> {
    type Error = Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(buffer)
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d, T: Instance> embedded_io::ErrorType for UsartTx<'d, T, Blocking> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_io::Write for UsartTx<'d, T, Blocking> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf).map(|_| buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d, T: Instance> embedded_io::ErrorType for UsartRx<'d, T, Blocking> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_io::Read for UsartRx<'d, T, Blocking> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.blocking_read(buf).map(|_| buf.len())
    }
}

impl<'d, T: Instance> embedded_io::ErrorType for Usart<'d, T, Blocking> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_io::Write for Usart<'d, T, Blocking> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf).map(|_| buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d, T: Instance> embedded_io::Read for Usart<'d, T, Blocking> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.blocking_read(buf).map(|_| buf.len())
    }
}

type UsartRegBlock = crate::pac::usart0::RegisterBlock;

mod sealed {
    use crate::usart::inner::UsartRegBlock;
    use crate::usart::{Config, Error};
    pub trait SealedInstance {
        fn usart_reg() -> &'static UsartRegBlock;
        fn enable_clock();
        fn select_clock(baudrate: u32) -> u32;
        fn configure_flexcomm();
        fn set_baudrate(mult_value: u8, brg_value: u8);
        fn reset_flexcomm();
        fn tx_pin_config();
        fn rx_pin_config();

        fn configure_usart(config: Config) {
            // See section 34.6.1
            Self::usart_reg().cfg.modify(|_, w| {
                // LIN break mode enable
                w.linmode()
                    // Disabled. Break detect and generate is configured for normal operation.
                    .disabled()
                    //CTS Enable. Determines whether CTS is used for flow control. CTS can be from the
                    //input pin, or from the USART’s own RTS if loopback mode is enabled.
                    .ctsen()
                    // No flow control. The transmitter does not receive any automatic flow control signal.
                    .disabled()
                    // Selects synchronous or asynchronous operation.
                    .syncen()
                    .asynchronous_mode()
                    // Selects the clock polarity and sampling edge of received data in synchronous mode.
                    .clkpol()
                    .rising_edge()
                    // Synchronous mode Master select.
                    .syncmst()
                    // When synchronous mode is enabled, the USART is a master.
                    .master()
                    // Selects data loopback mode
                    .loop_()
                    // Normal operation
                    .normal()
                    // Output Enable Turnaround time enable for RS-485 operation.
                    .oeta()
                    // Disabled. If selected by OESEL, the Output Enable signal deasserted at the end of
                    // the last stop bit of a transmission.
                    .disabled()
                    // Output enable select.
                    .oesel()
                    // Standard. The RTS signal is used as the standard flow control function.
                    .standard()
                    // Automatic address matching enable.
                    .autoaddr()
                    // Disabled. When addressing is enabled by ADDRDET, address matching is done by
                    // software. This provides the possibility of versatile addressing (e.g. respond to more
                    // than one address)
                    .disabled()
                    // Output enable polarity.
                    .oepol()
                    // Low. If selected by OESEL, the output enable is active low.
                    .low()
            });

            Self::usart_reg().cfg.modify(|_, w| unsafe {
                w.datalen()
                    .bits(config.data_bits.bits())
                    .paritysel()
                    .bits(config.parity.bits())
                    .stoplen()
                    .bit(config.stop_bits.bits())
                    .rxpol()
                    .bit(config.invert_rx)
                    .txpol()
                    .bit(config.invert_tx)
            });
        }
        fn disable_dma() {
            Self::usart_reg()
                .fifocfg
                .modify(|_, w| w.dmatx().disabled().dmarx().disabled());
        }
        fn enable_usart() {
            Self::usart_reg()
                .fifocfg
                .modify(|_, w| w.enabletx().enabled().enablerx().enabled());
            Self::usart_reg().cfg.modify(|_, w| w.enable().enabled());
            while Self::usart_reg().fifostat.read().rxnotempty().bit_is_set() {
                let _ = Self::usart_reg().fiford.read().bits();
            }
        }
        fn blocking_write(buffer: &[u8]) -> Result<(), Error> {
            for &b in buffer {
                while Self::usart_reg().fifostat.read().txnotfull().bit_is_clear() {}
                Self::usart_reg()
                    .fifowr
                    .modify(|_, w| unsafe { w.txdata().bits(b as u16) });
            }
            Ok(())
        }
        fn blocking_flush() -> Result<(), Error> {
            while Self::usart_reg().fifostat.read().txempty().bit_is_clear() {}
            Ok(())
        }
        fn tx_busy() -> bool {
            Self::usart_reg().fifostat.read().txempty().bit_is_clear()
        }
        fn drain_fifo(buffer: &mut [u8]) -> Result<usize, (usize, Error)> {
            for (i, b) in buffer.iter_mut().enumerate() {
                while Self::usart_reg().fifostat.read().rxnotempty().bit_is_clear() {}

                if Self::usart_reg().fifostat.read().rxerr().bit_is_set() {
                    return Err((i, Error::Overrun));
                } else if Self::usart_reg().fifordnopop.read().parityerr().bit_is_set() {
                    return Err((i, Error::Parity));
                } else if Self::usart_reg().fifordnopop.read().framerr().bit_is_set() {
                    return Err((i, Error::Framing));
                } else if Self::usart_reg().fifordnopop.read().rxnoise().bit_is_set() {
                    return Err((i, Error::Noise));
                } else if Self::usart_reg().intstat.read().deltarxbrk().bit_is_set() {
                    return Err((i, Error::Break));
                }
                let dr = Self::usart_reg().fiford.read().bits() as u8;
                *b = dr;
            }
            Ok(buffer.len())
        }
    }
}

/// UART instance.
#[allow(private_bounds)]
pub trait Instance: sealed::SealedInstance + PeripheralType {}

#[macro_export]
macro_rules! impl_instance {
    (
        $inst:ident,
        usart_peripheral: $USARTX:ident,
        usart_crate: $usartX:ident,

        flexcomm: {
          field: $FLEXCOMM_FIELD:ident,
          clock_field: $FLEXCOMM_CLK_FIELD:ident
        },

        reset: {
            bit: $RESET_BIT:ident
        },

        clock: {
            sel_field: $CLKSEL_FIELD:ident,
            frg_field: $FRG_FIELD:ident
        },

        pins: {
            tx: $TX_IOCON:ident => $TX_FUNC:expr,
            rx: $RX_IOCON:ident => $RX_FUNC:expr
        }

    ) => {
        impl $crate::usart::SealedInstance for $crate::peripherals::$inst {
            fn usart_reg() -> &'static UsartRegBlock {
                unsafe { &*$crate::pac::$USARTX::ptr() }
            }

            fn enable_clock() {
                critical_section::with(|_cs| {
                    if syscon_reg().ahbclkctrl0.read().iocon().is_disable() {
                        syscon_reg().ahbclkctrl0.modify(|_, w| w.iocon().enable());
                    }
                    if syscon_reg().ahbclkctrl1.read().$FLEXCOMM_CLK_FIELD().is_disable() {
                        syscon_reg()
                            .ahbclkctrl1
                            .modify(|_, w| w.$FLEXCOMM_CLK_FIELD().enable());
                    }
                });
            }

            fn configure_flexcomm() {
                let flexcomm = unsafe { &*$crate::pac::$FLEXCOMM_FIELD::ptr() };
                flexcomm.pselid.modify(|_, w| w.persel().usart());
            }

            fn reset_flexcomm() {
                syscon_reg().presetctrl1.modify(|_, w| w.$RESET_BIT().set_bit());
                syscon_reg().presetctrl1.modify(|_, w| w.$RESET_BIT().clear_bit());
            }

            fn select_clock(baudrate: u32) -> u32 {
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
                return match baudrate {
                    750_001..=6000000 => {
                        syscon_reg().$CLKSEL_FIELD().write(|w| w.sel().enum_0x3()); // 96 MHz
                        96_000_000
                    }
                    1501..=750_000 => {
                        syscon_reg().$CLKSEL_FIELD().write(|w| w.sel().enum_0x2()); // 12 MHz
                        12_000_000
                    }
                    121..=1500 => {
                        syscon_reg().$CLKSEL_FIELD().write(|w| w.sel().enum_0x4()); // 1 MHz
                        1_000_000
                    }
                    _ => {
                        panic!("{} baudrate is not permitted in this mode", baudrate);
                    }
                };
            }

            fn set_baudrate(mult_value: u8, brg_value: u8) {
                // FCLK =  (clock selected via FCCLKSEL) / (1+ MULT / DIV)
                // Remark: To use the fractional baud rate generator, 0xFF must be wirtten to the DIV value
                // to yield a denominator vale of 256. All other values are not supported
                syscon_reg()
                    .$FRG_FIELD()
                    .modify(|_, w| unsafe { w.div().bits(0xFF).mult().bits(mult_value as u8) });

                // Baud rate = [FCLK / (OSRVAL+1)] / (BRGVAL + 1)
                // By default, oversampling is 16x, i.e. OSRVAL = 15

                // Typical industry standard USARTs use a 16x oversample clock to transmit and receive
                // asynchronous data. This is the number of BRG clocks used for one data bit. The
                // Oversample Select Register (OSR) allows this USART to use a 16x down to a 5x
                // oversample clock. There is no oversampling in synchronous modes.
                Self::usart_reg()
                    .brg
                    .modify(|_, w| unsafe { w.brgval().bits((brg_value - 1) as u16) });
            }

            fn tx_pin_config() {
                iocon_reg().$TX_IOCON.modify(|_, w| unsafe {
                    w.func()
                        .bits($TX_FUNC)
                        .digimode()
                        .digital()
                        .slew()
                        .standard()
                        .mode()
                        .inactive()
                        .invert()
                        .disabled()
                        .od()
                        .normal()
                });
            }

            fn rx_pin_config() {
                iocon_reg().$RX_IOCON.modify(|_, w| unsafe {
                    w.func()
                        .bits($RX_FUNC)
                        .digimode()
                        .digital()
                        .slew()
                        .standard()
                        .mode()
                        .inactive()
                        .invert()
                        .disabled()
                        .od()
                        .normal()
                });
            }
        }

        impl $crate::usart::Instance for $crate::peripherals::$inst {}
    };
}

impl_instance!(USART0, usart_peripheral: USART0, usart_crate: usart0,
    flexcomm: {
        field: FLEXCOMM0,
        clock_field: fc0
    },

    reset: {
        bit: fc0_rst
    },

    clock: {
        sel_field: fcclksel0,
        frg_field: flexfrg0ctrl
    },

    pins: {
        tx: pio1_6 => 1,
        rx: pio1_5 => 1
    }
);

impl_instance!(USART1, usart_peripheral: USART1, usart_crate: usart1,
    flexcomm: {
        field: FLEXCOMM1,
        clock_field: fc1
    },

    reset: {
        bit: fc1_rst
    },

    clock: {
        sel_field: fcclksel1,
        frg_field: flexfrg1ctrl
    },

    pins: {
        tx: pio1_11 => 2,
        rx: pio1_10 => 2
    }
);

impl_instance!(USART2, usart_peripheral: USART2, usart_crate: usart2,
    flexcomm: {
        field: FLEXCOMM2,
        clock_field: fc2
    },

    reset: {
        bit: fc2_rst
    },

    clock: {
        sel_field: fcclksel2,
        frg_field: flexfrg2ctrl
    },

    pins: {
        tx: pio0_27 => 1,
        rx: pio1_24 => 1
    }
);

impl_instance!(USART3, usart_peripheral: USART3, usart_crate: usart3,
    flexcomm: {
        field: FLEXCOMM3,
        clock_field: fc3
    },

    reset: {
        bit: fc3_rst
    },

    clock: {
        sel_field: fcclksel3,
        frg_field: flexfrg3ctrl
    },

    pins: {
        tx: pio0_2 => 1,
        rx: pio0_3 => 1
    }
);

impl_instance!(USART4, usart_peripheral: USART4, usart_crate: usart4,
    flexcomm: {
        field: FLEXCOMM4,
        clock_field: fc4
    },

    reset: {
        bit: fc4_rst
    },

    clock: {
        sel_field: fcclksel4,
        frg_field: flexfrg4ctrl
    },

    pins: {
        tx: pio0_16 => 1,
        rx: pio0_5 => 2
    }
);

impl_instance!(USART5, usart_peripheral: USART5, usart_crate: usart5,
    flexcomm: {
        field: FLEXCOMM5,
        clock_field: fc5
    },

    reset: {
        bit: fc5_rst
    },

    clock: {
        sel_field: fcclksel5,
        frg_field: flexfrg5ctrl
    },

    pins: {
        tx: pio0_9 => 3,
        rx: pio0_8 => 3
    }
);

impl_instance!(USART6, usart_peripheral: USART6, usart_crate: usart6,
    flexcomm: {
        field: FLEXCOMM6,
        clock_field: fc6
    },

    reset: {
        bit: fc6_rst
    },

    clock: {
        sel_field: fcclksel6,
        frg_field: flexfrg6ctrl
    },

    pins: {
        tx: pio1_16 => 2,
        rx: pio1_13 => 2
    }
);

impl_instance!(USART7, usart_peripheral: USART7, usart_crate: usart7,
    flexcomm: {
        field: FLEXCOMM7,
        clock_field: fc7
    },

    reset: {
        bit: fc7_rst
    },

    clock: {
        sel_field: fcclksel7,
        frg_field: flexfrg7ctrl
    },

    pins: {
        tx: pio0_19 => 7,
        rx: pio0_20 => 7
    }
);

/// Trait for TX pins.
pub trait TxPin<T: Instance>: crate::gpio::Pin {}
/// Trait for RX pins.
pub trait RxPin<T: Instance>: crate::gpio::Pin {}

// TODO: Add RTS, CTS and CLK pin traits

macro_rules! impl_pin {
    ($pin:ident, $instance:ident, Tx) => {
        impl TxPin<crate::peripherals::$instance> for crate::peripherals::$pin {}
    };
    ($pin:ident, $instance:ident, Rx) => {
        impl RxPin<crate::peripherals::$instance> for crate::peripherals::$pin {}
    };
}

impl_pin!(PIO1_5, USART0, Rx);
impl_pin!(PIO1_6, USART0, Tx);
impl_pin!(PIO1_10, USART1, Rx);
impl_pin!(PIO1_11, USART1, Tx);
impl_pin!(PIO0_27, USART2, Tx);
impl_pin!(PIO1_24, USART2, Rx);
impl_pin!(PIO0_2, USART3, Tx);
impl_pin!(PIO0_3, USART3, Rx);
impl_pin!(PIO0_16, USART4, Tx);
impl_pin!(PIO0_5, USART4, Rx);
impl_pin!(PIO0_8, USART5, Rx);
impl_pin!(PIO0_9, USART5, Tx);
impl_pin!(PIO1_16, USART6, Tx);
impl_pin!(PIO1_13, USART6, Rx);
impl_pin!(PIO0_20, USART7, Rx);
impl_pin!(PIO0_19, USART7, Tx);

/// Get the SYSCON register block.
///
/// # Safety
/// Read/Write operations on a single registers are NOT atomic. You must ensure that the GPIO
/// registers are not accessed concurrently by multiple threads.
pub(crate) fn syscon_reg() -> &'static crate::pac::syscon::RegisterBlock {
    unsafe { &*crate::pac::SYSCON::ptr() }
}

/// Get the IOCON register block.
///
/// # Safety
/// Read/Write operations on a single registers are NOT atomic. You must ensure that the GPIO
/// registers are not accessed concurrently by multiple threads.
pub(crate) fn iocon_reg() -> &'static crate::pac::iocon::RegisterBlock {
    unsafe { &*crate::pac::IOCON::ptr() }
}
