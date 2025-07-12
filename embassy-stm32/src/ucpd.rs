//! USB Type-C/USB Power Delivery Interface (UCPD)

// Implementation Notes
//
// As of Feb. 2024 the UCPD peripheral is availalbe on: G0, G4, H5, L5, U5
//
// Cube HAL LL Driver (g0):
// https://github.com/STMicroelectronics/stm32g0xx_hal_driver/blob/v1.4.6/Inc/stm32g0xx_ll_ucpd.h
// https://github.com/STMicroelectronics/stm32g0xx_hal_driver/blob/v1.4.6/Src/stm32g0xx_ll_ucpd.c
// Except for a the `LL_UCPD_RxAnalogFilterEnable/Disable()` functions the Cube HAL implementation of
// all families is the same.
//
// Dead battery pull-down resistors functionality is enabled by default on startup and must
// be disabled by setting a bit in PWR/SYSCFG registers. The exact name and location for that
// bit is different for each familily.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::{ChannelAndRequest, TransferOptions};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::ucpd::vals::{Anamode, Ccenable, PscUsbpdclk, Txmode};
pub use crate::pac::ucpd::vals::{Phyccsel as CcSel, Rxordset, TypecVstateCc as CcVState};
use crate::rcc::{self, RccPeripheral};
use crate::{interrupt, Peri};

pub(crate) fn init(
    _cs: critical_section::CriticalSection,
    #[cfg(peri_ucpd1)] ucpd1_db_enable: bool,
    #[cfg(peri_ucpd2)] ucpd2_db_enable: bool,
) {
    #[cfg(stm32g0x1)]
    {
        // according to RM0444 (STM32G0x1) section 8.1.1:
        // when UCPD is disabled setting the strobe will disable dead battery
        // (which is enabled after reset) but if UCPD is enabled, setting the
        // strobe will apply the CC pin configuration from the control register
        // (which is why we need to be careful about when we call this)
        crate::pac::SYSCFG.cfgr1().modify(|w| {
            w.set_ucpd1_strobe(!ucpd1_db_enable);
            w.set_ucpd2_strobe(!ucpd2_db_enable);
        });
    }

    #[cfg(any(stm32g4, stm32l5))]
    {
        crate::pac::PWR.cr3().modify(|w| {
            #[cfg(stm32g4)]
            w.set_ucpd1_dbdis(!ucpd1_db_enable);
            #[cfg(stm32l5)]
            w.set_ucpd_dbdis(!ucpd1_db_enable);
        })
    }

    #[cfg(any(stm32h5, stm32u5, stm32h7rs))]
    {
        crate::pac::PWR.ucpdr().modify(|w| {
            w.set_ucpd_dbdis(!ucpd1_db_enable);
        })
    }
}

/// Pull-up or Pull-down resistor state of both CC lines.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CcPull {
    /// Analog PHY for CC pin disabled.
    Disabled,

    /// Rd=5.1k pull-down resistor.
    Sink,

    /// Rp=56k pull-up resistor to indicate default USB power.
    SourceDefaultUsb,

    /// Rp=22k pull-up resistor to indicate support for up to 1.5A.
    Source1_5A,

    /// Rp=10k pull-up resistor to indicate support for up to 3.0A.
    Source3_0A,
}

/// UCPD configuration
#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub struct Config {
    /// Receive SOP packets
    pub sop: bool,
    /// Receive SOP' packets
    pub sop_prime: bool,
    /// Receive SOP'' packets
    pub sop_double_prime: bool,
    /// Receive SOP'_Debug packets
    pub sop_prime_debug: bool,
    /// Receive SOP''_Debug packets
    pub sop_double_prime_debug: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sop: true,
            sop_prime: false,
            sop_double_prime: false,
            sop_prime_debug: false,
            sop_double_prime_debug: false,
        }
    }
}

/// UCPD driver.
pub struct Ucpd<'d, T: Instance> {
    cc_phy: CcPhy<'d, T>,
}

