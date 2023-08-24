use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use atomic_polyfill::{AtomicU16, Ordering};
use embassy_futures::select::{select, Either};
use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use embassy_time::{Duration, Timer};
use pac::uart::regs::Uartris;

use crate::clocks::clk_peri_freq;
use crate::dma::{AnyChannel, Channel};
use crate::gpio::sealed::Pin;
use crate::gpio::AnyPin;
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::pac::io::vals::{Inover, Outover};
use crate::{interrupt, pac, peripherals, Peripheral, RegExt};

#[cfg(feature = "nightly")]
mod buffered;
#[cfg(feature = "nightly")]
pub use buffered::{BufferedInterruptHandler, BufferedUart, BufferedUartRx, BufferedUartTx};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DataBits {
    DataBits5,
    DataBits6,
    DataBits7,
    DataBits8,
}

impl DataBits {
    fn bits(&self) -> u8 {
        match self {
            Self::DataBits5 => 0b00,
            Self::DataBits6 => 0b01,
            Self::DataBits7 => 0b10,
            Self::DataBits8 => 0b11,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Parity {
    ParityNone,
    ParityEven,
    ParityOdd,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StopBits {
    #[doc = "1 stop bit"]
    STOP1,
    #[doc = "2 stop bits"]
    STOP2,
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Config {
    pub baudrate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
    /// Invert the tx pin output
    pub invert_tx: bool,
    /// Invert the rx pin input
    pub invert_rx: bool,
    // Invert the rts pin
    pub invert_rts: bool,
    // Invert the cts pin
    pub invert_cts: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            baudrate: 115200,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::STOP1,
            parity: Parity::ParityNone,
            invert_rx: false,
            invert_tx: false,
            invert_rts: false,
            invert_cts: false,
        }
    }
}

/// Serial error
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Triggered when the FIFO (or shift-register) is overflowed.
    Overrun,
    /// Triggered when a break is received
    Break,
    /// Triggered when there is a parity mismatch between what's received and
    /// our settings.
    Parity,
    /// Triggered when the received character didn't have a valid stop bit.
    Framing,
}

pub struct DmaState {
    rx_err_waker: AtomicWaker,
    rx_errs: AtomicU16,
}

pub struct Uart<'d, T: Instance, M: Mode> {
    tx: UartTx<'d, T, M>,
    rx: UartRx<'d, T, M>,
}

pub struct UartTx<'d, T: Instance, M: Mode> {
    tx_dma: Option<PeripheralRef<'d, AnyChannel>>,
    phantom: PhantomData<(&'d mut T, M)>,
}

pub struct UartRx<'d, T: Instance, M: Mode> {
    rx_dma: Option<PeripheralRef<'d, AnyChannel>>,
    phantom: PhantomData<(&'d mut T, M)>,
}

impl<'d, T: Instance, M: Mode> UartTx<'d, T, M> {
    /// Create a new DMA-enabled UART which can only send data
    pub fn new(
        _uart: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        tx_dma: impl Peripheral<P = impl Channel> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(tx, tx_dma);
        Uart::<T, M>::init(Some(tx.map_into()), None, None, None, config);
        Self::new_inner(Some(tx_dma.map_into()))
    }

