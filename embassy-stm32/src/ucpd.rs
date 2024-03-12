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
use core::sync::atomic::Ordering;
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

use crate::dma::{AnyChannel, Request, Transfer, TransferOptions};
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::ucpd::vals::{Anamode, Ccenable, PscUsbpdclk, Txmode};
pub use crate::pac::ucpd::vals::{Phyccsel as CcSel, TypecVstateCc as CcVState};
use crate::rcc::RccPeripheral;

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
            w.set_ucpd1_strobe(ucpd1_db_enable);
            w.set_ucpd2_strobe(ucpd2_db_enable);
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

    #[cfg(any(stm32h5, stm32u5))]
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

/// UCPD driver.
pub struct Ucpd<'d, T: Instance> {
    cc_phy: CcPhy<'d, T>,
}

impl<'d, T: Instance> Ucpd<'d, T> {
    /// Creates a new UCPD driver instance.
    pub fn new(
        _peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        _cc1: impl Peripheral<P = impl Cc1Pin<T>> + 'd,
        _cc2: impl Peripheral<P = impl Cc2Pin<T>> + 'd,
    ) -> Self {
        T::enable_and_reset();
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let r = T::REGS;
        r.cfgr1().write(|w| {
            // "The receiver is designed to work in the clock frequency range from 6 to 18 MHz.
            // However, the optimum performance is ensured in the range from 6 to 12 MHz"
            // UCPD is driven by HSI16 (16MHz internal oscillator), which we need to divide by 2.
            w.set_psc_usbpdclk(PscUsbpdclk::DIV2);

            // Prescaler to produce a target half-bit frequency of 600kHz which is required
            // to produce transmit with a nominal nominal bit rate of 300Kbps+-10% using
            // biphase mark coding (BMC, aka differential manchester coding).
            // A divider of 13 gives the target frequency closest to spec (~615kHz, 1.625us)
            // but we go with the (hopefully well tested) default value used by the Cube HAL
            // which is 14 divides the clock down to ~571kHz, 1.75us.
            w.set_hbitclkdiv(14 - 1);

            // Time window for detecting non-idle (12-20us).
            // 1.75us * 8 = 14us.
            w.set_transwin(8 - 1);

            // Time from the end of last bit of a Frame until the start of the first bit of the
            // next Preamble (min 25us).
            // 1.75us * 17 = ~30us
            w.set_ifrgap(17 - 1);

            w.set_ucpden(true);
        });

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
        rx_dma: impl Peripheral<P = impl RxDma<T>> + 'd,
        tx_dma: impl Peripheral<P = impl TxDma<T>> + 'd,
        cc_sel: CcSel,
    ) -> (CcPhy<'d, T>, PdPhy<'d, T>) {
        let r = T::REGS;

        // TODO: Currently only SOP messages are supported.
        r.tx_ordsetr().write(|w| w.set_txordset(0b10001_11000_11000_11000));

        r.cfgr1().modify(|w| {
            // TODO: Currently only hard reset and SOP messages can be received.
            w.set_rxordseten(0b1001);

            // Enable DMA
            w.set_txdmaen(true);
            w.set_rxdmaen(true);
        });

        // Enable the receiver on one of the two CC lines.
        r.cr().modify(|w| {
            w.set_phyccsel(cc_sel);
            w.set_phyrxen(true);
        });

        // Enable hard reset receive interrupt.
        r.imr().modify(|w| w.set_rxhrstdetie(true));

        // Both parts must be dropped before the peripheral can be disabled.
        T::state().drop_not_ready.store(true, Ordering::Relaxed);

        into_ref!(rx_dma, tx_dma);
        let rx_dma_req = rx_dma.request();
        let tx_dma_req = tx_dma.request();
        (
            self.cc_phy,
            PdPhy {
                _lifetime: PhantomData,
                rx_dma_ch: rx_dma.map_into(),
                rx_dma_req,
                tx_dma_ch: tx_dma.map_into(),
                tx_dma_req,
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
            drop_not_ready.store(true, Ordering::Relaxed);
        } else {
            r.cfgr1().write(|w| w.set_ucpden(false));
            T::disable();
            T::Interrupt::disable();
        }
    }
}

impl<'d, T: Instance> CcPhy<'d, T> {
    /// Sets the pull-up/pull-down resistor values exposed on the CC pins.
    pub fn set_pull(&mut self, cc_pull: CcPull) {
        T::REGS.cr().write(|w| {
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
    rx_dma_ch: PeripheralRef<'d, AnyChannel>,
    rx_dma_req: Request,
    tx_dma_ch: PeripheralRef<'d, AnyChannel>,
    tx_dma_req: Request,
}

impl<'d, T: Instance> Drop for PdPhy<'d, T> {
    fn drop(&mut self) {
        let r = T::REGS;
        r.cr().modify(|w| w.set_phyrxen(false));

        // Check if the Type-C part was dropped already.
        let drop_not_ready = &T::state().drop_not_ready;
        if drop_not_ready.load(Ordering::Relaxed) {
            drop_not_ready.store(true, Ordering::Relaxed);
        } else {
            r.cfgr1().write(|w| w.set_ucpden(false));
            T::disable();
            T::Interrupt::disable();
        }
    }
}

impl<'d, T: Instance> PdPhy<'d, T> {
    /// Receives a PD message into the provided buffer.
    ///
    /// Returns the number of received bytes or an error.
    pub async fn receive(&mut self, buf: &mut [u8]) -> Result<usize, RxError> {
        let r = T::REGS;

        // Check if a message is already being received. If yes, wait until its
        // done, ignore errors and try to receive the next message.
        if r.sr().read().rxorddet() {
            if let Err(RxError::HardReset) = self.wait_rx_done().await {
                return Err(RxError::HardReset);
            }
            r.rxdr().read(); // Clear the RX buffer.
        }

        // Keep the DMA transfer alive so its drop code does not stop it right away.
        let dma = unsafe {
            Transfer::new_read(
                &self.rx_dma_ch,
                self.rx_dma_req,
                r.rxdr().as_ptr() as *mut u8,
                buf,
                TransferOptions::default(),
            )
        };

        self.wait_rx_done().await?;

        // Make sure the the last byte to byte was fetched by DMA.
        while r.sr().read().rxne() {
            if dma.get_remaining_transfers() == 0 {
                return Err(RxError::Overrun);
            }
        }

        Ok(r.rx_payszr().read().rxpaysz().into())
    }

    async fn wait_rx_done(&self) -> Result<(), RxError> {
        let _on_drop = OnDrop::new(|| self.enable_rx_interrupt(false));
        poll_fn(|cx| {
            let r = T::REGS;
            let sr = r.sr().read();
            if sr.rxhrstdet() {
                // Clean and re-enable hard reset receive interrupt.
                r.icr().write(|w| w.set_rxhrstdetcf(true));
                r.imr().modify(|w| w.set_rxhrstdetie(true));
                Poll::Ready(Err(RxError::HardReset))
            } else if sr.rxmsgend() {
                let ret = if sr.rxovr() {
                    Err(RxError::Overrun)
                } else if sr.rxerr() {
                    Err(RxError::Crc)
                } else {
                    Ok(())
                };
                // Message received, clear interrupt flags.
                r.icr().write(|w| {
                    w.set_rxorddetcf(true);
                    w.set_rxovrcf(true);
                    w.set_rxmsgendcf(true);
                });
                Poll::Ready(ret)
            } else {
                T::state().waker.register(cx.waker());
                self.enable_rx_interrupt(true);
                Poll::Pending
            }
        })
        .await
    }

    fn enable_rx_interrupt(&self, enable: bool) {
        T::REGS.imr().modify(|w| w.set_rxmsgendie(enable));
    }

    /// Transmits a PD message.
    pub async fn transmit(&mut self, buf: &[u8]) -> Result<(), TxError> {
        let r = T::REGS;

        // When a previous transmission was dropped before it had finished it
        // might still be running because there is no way to abort an ongoing
        // message transmission. Wait for it to finish but ignore errors.
        if r.cr().read().txsend() {
            if let Err(TxError::HardReset) = self.wait_tx_done().await {
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
            Transfer::new_write(
                &self.tx_dma_ch,
                self.tx_dma_req,
                buf,
                r.txdr().as_ptr() as *mut u8,
                TransferOptions::default(),
            )
        };

        // Configure and start the transmission.
        r.tx_payszr().write(|w| w.set_txpaysz(buf.len() as _));
        r.cr().modify(|w| {
            w.set_txmode(Txmode::PACKET);
            w.set_txsend(true);
        });

        self.wait_tx_done().await
    }

    async fn wait_tx_done(&self) -> Result<(), TxError> {
        let _on_drop = OnDrop::new(|| self.enable_tx_interrupts(false));
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
                self.enable_tx_interrupts(true);
                Poll::Pending
            }
        })
        .await
    }

    fn enable_tx_interrupts(&self, enable: bool) {
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
            w.set_txmsgdisccf(true);
            w.set_txmsgsentcf(true);
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

/// UCPD instance trait.
pub trait Instance: sealed::Instance + RccPeripheral {}

mod sealed {
    use core::sync::atomic::AtomicBool;

    use embassy_sync::waitqueue::AtomicWaker;

    pub struct State {
        pub waker: AtomicWaker,
        // Inverted logic for a default state of 0 so that the data goes into the .bss section.
        pub drop_not_ready: AtomicBool,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                waker: AtomicWaker::new(),
                drop_not_ready: AtomicBool::new(false),
            }
        }
    }

    pub trait Instance {
        type Interrupt: crate::interrupt::typelevel::Interrupt;
        const REGS: crate::pac::ucpd::Ucpd;
        fn state() -> &'static crate::ucpd::sealed::State;
    }
}

foreach_interrupt!(
    ($inst:ident, ucpd, UCPD, GLOBAL, $irq:ident) => {
        impl sealed::Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;

            const REGS: crate::pac::ucpd::Ucpd = crate::pac::$inst;

            fn state() -> &'static crate::ucpd::sealed::State {
                static STATE: crate::ucpd::sealed::State = crate::ucpd::sealed::State::new();
                &STATE
            }
        }

        impl Instance for crate::peripherals::$inst {}
    };
);

pin_trait!(Cc1Pin, Instance);
pin_trait!(Cc2Pin, Instance);

dma_trait!(TxDma, Instance);
dma_trait!(RxDma, Instance);