impl<'d, T: Instance> Ucpd<'d, T> {
    /// Creates a new UCPD driver instance.
    pub fn new(
        _peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        cc1: Peri<'d, impl Cc1Pin<T>>,
        cc2: Peri<'d, impl Cc2Pin<T>>,
        config: Config,
    ) -> Self {
        cc1.set_as_analog();
        cc2.set_as_analog();

        rcc::enable_and_reset::<T>();
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let r = T::REGS;

        #[cfg(stm32h5)]
        r.cfgr2().write(|w| {
            // Only takes effect, when UCPDEN=0.
            w.set_rxafilten(true);
        });

        r.cfgr1().write(|w| {
            // "The receiver is designed to work in the clock frequency range from 6 to 18 MHz.
            // However, the optimum performance is ensured in the range from 6 to 12 MHz"
            // UCPD is driven by HSI16 (16MHz internal oscillator), which we need to divide by 2.
            w.set_psc_usbpdclk(PscUsbpdclk::DIV2);

            // Prescaler to produce a target half-bit frequency of 600kHz which is required
            // to produce transmit with a nominal nominal bit rate of 300Kbps+-10% using
            // biphase mark coding (BMC, aka differential manchester coding).
            // A divider of 13 gives the target frequency closest to spec (~615kHz, 1.625us).
            w.set_hbitclkdiv(13 - 1);

            // Time window for detecting non-idle (12-20us).
            // 1.75us * 8 = 14us.
            w.set_transwin(8 - 1);

            // Time from the end of last bit of a Frame until the start of the first bit of the
            // next Preamble (min 25us).
            // 1.75us * 17 = ~30us
            w.set_ifrgap(17 - 1);

            // UNDOCUMENTED: This register can only be written while UCPDEN=0 (found by testing).
            let rxordset = (config.sop as u16) << 0
                | (config.sop_prime as u16) << 1
                | (config.sop_double_prime as u16) << 2
                // Hard reset
                | 0x1 << 3
                | (config.sop_prime_debug as u16) << 4
                | (config.sop_double_prime_debug as u16) << 5;
            w.set_rxordseten(rxordset);

            // Enable DMA
            w.set_txdmaen(true);
            w.set_rxdmaen(true);

            w.set_ucpden(true);
        });

        // Software trim according to RM0481, p. 2650/2668
        #[cfg(stm32h5)]
        {
            let trim_rd_cc1 = unsafe { *(0x4002_242C as *const u32) & 0xF };
            let trim_rd_cc2 = unsafe { ((*(0x4002_242C as *const u32)) >> 8) & 0xF };

            r.cfgr3().write(|w| {
                w.set_trim_cc1_rd(trim_rd_cc1 as u8);
                w.set_trim_cc2_rd(trim_rd_cc2 as u8);
            });
        }

        Self {
            cc_phy: CcPhy { _lifetime: PhantomData },
        }
    }

    /// Returns the TypeC CC PHY.
    pub fn cc_phy(&mut self) -> &mut CcPhy<'d, T> {
        &mut self.cc_phy
    }

    /// Splits the UCPD driver into a TypeC PHY to control and monitor CC voltage
    /// and a Power Delivery (PD) PHY with receiver and transmitter.
    pub fn split_pd_phy(
        self,
        rx_dma: Peri<'d, impl RxDma<T>>,
        tx_dma: Peri<'d, impl TxDma<T>>,
        cc_sel: CcSel,
    ) -> (CcPhy<'d, T>, PdPhy<'d, T>) {
        let r = T::REGS;

        // TODO: Currently only SOP messages are supported.
        r.tx_ordsetr().write(|w| w.set_txordset(0b10001_11000_11000_11000));

        // Enable the receiver on one of the two CC lines.
        r.cr().modify(|w| w.set_phyccsel(cc_sel));

        // Enable hard reset receive interrupt.
        r.imr().modify(|w| w.set_rxhrstdetie(true));

        // Enable PD packet reception
        r.cr().modify(|w| w.set_phyrxen(true));

        // Both parts must be dropped before the peripheral can be disabled.
        T::state().drop_not_ready.store(true, Ordering::Relaxed);

        let rx_dma_req = rx_dma.request();
        let tx_dma_req = tx_dma.request();
        (
            self.cc_phy,
            PdPhy {
                _lifetime: PhantomData,
                rx_dma: ChannelAndRequest {
                    channel: rx_dma.into(),
                    request: rx_dma_req,
                },
                tx_dma: ChannelAndRequest {
                    channel: tx_dma.into(),
                    request: tx_dma_req,
                },
            },
        )
    }
}

/// Control and monitoring of TypeC CC pin functionailty.
pub struct CcPhy<'d, T: Instance> {
    _lifetime: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Drop for CcPhy<'d, T> {
    fn drop(&mut self) {
        let r = T::REGS;
        r.cr().modify(|w| {
            w.set_cc1tcdis(true);
            w.set_cc2tcdis(true);
            w.set_ccenable(Ccenable::DISABLED);
        });

        // Check if the PdPhy part was dropped already.
        let drop_not_ready = &T::state().drop_not_ready;
        if drop_not_ready.load(Ordering::Relaxed) {
            drop_not_ready.store(false, Ordering::Relaxed);
        } else {
            r.cfgr1().write(|w| w.set_ucpden(false));
            rcc::disable::<T>();
            T::Interrupt::disable();
        }
    }
}

