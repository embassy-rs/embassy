#![macro_use]

//! Async UART
//!
//! Async UART is provided in two flavors - this one and also [crate::buffered_uarte::BufferedUarte].
//! The [Uarte] here is useful for those use-cases where reading the UARTE peripheral is
//! exclusively awaited on. If the [Uarte] is required to be awaited on with some other future,
//! for example when using `futures_util::future::select`, then you should consider
//! [crate::buffered_uarte::BufferedUarte] so that reads may continue while processing these
//! other futures. If you do not then you may lose data between reads.
//!
//! An advantage of the [Uarte] has over [crate::buffered_uarte::BufferedUarte] is that less
//! memory may be used given that buffers are passed in directly to its read and write
//! methods.

use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;
use embassy::interrupt::InterruptExt;
use embassy::util::Unborrow;
use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::unborrow;
use futures::future::poll_fn;

use crate::chip::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE};
use crate::gpio::sealed::Pin as _;
use crate::gpio::{self, AnyPin, Pin as GpioPin, PselBits};
use crate::interrupt::Interrupt;
use crate::pac;
use crate::ppi::{AnyConfigurableChannel, ConfigurableChannel, Event, Ppi, Task};
use crate::timer::Instance as TimerInstance;
use crate::timer::{Frequency, Timer};
use crate::util::slice_in_ram_or;

// Re-export SVD variants to allow user to directly set values.
pub use pac::uarte0::{baudrate::BAUDRATE_A as Baudrate, config::PARITY_A as Parity};

#[non_exhaustive]
pub struct Config {
    pub parity: Parity,
    pub baudrate: Baudrate,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            parity: Parity::EXCLUDED,
            baudrate: Baudrate::BAUD115200,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    BufferTooLong,
    BufferZeroLength,
    DMABufferNotInDataMemory,
    // TODO: add other error variants.
}

/// Interface to the UARTE peripheral using EasyDMA to offload the transmission and reception workload.
///
/// For more details about EasyDMA, consult the module documentation.
pub struct Uarte<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    tx: UarteTx<'d, T>,
    rx: UarteRx<'d, T>,
}

/// Transmitter interface to the UARTE peripheral obtained
/// via [Uarte]::split.
pub struct UarteTx<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

