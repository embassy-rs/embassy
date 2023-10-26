//! NFC Tag Driver

#![macro_use]

use core::future::poll_fn;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::InterruptExt;
// use crate::chip::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE};
use crate::peripherals::NFCT;
use crate::util::slice_in_ram_or;
use crate::{interrupt, pac, Peripheral};

/// Interrupt handler.
pub struct InterruptHandler {
    _private: (),
}

impl interrupt::typelevel::Handler<interrupt::typelevel::NFCT> for InterruptHandler {
    unsafe fn on_interrupt() {
        let r = unsafe { &*pac::NFCT::ptr() };
        if r.events_rxframeend.read().bits() != 0 {
            r.intenclr.write(|w| w.rxframeend().set_bit());
            WAKER.wake();
            info!("NFC Interrupt: rxframeend")
        }

        if r.events_rxerror.read().bits() != 0 {
            r.intenclr.write(|w| w.rxerror().set_bit());
            WAKER.wake();
            info!("NFC Interrupt: rxerror")
        }

        if r.events_endtx.read().bits() != 0 {
            r.intenclr.write(|w| w.endtx().set_bit());
            WAKER.wake();
            info!("NFC Interrupt: endtx")
        }

        if r.events_fielddetected.read().bits() != 0 {
            r.intenclr.write(|w| w.fielddetected().set_bit());
            WAKER.wake();
            info!("NFC Interrupt: fielddetected")
        }

        if r.events_fieldlost.read().bits() != 0 {
            r.intenclr.write(|w| w.fieldlost().set_bit());
            WAKER.wake();
            info!("NFC Interrupt: fieldlost")
        }

        if r.events_ready.read().bits() != 0 {
            r.intenclr.write(|w| w.ready().set_bit());
            WAKER.wake();
            info!("NFC Interrupt: ready")
        }
    }
}

static WAKER: AtomicWaker = AtomicWaker::new();

/// NFC error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Rx Error received while waiting for frame
    RxError,
    /// Rx buffer was overrun, increase your buffer size to resolve this
    RxOverrun,
    /// The buffer is not in data RAM. It's most likely in flash, and nRF's DMA cannot access flash.
    BufferNotInRAM,
}

/// Nfc Tag Read/Writer driver
pub struct NfcT<'d> {
    _p: PeripheralRef<'d, NFCT>,
}

impl<'d> NfcT<'d> {
    /// Create an Nfc Tag driver
    pub fn new(
        _p: impl Peripheral<P = NFCT> + 'd,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::NFCT, InterruptHandler> + 'd,
    ) -> Self {
        into_ref!(_p);

        let r = unsafe { &*pac::NFCT::ptr() };
        // r.intenset.write(|w| unsafe { w.bits(u32::MAX) });
        r.intenset.write(|w| w.ready().set());

        interrupt::NFCT.unpend();
        unsafe { interrupt::NFCT.enable() };

        // r.tasks_activate.write(|w| w.tasks_activate().set_bit());
        Self { _p }
    }