impl<'d, T: Instance> CcPhy<'d, T> {
    /// Sets the pull-up/pull-down resistor values exposed on the CC pins.
    pub fn set_pull(&mut self, cc_pull: CcPull) {
        T::REGS.cr().modify(|w| {
            w.set_anamode(if cc_pull == CcPull::Sink {
                Anamode::SINK
            } else {
                Anamode::SOURCE
            });
            w.set_anasubmode(match cc_pull {
                CcPull::SourceDefaultUsb => 1,
                CcPull::Source1_5A => 2,
                CcPull::Source3_0A => 3,
                _ => 0,
            });
            w.set_ccenable(if cc_pull == CcPull::Disabled {
                Ccenable::DISABLED
            } else {
                Ccenable::BOTH
            });
        });

        // Software trim according to RM0481, p. 2650/2668
        #[cfg(stm32h5)]
        T::REGS.cfgr3().modify(|w| match cc_pull {
            CcPull::Source1_5A => {
                let trim_1a5_cc1 = unsafe { *(0x08FF_F844 as *const u32) & 0xF };
                let trim_1a5_cc2 = unsafe { ((*(0x08FF_F844 as *const u32)) >> 16) & 0xF };

                w.set_trim_cc1_rp(trim_1a5_cc1 as u8);
                w.set_trim_cc2_rp(trim_1a5_cc2 as u8);
            }
            _ => {
                let trim_3a0_cc1 = unsafe { (*(0x4002_242C as *const u32) >> 4) & 0xF };
                let trim_3a0_cc2 = unsafe { ((*(0x4002_242C as *const u32)) >> 12) & 0xF };

                w.set_trim_cc1_rp(trim_3a0_cc1 as u8);
                w.set_trim_cc2_rp(trim_3a0_cc2 as u8);
            }
        });

        // Disable dead-battery pull-down resistors which are enabled by default on boot.
        critical_section::with(|cs| {
            init(
                cs,
                false,
                #[cfg(peri_ucpd2)]
                false,
            );
        });
    }

    /// Returns the current voltage level of CC1 and CC2 pin as tuple.
    ///
    /// Interpretation of the voltage levels depends on the configured CC line
    /// pull-up/pull-down resistance.
    pub fn vstate(&self) -> (CcVState, CcVState) {
        let sr = T::REGS.sr().read();
        (sr.typec_vstate_cc1(), sr.typec_vstate_cc2())
    }

    /// Waits for a change in voltage state on either CC line.
    pub async fn wait_for_vstate_change(&self) -> (CcVState, CcVState) {
        let _on_drop = OnDrop::new(|| self.enable_cc_interrupts(false));
        let prev_vstate = self.vstate();
        poll_fn(|cx| {
            let vstate = self.vstate();
            if vstate != prev_vstate {
                Poll::Ready(vstate)
            } else {
                T::state().waker.register(cx.waker());
                self.enable_cc_interrupts(true);
                Poll::Pending
            }
        })
        .await
    }

    fn enable_cc_interrupts(&self, enable: bool) {
        T::REGS.imr().modify(|w| {
            w.set_typecevt1ie(enable);
            w.set_typecevt2ie(enable);
        });
    }
}

/// Receive SOP.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Sop {
    /// SOP
    Sop,
    /// SOP'
    SopPrime,
    /// SOP''
    SopDoublePrime,
    /// SOP'_Debug
    SopPrimeDebug,
    /// SOP''_Debug
    SopDoublePrimeDebug,
}

/// Receive Error.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RxError {
    /// Incorrect CRC or truncated message (a line becoming static before EOP is met).
    Crc,

    /// Provided buffer was too small for the received message.
    Overrun,

    /// Hard Reset received before or during reception.
    HardReset,
}

/// Transmit Error.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TxError {
    /// Concurrent receive in progress or excessive noise on the line.
    Discarded,

    /// Hard Reset received before or during transmission.
    HardReset,
}

/// Power Delivery (PD) PHY.
pub struct PdPhy<'d, T: Instance> {
    _lifetime: PhantomData<&'d mut T>,
    rx_dma: ChannelAndRequest<'d>,
    tx_dma: ChannelAndRequest<'d>,
}

