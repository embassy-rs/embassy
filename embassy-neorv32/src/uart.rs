//! Universal Asynchronous Receiver and Transmitter (UART)
use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::{self, Dma};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::peripherals::{UART0, UART1};

/// UART interrupt handler binding.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // If RX FIFO is not empty, disable RX not empty IRQ and wake RX task
        let rx_nempty_irq_set = T::info().reg.ctrl().read().uart_ctrl_irq_rx_nempty().bit_is_set();
        let rx_nempty = T::info().reg.ctrl().read().uart_ctrl_rx_nempty().bit_is_set();

        if rx_nempty_irq_set && rx_nempty {
            T::info()
                .reg
                .ctrl()
                .modify(|_, w| w.uart_ctrl_irq_rx_nempty().clear_bit());
            T::info().rx_waker.wake();
        }

        // If RX FIFO is full, disable RX full IRQ and wake RX task
        let rx_full_irq_set = T::info().reg.ctrl().read().uart_ctrl_irq_rx_full().bit_is_set();
        let rx_full = T::info().reg.ctrl().read().uart_ctrl_rx_full().bit_is_set();

        if rx_full_irq_set && rx_full {
            T::info()
                .reg
                .ctrl()
                .modify(|_, w| w.uart_ctrl_irq_rx_full().clear_bit());
            T::info().rx_waker.wake();
        }

        // If TX FIFO is empty, disable TX empty IRQ and wake TX task
        let tx_empty_irq_set = T::info().reg.ctrl().read().uart_ctrl_irq_tx_empty().bit_is_set();
        let tx_empty = T::info().reg.ctrl().read().uart_ctrl_tx_empty().bit_is_set();

        if tx_empty_irq_set && tx_empty {
            T::info()
                .reg
                .ctrl()
                .modify(|_, w| w.uart_ctrl_irq_tx_empty().clear_bit());
            T::info().tx_waker.wake();
        }
    }
}

/// UART error.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// The NEORV32 configuration does not support UART.
    NotSupported,
    /// A DMA error occurred.
    Dma(dma::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::NotSupported => write!(f, "The NEORV32 configuration does not support UART"),
            Error::Dma(e) => write!(f, "A DMA error occurred: {e:?}"),
        }
    }
}

impl core::error::Error for Error {}

/// UART driver.
pub struct Uart<'d, M: IoMode> {
    rx: UartRx<'d, M>,
    tx: UartTx<'d, M>,
}

impl<'d, M: IoMode> Uart<'d, M> {
    fn init<T: Instance>(_instance: Peri<'d, T>, baud_rate: u32, sim: bool, flow_control: bool) {
        // Enable simulation mode if applicable
        if sim {
            T::info().reg.ctrl().modify(|_, w| w.uart_ctrl_sim_mode().set_bit());
        }

        // Enable flow control if applicable
        if flow_control {
            T::info().reg.ctrl().modify(|_, w| w.uart_ctrl_hwfc_en().set_bit());
        }

        // baud div is max 10-bits wide
        const U10_MAX: u16 = 0x3ff;
        let cpu_freq = crate::sysinfo::SysInfo::clock_freq();
        let mut baud_div = cpu_freq / (2 * baud_rate);
        let mut prsc_sel = 0;

        // Calculate clock prescaler and baud rate prescaler
        // See: https://github.com/stnolting/neorv32/blob/main/sw/lib/source/neorv32_uart.c#L47
        while baud_div >= U10_MAX as u32 {
            if prsc_sel == 2 || prsc_sel == 4 {
                baud_div >>= 3;
            } else {
                baud_div >>= 1;
            }
            prsc_sel += 1;
        }

        // Set the clock and baudrate prescalers
        // SAFETY: The calculation above ensures we are writing valid prscv and baud div
        T::info().reg.ctrl().modify(|_, w| unsafe {
            w.uart_ctrl_prsc()
                .bits(prsc_sel)
                .uart_ctrl_baud()
                .bits((baud_div as u16 - 1) & U10_MAX)
        });

        // Enable UART
        T::info().reg.ctrl().modify(|_, w| w.uart_ctrl_en().set_bit());
    }