/// Receiver interface to the UARTE peripheral obtained
/// via [Uarte]::split.
pub struct UarteRx<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Uarte<'d, T> {
    /// Create a new UARTE without hardware flow control
    pub fn new(
        uarte: impl Unborrow<Target = T> + 'd,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
        rxd: impl Unborrow<Target = impl GpioPin> + 'd,
        txd: impl Unborrow<Target = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(rxd, txd);
        Self::new_inner(uarte, irq, rxd.degrade(), txd.degrade(), None, None, config)
    }

    /// Create a new UARTE with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts(
        uarte: impl Unborrow<Target = T> + 'd,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
        rxd: impl Unborrow<Target = impl GpioPin> + 'd,
        txd: impl Unborrow<Target = impl GpioPin> + 'd,
        cts: impl Unborrow<Target = impl GpioPin> + 'd,
        rts: impl Unborrow<Target = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(rxd, txd, cts, rts);
        Self::new_inner(
            uarte,
            irq,
            rxd.degrade(),
            txd.degrade(),
            Some(cts.degrade()),
            Some(rts.degrade()),
            config,
        )
    }

    fn new_inner(
        _uarte: impl Unborrow<Target = T> + 'd,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
        rxd: AnyPin,
        txd: AnyPin,
        cts: Option<AnyPin>,
        rts: Option<AnyPin>,
        config: Config,
    ) -> Self {
        unborrow!(irq);

        let r = T::regs();

        rxd.conf().write(|w| w.input().connect().drive().h0h1());
        r.psel.rxd.write(|w| unsafe { w.bits(rxd.psel_bits()) });

        txd.set_high();
        txd.conf().write(|w| w.dir().output().drive().h0h1());
        r.psel.txd.write(|w| unsafe { w.bits(txd.psel_bits()) });

        if let Some(pin) = &cts {
            pin.conf().write(|w| w.input().connect().drive().h0h1());
        }
        r.psel.cts.write(|w| unsafe { w.bits(cts.psel_bits()) });

        if let Some(pin) = &rts {
            pin.set_high();
            pin.conf().write(|w| w.dir().output().drive().h0h1());
        }
        r.psel.rts.write(|w| unsafe { w.bits(rts.psel_bits()) });

        // Configure
        let hardware_flow_control = match (rts.is_some(), cts.is_some()) {
            (false, false) => false,
            (true, true) => true,
            _ => panic!("RTS and CTS pins must be either both set or none set."),
        };
        r.config.write(|w| {
            w.hwfc().bit(hardware_flow_control);
            w.parity().variant(config.parity);
            w
        });
        r.baudrate.write(|w| w.baudrate().variant(config.baudrate));

        // Disable all interrupts
        r.intenclr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });

        // Reset rxstarted, txstarted. These are used by drop to know whether a transfer was
        // stopped midway or not.
        r.events_rxstarted.reset();
        r.events_txstarted.reset();

        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        // Enable
        apply_workaround_for_enable_anomaly(&r);
        r.enable.write(|w| w.enable().enabled());

        let s = T::state();

        s.tx_rx_refcount.store(2, Ordering::Relaxed);

        Self {
            phantom: PhantomData,
            tx: UarteTx {
                phantom: PhantomData,
            },
            rx: UarteRx {
                phantom: PhantomData,
            },
        }
    }

    /// Split the Uarte into a transmitter and receiver, which is
    /// particuarly useful when having two tasks correlating to
    /// transmitting and receiving.
    pub fn split(self) -> (UarteTx<'d, T>, UarteRx<'d, T>) {
        (self.tx, self.rx)
    }

    /// Return the endtx event for use with PPI
    pub fn event_endtx(&self) -> Event {
        let r = T::regs();
        Event::from_reg(&r.events_endtx)
    }

    fn on_interrupt(_: *mut ()) {
        let r = T::regs();
        let s = T::state();

        if r.events_endrx.read().bits() != 0 {
            s.endrx_waker.wake();
            r.intenclr.write(|w| w.endrx().clear());
        }
        if r.events_endtx.read().bits() != 0 {
            s.endtx_waker.wake();
            r.intenclr.write(|w| w.endtx().clear());
        }
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.read(buffer).await
    }

    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write(buffer).await
    }

    /// Same as [`write`](Uarte::write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub async fn write_from_ram(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write_from_ram(buffer).await
    }

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }

    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buffer)
    }

    /// Same as [`blocking_write`](Uarte::blocking_write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub fn blocking_write_from_ram(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write_from_ram(buffer)
    }
}

