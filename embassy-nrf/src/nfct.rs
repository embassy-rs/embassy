//! NFC tag emulator driver.
//!
//! This driver implements support for emulating an ISO14443-3 card. Anticollision and selection
//! are handled automatically in hardware, then the driver lets you receive and reply to
//! raw ISO14443-3 frames in software.
//!
//! Higher layers such as ISO14443-4 aka ISO-DEP and ISO7816 must be handled on top
//! in software.

#![macro_use]

use core::future::poll_fn;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;
pub use vals::{Bitframesdd as SddPat, Discardmode as DiscardMode};

use crate::interrupt::InterruptExt;
use crate::pac::nfct::vals;
use crate::pac::NFCT;
use crate::peripherals::NFCT;
use crate::util::slice_in_ram;
use crate::{interrupt, pac, Peri};

/// NFCID1 (aka UID) of different sizes.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum NfcId {
    /// 4-byte UID.
    SingleSize([u8; 4]),
    /// 7-byte UID.
    DoubleSize([u8; 7]),
    /// 10-byte UID.
    TripleSize([u8; 10]),
}

/// The protocol field to be sent in the `SEL_RES` response byte (b6-b7).
#[derive(Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum SelResProtocol {
    /// Configured for Type 2 Tag platform.
    #[default]
    Type2 = 0,
    /// Configured for Type 4A Tag platform, compliant with ISO/IEC_14443.
    Type4A = 1,
    /// Configured for the NFC-DEP Protocol.
    NfcDep = 2,
    /// Configured for the NFC-DEP Protocol and Type 4A Tag platform.
    NfcDepAndType4A = 3,
}

/// Config for the `NFCT` peripheral driver.
#[derive(Clone)]
pub struct Config {
    /// NFCID1 to use during autocollision.
    pub nfcid1: NfcId,
    /// SDD pattern to be sent in `SENS_RES`.
    pub sdd_pat: SddPat,
    /// Platform config to be sent in `SEL_RES`.
    pub plat_conf: u8,
    /// Protocol to be sent in the `SEL_RES` response.
    pub protocol: SelResProtocol,
}

/// Interrupt handler.
pub struct InterruptHandler {
    _private: (),
}

impl interrupt::typelevel::Handler<interrupt::typelevel::NFCT> for InterruptHandler {
    unsafe fn on_interrupt() {
        trace!("irq");
        pac::NFCT.inten().write(|w| w.0 = 0);
        WAKER.wake();
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
    /// Lost field.
    Deactivated,
    /// Collision
    Collision,
    /// The buffer is not in data RAM. It's most likely in flash, and nRF's DMA cannot access flash.
    BufferNotInRAM,
}

/// NFC tag emulator driver.
pub struct NfcT<'d> {
    _p: Peri<'d, NFCT>,
    rx_buf: [u8; 256],
    tx_buf: [u8; 256],
}

impl<'d> NfcT<'d> {
    /// Create an Nfc Tag driver
    pub fn new(
        _p: Peri<'d, NFCT>,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::NFCT, InterruptHandler> + 'd,
        config: &Config,
    ) -> Self {
        let r = pac::NFCT;

        unsafe {
            let reset = (r.as_ptr() as *mut u32).add(0xFFC / 4);
            reset.write_volatile(0);
            reset.read_volatile();
            reset.write_volatile(1);
        }

        let nfcid_size = match &config.nfcid1 {
            NfcId::SingleSize(bytes) => {
                r.nfcid1_last().write(|w| w.0 = u32::from_be_bytes(*bytes));

                vals::Nfcidsize::NFCID1SINGLE
            }
            NfcId::DoubleSize(bytes) => {
                let (bytes, chunk) = bytes.split_last_chunk::<4>().unwrap();
                r.nfcid1_last().write(|w| w.0 = u32::from_be_bytes(*chunk));

                let mut chunk = [0u8; 4];
                chunk[1..].copy_from_slice(bytes);
                r.nfcid1_2nd_last().write(|w| w.0 = u32::from_be_bytes(chunk));

                vals::Nfcidsize::NFCID1DOUBLE
            }
            NfcId::TripleSize(bytes) => {
                let (bytes, chunk) = bytes.split_last_chunk::<4>().unwrap();
                r.nfcid1_last().write(|w| w.0 = u32::from_be_bytes(*chunk));

                let (bytes, chunk2) = bytes.split_last_chunk::<3>().unwrap();
                let mut chunk = [0u8; 4];
                chunk[1..].copy_from_slice(chunk2);
                r.nfcid1_2nd_last().write(|w| w.0 = u32::from_be_bytes(chunk));

                let mut chunk = [0u8; 4];
                chunk[1..].copy_from_slice(bytes);
                r.nfcid1_3rd_last().write(|w| w.0 = u32::from_be_bytes(chunk));

                vals::Nfcidsize::NFCID1TRIPLE
            }
        };

        r.sensres().write(|w| {
            w.set_nfcidsize(nfcid_size);
            w.set_bitframesdd(config.sdd_pat);
            w.set_platfconfig(config.plat_conf & 0xF);
        });

        r.selres().write(|w| {
            w.set_protocol(config.protocol as u8);
        });

        // errata
        #[cfg(feature = "nrf52832")]
        unsafe {
            // Errata 57 nrf52832 only
            //(0x40005610 as *mut u32).write_volatile(0x00000005);
            //(0x40005688 as *mut u32).write_volatile(0x00000001);
            //(0x40005618 as *mut u32).write_volatile(0x00000000);
            //(0x40005614 as *mut u32).write_volatile(0x0000003F);

            // Errata 98
            (0x4000568C as *mut u32).write_volatile(0x00038148);
        }

        r.inten().write(|w| w.0 = 0);

        interrupt::NFCT.unpend();
        unsafe { interrupt::NFCT.enable() };

        // clear all shorts
        r.shorts().write(|_| {});

        let res = Self {
            _p,
            tx_buf: [0u8; 256],
            rx_buf: [0u8; 256],
        };

        assert!(slice_in_ram(&res.tx_buf), "TX Buf not in ram");
        assert!(slice_in_ram(&res.rx_buf), "RX Buf not in ram");

        res
    }