    fn new_inner<T: Instance>(rx_dma: Option<Dma<'d>>, tx_dma: Option<Dma<'d>>) -> Result<Self, Error> {
        let rx = UartRx::new_inner::<T>(rx_dma)?;
        let tx = UartTx::new_inner::<T>(tx_dma)?;
        Ok(Self { rx, tx })
    }

    fn blocking_flush(&mut self) {
        self.tx.blocking_flush();
    }

    /// Reads a byte from RX FIFO, blocking if empty.
    pub fn blocking_read_byte(&self) -> u8 {
        self.rx.blocking_read_byte()
    }

    /// Reads bytes from RX FIFO until buffer is full, blocking if empty.
    pub fn blocking_read(&self, buf: &mut [u8]) {
        self.rx.blocking_read(buf);
    }

    /// Writes a byte to TX FIFO, blocking if full.
    pub fn blocking_write_byte(&mut self, byte: u8) {
        self.tx.blocking_write_byte(byte);
    }

    /// Writes bytes to TX FIFO, blocking if full.
    pub fn blocking_write(&mut self, bytes: &[u8]) {
        self.tx.blocking_write(bytes);
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
    /// Creates a new blocking UART driver with given baud rate.
    ///
    /// Enables simulation mode if `sim` is true and hardware flow control if `flow_control` is true.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotSupported`] if UART is not supported.
    pub fn new_blocking<T: Instance>(
        _instance: Peri<'d, T>,
        baud_rate: u32,
        sim: bool,
        flow_control: bool,
    ) -> Result<Self, Error> {
        Self::init(_instance, baud_rate, sim, flow_control);
        Self::new_inner::<T>(None, None)
    }
}

impl<'d> Uart<'d, Async> {
    fn new_async_inner<T: Instance>(
        _instance: Peri<'d, T>,
        baud_rate: u32,
        sim: bool,
        flow_control: bool,
        rx_dma: Option<Dma<'d>>,
        tx_dma: Option<Dma<'d>>,
    ) -> Result<Self, Error> {
        let uart = Self::new_inner::<T>(rx_dma, tx_dma)?;
        Self::init(_instance, baud_rate, sim, flow_control);
        // SAFETY: It is valid to enable UART interrupt here
        unsafe { T::Interrupt::enable() }
        Ok(uart)
    }

    fn flush(&mut self) -> impl Future<Output = ()> {
        self.tx.flush()
    }

