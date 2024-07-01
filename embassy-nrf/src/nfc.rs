//! NFC Tag Driver

#![macro_use]

use core::future::poll_fn;
use core::sync::atomic::{compiler_fence, AtomicU32, Ordering};
use core::task::Poll;
use core::ops::Range;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::InterruptExt;
// use crate::chip::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE};
use crate::peripherals::NFCT;
use crate::util::slice_in_ram_or;
use crate::{interrupt, pac, Peripheral};

/// Bit frame SDD as defined by the b5:b1 of byte 1 in SENS_RES response in the NFC Forum, NFC Digital 
/// Protocol Technical Specification.
#[derive(Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum SddPat {
    /// SDD pattern 00000.
    Sdd00000 = 0,
    /// SDD pattern 00001.
    #[default]
    Sdd00001 = 1,
    /// SDD pattern 00010.
    Sdd00010 = 2,
    /// SDD pattern 00100.
    Sdd00100 = 4,
    /// SDD pattern 01000.
    Sdd01000 = 8,
    /// SDD pattern 10000.
    Sdd10000 = 16,
}

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

/// Hardware Autocollision config.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct AutoCollConfig {
    /// NFCID1 to use during autocollision.
    pub nfcid1: NfcId,
    /// SDD pattern to be sent in `SENS_RES`.
    pub sdd_pat: SddPat,
    /// Platform config to be sent in `SEL_RES`.
    pub plat_conf: u8,
    /// Protocol to be sent in the `SEL_RES` response.
    pub protocol: SelResProtocol,
}

/// Whether unused bits are discarded at start or end of a frame.
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum DiscardMode {
    /// Unused bits are discarded at end of frame (EoF).
    DiscardEnd = 0,
    /// Unused bits are discarded at start of frame (SoF).
    #[default]
    DiscardStart = 1,
}

/// Specifies CRC mode.
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum CrcMode {
    /// CRC calculation/verification is disabled.
    NoCrc,
    /// Use CRC16.
    #[default]
    Crc16,
}

/// Config for outgoing frames.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TxdFrameConfig {
    /// Indicates if parity is added to the frame.
    pub parity: bool,
    /// Discarding unused bits at start or end of a frame.
    pub discard_mode: DiscardMode,
    /// Add SoF symbol.
    pub add_sof: bool,
    /// CRC mode for outgoing frames.
    pub crc_mode: CrcMode,
}

impl Default for TxdFrameConfig {
    fn default() -> Self {
        TxdFrameConfig {
            parity: true,
            discard_mode: DiscardMode::DiscardStart,
            add_sof: true,
            crc_mode: CrcMode::Crc16,
        }
    }
}

/// Config for incoming frames.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct RxdFrameConfig {
    /// Indicates if parity is expected in the received frames.
    pub parity: bool,
    /// Indicates whether SoF symbol is expected.
    pub add_sof: bool,
    /// CRC mode for incoming frames.
    /// 
    /// When set to [`CrcMode::NoCrc`] no CRC is expected in a frame, otherwise CRC is verified 
    /// and `CRCSTATUS` is updated.
    pub crc_mode: CrcMode,
}

impl Default for RxdFrameConfig {
    fn default() -> Self {
        RxdFrameConfig {
            parity: true,
            add_sof: true,
            crc_mode: CrcMode::Crc16,
        }
    }
}

/// Config for the Frame Delay Timer.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum FrameDelayConfig {
    /// Transmission is independent of frame timer and will start when the `STARTTX`` task is 
    /// triggered. No timeout.
    FreeRun,
    /// Frame is transmitted a range of 13.56 Mhz clocks.
    /// 
    /// The start value should fit in 16 bits, the end value should fit in 20 bits.
    Window(Range<u32>),
    /// Frame is transmitted exactly after a certain amount of 13.56 Mhz clocks.
    /// 
    /// The value should fit in 20 bits.
    ExactVal(u32),
    /// Frame is transmitted on a bit grid between a range of 13.56 Mhz clocks.
    /// 
    /// The start value should fit in 16 bits, the end value should fit in 20 bits.
    WindowGrid(Range<u32>),
}