    /// Wait for field on and select.
    ///
    /// This waits for the field to become on, and then for a reader to select us. The ISO14443-3
    /// sense, anticollision and select procedure is handled entirely in hardware.
    ///
    /// When this returns, we have successfully been selected as a card. You must then
    /// loop calling [`receive`](Self::receive) and responding with [`transmit`](Self::transmit).
    pub async fn activate(&mut self) {
        let r = pac::NFCT;
        loop {
            r.events_fieldlost().write_value(0);
            r.events_fielddetected().write_value(0);
            r.tasks_sense().write_value(1);

            // enable autocoll
            #[cfg(not(feature = "nrf52832"))]
            r.autocolresconfig().write(|w| w.0 = 0b10);

            // framedelaymax=4096 is needed to make it work with phones from
            // a certain company named after some fruit.
            r.framedelaymin().write(|w| w.set_framedelaymin(1152));
            r.framedelaymax().write(|w| w.set_framedelaymax(4096));
            r.framedelaymode().write(|w| {
                w.set_framedelaymode(vals::Framedelaymode::WINDOW_GRID);
            });

            info!("waiting for field");
            poll_fn(|cx| {
                WAKER.register(cx.waker());

                if r.events_fielddetected().read() != 0 {
                    r.events_fielddetected().write_value(0);
                    return Poll::Ready(());
                }

                r.inten().write(|w| {
                    w.set_fielddetected(true);
                });
                Poll::Pending
            })
            .await;

            #[cfg(feature = "time")]
            embassy_time::Timer::after_millis(1).await; // workaround errata 190

            r.events_selected().write_value(0);
            r.tasks_activate().write_value(1);

            trace!("Waiting to be selected");
            poll_fn(|cx| {
                let r = pac::NFCT;

                WAKER.register(cx.waker());

                if r.events_selected().read() != 0 || r.events_fieldlost().read() != 0 {
                    return Poll::Ready(());
                }

                r.inten().write(|w| {
                    w.set_selected(true);
                    w.set_fieldlost(true);
                });
                Poll::Pending
            })
            .await;
            if r.events_fieldlost().read() != 0 {
                continue;
            }

            // disable autocoll
            #[cfg(not(feature = "nrf52832"))]
            r.autocolresconfig().write(|w| w.0 = 0b11u32);

            // once anticoll is done, set framedelaymax to the maximum possible.
            // this gives the firmware as much time as possible to reply.
            // higher layer still has to reply faster than the FWT it specifies in the iso14443-4 ATS,
            // but that's not our concern.
            //
            // nrf52832 field is 16bit instead of 20bit. this seems to force a too short timeout, maybe it's a SVD bug?
            #[cfg(not(feature = "nrf52832"))]
            r.framedelaymax().write(|w| w.set_framedelaymax(0xF_FFFF));
            #[cfg(feature = "nrf52832")]
            r.framedelaymax().write(|w| w.set_framedelaymax(0xFFFF));

            return;
        }
    }