    /// Creates a new async UART driver with given baud rate.
    ///
    /// Enables simulation mode if `sim` is true and hardware flow control if `flow_control` is true.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotSupported`] if UART is not supported.
    pub fn new_async<T: Instance>(
        _instance: Peri<'d, T>,
        baud_rate: u32,
        sim: bool,
        flow_control: bool,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Result<Self, Error> {
        Self::new_async_inner(_instance, baud_rate, sim, flow_control, None, None)
    }

    /// Creates a new async UART driver with given baud rate.
    ///
    /// Enables simulation mode if `sim` is true and hardware flow control if `flow_control` is true.
    ///
    /// Additionally provides the DMA peripheral for TX transfers.
    /// See [`UartTx::new_async_with_dma`] for considerations on whether to use DMA or not.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotSupported`] if UART is not supported.
    ///
    /// Returns [`Error::Dma`] if DMA is not supported.
    pub fn new_async_with_tx_dma<T: Instance, D: dma::Instance>(
        _instance: Peri<'d, T>,
        baud_rate: u32,
        sim: bool,
        flow_control: bool,
        dma: Peri<'d, D>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + Binding<D::Interrupt, dma::InterruptHandler<D>> + 'd,
    ) -> Result<Self, Error> {
        let dma = dma::Dma::new(dma, _irq).map_err(Error::Dma)?;
        Self::new_async_inner(_instance, baud_rate, sim, flow_control, None, Some(dma))
    }

    /// Creates a new async UART driver with given baud rate.
    ///
    /// Enables simulation mode if `sim` is true and hardware flow control if `flow_control` is true.
    ///
    /// Additionally provides the DMA peripheral for RX transfers.
    /// See [`UartRx::new_async_with_dma`] for considerations on whether to use DMA or not.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotSupported`] if UART is not supported.
    ///
    /// Returns [`Error::Dma`] if DMA is not supported.
    pub fn new_async_with_rx_dma<T: Instance, D: dma::Instance>(
        _instance: Peri<'d, T>,
        baud_rate: u32,
        sim: bool,
        flow_control: bool,
        dma: Peri<'d, D>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + Binding<D::Interrupt, dma::InterruptHandler<D>> + 'd,
    ) -> Result<Self, Error> {
        let dma = dma::Dma::new(dma, _irq).map_err(Error::Dma)?;
        Self::new_async_inner(_instance, baud_rate, sim, flow_control, Some(dma), None)
    }

    /// Reads bytes from RX FIFO until buffer is full.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dma`] if DMA error occurred during transfer.
    pub fn read(&mut self, buf: &mut [u8]) -> impl Future<Output = Result<(), Error>> {
        self.rx.read(buf)
    }

    /// Writes bytes from buffer to TX FIFO.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dma`] if DMA error occurred during transfer.
    pub fn write(&mut self, bytes: &[u8]) -> impl Future<Output = Result<(), Error>> {
        self.tx.write(bytes)
    }
}

/// RX-only UART driver.
pub struct UartRx<'d, M: IoMode> {
    info: Info,
    fifo_depth: usize,
    dma: Option<dma::Dma<'d>>,
    _phantom: PhantomData<&'d M>,
}

// Allows for use in a Mutex (to share safely between harts and tasks)
// Revisit: Is this actually safe?
// SAFETY: This is a consequence of the register block generated by the PAC being !Sync,
// and since we hold a reference to it that makes us !Send
unsafe impl<'d, M: IoMode> Send for UartRx<'d, M> {}

impl<'d, M: IoMode> UartRx<'d, M> {
    fn new_inner<T: Instance>(dma: Option<dma::Dma<'d>>) -> Result<Self, Error> {
        if !T::supported() {
            return Err(Error::NotSupported);
        }

        // Mark RX as active
        T::info().active.rx.store(true, Ordering::Release);

        // FIFO depth is part of DATA register, which has side effects when read, so we do it once and cache it
        // FIFO depth is bits 11:8 of DATA
        // Revisit: The SVD does not define FIFO depths as separate fields. Upstream patch?
        // This is used to chunk up DMA transfers into sizes that will fit in the FIFO
        let fifo_depth = (T::info().reg.data().read().bits() >> 8) & (0b1111);
        let fifo_depth = 1 << fifo_depth;

        Ok(Self {
            info: T::info(),
            fifo_depth,
            dma,
            _phantom: PhantomData,
        })
    }

    fn read_inner(&self) -> u8 {
        self.info.reg.data().read().bits() as u8
    }

    fn enable_irq_rx_nempty(&mut self) {
        self.info
            .reg
            .ctrl()
            .modify(|_, w| w.uart_ctrl_irq_rx_nempty().set_bit());
    }

    fn enable_irq_rx_full(&mut self) {
        self.info.reg.ctrl().modify(|_, w| w.uart_ctrl_irq_rx_full().set_bit());
    }

    fn fifo_empty(&self) -> bool {
        self.info.reg.ctrl().read().uart_ctrl_rx_nempty().bit_is_clear()
    }

    fn fifo_full(&self) -> bool {
        self.info.reg.ctrl().read().uart_ctrl_rx_full().bit_is_set()
    }

    /// Reads a byte from RX FIFO, blocking if empty.
    pub fn blocking_read_byte(&self) -> u8 {
        while self.fifo_empty() {}
        self.read_inner()
    }

