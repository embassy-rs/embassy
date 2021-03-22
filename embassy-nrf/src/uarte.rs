//! Async UART

use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;
use embassy::traits::uart::{Error, Read, Write};
use embassy::util::{wake_on_interrupt, OnDrop, PeripheralBorrow};
use embassy_extras::unborrow;
use futures::future::poll_fn;

use crate::fmt::{assert, *};
use crate::gpio::sealed::Pin as _;
use crate::gpio::{OptionalPin as GpioOptionalPin, Pin as GpioPin};
use crate::hal::pac;
use crate::hal::target_constants::EASY_DMA_SIZE;
use crate::interrupt;
use crate::interrupt::Interrupt;
use crate::peripherals;

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

/// Interface to the UARTE peripheral
pub struct Uarte<'d, T: Instance> {
    peri: T,
    irq: T::Interrupt,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Uarte<'d, T> {
    /// Creates the interface to a UARTE instance.
    /// Sets the baud rate, parity and assigns the pins to the UARTE peripheral.
    ///
    /// # Safety
    ///
    /// The returned API is safe unless you use `mem::forget` (or similar safe mechanisms)
    /// on stack allocated buffers which which have been passed to [`send()`](Uarte::send)
    /// or [`receive`](Uarte::receive).
    #[allow(unused_unsafe)]
    pub unsafe fn new(
        uarte: impl PeripheralBorrow<Target = T> + 'd,
        irq: impl PeripheralBorrow<Target = T::Interrupt> + 'd,
        rxd: impl PeripheralBorrow<Target = impl GpioPin> + 'd,
        txd: impl PeripheralBorrow<Target = impl GpioPin> + 'd,
        cts: impl PeripheralBorrow<Target = impl GpioOptionalPin> + 'd,
        rts: impl PeripheralBorrow<Target = impl GpioOptionalPin> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(uarte, irq, rxd, txd, cts, rts);

        let r = uarte.regs();

        assert!(r.enable.read().enable().is_disabled());

        rxd.conf().write(|w| w.input().connect().drive().h0h1());
        r.psel.rxd.write(|w| unsafe { w.bits(rxd.psel_bits()) });

        txd.set_high();
        txd.conf().write(|w| w.dir().output().drive().h0h1());
        r.psel.txd.write(|w| unsafe { w.bits(txd.psel_bits()) });

        if let Some(pin) = rts.pin_mut() {
            pin.set_high();
            pin.conf().write(|w| w.dir().output().drive().h0h1());
        }
        r.psel.cts.write(|w| unsafe { w.bits(cts.psel_bits()) });

        if let Some(pin) = cts.pin_mut() {
            pin.conf().write(|w| w.input().connect().drive().h0h1());
        }
        r.psel.rts.write(|w| unsafe { w.bits(rts.psel_bits()) });

        r.baudrate.write(|w| w.baudrate().variant(config.baudrate));
        r.config.write(|w| w.parity().variant(config.parity));

        // Enable
        r.enable.write(|w| w.enable().enabled());

        Self {
            peri: uarte,
            irq,
            phantom: PhantomData,
        }
    }
}

impl<'d, T: Instance> Drop for Uarte<'d, T> {
    fn drop(&mut self) {
        let r = self.peri.regs();
        r.enable.write(|w| w.enable().disabled());

        // todo disable pins
    }
}

impl<'d, T: Instance> Read for Uarte<'d, T> {
    #[rustfmt::skip]
    type ReadFuture<'a> where Self: 'a = impl Future<Output = Result<(), Error>> + 'a;

    fn read<'a>(self: Pin<&'a mut Self>, rx_buffer: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move {
            let this = unsafe { self.get_unchecked_mut() };

            let ptr = rx_buffer.as_ptr();
            let len = rx_buffer.len();
            assert!(len <= EASY_DMA_SIZE);

            let r = this.peri.regs();

            let drop = OnDrop::new(move || {
                info!("read drop: stopping");

                r.intenclr.write(|w| w.endrx().clear());
                r.tasks_stoprx.write(|w| unsafe { w.bits(1) });

                // TX is stopped almost instantly, spinning is fine.
                while r.events_endrx.read().bits() == 0 {}
                info!("read drop: stopped");
            });

            r.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
            r.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

            r.events_endrx.reset();
            r.intenset.write(|w| w.endrx().set());

            compiler_fence(Ordering::SeqCst);

            trace!("startrx");
            r.tasks_startrx.write(|w| unsafe { w.bits(1) });

            let irq = &mut this.irq;
            poll_fn(|cx| {
                if r.events_endrx.read().bits() != 0 {
                    r.events_endrx.reset();
                    return Poll::Ready(());
                }

                wake_on_interrupt(irq, cx.waker());

                Poll::Pending
            })
            .await;

            compiler_fence(Ordering::SeqCst);
            r.intenclr.write(|w| w.endrx().clear());
            drop.defuse();

            Ok(())
        }
    }
}

impl<'d, T: Instance> Write for Uarte<'d, T> {
    #[rustfmt::skip]
    type WriteFuture<'a> where Self: 'a = impl Future<Output = Result<(), Error>> + 'a;