    fn regs() -> &'static pac::nfct::RegisterBlock {
        unsafe { &*pac::NFCT::ptr() }
    }

    fn stop_recv_frame() {
        let r = Self::regs();

        r.intenclr.write(|w| w.rxframeend().set_bit());
        r.intenclr.write(|w| w.rxerror().set_bit());

        compiler_fence(Ordering::SeqCst);

        // FIXME: this might take too long, maybe on start we clear?
        while r.events_rxframeend.read().bits() == 0 && r.events_rxerror.read().bits() == 0 {}
        r.events_rxframeend.reset();
        r.events_rxerror.reset();
    }

    fn stop_tx_frame() {
        let r = Self::regs();
        r.intenclr.write(|w| w.endtx().set_bit());

        compiler_fence(Ordering::SeqCst);

        // FIXME: this might take too long, maybe on start we clear?
        while r.events_endtx.read().bits() == 0 {}
        r.events_endtx.reset();
    }

    /// Checks if field is already present
    pub fn is_field_present(&self) -> bool {
        let r = Self::regs();
        return r.fieldpresent.read().fieldpresent().bit();
    }

    /// Blocks until field-detected event is triggered
    pub async fn wait_for_field(&mut self) {
        let r = Self::regs();

        r.tasks_sense.write(|w| w.tasks_sense().set_bit());
        r.intenset.write(|w| w.fielddetected().set());
        r.intenset.write(|w| w.fieldlost().set());
        poll_fn(|cx| {
            let r = Self::regs();

            WAKER.register(cx.waker());

            if r.events_fielddetected.read().bits() != 0 {
                r.events_fielddetected.reset();
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }

    /// Transmit an NFC frame
    /// `buf` is not pointing to the Data RAM region, an EasyDMA transfer may result in a hard fault or RAM corruption.
    pub async fn tx_frame(&mut self, buf: &[u8]) -> Result<(), Error> {
        slice_in_ram_or(buf, Error::BufferNotInRAM)?;

        let r = Self::regs();

        let on_drop = OnDrop::new(Self::stop_tx_frame);

        //Setup DMA
        r.packetptr.write(|w| unsafe { w.bits(buf.as_ptr() as u32) });
        r.maxlen.write(|w| unsafe { w.bits(buf.len() as _) });

        r.events_endtx.reset();
        r.intenset.write(|w| w.endtx().set());

        // Start enablerxdata only after configs are finished writing
        compiler_fence(Ordering::SeqCst);

        // Enter TX state
        r.tasks_starttx.write(|w| w.tasks_starttx().set_bit());

        // Wait for 'rxframeend'/'rxerror' event.
        poll_fn(|cx| {
            let r = Self::regs();

            WAKER.register(cx.waker());

            if r.events_endtx.read().bits() != 0 {
                r.events_endtx.reset();
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;

        drop(on_drop);
        Ok(())
    }

    /// Waits for a single frame to be loaded into `buf`
    /// `buf` is not pointing to the Data RAM region, an EasyDMA transfer may result in a hard fault or RAM corruption.
    pub async fn recv_frame<'a, const N: usize>(&mut self, buf: &'a mut [u8; N]) -> Result<&'a [u8], Error> {
        slice_in_ram_or(buf, Error::BufferNotInRAM)?;
        let r = Self::regs();

        let on_drop = OnDrop::new(Self::stop_recv_frame);

        //Setup DMA
        r.packetptr.write(|w| unsafe { w.bits(buf.as_mut_ptr() as u32) });
        r.maxlen.write(|w| unsafe { w.bits(N as _) });

        // Reset and enable the end event
        r.events_rxframeend.reset();
        r.events_rxerror.reset();
        r.intenset.write(|w| w.rxframeend().set());
        r.intenset.write(|w| w.rxerror().set());

        // Start enablerxdata only after configs are finished writing
        compiler_fence(Ordering::SeqCst);

        // Enter RX state
        r.tasks_enablerxdata.write(|w| w.tasks_enablerxdata().set_bit());

        // Wait for 'rxframeend'/'rxerror' event.
        let res = poll_fn(|cx| {
            let r = Self::regs();

            WAKER.register(cx.waker());

            if r.events_rxframeend.read().bits() != 0 {
                r.events_rxframeend.reset();
                let amount_read = r.rxd.amount.read().bits() as usize;
                return Poll::Ready(Ok(&buf[..amount_read]));
            }

            if r.events_rxerror.read().bits() != 0 {
                r.events_rxerror.reset();
                // If rx buffer is overrun, rxd.amount will indicate a longer message than maxlen & rxerror will be emitted
                if r.rxd.amount.read().bits() > r.maxlen.read().bits() {
                    return Poll::Ready(Err(Error::RxOverrun));
                }
                return Poll::Ready(Err(Error::RxError));
            }

            Poll::Pending
        })
        .await?;

        drop(on_drop);
        Ok(res)
    }
}