impl<'d, T: Instance> UarteTx<'d, T> {
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        match self.write_from_ram(buffer).await {
            Ok(_) => Ok(()),
            Err(Error::DMABufferNotInDataMemory) => {
                trace!("Copying UARTE tx buffer into RAM for DMA");
                let ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..buffer.len()];
                ram_buf.copy_from_slice(buffer);
                self.write_from_ram(&ram_buf).await
            }
            Err(error) => Err(error),
        }
    }

    pub async fn write_from_ram(&mut self, buffer: &[u8]) -> Result<(), Error> {
        slice_in_ram_or(buffer, Error::DMABufferNotInDataMemory)?;
        if buffer.len() == 0 {
            return Err(Error::BufferZeroLength);
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let ptr = buffer.as_ptr();
        let len = buffer.len();

        let r = T::regs();
        let s = T::state();

        let drop = OnDrop::new(move || {
            trace!("write drop: stopping");

            r.intenclr.write(|w| w.endtx().clear());
            r.events_txstopped.reset();
            r.tasks_stoptx.write(|w| unsafe { w.bits(1) });

            // TX is stopped almost instantly, spinning is fine.
            while r.events_endtx.read().bits() == 0 {}
            trace!("write drop: stopped");
        });

        r.txd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        r.txd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

        r.events_endtx.reset();
        r.intenset.write(|w| w.endtx().set());

        compiler_fence(Ordering::SeqCst);

        trace!("starttx");
        r.tasks_starttx.write(|w| unsafe { w.bits(1) });

        poll_fn(|cx| {
            s.endtx_waker.register(cx.waker());
            if r.events_endtx.read().bits() != 0 {
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;

        compiler_fence(Ordering::SeqCst);
        r.events_txstarted.reset();
        drop.defuse();

        Ok(())
    }

    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        match self.blocking_write_from_ram(buffer) {
            Ok(_) => Ok(()),
            Err(Error::DMABufferNotInDataMemory) => {
                trace!("Copying UARTE tx buffer into RAM for DMA");
                let ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..buffer.len()];
                ram_buf.copy_from_slice(buffer);
                self.blocking_write_from_ram(&ram_buf)
            }
            Err(error) => Err(error),
        }
    }

    pub fn blocking_write_from_ram(&mut self, buffer: &[u8]) -> Result<(), Error> {
        slice_in_ram_or(buffer, Error::DMABufferNotInDataMemory)?;
        if buffer.len() == 0 {
            return Err(Error::BufferZeroLength);
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let ptr = buffer.as_ptr();
        let len = buffer.len();

        let r = T::regs();

        r.txd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        r.txd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

        r.events_endtx.reset();
        r.intenclr.write(|w| w.endtx().clear());

        compiler_fence(Ordering::SeqCst);

        trace!("starttx");
        r.tasks_starttx.write(|w| unsafe { w.bits(1) });

        while r.events_endtx.read().bits() == 0 {}

        compiler_fence(Ordering::SeqCst);
        r.events_txstarted.reset();

        Ok(())
    }
}

impl<'a, T: Instance> Drop for UarteTx<'a, T> {
    fn drop(&mut self) {
        trace!("uarte tx drop");

        let r = T::regs();

        let did_stoptx = r.events_txstarted.read().bits() != 0;
        trace!("did_stoptx {}", did_stoptx);

        // Wait for txstopped, if needed.
        while did_stoptx && r.events_txstopped.read().bits() == 0 {}

        let s = T::state();

        drop_tx_rx(&r, &s);
    }
}

impl<'d, T: Instance> UarteRx<'d, T> {
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        if buffer.len() == 0 {
            return Err(Error::BufferZeroLength);
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let ptr = buffer.as_ptr();
        let len = buffer.len();

        let r = T::regs();
        let s = T::state();

        let drop = OnDrop::new(move || {
            trace!("read drop: stopping");

            r.intenclr.write(|w| w.endrx().clear());
            r.events_rxto.reset();
            r.tasks_stoprx.write(|w| unsafe { w.bits(1) });

            while r.events_endrx.read().bits() == 0 {}

            trace!("read drop: stopped");
        });

        r.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        r.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

        r.events_endrx.reset();
        r.intenset.write(|w| w.endrx().set());

        compiler_fence(Ordering::SeqCst);

        trace!("startrx");
        r.tasks_startrx.write(|w| unsafe { w.bits(1) });

        poll_fn(|cx| {
            s.endrx_waker.register(cx.waker());
            if r.events_endrx.read().bits() != 0 {
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;

        compiler_fence(Ordering::SeqCst);
        r.events_rxstarted.reset();
        drop.defuse();

        Ok(())
    }

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        if buffer.len() == 0 {
            return Err(Error::BufferZeroLength);
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let ptr = buffer.as_ptr();
        let len = buffer.len();

        let r = T::regs();

        r.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        r.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

        r.events_endrx.reset();
        r.intenclr.write(|w| w.endrx().clear());

        compiler_fence(Ordering::SeqCst);

        trace!("startrx");
        r.tasks_startrx.write(|w| unsafe { w.bits(1) });

        while r.events_endrx.read().bits() == 0 {}

        compiler_fence(Ordering::SeqCst);
        r.events_rxstarted.reset();

        Ok(())
    }
}

impl<'a, T: Instance> Drop for UarteRx<'a, T> {
    fn drop(&mut self) {
        trace!("uarte rx drop");

        let r = T::regs();

        let did_stoprx = r.events_rxstarted.read().bits() != 0;
        trace!("did_stoprx {}", did_stoprx);

        // Wait for rxto, if needed.
        while did_stoprx && r.events_rxto.read().bits() == 0 {}

        let s = T::state();

        drop_tx_rx(&r, &s);
    }
}

#[cfg(not(any(feature = "_nrf9160", feature = "nrf5340")))]
pub(in crate) fn apply_workaround_for_enable_anomaly(_r: &crate::pac::uarte0::RegisterBlock) {
    // Do nothing
}

#[cfg(any(feature = "_nrf9160", feature = "nrf5340"))]
pub(in crate) fn apply_workaround_for_enable_anomaly(r: &crate::pac::uarte0::RegisterBlock) {
    use core::ops::Deref;

    // Apply workaround for anomalies:
    // - nRF9160 - anomaly 23
    // - nRF5340 - anomaly 44
    let rxenable_reg: *const u32 = ((r.deref() as *const _ as usize) + 0x564) as *const u32;
    let txenable_reg: *const u32 = ((r.deref() as *const _ as usize) + 0x568) as *const u32;

    // NB Safety: This is taken from Nordic's driver -
    // https://github.com/NordicSemiconductor/nrfx/blob/master/drivers/src/nrfx_uarte.c#L197
    if unsafe { core::ptr::read_volatile(txenable_reg) } == 1 {
        r.tasks_stoptx.write(|w| unsafe { w.bits(1) });
    }

    // NB Safety: This is taken from Nordic's driver -
    // https://github.com/NordicSemiconductor/nrfx/blob/master/drivers/src/nrfx_uarte.c#L197
    if unsafe { core::ptr::read_volatile(rxenable_reg) } == 1 {
        r.enable.write(|w| w.enable().enabled());
        r.tasks_stoprx.write(|w| unsafe { w.bits(1) });

        let mut workaround_succeded = false;
        // The UARTE is able to receive up to four bytes after the STOPRX task has been triggered.
        // On lowest supported baud rate (1200 baud), with parity bit and two stop bits configured
        // (resulting in 12 bits per data byte sent), this may take up to 40 ms.
        for _ in 0..40000 {
            // NB Safety: This is taken from Nordic's driver -
            // https://github.com/NordicSemiconductor/nrfx/blob/master/drivers/src/nrfx_uarte.c#L197
            if unsafe { core::ptr::read_volatile(rxenable_reg) } == 0 {
                workaround_succeded = true;
                break;
            } else {
                // Need to sleep for 1us here
            }
        }

        if !workaround_succeded {
            panic!("Failed to apply workaround for UART");
        }

        let errors = r.errorsrc.read().bits();
        // NB Safety: safe to write back the bits we just read to clear them
        r.errorsrc.write(|w| unsafe { w.bits(errors) });
        r.enable.write(|w| w.enable().disabled());
    }
}

pub(in crate) fn drop_tx_rx(r: &pac::uarte0::RegisterBlock, s: &sealed::State) {
    if s.tx_rx_refcount.fetch_sub(1, Ordering::Relaxed) == 1 {
        // Finally we can disable, and we do so for the peripheral
        // i.e. not just rx concerns.
        r.enable.write(|w| w.enable().disabled());

        gpio::deconfigure_pin(r.psel.rxd.read().bits());
        gpio::deconfigure_pin(r.psel.txd.read().bits());
        gpio::deconfigure_pin(r.psel.rts.read().bits());
        gpio::deconfigure_pin(r.psel.cts.read().bits());

        trace!("uarte tx and rx drop: done");
    }
}

/// Interface to an UARTE peripheral that uses an additional timer and two PPI channels,
/// allowing it to implement the ReadUntilIdle trait.
pub struct UarteWithIdle<'d, U: Instance, T: TimerInstance> {
    tx: UarteTx<'d, U>,
    rx: UarteRxWithIdle<'d, U, T>,
}

impl<'d, U: Instance, T: TimerInstance> UarteWithIdle<'d, U, T> {
    /// Create a new UARTE without hardware flow control
    pub fn new(
        uarte: impl Unborrow<Target = U> + 'd,
        timer: impl Unborrow<Target = T> + 'd,
        ppi_ch1: impl Unborrow<Target = impl ConfigurableChannel + 'd> + 'd,
        ppi_ch2: impl Unborrow<Target = impl ConfigurableChannel + 'd> + 'd,
        irq: impl Unborrow<Target = U::Interrupt> + 'd,
        rxd: impl Unborrow<Target = impl GpioPin> + 'd,
        txd: impl Unborrow<Target = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(rxd, txd);
        Self::new_inner(
            uarte,
            timer,
            ppi_ch1,
            ppi_ch2,
            irq,
            rxd.degrade(),
            txd.degrade(),
            None,
            None,
            config,
        )
    }

    /// Create a new UARTE with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts(
        uarte: impl Unborrow<Target = U> + 'd,
        timer: impl Unborrow<Target = T> + 'd,
        ppi_ch1: impl Unborrow<Target = impl ConfigurableChannel + 'd> + 'd,
        ppi_ch2: impl Unborrow<Target = impl ConfigurableChannel + 'd> + 'd,
        irq: impl Unborrow<Target = U::Interrupt> + 'd,
        rxd: impl Unborrow<Target = impl GpioPin> + 'd,
        txd: impl Unborrow<Target = impl GpioPin> + 'd,
        cts: impl Unborrow<Target = impl GpioPin> + 'd,
        rts: impl Unborrow<Target = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(rxd, txd, cts, rts);
        Self::new_inner(
            uarte,
            timer,
            ppi_ch1,
            ppi_ch2,
            irq,
            rxd.degrade(),
            txd.degrade(),
            Some(cts.degrade()),
            Some(rts.degrade()),
            config,
        )
    }

    fn new_inner(
        uarte: impl Unborrow<Target = U> + 'd,
        timer: impl Unborrow<Target = T> + 'd,
        ppi_ch1: impl Unborrow<Target = impl ConfigurableChannel + 'd> + 'd,
        ppi_ch2: impl Unborrow<Target = impl ConfigurableChannel + 'd> + 'd,
        irq: impl Unborrow<Target = U::Interrupt> + 'd,
        rxd: AnyPin,
        txd: AnyPin,
        cts: Option<AnyPin>,
        rts: Option<AnyPin>,
        config: Config,
    ) -> Self {
        let baudrate = config.baudrate;
        let (tx, rx) = Uarte::new_inner(uarte, irq, rxd, txd, cts, rts, config).split();

        let mut timer = Timer::new(timer);

        unborrow!(ppi_ch1, ppi_ch2);

        let r = U::regs();

        // BAUDRATE register values are `baudrate * 2^32 / 16000000`
        // source: https://devzone.nordicsemi.com/f/nordic-q-a/391/uart-baudrate-register-values
        //
        // We want to stop RX if line is idle for 2 bytes worth of time
        // That is 20 bits (each byte is 1 start bit + 8 data bits + 1 stop bit)
        // This gives us the amount of 16M ticks for 20 bits.
        let timeout = 0x8000_0000 / (baudrate as u32 / 40);

        timer.set_frequency(Frequency::F16MHz);
        timer.cc(0).write(timeout);
        timer.cc(0).short_compare_clear();
        timer.cc(0).short_compare_stop();

        let mut ppi_ch1 = Ppi::new_one_to_two(
            ppi_ch1.degrade(),
            Event::from_reg(&r.events_rxdrdy),
            timer.task_clear(),
            timer.task_start(),
        );
        ppi_ch1.enable();

        let mut ppi_ch2 = Ppi::new_one_to_one(
            ppi_ch2.degrade(),
            timer.cc(0).event_compare(),
            Task::from_reg(&r.tasks_stoprx),
        );
        ppi_ch2.enable();

        Self {
            tx,
            rx: UarteRxWithIdle {
                rx,
                timer,
                ppi_ch1: ppi_ch1,
                _ppi_ch2: ppi_ch2,
            },
        }
    }

    /// Split the Uarte into a transmitter and receiver, which is
    /// particuarly useful when having two tasks correlating to
    /// transmitting and receiving.
    pub fn split(self) -> (UarteTx<'d, U>, UarteRxWithIdle<'d, U, T>) {
        (self.tx, self.rx)
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.read(buffer).await
    }

    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write(buffer).await
    }

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }

    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buffer)
    }

    pub async fn read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        self.rx.read_until_idle(buffer).await
    }

    pub fn blocking_read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        self.rx.blocking_read_until_idle(buffer)
    }
}