    fn new_inner(tx_dma: Option<PeripheralRef<'d, AnyChannel>>) -> Self {
        Self {
            tx_dma,
            phantom: PhantomData,
        }
    }

    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = T::regs();
        for &b in buffer {
            while r.uartfr().read().txff() {}
            r.uartdr().write(|w| w.set_data(b));
        }
        Ok(())
    }

    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        let r = T::regs();
        while !r.uartfr().read().txfe() {}
        Ok(())
    }

    pub fn busy(&self) -> bool {
        T::regs().uartfr().read().busy()
    }

    /// Assert a break condition after waiting for the transmit buffers to empty,
    /// for the specified number of bit times. This condition must be asserted
    /// for at least two frame times to be effective, `bits` will adjusted
    /// according to frame size, parity, and stop bit settings to ensure this.
    ///
    /// This method may block for a long amount of time since it has to wait
    /// for the transmit fifo to empty, which may take a while on slow links.
    pub async fn send_break(&mut self, bits: u32) {
        let regs = T::regs();
        let bits = bits.max({
            let lcr = regs.uartlcr_h().read();
            let width = lcr.wlen() as u32 + 5;
            let parity = lcr.pen() as u32;
            let stops = 1 + lcr.stp2() as u32;
            2 * (1 + width + parity + stops)
        });
        let divx64 = (((regs.uartibrd().read().baud_divint() as u32) << 6)
            + regs.uartfbrd().read().baud_divfrac() as u32) as u64;
        let div_clk = clk_peri_freq() as u64 * 64;
        let wait_usecs = (1_000_000 * bits as u64 * divx64 * 16 + div_clk - 1) / div_clk;

        self.blocking_flush().unwrap();
        while self.busy() {}
        regs.uartlcr_h().write_set(|w| w.set_brk(true));
        Timer::after(Duration::from_micros(wait_usecs)).await;
        regs.uartlcr_h().write_clear(|w| w.set_brk(true));
    }
}

impl<'d, T: Instance> UartTx<'d, T, Blocking> {
    #[cfg(feature = "nightly")]
    pub fn into_buffered(
        self,
        irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        tx_buffer: &'d mut [u8],
    ) -> BufferedUartTx<'d, T> {
        buffered::init_buffers::<T>(irq, tx_buffer, &mut []);

        BufferedUartTx { phantom: PhantomData }
    }
}

impl<'d, T: Instance> UartTx<'d, T, Async> {
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let ch = self.tx_dma.as_mut().unwrap();
        let transfer = unsafe {
            T::regs().uartdmacr().write_set(|reg| {
                reg.set_txdmae(true);
            });
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            crate::dma::write(ch, buffer, T::regs().uartdr().as_ptr() as *mut _, T::TX_DREQ)
        };
        transfer.await;
        Ok(())
    }
}

impl<'d, T: Instance, M: Mode> UartRx<'d, T, M> {
    /// Create a new DMA-enabled UART which can only receive data
    pub fn new(
        _uart: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        rx_dma: impl Peripheral<P = impl Channel> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(rx, rx_dma);
        Uart::<T, M>::init(None, Some(rx.map_into()), None, None, config);
        Self::new_inner(true, Some(rx_dma.map_into()))
    }

    fn new_inner(has_irq: bool, rx_dma: Option<PeripheralRef<'d, AnyChannel>>) -> Self {
        debug_assert_eq!(has_irq, rx_dma.is_some());
        if has_irq {
            // disable all error interrupts initially
            T::regs().uartimsc().write(|w| w.0 = 0);
            T::Interrupt::unpend();
            unsafe { T::Interrupt::enable() };
        }
        Self {
            rx_dma,
            phantom: PhantomData,
        }
    }

    pub fn blocking_read(&mut self, mut buffer: &mut [u8]) -> Result<(), Error> {
        while buffer.len() > 0 {
            let received = self.drain_fifo(buffer)?;
            buffer = &mut buffer[received..];
        }
        Ok(())
    }

    fn drain_fifo(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        let r = T::regs();
        for (i, b) in buffer.iter_mut().enumerate() {
            if r.uartfr().read().rxfe() {
                return Ok(i);
            }

            let dr = r.uartdr().read();

            if dr.oe() {
                return Err(Error::Overrun);
            } else if dr.be() {
                return Err(Error::Break);
            } else if dr.pe() {
                return Err(Error::Parity);
            } else if dr.fe() {
                return Err(Error::Framing);
            } else {
                *b = dr.data();
            }
        }
        Ok(buffer.len())
    }
}

impl<'d, T: Instance, M: Mode> Drop for UartRx<'d, T, M> {
    fn drop(&mut self) {
        if let Some(_) = self.rx_dma {
            T::Interrupt::disable();
            // clear dma flags. irq handlers use these to disambiguate among themselves.
            T::regs().uartdmacr().write_clear(|reg| {
                reg.set_rxdmae(true);
                reg.set_txdmae(true);
                reg.set_dmaonerr(true);
            });
        }
    }
}