impl Default for FrameDelayConfig {
    fn default() -> Self {
        FrameDelayConfig::Window(1152..4096)
    }
}

/// Config for shortcuts the `NFCT` peripheral might take.
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ShortsConfig {
    /// Activates a shortcut between the `FIELDDETECTED` event and `ACTIVATE` task.
    pub fielddetected_activate: bool,
    /// Activates a shortcut between the `FIELDLOST` event and `SENSE` task.
    pub fieldlost_sense: bool,
    /// Activates a shortcut between the `TXFRAMEEND` event and `ENABLERXDATA` task.
    pub txframeend_enablerxdata: bool,
}

/// Config for the `NFCT` peripheral driver.
#[derive(Clone, Debug)]
pub struct Config {
    /// Hardware autocollision resolution config.
    /// 
    /// Hardware autocollision resolution is disabled when this is set to [`None`].
    pub autocoll_config: Option<AutoCollConfig>,
    /// Configuration for transmitting frames.
    pub txd_frame_config: TxdFrameConfig,
    /// Configuration for receiving frames.
    pub rxd_frame_config: RxdFrameConfig,
    /// Configuration for the frame delay controller.
    pub frame_delay_config: FrameDelayConfig,
}

/// Interrupt handler.
pub struct InterruptHandler {
    _private: (),
}

impl interrupt::typelevel::Handler<interrupt::typelevel::NFCT> for InterruptHandler {
    unsafe fn on_interrupt() {
        info!("NFC Interrupt entry");

        let r = unsafe { &*pac::NFCT::ptr() };
        let mut wake = false;
        
        let field_detected = r.events_fielddetected.read().bits() != 0;
        if field_detected {
            r.intenclr.write(|w| w.fielddetected().set_bit());
            wake = true;
            info!("NFC Interrupt: fielddetected")
        }

        let field_lost = r.events_fieldlost.read().bits() != 0;
        if field_lost {
            r.intenclr.write(|w| w.fieldlost().set_bit());
            wake = true;
            info!("NFC Interrupt: fieldlost")
        }

        if r.events_rxframestart.read().bits() != 0 {
            r.intenclr.write(|w| w.rxframestart().set_bit());
            r.events_rxframestart.reset();
            wake = true;
            info!("NFC Interrupt: rxframestart")
        }

        if r.events_txframestart.read().bits() != 0 {
            r.intenclr.write(|w| w.txframestart().set_bit());
            r.events_txframestart.reset();
            wake = true;
            info!("NFC Interrupt: txframestart")
        }

        if r.events_rxframeend.read().bits() != 0 {
            r.intenclr.write(|w| w.rxframeend().set_bit());
            wake = true;
            info!("NFC Interrupt: rxframeend")
        }

        if r.events_txframeend.read().bits() != 0 {
            r.intenclr.write(|w| w.txframeend().set_bit());
            wake = true;
            info!("NFC Interrupt: txframeend")
        }

        if r.events_endrx.read().bits() != 0 {
            r.intenclr.write(|w| w.endrx().set_bit());
            wake = true;
            info!("NFC Interrupt: endrx")
        }

        if r.events_endtx.read().bits() != 0 {
            r.intenclr.write(|w| w.endtx().set_bit());
            wake = true;
            info!("NFC Interrupt: endtx")
        }

        if r.events_rxerror.read().bits() != 0 {
            r.intenclr.write(|w| w.rxerror().set_bit());
            wake = true;
            info!("NFC Interrupt: rxerror")
        }

        if r.events_error.read().bits() != 0 {
            r.intenclr.write(|w| w.error().set_bit());
            wake = true;
            info!("NFC Interrupt: error")
        }

        if r.events_ready.read().bits() != 0 {
            r.intenclr.write(|w| w.ready().set_bit());
            wake = true;
            info!("NFC Interrupt: ready")
        }

        if r.events_selected.read().bits() != 0 {
            r.intenclr.write(|w| w.selected().set_bit());
            wake = true;
            info!("NFC Interrupt: selected")
        }

        if r.events_collision.read().bits() != 0 {
            r.intenclr.write(|w| w.collision().set_bit());
            wake = true;
            info!("NFC Interrupt: collision")
        }

        if r.events_started.read().bits() != 0 {
            r.intenclr.write(|w| w.started().set_bit());
            r.events_started.reset();
            wake = true;
            info!("NFC Interrupt: started")
        }

        if wake {
            WAKER.wake();
        }

        r.framedelaymax.write(|w| unsafe { w.bits(FRM_DELAY_MAX.load(Ordering::Relaxed)) });

        info!("NFC Interrupt exit");
    }
}