    /// Reads bytes from RX FIFO until buffer is full, blocking if empty.
    pub fn blocking_read(&self, buf: &mut [u8]) {
        for byte in buf {
            *byte = self.blocking_read_byte();
        }
    }
}

impl<'d> UartRx<'d, Blocking> {
    /// Creates a new RX-only blocking UART driver with given baud rate.
    ///
    /// Enables hardware flow control if `flow_control` is true.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotSupported`] if UART is not supported.
    pub fn new_blocking<T: Instance>(
        _instance: Peri<'d, T>,
        baud_rate: u32,
        flow_control: bool,
    ) -> Result<Self, Error> {
        let uart = Self::new_inner::<T>(None)?;
        Uart::<Blocking>::init(_instance, baud_rate, false, flow_control);
        Ok(uart)
    }
}

impl<'d> UartRx<'d, Async> {
    async fn wait_fifo_nempty(&mut self) {
        poll_fn(|cx| {
            self.info.rx_waker.register(cx.waker());
            if !self.fifo_empty() {
                Poll::Ready(())
            } else {
                // CS used here since interrupt modifies register
                critical_section::with(|_| self.enable_irq_rx_nempty());
                Poll::Pending
            }
        })
        .await
    }

    async fn wait_fifo_full(&mut self) {
        poll_fn(|cx| {
            self.info.rx_waker.register(cx.waker());
            if self.fifo_full() {
                Poll::Ready(())
            } else {
                // CS used here since interrupt modifies register
                critical_section::with(|_| self.enable_irq_rx_full());
                Poll::Pending
            }
        })
        .await
    }

    async fn read_chunk(&mut self, chunk: &mut [u8]) -> Result<(), Error> {
        // If DMA available, use it to transfer data from RX FIFO to buffer
        if let Some(dma) = &mut self.dma {
            let src = self.info.reg.data().as_ptr() as *const u8;
            // SAFETY: The PAC ensures the data register pointer is not-null and properly aligned
            let src = unsafe { src.as_ref().unwrap_unchecked() };
            dma.read(src, chunk, false).await.map_err(Error::Dma)?;

        // Otherwise, manually read each byte from RX FIFO
        } else {
            for byte in chunk {
                *byte = self.read_inner();
            }
        }

        Ok(())
    }

    fn new_async_inner<T: Instance>(
        _instance: Peri<'d, T>,
        baud_rate: u32,
        flow_control: bool,
        dma: Option<Dma<'d>>,
    ) -> Result<Self, Error> {
        let uart = Self::new_inner::<T>(dma)?;
        Uart::<Async>::init(_instance, baud_rate, false, flow_control);
        // SAFETY: It is valid to enable UART interrupt here
        unsafe { T::Interrupt::enable() }
        Ok(uart)
    }

    /// Creates a new RX-only async UART driver with given baud rate.
    ///
    /// Enables hardware flow control if `flow_control` is true.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotSupported`] if UART is not supported.
    pub fn new_async<T: Instance>(
        _instance: Peri<'d, T>,
        baud_rate: u32,
        flow_control: bool,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Result<Self, Error> {
        Self::new_async_inner(_instance, baud_rate, flow_control, None)
    }

    /// Creates a new RX-only async UART driver with given baud rate.
    ///
    /// Enables hardware flow control if `flow_control` is true.
    ///
    /// Additionally provides the DMA peripheral for transfers.
    ///
    /// **Note**: The DMA peripheral is limited in that it is single-channel only so you have to
    /// decide which peripheral (if any) will use it. Without DMA, the driver will manually
    /// copy each byte into the FIFO. However, depending on the configured FIFO size and how many
    /// bytes you are expecting to transfer, this may be more efficient as there is overhead in
    /// setting up the DMA transfer.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotSupported`] if UART is not supported.
    ///
    /// Returns [`Error::Dma`] if DMA is not supported.
    pub fn new_async_with_dma<T: Instance, D: dma::Instance>(
        _instance: Peri<'d, T>,
        baud_rate: u32,
        flow_control: bool,
        dma: Peri<'d, D>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + Binding<D::Interrupt, dma::InterruptHandler<D>> + 'd,
    ) -> Result<Self, Error> {
        let dma = dma::Dma::new(dma, _irq).map_err(Error::Dma)?;
        Self::new_async_inner(_instance, baud_rate, flow_control, Some(dma))
    }

    /// Reads bytes from RX FIFO until buffer is full.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dma`] if DMA error occurred during transfer.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        let mut chunks = buf.chunks_exact_mut(self.fifo_depth);

        // For chunks that match the FIFO depth, we can wait for FIFO full
        // then read all bytes from the FIFO in one shot
        for chunk in chunks.by_ref() {
            self.wait_fifo_full().await;
            self.read_chunk(chunk).await?;
        }

        // But for the remainder need to interrupt every time the FIFO is not empty
        // and manually read a single byte
        for byte in chunks.into_remainder() {
            self.wait_fifo_nempty().await;
            *byte = self.read_inner();
        }

        Ok(())
    }
}

impl<'d, M: IoMode> Drop for UartRx<'d, M> {
    fn drop(&mut self) {
        self.info.active.rx.store(false, Ordering::Release);
        drop_uart(&self.info);
    }
}

/// TX-only UART driver.
pub struct UartTx<'d, M: IoMode> {
    info: Info,
    fifo_depth: usize,
    dma: Option<dma::Dma<'d>>,
    _phantom: PhantomData<&'d M>,
}