impl<'d, T: Instance> UartRx<'d, T, Blocking> {
    pub fn new_blocking(
        _uart: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(rx);
        Uart::<T, Blocking>::init(None, Some(rx.map_into()), None, None, config);
        Self::new_inner(false, None)
    }

    #[cfg(feature = "nightly")]
    pub fn into_buffered(
        self,
        irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        rx_buffer: &'d mut [u8],
    ) -> BufferedUartRx<'d, T> {
        buffered::init_buffers::<T>(irq, &mut [], rx_buffer);

        BufferedUartRx { phantom: PhantomData }
    }
}

pub struct InterruptHandler<T: Instance> {
    _uart: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let uart = T::regs();
        if !uart.uartdmacr().read().rxdmae() {
            return;
        }

        let state = T::dma_state();
        let errs = uart.uartris().read();
        state.rx_errs.store(errs.0 as u16, Ordering::Relaxed);
        state.rx_err_waker.wake();
        // disable the error interrupts instead of clearing the flags. clearing the
        // flags would allow the dma transfer to continue, potentially signaling
        // completion before we can check for errors that happened *during* the transfer.
        uart.uartimsc().write_clear(|w| w.0 = errs.0);
    }
}

impl<'d, T: Instance> UartRx<'d, T, Async> {
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        // clear error flags before we drain the fifo. errors that have accumulated
        // in the flags will also be present in the fifo.
        T::dma_state().rx_errs.store(0, Ordering::Relaxed);
        T::regs().uarticr().write(|w| {
            w.set_oeic(true);
            w.set_beic(true);
            w.set_peic(true);
            w.set_feic(true);
        });

        // then drain the fifo. we need to read at most 32 bytes. errors that apply
        // to fifo bytes will be reported directly.
        let buffer = match {
            let limit = buffer.len().min(32);
            self.drain_fifo(&mut buffer[0..limit])
        } {
            Ok(len) if len < buffer.len() => &mut buffer[len..],
            Ok(_) => return Ok(()),
            Err(e) => return Err(e),
        };

        // start a dma transfer. if errors have happened in the interim some error
        // interrupt flags will have been raised, and those will be picked up immediately
        // by the interrupt handler.
        let ch = self.rx_dma.as_mut().unwrap();
        T::regs().uartimsc().write_set(|w| {
            w.set_oeim(true);
            w.set_beim(true);
            w.set_peim(true);
            w.set_feim(true);
        });
        T::regs().uartdmacr().write_set(|reg| {
            reg.set_rxdmae(true);
            reg.set_dmaonerr(true);
        });
        let transfer = unsafe {
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            crate::dma::read(ch, T::regs().uartdr().as_ptr() as *const _, buffer, T::RX_DREQ)
        };

        // wait for either the transfer to complete or an error to happen.
        let transfer_result = select(
            transfer,
            poll_fn(|cx| {
                T::dma_state().rx_err_waker.register(cx.waker());
                match T::dma_state().rx_errs.swap(0, Ordering::Relaxed) {
                    0 => Poll::Pending,
                    e => Poll::Ready(Uartris(e as u32)),
                }
            }),
        )
        .await;

        let errors = match transfer_result {
            Either::First(()) => return Ok(()),
            Either::Second(e) => e,
        };

        if errors.0 == 0 {
            return Ok(());
        } else if errors.oeris() {
            return Err(Error::Overrun);
        } else if errors.beris() {
            return Err(Error::Break);
        } else if errors.peris() {
            return Err(Error::Parity);
        } else if errors.feris() {
            return Err(Error::Framing);
        }
        unreachable!("unrecognized rx error");
    }
}

impl<'d, T: Instance> Uart<'d, T, Blocking> {
    /// Create a new UART without hardware flow control
    pub fn new_blocking(
        uart: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(tx, rx);
        Self::new_inner(
            uart,
            tx.map_into(),
            rx.map_into(),
            None,
            None,
            false,
            None,
            None,
            config,
        )
    }