impl<'d, T: Instance> Drop for PdPhy<'d, T> {
    fn drop(&mut self) {
        let r = T::REGS;
        r.cr().modify(|w| w.set_phyrxen(false));
        // Check if the CcPhy part was dropped already.
        let drop_not_ready = &T::state().drop_not_ready;
        if drop_not_ready.load(Ordering::Relaxed) {
            drop_not_ready.store(false, Ordering::Relaxed);
        } else {
            r.cfgr1().write(|w| w.set_ucpden(false));
            rcc::disable::<T>();
            T::Interrupt::disable();
        }
    }
}

impl<'d, T: Instance> PdPhy<'d, T> {
    /// Receives a PD message into the provided buffer.
    ///
    /// Returns the number of received bytes or an error.
    pub async fn receive(&mut self, buf: &mut [u8]) -> Result<usize, RxError> {
        self.receive_with_sop(buf).await.map(|(_sop, size)| size)
    }

    /// Receives SOP and a PD message into the provided buffer.
    ///
    /// Returns the start of packet type and number of received bytes or an error.
    pub async fn receive_with_sop(&mut self, buf: &mut [u8]) -> Result<(Sop, usize), RxError> {
        let r = T::REGS;

        let mut dma = unsafe {
            self.rx_dma
                .read(r.rxdr().as_ptr() as *mut u8, buf, TransferOptions::default())
        };

        let _on_drop = OnDrop::new(|| {
            Self::enable_rx_interrupt(false);
            // Clear interrupt flags
            r.icr().write(|w| {
                w.set_rxorddetcf(true);
                w.set_rxovrcf(true);
                w.set_rxmsgendcf(true);
            });
        });

        let mut rxpaysz = 0;

        // Stop DMA reception immediately after receiving a packet, to prevent storing multiple packets in the same buffer.
        poll_fn(|cx| {
            let sr = r.sr().read();

            if sr.rxhrstdet() {
                dma.request_stop();

                // Clean and re-enable hard reset receive interrupt.
                r.icr().write(|w| w.set_rxhrstdetcf(true));
                r.imr().modify(|w| w.set_rxhrstdetie(true));
                Poll::Ready(Err(RxError::HardReset))
            } else if sr.rxmsgend() {
                dma.request_stop();
                // Should be read immediately on interrupt.
                rxpaysz = r.rx_payszr().read().rxpaysz().into();

                let ret = if sr.rxovr() {
                    Err(RxError::Overrun)
                } else if sr.rxerr() {
                    Err(RxError::Crc)
                } else {
                    Ok(())
                };
                Poll::Ready(ret)
            } else {
                T::state().waker.register(cx.waker());
                Self::enable_rx_interrupt(true);
                Poll::Pending
            }
        })
        .await?;

        // Make sure that the last byte was fetched by DMA.
        while r.sr().read().rxne() {
            if dma.get_remaining_transfers() == 0 {
                return Err(RxError::Overrun);
            }
        }

        let sop = match r.rx_ordsetr().read().rxordset() {
            Rxordset::SOP => Sop::Sop,
            Rxordset::SOP_PRIME => Sop::SopPrime,
            Rxordset::SOP_DOUBLE_PRIME => Sop::SopDoublePrime,
            Rxordset::SOP_PRIME_DEBUG => Sop::SopPrimeDebug,
            Rxordset::SOP_DOUBLE_PRIME_DEBUG => Sop::SopDoublePrimeDebug,
            Rxordset::CABLE_RESET => return Err(RxError::HardReset),
            // Extension headers are not supported
            _ => unreachable!(),
        };

        Ok((sop, rxpaysz))
    }

    fn enable_rx_interrupt(enable: bool) {
        T::REGS.imr().modify(|w| w.set_rxmsgendie(enable));
    }

    /// Transmits a PD message.
    pub async fn transmit(&mut self, buf: &[u8]) -> Result<(), TxError> {
        let r = T::REGS;

        // When a previous transmission was dropped before it had finished it
        // might still be running because there is no way to abort an ongoing
        // message transmission. Wait for it to finish but ignore errors.
        if r.cr().read().txsend() {
            if let Err(TxError::HardReset) = Self::wait_tx_done().await {
                return Err(TxError::HardReset);
            }
        }

        // Clear the TX interrupt flags.
        T::REGS.icr().write(|w| {
            w.set_txmsgdisccf(true);
            w.set_txmsgsentcf(true);
        });

        // Start the DMA and let it do its thing in the background.
        let _dma = unsafe {
            self.tx_dma
                .write(buf, r.txdr().as_ptr() as *mut u8, TransferOptions::default())
        };

        // Configure and start the transmission.
        r.tx_payszr().write(|w| w.set_txpaysz(buf.len() as _));
        r.cr().modify(|w| {
            w.set_txmode(Txmode::PACKET);
            w.set_txsend(true);
        });

        Self::wait_tx_done().await
    }