    fn write<'a>(self: Pin<&'a mut Self>, tx_buffer: &'a [u8]) -> Self::WriteFuture<'a> {
        async move {
            let this = unsafe { self.get_unchecked_mut() };

            let ptr = tx_buffer.as_ptr();
            let len = tx_buffer.len();
            assert!(len <= EASY_DMA_SIZE);
            // TODO: panic if buffer is not in SRAM

            let r = this.peri.regs();

            let drop = OnDrop::new(move || {
                info!("write drop: stopping");

                r.intenclr.write(|w| w.endtx().clear());
                r.tasks_stoptx.write(|w| unsafe { w.bits(1) });

                // TX is stopped almost instantly, spinning is fine.
                while r.events_endtx.read().bits() == 0 {}
                info!("write drop: stopped");
            });

            r.txd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
            r.txd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

            r.events_endtx.reset();
            r.intenset.write(|w| w.endtx().set());

            compiler_fence(Ordering::SeqCst);

            trace!("starttx");
            r.tasks_starttx.write(|w| unsafe { w.bits(1) });

            let irq = &mut this.irq;
            poll_fn(|cx| {
                if r.events_endtx.read().bits() != 0 {
                    r.events_endtx.reset();
                    return Poll::Ready(());
                }

                wake_on_interrupt(irq, cx.waker());

                Poll::Pending
            })
            .await;

            compiler_fence(Ordering::SeqCst);
            r.intenclr.write(|w| w.endtx().clear());
            drop.defuse();

            Ok(())
        }
    }
}

/*
/// Future for the [`Uarte::send()`] method.
pub struct SendFuture<'a, T>
where
    T: Instance,
{
    uarte: &'a mut Uarte<T>,
    buf: &'a [u8],
}

impl<'a, T> Drop for SendFuture<'a, T>
where
    T: Instance,
{
    fn drop(self: &mut Self) {
        if self.uarte.tx_started() {
            trace!("stoptx");

            // Stop the transmitter to minimize the current consumption.
            self.uarte.peri.events_txstarted.reset();
            self.uarte.peri.tasks_stoptx.write(|w| unsafe { w.bits(1) });

            // TX is stopped almost instantly, spinning is fine.
            while !T::state().tx_done.signaled() {}
        }
    }
}

/// Future for the [`Uarte::receive()`] method.
pub struct ReceiveFuture<'a, T>
where
    T: Instance,
{
    uarte: &'a mut Uarte<T>,
    buf: &'a mut [u8],
}

impl<'a, T> Drop for ReceiveFuture<'a, T>
where
    T: Instance,
{
    fn drop(self: &mut Self) {
        if self.uarte.rx_started() {
            trace!("stoprx (drop)");

            self.uarte.peri.events_rxstarted.reset();
            self.uarte.peri.tasks_stoprx.write(|w| unsafe { w.bits(1) });

            embassy_extras::low_power_wait_until(|| T::state().rx_done.signaled())
        }
    }
}

impl<'a, T> Future for ReceiveFuture<'a, T>
where
    T: Instance,
{
    type Output = Result<(), embassy::traits::uart::Error>;

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { uarte, buf } = unsafe { self.get_unchecked_mut() };

        match T::state().rx_done.poll_wait(cx) {
            Poll::Pending if !uarte.rx_started() => {
                let ptr = buf.as_ptr();
                let len = buf.len();
                assert!(len <= EASY_DMA_SIZE);

                uarte.enable();

                compiler_fence(Ordering::SeqCst);
                r.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
                r.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

                trace!("startrx");
                uarte.peri.tasks_startrx.write(|w| unsafe { w.bits(1) });
                while !uarte.rx_started() {} // Make sure reception has started

                Poll::Pending
            }
            Poll::Pending => Poll::Pending,
            Poll::Ready(_) => Poll::Ready(Ok(())),
        }
    }
}

/// Future for the [`receive()`] method.
impl<'a, T> ReceiveFuture<'a, T>
where
    T: Instance,
{
    /// Stops the ongoing reception and returns the number of bytes received.
    pub async fn stop(self) -> usize {
        let len = if self.uarte.rx_started() {
            trace!("stoprx (stop)");

            self.uarte.peri.events_rxstarted.reset();
            self.uarte.peri.tasks_stoprx.write(|w| unsafe { w.bits(1) });
            T::state().rx_done.wait().await
        } else {
            // Transfer was stopped before it even started. No bytes were sent.
            0
        };
        len as _
    }
}
 */

mod sealed {
    use super::*;

    pub trait Instance {
        fn regs(&self) -> &pac::uarte0::RegisterBlock;
    }
}

pub trait Instance: sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

macro_rules! make_impl {
    ($type:ident, $irq:ident) => {
        impl sealed::Instance for peripherals::$type {
            fn regs(&self) -> &pac::uarte0::RegisterBlock {
                unsafe { &*pac::$type::ptr() }
            }
        }
        impl Instance for peripherals::$type {
            type Interrupt = interrupt::$irq;
        }
    };
}

make_impl!(UARTE0, UARTE0_UART0);
#[cfg(any(feature = "52833", feature = "52840", feature = "9160"))]
make_impl!(UARTE1, UARTE1);
