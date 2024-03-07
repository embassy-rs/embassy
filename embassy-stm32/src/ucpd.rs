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
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::AnyChannel;
use crate::interrupt;
use crate::pac::ucpd::vals::{Anamode, Ccenable, PscUsbpdclk};
pub use crate::pac::ucpd::vals::{Phyccsel as CcSel, TypecVstateCc as CcVState};
use crate::rcc::RccPeripheral;

/// Pull-up or Pull-down resistor state of both CC lines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CcPull {
    /// Analog PHY for CC pin disabled.
    Disabled,

    /// Rd=5.1k pull-down resistor enabled when the corresponding DBCC pin is high.
    SinkDeadBattery,

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
    _peri: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Drop for Ucpd<'d, T> {
    fn drop(&mut self) {
        T::REGS.cr().modify(|w| {
            w.set_ccenable(Ccenable::DISABLED);
            w.set_cc1tcdis(true);
            w.set_cc2tcdis(true);
        });
    }
}

impl<'d, T: Instance> Ucpd<'d, T> {
    /// Creates a new UCPD driver instance.
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        _cc1: impl Peripheral<P = impl Cc1Pin<T>> + 'd,
        _cc2: impl Peripheral<P = impl Cc2Pin<T>> + 'd,
        cc_pull: CcPull,
    ) -> Self {
        T::enable_and_reset();

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

            // TODO: Only receive SOP messages
            w.set_rxordseten(0x1);

            // Enable DMA and the peripheral
            w.set_txdmaen(true);
            w.set_rxdmaen(true);
            w.set_ucpden(true);
        });

        r.cr().write(|w| {
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
            w.set_ccenable(if cc_pull != CcPull::SinkDeadBattery {
                Ccenable::BOTH
            } else {
                Ccenable::DISABLED
            });

            // Make sure detector is enabled on both pins.
            w.set_cc1tcdis(false);
            w.set_cc2tcdis(false);
        });

        // Disable dead-battery pull-down resistors which are enabled by default on boot.
        critical_section::with(|_| {
            // TODO: other families
            #[cfg(stm32g4)]
            crate::pac::PWR
                .cr3()
                .modify(|w| w.set_ucpd1_dbdis(cc_pull != CcPull::SinkDeadBattery));
        });

        into_ref!(peri);
        Self { _peri: peri }
    }

    /// Returns the current voltage level of CC1 and CC2 pin as tuple.
    ///
    /// Interpretation of the voltage levels depends on the configured CC line
    /// pull-up/pull-down resistance.
    pub fn cc_vstate(&self) -> (CcVState, CcVState) {
        let sr = T::REGS.sr().read();
        (sr.typec_vstate_cc1(), sr.typec_vstate_cc2())
    }

    /// Waits for a change in voltage state on either CC line.
    pub async fn wait_for_cc_vstate_change(&self) -> (CcVState, CcVState) {
        let _on_drop = OnDrop::new(|| critical_section::with(|_| self.enable_cc_interrupts(false)));
        let prev_vstate = self.cc_vstate();
        poll_fn(|cx| {
            let vstate = self.cc_vstate();
            if vstate != prev_vstate {
                Poll::Ready(vstate)
            } else {
                T::waker().register(cx.waker());
                self.enable_cc_interrupts(true);
                Poll::Pending
            }
        })
        .await
    }

    fn enable_cc_interrupts(&self, enable: bool) {
        critical_section::with(|_| {
            T::REGS.imr().modify(|w| {
                w.set_typecevt1ie(enable);
                w.set_typecevt2ie(enable);
            })
        });
    }

    /// Returns PD receiver and transmitter.
    pub fn pd(
        &mut self,
        rx_dma: impl Peripheral<P = impl RxDma<T>> + 'd,
        tx_dma: impl Peripheral<P = impl TxDma<T>> + 'd,
        cc_sel: CcSel,
    ) -> (PdRx<'_, T>, PdTx<'_, T>) {
        into_ref!(rx_dma, tx_dma);
        let rx_dma_req = rx_dma.request();
        let tx_dma_req = tx_dma.request();
        (
            PdRx {
                _ucpd: self,
                dma_ch: rx_dma.map_into(),
                dma_req: rx_dma_req,
            },
            PdTx {
                _ucpd: self,
                dma_ch: tx_dma.map_into(),
                dma_req: tx_dma_req,
            },
        )
    }
}

/// Power Delivery (PD) Receiver.
pub struct PdRx<'d, T: Instance> {
    _ucpd: &'d Ucpd<'d, T>,
    dma_ch: PeripheralRef<'d, AnyChannel>,
    dma_req: Request,
}

impl<'d, T: Instance> Drop for PdRx<'d, T> {
    fn drop(&mut self) {
        T::REGS.cr().modify(|w| w.set_phyrxen(false));
    }
}

/// Power Delivery (PD) Transmitter.
pub struct PdTx<'d, T: Instance> {
    _ucpd: &'d Ucpd<'d, T>,
    dma_ch: PeripheralRef<'d, AnyChannel>,
    dma_req: Request,
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

        // Wake the task to clear and re-enabled interrupts.
        T::waker().wake();
    }
}

/// UCPD instance trait.
pub trait Instance: sealed::Instance + RccPeripheral {}

pub(crate) mod sealed {
    pub trait Instance {
        type Interrupt: crate::interrupt::typelevel::Interrupt;
        const REGS: crate::pac::ucpd::Ucpd;
        fn waker() -> &'static embassy_sync::waitqueue::AtomicWaker;
    }
}

foreach_interrupt!(
    ($inst:ident, ucpd, UCPD, GLOBAL, $irq:ident) => {
        impl sealed::Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;

            const REGS: crate::pac::ucpd::Ucpd = crate::pac::$inst;

            fn waker() -> &'static AtomicWaker {
                static WAKER: AtomicWaker = AtomicWaker::new();
                &WAKER
            }
        }

        impl Instance for crate::peripherals::$inst {}
    };
);

pin_trait!(Cc1Pin, Instance);
pin_trait!(Cc2Pin, Instance);

dma_trait!(TxDma, Instance);
dma_trait!(RxDma, Instance);