    async fn wait_tx_done() -> Result<(), TxError> {
        let _on_drop = OnDrop::new(|| Self::enable_tx_interrupts(false));
        poll_fn(|cx| {
            let r = T::REGS;
            let sr = r.sr().read();
            if sr.rxhrstdet() {
                // Clean and re-enable hard reset receive interrupt.
                r.icr().write(|w| w.set_rxhrstdetcf(true));
                r.imr().modify(|w| w.set_rxhrstdetie(true));
                Poll::Ready(Err(TxError::HardReset))
            } else if sr.txmsgdisc() {
                Poll::Ready(Err(TxError::Discarded))
            } else if sr.txmsgsent() {
                Poll::Ready(Ok(()))
            } else {
                T::state().waker.register(cx.waker());
                Self::enable_tx_interrupts(true);
                Poll::Pending
            }
        })
        .await
    }

    fn enable_tx_interrupts(enable: bool) {
        T::REGS.imr().modify(|w| {
            w.set_txmsgdiscie(enable);
            w.set_txmsgsentie(enable);
        });
    }

    /// Transmit a hard reset.
    pub async fn transmit_hardreset(&mut self) -> Result<(), TxError> {
        let r = T::REGS;

        // Clear the hardreset interrupt flags.
        T::REGS.icr().write(|w| {
            w.set_hrstdisccf(true);
            w.set_hrstsentcf(true);
        });

        // Trigger hard reset transmission.
        r.cr().modify(|w| {
            w.set_txhrst(true);
        });

        let _on_drop = OnDrop::new(|| self.enable_hardreset_interrupts(false));
        poll_fn(|cx| {
            let r = T::REGS;
            let sr = r.sr().read();
            if sr.rxhrstdet() {
                // Clean and re-enable hard reset receive interrupt.
                r.icr().write(|w| w.set_rxhrstdetcf(true));
                r.imr().modify(|w| w.set_rxhrstdetie(true));
                Poll::Ready(Err(TxError::HardReset))
            } else if sr.hrstdisc() {
                Poll::Ready(Err(TxError::Discarded))
            } else if sr.hrstsent() {
                Poll::Ready(Ok(()))
            } else {
                T::state().waker.register(cx.waker());
                self.enable_hardreset_interrupts(true);
                Poll::Pending
            }
        })
        .await
    }

    fn enable_hardreset_interrupts(&self, enable: bool) {
        T::REGS.imr().modify(|w| {
            w.set_hrstdiscie(enable);
            w.set_hrstsentie(enable);
        });
    }
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::REGS;
        let sr = r.sr().read();

        if sr.typecevt1() || sr.typecevt2() {
            r.icr().write(|w| {
                w.set_typecevt1cf(true);
                w.set_typecevt2cf(true);
            });
        }

        if sr.rxhrstdet() {
            r.imr().modify(|w| w.set_rxhrstdetie(false));
        }

        if sr.rxmsgend() {
            r.imr().modify(|w| w.set_rxmsgendie(false));
        }

        if sr.txmsgdisc() || sr.txmsgsent() {
            r.imr().modify(|w| {
                w.set_txmsgdiscie(false);
                w.set_txmsgsentie(false);
            });
        }

        if sr.hrstdisc() || sr.hrstsent() {
            r.imr().modify(|w| {
                w.set_hrstdiscie(false);
                w.set_hrstsentie(false);
            });
        }

        // Wake the task to clear and re-enabled interrupts.
        T::state().waker.wake();
    }
}

struct State {
    waker: AtomicWaker,
    // Inverted logic for a default state of 0 so that the data goes into the .bss section.
    drop_not_ready: AtomicBool,
}

impl State {
    pub const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
            drop_not_ready: AtomicBool::new(false),
        }
    }
}

trait SealedInstance {
    const REGS: crate::pac::ucpd::Ucpd;
    fn state() -> &'static State;
}

/// UCPD instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral {
    /// Interrupt for this instance.
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}

foreach_interrupt!(
    ($inst:ident, ucpd, UCPD, GLOBAL, $irq:ident) => {
        impl SealedInstance for crate::peripherals::$inst {
            const REGS: crate::pac::ucpd::Ucpd = crate::pac::$inst;

            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        }

        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
);

pin_trait!(Cc1Pin, Instance);
pin_trait!(Cc2Pin, Instance);

dma_trait!(TxDma, Instance);
dma_trait!(RxDma, Instance);