    /// Create a new UART with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts_blocking(
        uart: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(tx, rx, cts, rts);
        Self::new_inner(
            uart,
            tx.map_into(),
            rx.map_into(),
            Some(rts.map_into()),
            Some(cts.map_into()),
            false,
            None,
            None,
            config,
        )
    }

    #[cfg(feature = "nightly")]
    pub fn into_buffered(
        self,
        irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
    ) -> BufferedUart<'d, T> {
        buffered::init_buffers::<T>(irq, tx_buffer, rx_buffer);

        BufferedUart {
            rx: BufferedUartRx { phantom: PhantomData },
            tx: BufferedUartTx { phantom: PhantomData },
        }
    }
}

impl<'d, T: Instance> Uart<'d, T, Async> {
    /// Create a new DMA enabled UART without hardware flow control
    pub fn new(
        uart: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        tx_dma: impl Peripheral<P = impl Channel> + 'd,
        rx_dma: impl Peripheral<P = impl Channel> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(tx, rx, tx_dma, rx_dma);
        Self::new_inner(
            uart,
            tx.map_into(),
            rx.map_into(),
            None,
            None,
            true,
            Some(tx_dma.map_into()),
            Some(rx_dma.map_into()),
            config,
        )
    }

    /// Create a new DMA enabled UART with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts(
        uart: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<T>> + 'd,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        tx_dma: impl Peripheral<P = impl Channel> + 'd,
        rx_dma: impl Peripheral<P = impl Channel> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(tx, rx, cts, rts, tx_dma, rx_dma);
        Self::new_inner(
            uart,
            tx.map_into(),
            rx.map_into(),
            Some(rts.map_into()),
            Some(cts.map_into()),
            true,
            Some(tx_dma.map_into()),
            Some(rx_dma.map_into()),
            config,
        )
    }
}