// Allows for use in a Mutex (to share safely between harts and tasks)
// Revisit: Is this actually safe?
// SAFETY: This is a consequence of the register block generated by the PAC being !Sync,
// and since we hold a reference to it that makes us !Send
unsafe impl<'d, M: IoMode> Send for UartTx<'d, M> {}

impl<'d, M: IoMode> UartTx<'d, M> {
    fn new_inner<T: Instance>(dma: Option<dma::Dma<'d>>) -> Result<Self, Error> {
        if !T::supported() {
            return Err(Error::NotSupported);
        }

        // Mark TX as active
        T::info().active.rx.store(true, Ordering::Release);

        // FIFO depth is part of DATA register, which has side effects when read, so we do it once and cache it
        // FIFO depth is bits 15:12 of DATA
        // Revisit: The SVD does not define FIFO depths as separate fields. Upstream patch?
        // This is used to chunk up DMA transfers into sizes that will fit in the FIFO
        let fifo_depth = (T::info().reg.data().read().bits() >> 12) & (0b1111);
        let fifo_depth = 1 << fifo_depth;

        Ok(Self {
            info: T::info(),
            dma,
            fifo_depth,
            _phantom: PhantomData,
        })
    }

    fn write_inner(&mut self, byte: u8) {
        // SAFETY: We are just writing a byte, the MSB bits are read-only
        self.info.reg.data().write(|w| unsafe { w.bits(byte as u32) });
    }

    fn enable_irq_tx_empty(&mut self) {
        self.info.reg.ctrl().modify(|_, w| w.uart_ctrl_irq_tx_empty().set_bit());
    }

    fn fifo_full(&self) -> bool {
        self.info.reg.ctrl().read().uart_ctrl_tx_nfull().bit_is_clear()
    }

    fn busy(&self) -> bool {
        self.info.reg.ctrl().read().uart_ctrl_tx_busy().bit_is_set()
    }

    fn blocking_flush(&mut self) {
        while self.busy() {}
    }

    /// Writes a byte to TX FIFO, blocking if full.
    pub fn blocking_write_byte(&mut self, byte: u8) {
        while self.fifo_full() {}
        self.write_inner(byte);
        self.blocking_flush();
    }

    /// Writes bytes to TX FIFO, blocking if full.
    pub fn blocking_write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            while self.fifo_full() {}
            self.write_inner(*byte);
        }
        self.blocking_flush();
    }
}

impl<'d> UartTx<'d, Blocking> {
    /// Creates a new TX-only blocking UART driver with given baud rate.
    ///
    /// Enables simulation mode if `sim` is true and hardware flow control if `flow_control` is true.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotSupported`] if UART is not supported.
    pub fn new_blocking<T: Instance>(
        _instance: Peri<'d, T>,
        baud_rate: u32,
        sim: bool,
        flow_control: bool,
    ) -> Result<Self, Error> {
        let uart = Self::new_inner::<T>(None)?;
        Uart::<Blocking>::init(_instance, baud_rate, sim, flow_control);
        Ok(uart)
    }
}