static WAKER: AtomicWaker = AtomicWaker::new();
static FRM_DELAY_MAX: AtomicU32 = AtomicU32::new(0);

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
    LostField,
    /// Collision
    Collision,
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
        config: &Config,
    ) -> Self {
        into_ref!(_p);

        let r = unsafe { &*pac::NFCT::ptr() };

        match &config.frame_delay_config {
            FrameDelayConfig::FreeRun => r.framedelaymode.write(|w| unsafe { w.bits(0) }),
            FrameDelayConfig::Window(range) => {
                FRM_DELAY_MAX.store(range.start, Ordering::Relaxed);
                r.framedelaymin.write(|w| unsafe { w.bits(range.start) });
                r.framedelaymax.write(|w| unsafe { w.bits(range.end) });
                r.framedelaymode.write(|w| unsafe { w.bits(1) });
            }
            &FrameDelayConfig::ExactVal(val) => {
                FRM_DELAY_MAX.store(val, Ordering::Relaxed);
                r.framedelaymax.write(|w| unsafe { w.bits(val) });
                r.framedelaymode.write(|w| unsafe { w.bits(2) });
            }
            FrameDelayConfig::WindowGrid(range) => {
                FRM_DELAY_MAX.store(range.start, Ordering::Relaxed);
                r.framedelaymin.write(|w| unsafe { w.bits(range.start) });
                r.framedelaymax.write(|w| unsafe { w.bits(range.end) });
                r.framedelaymode.write(|w| unsafe { w.bits(3) });
            }
        }

        if let Some(autocoll_config) = config.autocoll_config.as_ref() {
            Self::set_autocoll_cfg(autocoll_config);
        } else {
            r.autocolresconfig.write(|w| unsafe {
                w.bits(0b11u32)
            });
        }

        // errata
        unsafe {
            // Errata 57 nrf52832 only
            //(0x40005610 as *mut u32).write_volatile(0x00000005);
            //(0x40005688 as *mut u32).write_volatile(0x00000001);
            //(0x40005618 as *mut u32).write_volatile(0x00000000);
            //(0x40005614 as *mut u32).write_volatile(0x0000003F);

            // Errata 98
            (0x4000568C as *mut u32).write_volatile(0x00038148);
        }

        // TODO: other configs

        r.intenclr.write(|w| {
            w.fielddetected().set_bit();
            w.fieldlost().set_bit();
            w.txframestart().set_bit();
            w.txframeend().set_bit();
            w.rxframestart().set_bit();
            w.rxframeend().set_bit();
            w.error().set_bit();
            w.rxerror().set_bit();
            w.endtx().set_bit();
            w.endrx().set_bit();
            w.autocolresstarted().set_bit();
            w.collision().set_bit();
            w.selected().set_bit();
            w.started().set_bit();
            w
        });

        r.events_autocolresstarted.reset();
        r.events_collision.reset();
        r.events_endrx.reset();
        r.events_endtx.reset();
        r.events_error.reset();
        r.events_fielddetected.reset();
        r.events_fieldlost.reset();
        r.events_ready.reset();
        r.events_rxerror.reset();
        r.events_rxframeend.reset();
        r.events_rxframestart.reset();
        r.events_txframeend.reset();
        r.events_txframestart.reset();
        r.events_selected.reset();
        r.events_started.reset();

        compiler_fence(Ordering::SeqCst);

        interrupt::NFCT.unpend();
        unsafe { interrupt::NFCT.enable() };


        r.intenset.write(|w| {
            w
                .ready().set()
                .txframestart().set()
                .txframeend().set()
                .rxframestart().set()
                .rxframeend().set()
                .fielddetected().set()
                .fieldlost().set()
                .error().set()
                .rxerror().set()
        });
        r.tasks_activate.write(|w| w.tasks_activate().set_bit());

        // r.tasks_activate.write(|w| w.tasks_activate().set_bit());
        Self { _p, tx_buf: [0u8; 256], rx_buf: [0u8; 256] }
    }

    fn regs() -> &'static pac::nfct::RegisterBlock {
        unsafe { &*pac::NFCT::ptr() }
    }

    /// Checks if field is already present
    pub fn is_field_present(&self) -> bool {
        let r = Self::regs();
        return r.fieldpresent.read().fieldpresent().bit();
    }

    /// Blocks until field-detected event is triggered
    pub fn sense(&mut self) {
        let r = Self::regs();
        r.tasks_sense.write(|w| w.tasks_sense().set_bit());
    }

    /// Blocks until ready event is triggered
    pub async fn wait_for_active(&mut self) {
        let r = Self::regs();

        r.intenset.write(|w| w.ready().set());
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

    /// Blocks until ready event is triggered
    pub async fn wait_for_coll(&mut self) -> Result<(), Error> {
        let r = Self::regs();

        critical_section::with(|_sect| {
            r.events_selected.reset();
            r.events_collision.reset();
            r.events_fieldlost.reset();

            compiler_fence(Ordering::SeqCst);

            r.intenset.write(|w| w
                .selected().set()
                .collision().set()
                .fieldlost().set()
            );

            r.tasks_activate.write(|w| w.tasks_activate().set_bit());
        });

        poll_fn(|cx| {
            let r = Self::regs();

            WAKER.register(cx.waker());

            critical_section::with(|_sect| {
                if r.events_fieldlost.read().bits() != 0 {
                    r.events_fieldlost.reset();

                    return Poll::Ready(Err(Error::LostField));
                }

                if r.events_collision.read().bits() != 0 {
                    r.events_selected.reset();
                    r.events_collision.reset();

                    return Poll::Ready(Err(Error::Collision));
                }

                if r.events_selected.read().bits() != 0 {
                    r.events_selected.reset();
                    r.events_fieldlost.reset();

                    // clear other events as well
                    r.events_collision.reset();
                    r.events_fielddetected.reset();
                    r.events_rxframestart.reset();
                    r.events_rxframeend.reset();
                    r.events_rxerror.reset();
                    r.events_txframestart.reset();
                    r.events_txframeend.reset();

                    r.framedelaymax.write(|w| unsafe { w.bits(FRM_DELAY_MAX.load(Ordering::Relaxed)) });

                    return Poll::Ready(Ok(()));
                }

                Poll::Pending
            })
        })
        .await
    }

    /// Blocks until ready event is triggered
    pub async fn activate(&mut self) {
        let r = Self::regs();

        r.intenset.write(|w| w.ready().set());
        r.tasks_activate.write(|w| w.tasks_activate().set_bit());
        poll_fn(|cx| {
            let r = Self::regs();

            WAKER.register(cx.waker());

            if r.events_ready.read().bits() != 0 {
                r.events_ready.reset();
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }

    pub async fn tx_frame(&mut self, buf: &[u8], bits: u8) -> Result<(), Error> {
        self.tx_frame_with_config(buf, bits, TxdFrameConfig::default()).await
    }

    /// Transmit an NFC frame
    /// `buf` is not pointing to the Data RAM region, an EasyDMA transfer may result in a hard fault or RAM corruption.
    pub async fn tx_frame_with_config(&mut self, buf: &[u8], bits: u8, config: TxdFrameConfig) -> Result<(), Error> {
        //slice_in_ram_or(buf, Error::BufferNotInRAM)?;
        assert!(bits < 8);

        self.tx_buf[..buf.len()].copy_from_slice(buf);

        let r = Self::regs();

        let _on_drop = OnDrop::new(|| {
            r.intenclr.write(|w| w
                .txframestart().set_bit()
                .txframeend().set_bit()
                .error().set_bit()
                .fieldlost().set_bit()
            );
        });

        critical_section::with(|_sect| {
            //unsafe { (0x40005010 as *mut u32).write_volatile(1) };

            //Setup DMA
            r.packetptr.write(|w| unsafe { w.bits(self.tx_buf.as_ptr() as u32) });
            r.maxlen.write(|w| unsafe { w.bits(self.tx_buf.len() as _) });
            r.txd.amount.write(|w| unsafe {
                w.txdatabits().bits(bits);
                w.txdatabytes().bits(buf.len() as _);
                w
            });

            r.framedelaymode.write(|w| w.framedelaymode().bits(3));
            r.framedelaymin.write(|w| unsafe { w.bits(0x480) });
            r.framedelaymax.write(|w| unsafe { w.bits(0x1000) });

            r.txd.frameconfig.write(|w| 
                w
                    .crcmodetx().bit(config.crc_mode == CrcMode::Crc16)
                    .discardmode().bit(config.discard_mode == DiscardMode::DiscardStart)
                    .parity().bit(config.parity)
                    .sof().bit(config.add_sof)
            );

            r.events_txframestart.reset();
            r.events_txframeend.reset();
            r.events_error.reset();
            r.events_fieldlost.reset();

            r.errorstatus.write(|w| w.framedelaytimeout().bit(true));

            compiler_fence(Ordering::SeqCst);

            r.intenset.write(|w| w
                .txframestart().set()
                .txframeend().set()
                .error().set()
                .fieldlost().set()
            );

            compiler_fence(Ordering::SeqCst);

            trace!("nfctagstate before is {}", r.nfctagstate.read().bits());

            // Enter TX state
            r.tasks_starttx.write(|w| w.tasks_starttx().set_bit());
        });

        trace!("nfctagstate after is {}", r.nfctagstate.read().bits());

        poll_fn(|cx| {
            let r = Self::regs();

            WAKER.register(cx.waker());

            trace!("polling tx {}", r.nfctagstate.read().bits());

            critical_section::with(|_sect| {
                r.events_txframestart.reset();

                let mut finished = false;
                if r.events_fieldlost.read().bits() != 0 {
                    trace!("finished tx due to fieldlost");
                    r.events_fieldlost.reset();

                    unsafe { (0x40005010 as *mut u32).write_volatile(1) };

                    return Poll::Ready(Err(Error::LostField));
                }

                if r.events_txframeend.read().bits() != 0 {
                    trace!("clearing txframeend");
                    r.events_txframeend.reset();
                    finished = true;
                }

                if r.events_error.read().bits() != 0 {
                    trace!("clearing error");
                    r.events_error.reset();
                    finished = true;
                }

                if finished {
                    trace!("finished tx");

                    unsafe { (0x40005010 as *mut u32).write_volatile(1) };

                    Poll::Ready(Ok(()))
                } else {
                    trace!("tx pending");

                    Poll::Pending
                }
            })
        })
        .await?;

        Ok(())
    }

    pub async fn recv_frame<'a>(&mut self, buf: &'a mut [u8]) -> Result<(&'a [u8], u8), Error> {
        self.recv_frame_with_cfg(buf, Default::default()).await
    }

    /// Waits for a single frame to be loaded into `buf`
    /// `buf` is not pointing to the Data RAM region, an EasyDMA transfer may result in a hard fault or RAM corruption.
    pub async fn recv_frame_with_cfg<'a>(&mut self, buf: &'a mut [u8], cfg: RxdFrameConfig) -> Result<(&'a [u8], u8), Error> {
        let r = Self::regs();

        let _on_drop = OnDrop::new(|| {
            r.intenclr.write(|w| w
                .rxframestart().set_bit()
                .rxframeend().set_bit()
                .rxerror().set_bit()
                .fieldlost().set_bit()
            );
        });

        critical_section::with(|_sect| {
            r.rxd.frameconfig.write(|w| w
                .parity().bit(cfg.parity)
                .sof().bit(cfg.add_sof)
                .crcmoderx().bit(cfg.crc_mode == CrcMode::Crc16)
            );

            //Setup DMA
            r.packetptr.write(|w| unsafe { w.bits(self.rx_buf.as_mut_ptr() as u32) });
            r.maxlen.write(|w| unsafe { w.bits(self.rx_buf.len() as _) });

            // clear errors
            r.framestatus.rx.write(|w| w
                .crcerror().bit(true)
                .paritystatus().bit(true)
                .overrun().bit(true)
            );
            r.errorstatus.write(|w| w.framedelaytimeout().bit(true) );
            
            compiler_fence(Ordering::SeqCst);

            // Reset and enable the end event
            r.events_rxframeend.reset();
            r.events_rxerror.reset();
            r.events_fieldlost.reset();
            
            compiler_fence(Ordering::SeqCst);

            r.intenset.write(|w| w
                .rxframestart().set()
                .rxframeend().set()
                .rxerror().set()
                .fieldlost().set()
            );

            // Start enablerxdata only after configs are finished writing
            compiler_fence(Ordering::SeqCst);

            // Enter RX state
            r.tasks_enablerxdata.write(|w| w.tasks_enablerxdata().set_bit());
        });

        trace!("waiting for rx event");

        // Wait for 'rxframeend'/'rxerror' event.
        let (bytes, bits) = poll_fn(move |cx| {
            trace!("polling rx");

            let r = Self::regs();

            WAKER.register(cx.waker());

            critical_section::with(|_sect| {
                r.events_txframestart.reset();

                if r.events_fieldlost.read().bits() != 0 {
                    trace!("{} {}", r.events_fielddetected.read().bits(), r.fieldpresent.read().bits());
                    if r.events_fielddetected.read().bits() == 0 || r.fieldpresent.read().fieldpresent().bit_is_clear() {
                        r.events_fieldlost.reset();
                        r.events_endrx.reset();
                        r.events_rxframeend.reset();
                        r.events_rxerror.reset();
                        return Poll::Ready(Err(Error::LostField));
                    }
                }

                if r.events_rxerror.read().bits() != 0 {
                    r.events_endrx.reset();
                    r.events_rxframeend.reset();
                    r.events_rxerror.reset();

                    let framestatus = r.framestatus.rx.read();
                    let crc_error = framestatus.crcerror().bit();
                    let parity_status = framestatus.paritystatus().bit();
                    let overrun_status = framestatus.overrun().bit();
                    error!("rx error (crc {} parity {} overrun {})", crc_error, parity_status, overrun_status);
                    r.framestatus.rx.write(|w| w
                        .crcerror().bit(true)
                        .paritystatus().bit(true)
                        .overrun().bit(true)
                    );

                    if overrun_status {
                        return Poll::Ready(Err(Error::RxOverrun));
                    }
                    return Poll::Ready(Err(Error::RxError));
                }

                let mut finished = false;

                if r.events_endrx.read().bits() != 0 {
                    r.events_endrx.reset();
                    finished = true;

                    trace!("cleared endrx");
                }

                if r.events_rxframeend.read().bits() != 0 {
                    r.events_rxframeend.reset();
                    finished = true;

                    trace!("cleared rxframeend");
                }

                if finished {
                    let rxd_amount = r.rxd.amount.read();
                    let amount_read = rxd_amount.rxdatabytes().bits() as usize;
                    let amount_read_bits = rxd_amount.rxdatabits().bits() as usize;
                    let byte_size = (amount_read & 0xFF) + ((amount_read_bits + 7) / 8);

                    trace!("amount {} {} {}", amount_read, amount_read_bits, byte_size);
                    if amount_read > 257 {
                        error!("applying fixup");
                    }

                    Poll::Ready(Ok((byte_size, amount_read_bits as u8)))
                } else {
                    Poll::Pending
                }
            })
        })
        .await?;

        buf[..bytes].copy_from_slice(&self.rx_buf[..bytes]);

        Ok((&buf[..bytes], bits))
    }

    /// Sets the hardware auto collision resolution config.
    fn set_autocoll_cfg(config: &AutoCollConfig) {
        let r = Self::regs();

        let nfcid_size = match &config.nfcid1 {
            NfcId::SingleSize(bytes) => {
                r.nfcid1_last.write(|w| unsafe {
                    // First byte goes to the top.
                    w.bits(u32::from_be_bytes(*bytes))
                });

                0
            },
            NfcId::DoubleSize(bytes) => {
                let (bytes, chunk) = bytes.split_last_chunk::<4>().unwrap();
                r.nfcid1_last.write(|w| unsafe {
                    // First byte goes to the top.
                    w.bits(u32::from_be_bytes(*chunk))
                });

                let mut chunk = [0u8; 4];
                chunk[1..].copy_from_slice(bytes);
                r.nfcid1_2nd_last.write(|w| unsafe {
                    // First byte goes to the top.
                    w.bits(u32::from_be_bytes(chunk))
                });

                1
            },
            NfcId::TripleSize(bytes) => {
                let (bytes, chunk) = bytes.split_last_chunk::<4>().unwrap();
                r.nfcid1_last.write(|w| unsafe {
                    // First byte goes to the top.
                    w.bits(u32::from_be_bytes(*chunk))
                });

                let (bytes, chunk2) = bytes.split_last_chunk::<3>().unwrap();
                let mut chunk = [0u8; 4];
                chunk[1..].copy_from_slice(chunk2);
                r.nfcid1_2nd_last.write(|w| unsafe {
                    // First byte goes to the top.
                    w.bits(u32::from_be_bytes(chunk))
                });

                let mut chunk = [0u8; 4];
                chunk[1..].copy_from_slice(bytes);
                r.nfcid1_3rd_last.write(|w| unsafe {
                    // First byte goes to the top.
                    w.bits(u32::from_be_bytes(chunk))
                });

                2
            },
        };

        r.sensres.write(|w| unsafe {
            w.nfcidsize().bits(nfcid_size);
            w.bitframesdd().bits(config.sdd_pat as u8);
            w.platfconfig().bits(config.plat_conf & 0xF);
            w
        });

        r.selres.write(|w| unsafe {
            w.protocol().bits(config.protocol as u8);
            w
        });

        r.autocolresconfig.write(|w| unsafe {
            w.bits(0u32)
        });
    }

    /// Sets up shortcuts used by the NFCT peripheral.
    pub fn setup_shorts(&mut self, config: ShortsConfig) {
        let r = Self::regs();
        r.shorts.write(|w| unsafe { w.bits(
            (config.fielddetected_activate as u32) 
            | ((config.fieldlost_sense as u32) << 1) 
            | ((config.txframeend_enablerxdata as u32) << 5)
        ) });
    }

    /// Requests to enter the `SLEEP_A`` state.
    pub fn sleep(&mut self) {
        let r = Self::regs();
        r.tasks_gosleep.write(|w| unsafe { w.bits(1) });
    }

    /// Requests to enter the `IDLE`` state.
    pub fn idle(&mut self) {
        let r = Self::regs();
        r.tasks_goidle.write(|w| unsafe { w.bits(1) });
    }
}