impl<'d, T: Instance + 'd, M: Mode> Uart<'d, T, M> {
    fn new_inner(
        _uart: impl Peripheral<P = T> + 'd,
        mut tx: PeripheralRef<'d, AnyPin>,
        mut rx: PeripheralRef<'d, AnyPin>,
        mut rts: Option<PeripheralRef<'d, AnyPin>>,
        mut cts: Option<PeripheralRef<'d, AnyPin>>,
        has_irq: bool,
        tx_dma: Option<PeripheralRef<'d, AnyChannel>>,
        rx_dma: Option<PeripheralRef<'d, AnyChannel>>,
        config: Config,
    ) -> Self {
        Self::init(
            Some(tx.reborrow()),
            Some(rx.reborrow()),
            rts.as_mut().map(|x| x.reborrow()),
            cts.as_mut().map(|x| x.reborrow()),
            config,
        );

        Self {
            tx: UartTx::new_inner(tx_dma),
            rx: UartRx::new_inner(has_irq, rx_dma),
        }
    }

    fn init(
        tx: Option<PeripheralRef<'_, AnyPin>>,
        rx: Option<PeripheralRef<'_, AnyPin>>,
        rts: Option<PeripheralRef<'_, AnyPin>>,
        cts: Option<PeripheralRef<'_, AnyPin>>,
        config: Config,
    ) {
        let r = T::regs();
        if let Some(pin) = &tx {
            pin.gpio().ctrl().write(|w| {
                w.set_funcsel(2);
                w.set_outover(if config.invert_tx {
                    Outover::INVERT
                } else {
                    Outover::NORMAL
                });
            });
            pin.pad_ctrl().write(|w| w.set_ie(true));
        }
        if let Some(pin) = &rx {
            pin.gpio().ctrl().write(|w| {
                w.set_funcsel(2);
                w.set_inover(if config.invert_rx {
                    Inover::INVERT
                } else {
                    Inover::NORMAL
                });
            });
            pin.pad_ctrl().write(|w| w.set_ie(true));
        }
        if let Some(pin) = &cts {
            pin.gpio().ctrl().write(|w| {
                w.set_funcsel(2);
                w.set_inover(if config.invert_cts {
                    Inover::INVERT
                } else {
                    Inover::NORMAL
                });
            });
            pin.pad_ctrl().write(|w| w.set_ie(true));
        }
        if let Some(pin) = &rts {
            pin.gpio().ctrl().write(|w| {
                w.set_funcsel(2);
                w.set_outover(if config.invert_rts {
                    Outover::INVERT
                } else {
                    Outover::NORMAL
                });
            });
            pin.pad_ctrl().write(|w| w.set_ie(true));
        }

        Self::set_baudrate_inner(config.baudrate);

        let (pen, eps) = match config.parity {
            Parity::ParityNone => (false, false),
            Parity::ParityOdd => (true, false),
            Parity::ParityEven => (true, true),
        };

        r.uartlcr_h().write(|w| {
            w.set_wlen(config.data_bits.bits());
            w.set_stp2(config.stop_bits == StopBits::STOP2);
            w.set_pen(pen);
            w.set_eps(eps);
            w.set_fen(true);
        });

        r.uartifls().write(|w| {
            w.set_rxiflsel(0b000);
            w.set_txiflsel(0b000);
        });

        r.uartcr().write(|w| {
            w.set_uarten(true);
            w.set_rxe(true);
            w.set_txe(true);
            w.set_ctsen(cts.is_some());
            w.set_rtsen(rts.is_some());
        });
    }

    /// sets baudrate on runtime    
    pub fn set_baudrate(&mut self, baudrate: u32) {
        Self::set_baudrate_inner(baudrate);
    }

    fn set_baudrate_inner(baudrate: u32) {
        let r = T::regs();

        let clk_base = crate::clocks::clk_peri_freq();

        let baud_rate_div = (8 * clk_base) / baudrate;
        let mut baud_ibrd = baud_rate_div >> 7;
        let mut baud_fbrd = ((baud_rate_div & 0x7f) + 1) / 2;

        if baud_ibrd == 0 {
            baud_ibrd = 1;
            baud_fbrd = 0;
        } else if baud_ibrd >= 65535 {
            baud_ibrd = 65535;
            baud_fbrd = 0;
        }

        // Load PL011's baud divisor registers
        r.uartibrd().write_value(pac::uart::regs::Uartibrd(baud_ibrd));
        r.uartfbrd().write_value(pac::uart::regs::Uartfbrd(baud_fbrd));

        // PL011 needs a (dummy) line control register write to latch in the
        // divisors. We don't want to actually change LCR contents here.
        r.uartlcr_h().modify(|_| {});
    }
}

impl<'d, T: Instance, M: Mode> Uart<'d, T, M> {
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buffer)
    }

    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.tx.blocking_flush()
    }

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }

    pub fn busy(&self) -> bool {
        self.tx.busy()
    }

    pub async fn send_break(&mut self, bits: u32) {
        self.tx.send_break(bits).await
    }

    /// Split the Uart into a transmitter and receiver, which is particularly
    /// useful when having two tasks correlating to transmitting and receiving.
    pub fn split(self) -> (UartTx<'d, T, M>, UartRx<'d, T, M>) {
        (self.tx, self.rx)
    }
}

impl<'d, T: Instance> Uart<'d, T, Async> {
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write(buffer).await
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.read(buffer).await
    }
}

mod eh02 {
    use super::*;

    impl<'d, T: Instance, M: Mode> embedded_hal_02::serial::Read<u8> for UartRx<'d, T, M> {
        type Error = Error;
        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            let r = T::regs();
            if r.uartfr().read().rxfe() {
                return Err(nb::Error::WouldBlock);
            }

            let dr = r.uartdr().read();

            if dr.oe() {
                Err(nb::Error::Other(Error::Overrun))
            } else if dr.be() {
                Err(nb::Error::Other(Error::Break))
            } else if dr.pe() {
                Err(nb::Error::Other(Error::Parity))
            } else if dr.fe() {
                Err(nb::Error::Other(Error::Framing))
            } else {
                Ok(dr.data())
            }
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_02::serial::Write<u8> for UartTx<'d, T, M> {
        type Error = Error;