    /// Transmit an ISO14443-3 frame to the reader.
    ///
    /// You must call this only after receiving a frame with [`receive`](Self::receive),
    /// and only once. Higher-layer protocols usually define timeouts, so calling this
    /// too late can cause things to fail.
    ///
    /// This will fail with [`Error::Deactivated`] if we have been deselected due to either
    /// the field being switched off or due to the ISO14443 state machine. When this happens,
    /// you must stop calling [`receive`](Self::receive) and [`transmit`](Self::transmit), reset
    /// all protocol state, and go back to calling [`activate`](Self::activate).
    pub async fn transmit(&mut self, buf: &[u8]) -> Result<(), Error> {
        let r = pac::NFCT;

        //Setup DMA
        self.tx_buf[..buf.len()].copy_from_slice(buf);
        r.packetptr().write_value(self.tx_buf.as_ptr() as u32);
        r.maxlen().write(|w| w.0 = buf.len() as _);

        // Set packet length
        r.txd().amount().write(|w| {
            w.set_txdatabits(0);
            w.set_txdatabytes(buf.len() as _);
        });

        r.txd().frameconfig().write(|w| {
            w.set_crcmodetx(true);
            w.set_discardmode(DiscardMode::DISCARD_END);
            w.set_parity(true);
            w.set_sof(true);
        });

        r.events_error().write_value(0);
        r.events_txframeend().write_value(0);
        r.errorstatus().write(|w| w.0 = 0xffff_ffff);

        // Start starttx task
        compiler_fence(Ordering::SeqCst);
        r.tasks_starttx().write_value(1);

        poll_fn(move |cx| {
            trace!("polling tx");
            let r = pac::NFCT;
            WAKER.register(cx.waker());

            if r.events_fieldlost().read() != 0 {
                return Poll::Ready(Err(Error::Deactivated));
            }

            if r.events_txframeend().read() != 0 {
                trace!("Txframend hit, should be finished trasmitting");
                return Poll::Ready(Ok(()));
            }

            if r.events_error().read() != 0 {
                trace!("Got error?");
                let errs = r.errorstatus().read();
                r.errorstatus().write(|w| w.0 = 0xFFFF_FFFF);
                trace!("errors: {:08x}", errs.0);
                r.events_error().write_value(0);
                return Poll::Ready(Err(Error::RxError));
            }

            r.inten().write(|w| {
                w.set_txframeend(true);
                w.set_error(true);
                w.set_fieldlost(true);
            });

            Poll::Pending
        })
        .await
    }

    /// Receive an ISO14443-3 frame from the reader.
    ///
    /// After calling this, you must send back a response with [`transmit`](Self::transmit),
    /// and only once. Higher-layer protocols usually define timeouts, so calling this
    /// too late can cause things to fail.
    pub async fn receive(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let r = pac::NFCT;

        r.rxd().frameconfig().write(|w| {
            w.set_crcmoderx(true);
            w.set_parity(true);
            w.set_sof(true);
        });

        //Setup DMA
        r.packetptr().write_value(self.rx_buf.as_mut_ptr() as u32);
        r.maxlen().write(|w| w.0 = self.rx_buf.len() as _);

        // Reset and enable the end event
        r.events_rxframeend().write_value(0);
        r.events_rxerror().write_value(0);

        // Start enablerxdata only after configs are finished writing
        compiler_fence(Ordering::SeqCst);
        r.tasks_enablerxdata().write_value(1);

        poll_fn(move |cx| {
            trace!("polling rx");
            let r = pac::NFCT;
            WAKER.register(cx.waker());

            if r.events_fieldlost().read() != 0 {
                return Poll::Ready(Err(Error::Deactivated));
            }

            if r.events_rxerror().read() != 0 {
                trace!("RXerror got in recv frame, should be back in idle state");
                r.events_rxerror().write_value(0);
                let errs = r.framestatus().rx().read();
                r.framestatus().rx().write(|w| w.0 = 0xFFFF_FFFF);
                trace!("errors: {:08x}", errs.0);
                return Poll::Ready(Err(Error::RxError));
            }

            if r.events_rxframeend().read() != 0 {
                trace!("RX Frameend got in recv frame, should have data");
                r.events_rxframeend().write_value(0);
                return Poll::Ready(Ok(()));
            }

            r.inten().write(|w| {
                w.set_rxframeend(true);
                w.set_rxerror(true);
                w.set_fieldlost(true);
            });

            Poll::Pending
        })
        .await?;

        let n = r.rxd().amount().read().rxdatabytes() as usize - 2;
        buf[..n].copy_from_slice(&self.rx_buf[..n]);
        Ok(n)
    }
}

/// Wake the system if there if an NFC field close to the antenna
pub fn wake_on_nfc_sense() {
    NFCT.tasks_sense().write_value(0x01);
}