pub struct UarteRxWithIdle<'d, U: Instance, T: TimerInstance> {
    rx: UarteRx<'d, U>,
    timer: Timer<'d, T>,
    ppi_ch1: Ppi<'d, AnyConfigurableChannel, 1, 2>,
    _ppi_ch2: Ppi<'d, AnyConfigurableChannel, 1, 1>,
}

impl<'d, U: Instance, T: TimerInstance> UarteRxWithIdle<'d, U, T> {
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.ppi_ch1.disable();
        self.rx.read(buffer).await
    }

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.ppi_ch1.disable();
        self.rx.blocking_read(buffer)
    }

    pub async fn read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        if buffer.len() == 0 {
            return Err(Error::BufferZeroLength);
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let ptr = buffer.as_ptr();
        let len = buffer.len();

        let r = U::regs();
        let s = U::state();

        self.ppi_ch1.enable();

        let drop = OnDrop::new(|| {
            self.timer.stop();

            r.intenclr.write(|w| w.endrx().clear());
            r.events_rxto.reset();
            r.tasks_stoprx.write(|w| unsafe { w.bits(1) });

            while r.events_endrx.read().bits() == 0 {}
        });

        r.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        r.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

        r.events_endrx.reset();
        r.intenset.write(|w| w.endrx().set());

        compiler_fence(Ordering::SeqCst);

        r.tasks_startrx.write(|w| unsafe { w.bits(1) });

        poll_fn(|cx| {
            s.endrx_waker.register(cx.waker());
            if r.events_endrx.read().bits() != 0 {
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;

        compiler_fence(Ordering::SeqCst);
        let n = r.rxd.amount.read().amount().bits() as usize;

        self.timer.stop();
        r.events_rxstarted.reset();

        drop.defuse();

        Ok(n)
    }

    pub fn blocking_read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        if buffer.len() == 0 {
            return Err(Error::BufferZeroLength);
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let ptr = buffer.as_ptr();
        let len = buffer.len();

        let r = U::regs();

        self.ppi_ch1.enable();

        r.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        r.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

        r.events_endrx.reset();
        r.intenclr.write(|w| w.endrx().clear());

        compiler_fence(Ordering::SeqCst);

        r.tasks_startrx.write(|w| unsafe { w.bits(1) });

        while r.events_endrx.read().bits() == 0 {}

        compiler_fence(Ordering::SeqCst);
        let n = r.rxd.amount.read().amount().bits() as usize;

        self.timer.stop();
        r.events_rxstarted.reset();

        Ok(n)
    }
}
pub(crate) mod sealed {
    use core::sync::atomic::AtomicU8;

    use embassy::waitqueue::AtomicWaker;

    use super::*;

    pub struct State {
        pub endrx_waker: AtomicWaker,
        pub endtx_waker: AtomicWaker,
        pub tx_rx_refcount: AtomicU8,
    }
    impl State {
        pub const fn new() -> Self {
            Self {
                endrx_waker: AtomicWaker::new(),
                endtx_waker: AtomicWaker::new(),
                tx_rx_refcount: AtomicU8::new(0),
            }
        }
    }

    pub trait Instance {
        fn regs() -> &'static pac::uarte0::RegisterBlock;
        fn state() -> &'static State;
    }
}

pub trait Instance: Unborrow<Target = Self> + sealed::Instance + 'static + Send {
    type Interrupt: Interrupt;
}

macro_rules! impl_uarte {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::uarte::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::uarte0::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
            fn state() -> &'static crate::uarte::sealed::State {
                static STATE: crate::uarte::sealed::State = crate::uarte::sealed::State::new();
                &STATE
            }
        }
        impl crate::uarte::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::$irq;
        }
    };
}