        fn write(&mut self, word: u8) -> Result<(), nb::Error<Self::Error>> {
            let r = T::regs();
            if r.uartfr().read().txff() {
                return Err(nb::Error::WouldBlock);
            }

            r.uartdr().write(|w| w.set_data(word));
            Ok(())
        }

        fn flush(&mut self) -> Result<(), nb::Error<Self::Error>> {
            let r = T::regs();
            if !r.uartfr().read().txfe() {
                return Err(nb::Error::WouldBlock);
            }
            Ok(())
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::serial::Write<u8> for UartTx<'d, T, M> {
        type Error = Error;

        fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }

        fn bflush(&mut self) -> Result<(), Self::Error> {
            self.blocking_flush()
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_02::serial::Read<u8> for Uart<'d, T, M> {
        type Error = Error;

        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            embedded_hal_02::serial::Read::read(&mut self.rx)
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_02::serial::Write<u8> for Uart<'d, T, M> {
        type Error = Error;

        fn write(&mut self, word: u8) -> Result<(), nb::Error<Self::Error>> {
            embedded_hal_02::serial::Write::write(&mut self.tx, word)
        }

        fn flush(&mut self) -> Result<(), nb::Error<Self::Error>> {
            embedded_hal_02::serial::Write::flush(&mut self.tx)
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_02::blocking::serial::Write<u8> for Uart<'d, T, M> {
        type Error = Error;

        fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }

        fn bflush(&mut self) -> Result<(), Self::Error> {
            self.blocking_flush()
        }
    }
}

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

    impl embedded_hal_nb::serial::Error for Error {
        fn kind(&self) -> embedded_hal_nb::serial::ErrorKind {
            match *self {
                Self::Framing => embedded_hal_nb::serial::ErrorKind::FrameFormat,
                Self::Break => embedded_hal_nb::serial::ErrorKind::Other,
                Self::Overrun => embedded_hal_nb::serial::ErrorKind::Overrun,
                Self::Parity => embedded_hal_nb::serial::ErrorKind::Parity,
            }
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_nb::serial::ErrorType for UartRx<'d, T, M> {
        type Error = Error;
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_nb::serial::ErrorType for UartTx<'d, T, M> {
        type Error = Error;
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_nb::serial::ErrorType for Uart<'d, T, M> {
        type Error = Error;
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_nb::serial::Read for UartRx<'d, T, M> {
        fn read(&mut self) -> nb::Result<u8, Self::Error> {
            let r = T::regs();
            let dr = r.uartdr().read();

            if dr.oe() {
                Err(nb::Error::Other(Error::Overrun))
            } else if dr.be() {
                Err(nb::Error::Other(Error::Break))
            } else if dr.pe() {
                Err(nb::Error::Other(Error::Parity))
            } else if dr.fe() {
                Err(nb::Error::Other(Error::Framing))
            } else if dr.fe() {
                Ok(dr.data())
            } else {
                Err(nb::Error::WouldBlock)
            }
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_nb::serial::Write for UartTx<'d, T, M> {
        fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
            self.blocking_write(&[char]).map_err(nb::Error::Other)
        }

        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            self.blocking_flush().map_err(nb::Error::Other)
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_nb::serial::Read for Uart<'d, T, M> {
        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            embedded_hal_02::serial::Read::read(&mut self.rx)
        }
    }

    impl<'d, T: Instance, M: Mode> embedded_hal_nb::serial::Write for Uart<'d, T, M> {
        fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
            self.blocking_write(&[char]).map_err(nb::Error::Other)
        }

        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            self.blocking_flush().map_err(nb::Error::Other)
        }
    }
}

mod sealed {
    use super::*;

    pub trait Mode {}

    pub trait Instance {
        const TX_DREQ: u8;
        const RX_DREQ: u8;

        type Interrupt: interrupt::typelevel::Interrupt;

        fn regs() -> pac::uart::Uart;

        #[cfg(feature = "nightly")]
        fn buffered_state() -> &'static buffered::State;