impl<'d> UartTx<'d, Async> {
    async fn write_chunk(&mut self, chunk: &[u8]) -> Result<(), Error> {
        // If DMA available, use it to transfer data from buffer to TX FIFO
        if let Some(dma) = &mut self.dma {
            let dst = self.info.reg.data().as_ptr() as *mut u8;
            // SAFETY: The PAC ensures the data register pointer is not-null and properly aligned
            let dst = unsafe { dst.as_mut().unwrap_unchecked() };
            dma.write(chunk, dst, false).await.map_err(Error::Dma)?;

        // Otherwise, manually write each byte to TX FIFO
        } else {
            for byte in chunk.iter().copied() {
                self.write_inner(byte);
            }
        }

        Ok(())
    }

    fn new_async_inner<T: Instance>(
        _instance: Peri<'d, T>,
        baud_rate: u32,
        sim: bool,
        flow_control: bool,
        dma: Option<Dma<'d>>,
    ) -> Result<Self, Error> {
        let uart = Self::new_inner::<T>(dma)?;
        Uart::<Async>::init(_instance, baud_rate, sim, flow_control);
        // SAFETY: It is valid to enable UART interrupt here
        unsafe { T::Interrupt::enable() }
        Ok(uart)
    }

    async fn flush(&mut self) {
        poll_fn(|cx| {
            self.info.tx_waker.register(cx.waker());
            if !self.busy() {
                Poll::Ready(())
            } else {
                // CS used here since interrupt modifies register
                critical_section::with(|_| self.enable_irq_tx_empty());
                Poll::Pending
            }
        })
        .await
    }

    /// Creates a new TX-only async UART driver with given baud rate.
    ///
    /// Enables simulation mode if `sim` is true and hardware flow control if `flow_control` is true.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotSupported`] if UART is not supported.
    pub fn new_async<T: Instance>(
        _instance: Peri<'d, T>,
        baud_rate: u32,
        sim: bool,
        flow_control: bool,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Result<Self, Error> {
        Self::new_async_inner(_instance, baud_rate, sim, flow_control, None)
    }

    /// Creates a new TX-only async UART driver with given baud rate.
    ///
    /// Enables simulation mode if `sim` is true and hardware flow control if `flow_control` is true.
    ///
    /// Additionally provides the DMA peripheral for transfers.
    ///
    /// **Note**: The DMA peripheral is limited in that it is single-channel only so you have to
    /// decide which peripheral (if any) will use it. Without DMA, the driver will manually
    /// copy each byte into the FIFO. However, depending on the configured FIFO size and how many
    /// bytes you are expecting to transfer, this may be more efficient as there is overhead in
    /// setting up the DMA transfer.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotSupported`] if UART is not supported.
    ///
    /// Returns [`Error::Dma`] if DMA is not supported.
    pub fn new_async_with_dma<T: Instance, D: dma::Instance>(
        _instance: Peri<'d, T>,
        baud_rate: u32,
        sim: bool,
        flow_control: bool,
        dma: Peri<'d, D>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>> + Binding<D::Interrupt, dma::InterruptHandler<D>> + 'd,
    ) -> Result<Self, Error> {
        let dma = dma::Dma::new(dma, _irq).map_err(Error::Dma)?;
        Self::new_async_inner(_instance, baud_rate, sim, flow_control, Some(dma))
    }

    /// Writes bytes from buffer to TX FIFO.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dma`] if DMA error occurred during transfer.
    pub async fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        for chunk in bytes.chunks(self.fifo_depth) {
            self.write_chunk(chunk).await?;
            self.flush().await;
        }

        Ok(())
    }
}

impl<'d, M: IoMode> Drop for UartTx<'d, M> {
    fn drop(&mut self) {
        self.info.active.tx.store(false, Ordering::Release);
        drop_uart(&self.info);
    }
}

fn drop_uart(info: &Info) {
    // Only disable UART if both Rx and Tx have been dropped
    critical_section::with(|_| {
        if !info.active.rx.load(Ordering::Acquire) && !info.active.tx.load(Ordering::Acquire) {
            info.reg.ctrl().modify(|_, w| w.uart_ctrl_en().clear_bit());
        }
    })
}