// ====================

mod eh02 {
    use super::*;

    impl<'d, T: Instance> embedded_hal_02::blocking::serial::Write<u8> for Uarte<'d, T> {
        type Error = Error;

        fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }

        fn bflush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl<'d, T: Instance> embedded_hal_02::blocking::serial::Write<u8> for UarteTx<'d, T> {
        type Error = Error;

        fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }

        fn bflush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl<'d, U: Instance, T: TimerInstance> embedded_hal_02::blocking::serial::Write<u8>
        for UarteWithIdle<'d, U, T>
    {
        type Error = Error;

        fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }

        fn bflush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }
}

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

    impl embedded_hal_1::serial::Error for Error {
        fn kind(&self) -> embedded_hal_1::serial::ErrorKind {
            match *self {
                Self::BufferTooLong => embedded_hal_1::serial::ErrorKind::Other,
                Self::BufferZeroLength => embedded_hal_1::serial::ErrorKind::Other,
                Self::DMABufferNotInDataMemory => embedded_hal_1::serial::ErrorKind::Other,
            }
        }
    }

    // =====================

    impl<'d, T: Instance> embedded_hal_1::serial::ErrorType for Uarte<'d, T> {
        type Error = Error;
    }

    impl<'d, T: Instance> embedded_hal_1::serial::blocking::Write for Uarte<'d, T> {
        fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl<'d, T: Instance> embedded_hal_1::serial::ErrorType for UarteTx<'d, T> {
        type Error = Error;
    }

    impl<'d, T: Instance> embedded_hal_1::serial::blocking::Write for UarteTx<'d, T> {
        fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl<'d, T: Instance> embedded_hal_1::serial::ErrorType for UarteRx<'d, T> {
        type Error = Error;
    }

    impl<'d, U: Instance, T: TimerInstance> embedded_hal_1::serial::ErrorType
        for UarteWithIdle<'d, U, T>
    {
        type Error = Error;
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(feature = "unstable-traits", feature = "nightly", feature = "_todo_embedded_hal_serial"))] {
        use core::future::Future;

        impl<'d, T: Instance> embedded_hal_async::serial::Read for Uarte<'d, T> {
            type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn read<'a>(&'a mut self, buffer: &'a mut [u8]) -> Self::ReadFuture<'a> {
                self.read(buffer)
            }
        }

        impl<'d, T: Instance> embedded_hal_async::serial::Write for Uarte<'d, T> {
            type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn write<'a>(&'a mut self, buffer: &'a [u8]) -> Self::WriteFuture<'a> {
                self.write(buffer)
            }

            type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
                async move { Ok(()) }
            }
        }

        impl<'d, T: Instance> embedded_hal_async::serial::Write for UarteTx<'d, T> {
            type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn write<'a>(&'a mut self, buffer: &'a [u8]) -> Self::WriteFuture<'a> {
                self.write(buffer)
            }

            type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
                async move { Ok(()) }
            }
        }

        impl<'d, T: Instance> embedded_hal_async::serial::Read for UarteRx<'d, T> {
            type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn read<'a>(&'a mut self, buffer: &'a mut [u8]) -> Self::ReadFuture<'a> {
                self.read(buffer)
            }
        }

        impl<'d, U: Instance, T: TimerInstance> embedded_hal_async::serial::Read
            for UarteWithIdle<'d, U, T>
        {
            type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn read<'a>(&'a mut self, buffer: &'a mut [u8]) -> Self::ReadFuture<'a> {
                self.read(buffer)
            }
        }

        impl<'d, U: Instance, T: TimerInstance> embedded_hal_async::serial::Write
            for UarteWithIdle<'d, U, T>
        {
            type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn write<'a>(&'a mut self, buffer: &'a [u8]) -> Self::WriteFuture<'a> {
                self.write(buffer)
            }

            type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
                async move { Ok(()) }
            }
        }
    }
}