        fn dma_state() -> &'static DmaState;
    }
    pub trait TxPin<T: Instance> {}
    pub trait RxPin<T: Instance> {}
    pub trait CtsPin<T: Instance> {}
    pub trait RtsPin<T: Instance> {}
}

pub trait Mode: sealed::Mode {}

macro_rules! impl_mode {
    ($name:ident) => {
        impl sealed::Mode for $name {}
        impl Mode for $name {}
    };
}

pub struct Blocking;
pub struct Async;

impl_mode!(Blocking);
impl_mode!(Async);

pub trait Instance: sealed::Instance {}

macro_rules! impl_instance {
    ($inst:ident, $irq:ident, $tx_dreq:expr, $rx_dreq:expr) => {
        impl sealed::Instance for peripherals::$inst {
            const TX_DREQ: u8 = $tx_dreq;
            const RX_DREQ: u8 = $rx_dreq;

            type Interrupt = crate::interrupt::typelevel::$irq;

            fn regs() -> pac::uart::Uart {
                pac::$inst
            }

            #[cfg(feature = "nightly")]
            fn buffered_state() -> &'static buffered::State {
                static STATE: buffered::State = buffered::State::new();
                &STATE
            }

            fn dma_state() -> &'static DmaState {
                static STATE: DmaState = DmaState {
                    rx_err_waker: AtomicWaker::new(),
                    rx_errs: AtomicU16::new(0),
                };
                &STATE
            }
        }
        impl Instance for peripherals::$inst {}
    };
}

impl_instance!(UART0, UART0_IRQ, 20, 21);
impl_instance!(UART1, UART1_IRQ, 22, 23);

pub trait TxPin<T: Instance>: sealed::TxPin<T> + crate::gpio::Pin {}
pub trait RxPin<T: Instance>: sealed::RxPin<T> + crate::gpio::Pin {}
pub trait CtsPin<T: Instance>: sealed::CtsPin<T> + crate::gpio::Pin {}
pub trait RtsPin<T: Instance>: sealed::RtsPin<T> + crate::gpio::Pin {}

macro_rules! impl_pin {
    ($pin:ident, $instance:ident, $function:ident) => {
        impl sealed::$function<peripherals::$instance> for peripherals::$pin {}
        impl $function<peripherals::$instance> for peripherals::$pin {}
    };
}

impl_pin!(PIN_0, UART0, TxPin);
impl_pin!(PIN_1, UART0, RxPin);
impl_pin!(PIN_2, UART0, CtsPin);
impl_pin!(PIN_3, UART0, RtsPin);
impl_pin!(PIN_4, UART1, TxPin);
impl_pin!(PIN_5, UART1, RxPin);
impl_pin!(PIN_6, UART1, CtsPin);
impl_pin!(PIN_7, UART1, RtsPin);
impl_pin!(PIN_8, UART1, TxPin);
impl_pin!(PIN_9, UART1, RxPin);
impl_pin!(PIN_10, UART1, CtsPin);
impl_pin!(PIN_11, UART1, RtsPin);
impl_pin!(PIN_12, UART0, TxPin);
impl_pin!(PIN_13, UART0, RxPin);
impl_pin!(PIN_14, UART0, CtsPin);
impl_pin!(PIN_15, UART0, RtsPin);
impl_pin!(PIN_16, UART0, TxPin);
impl_pin!(PIN_17, UART0, RxPin);
impl_pin!(PIN_18, UART0, CtsPin);
impl_pin!(PIN_19, UART0, RtsPin);
impl_pin!(PIN_20, UART1, TxPin);
impl_pin!(PIN_21, UART1, RxPin);
impl_pin!(PIN_22, UART1, CtsPin);
impl_pin!(PIN_23, UART1, RtsPin);
impl_pin!(PIN_24, UART1, TxPin);
impl_pin!(PIN_25, UART1, RxPin);
impl_pin!(PIN_26, UART1, CtsPin);
impl_pin!(PIN_27, UART1, RtsPin);
impl_pin!(PIN_28, UART0, TxPin);
impl_pin!(PIN_29, UART0, RxPin);