// Serves as a "reference-counter" so we know when Uart is completely dropped
// Use two AtomicBools instead of AtomicU8 since fetch_add/fetch_sub are not available without A extension
struct Active {
    rx: AtomicBool,
    tx: AtomicBool,
}

impl Active {
    const fn new() -> Self {
        Self {
            rx: AtomicBool::new(false),
            tx: AtomicBool::new(false),
        }
    }
}

struct Info {
    // Note: uart0 and uart1 can both share uart0::RegisterBlock
    // PAC is able to coerce uart1::ptr() to it with correct base address
    reg: &'static crate::pac::uart0::RegisterBlock,
    active: &'static Active,
    rx_waker: &'static AtomicWaker,
    tx_waker: &'static AtomicWaker,
}

trait SealedIoMode {}

/// UART IO mode.
#[allow(private_bounds)]
pub trait IoMode: SealedIoMode {}

/// Blocking UART.
pub struct Blocking;
impl SealedIoMode for Blocking {}
impl IoMode for Blocking {}

/// Async UART.
pub struct Async;
impl SealedIoMode for Async {}
impl IoMode for Async {}

trait SealedInstance {
    fn info() -> Info;
    fn supported() -> bool;
}

/// A valid UART peripheral.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {
    type Interrupt: Interrupt;
}

macro_rules! impl_instance {
    ($periph:ident, $rb:ident, $soc_cfg:ident) => {
        impl SealedInstance for $periph {
            fn info() -> Info {
                static RX_WAKER: AtomicWaker = AtomicWaker::new();
                static TX_WAKER: AtomicWaker = AtomicWaker::new();
                static ACTIVE: Active = Active::new();

                Info {
                    // SAFETY: We are the sole users of the pointer and are sure to use it safely
                    reg: unsafe { &*crate::pac::$rb::ptr() },
                    active: &ACTIVE,
                    rx_waker: &RX_WAKER,
                    tx_waker: &TX_WAKER,
                }
            }

            fn supported() -> bool {
                crate::sysinfo::SysInfo::soc_config().$soc_cfg()
            }
        }
        impl Instance for $periph {
            type Interrupt = crate::interrupt::typelevel::$periph;
        }
    };
}

impl_instance!(UART0, Uart0, has_uart0);
impl_instance!(UART1, Uart1, has_uart1);

// Convenience for writing formatted strings to UART
impl<'d, M: IoMode> core::fmt::Write for Uart<'d, M> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.blocking_write(s.as_bytes());
        Ok(())
    }
}

impl<'d, M: IoMode> core::fmt::Write for UartTx<'d, M> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.blocking_write(s.as_bytes());
        Ok(())
    }
}

impl<'d, M: IoMode> embedded_hal_02::blocking::serial::Write<u8> for Uart<'d, M> {
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

impl<'d, M: IoMode> embedded_hal_02::blocking::serial::Write<u8> for UartTx<'d, M> {
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

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl<'d, M: IoMode> embedded_io::ErrorType for Uart<'d, M> {
    type Error = Error;
}

impl<'d, M: IoMode> embedded_io::Read for Uart<'d, M> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.blocking_read(buf);
        Ok(buf.len())
    }
}

impl<'d> embedded_io_async::Read for Uart<'d, Async> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read(buf).await.map(|_| buf.len())
    }
}

impl<'d, M: IoMode> embedded_io::Write for Uart<'d, M> {
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
        self.write(buf).await.map(|_| buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush().await;
        Ok(())
    }
}

impl<'d, M: IoMode> embedded_io::ErrorType for UartTx<'d, M> {
    type Error = Error;
}

impl<'d, M: IoMode> embedded_io::Write for UartTx<'d, M> {
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
        self.write(buf).await.map(|_| buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush().await;
        Ok(())
    }
}

impl<'d, M: IoMode> embedded_io::ErrorType for UartRx<'d, M> {
    type Error = Error;
}

impl<'d, M: IoMode> embedded_io::Read for UartRx<'d, M> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.blocking_read(buf);
        Ok(buf.len())
    }
}

impl<'d> embedded_io_async::Read for UartRx<'d, Async> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read(buf).await.map(|_| buf.len())
    }
}
