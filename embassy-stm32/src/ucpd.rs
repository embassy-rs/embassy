//! USB Type-C/USB Power Delivery Interface (UCPD)

// Implementation Notes
//
// As of Feb. 2024 the UCPD peripheral is available on: G0, G4, H5, L5, U5
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

use core::cell::RefCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::mem::DropGuard;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering, fence};
use core::task::Poll;
use core::{mem, slice};

use critical_section::Mutex;
use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::atomic::{AtomicClear, AtomicModify};
use crate::dma::{ChannelAndRequest, Transfer, TransferOptions};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::ucpd::vals::{Anamode, Ccenable, PscUsbpdclk, Txmode};
pub use crate::pac::ucpd::vals::{Phyccsel as CcSel, Rxordset, TypecVstateCc as CcVState};
use crate::rcc::{self, RccPeripheral};
use crate::{Peri, interrupt};

pub(crate) fn init(
    _cs: critical_section::CriticalSection,
    #[cfg(all(peri_ucpd1, not(stm32n6)))] ucpd1_db_enable: bool,
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
        cc1: Peri<'d, impl Cc1Pin<T>>,
        cc2: Peri<'d, impl Cc2Pin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Self {
        cc1.pin_number().set_as_analog();
        cc2.pin_number().set_as_analog();

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
            w.set_psc_usbpdclk(PscUsbpdclk::Div2);

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
            let rxordset = (config.sop as u16)
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

        // Software trim according to RM0456, p. 3480/3462
        #[cfg(stm32u5)]
        {
            let trim_rd_cc1 = unsafe { *(0x0BFA_0544 as *const u8) & 0xF };
            let trim_rd_cc2 = unsafe { *(0x0BFA_0546 as *const u8) & 0xF };

            r.cfgr3().write(|w| {
                w.set_trim_cc1_rd(trim_rd_cc1);
                w.set_trim_cc2_rd(trim_rd_cc2);
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
    ///
    /// PD messages are received into `rx_buffers` in the background, so that messages arriving
    /// between two calls of [`PdPhy::receive`] are not lost.
    pub fn split_pd_phy<RX, TX>(
        self,
        rx_dma: Peri<'d, RX>,
        tx_dma: Peri<'d, TX>,
        irqs: impl interrupt::typelevel::Binding<RX::Interrupt, crate::dma::InterruptHandler<RX>>
        + interrupt::typelevel::Binding<TX::Interrupt, crate::dma::InterruptHandler<TX>>
        + 'd,
        cc_sel: CcSel,
        rx_buffers: &'d mut RxBuffers,
    ) -> (CcPhy<'d, T>, PdPhy<'d, T>)
    where
        RX: RxDma<T>,
        TX: TxDma<T>,
    {
        let (cc_phy, mut pd_phy) = self.split_disabled_pd_phy(rx_dma, tx_dma, irqs);
        // SAFETY: `rx_buffers` is borrowed for `'d`, and dropping `pd_phy` stops reception.
        unsafe { pd_phy.start_reception(cc_sel, rx_buffers) };
        (cc_phy, pd_phy)
    }

    /// Splits the UCPD driver like [`split_pd_phy`](Self::split_pd_phy), but with PD
    /// communication disabled.
    ///
    /// Both parts can be kept for as long as the driver is needed. On attach, enable PD on the
    /// connected CC line with [`IdlePdPhy::activate`], and drop the returned [`ActivePdPhy`] on
    /// detach. Neither affects the CC terminations set with [`CcPhy::set_pull`].
    pub fn split_idle_pd_phy<RX, TX>(
        self,
        rx_dma: Peri<'d, RX>,
        tx_dma: Peri<'d, TX>,
        irqs: impl interrupt::typelevel::Binding<RX::Interrupt, crate::dma::InterruptHandler<RX>>
        + interrupt::typelevel::Binding<TX::Interrupt, crate::dma::InterruptHandler<TX>>
        + 'd,
    ) -> (CcPhy<'d, T>, IdlePdPhy<'d, T>)
    where
        RX: RxDma<T>,
        TX: TxDma<T>,
    {
        let (cc_phy, pd_phy) = self.split_disabled_pd_phy(rx_dma, tx_dma, irqs);
        (cc_phy, IdlePdPhy { pd_phy })
    }

    fn split_disabled_pd_phy<RX, TX>(
        self,
        rx_dma: Peri<'d, RX>,
        tx_dma: Peri<'d, TX>,
        irqs: impl interrupt::typelevel::Binding<RX::Interrupt, crate::dma::InterruptHandler<RX>>
        + interrupt::typelevel::Binding<TX::Interrupt, crate::dma::InterruptHandler<TX>>
        + 'd,
    ) -> (CcPhy<'d, T>, PdPhy<'d, T>)
    where
        RX: RxDma<T>,
        TX: TxDma<T>,
    {
        // Both parts must be dropped before the peripheral can be disabled.
        T::state().drop_not_ready.store(true, Ordering::Relaxed);

        let rx_dma = new_dma_nonopt!(rx_dma, irqs);
        let tx_dma = new_dma_nonopt!(tx_dma, irqs);
        (
            self.cc_phy,
            PdPhy {
                _lifetime: PhantomData,
                rx_dma,
                tx_dma,
            },
        )
    }
}

/// Control and monitoring of TypeC CC pin functionality.
pub struct CcPhy<'d, T: Instance> {
    _lifetime: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Drop for CcPhy<'d, T> {
    fn drop(&mut self) {
        let r = T::REGS;
        modify_cr::<T>(|w| {
            w.set_cc1tcdis(true);
            w.set_cc2tcdis(true);
            w.set_ccenable(Ccenable::Disabled);
        });

        // Check if the PdPhy part was dropped already.
        if !T::state().drop_not_ready.clear() {
            r.cfgr1().write(|w| w.set_ucpden(false));
            rcc::disable::<T>();
            T::Interrupt::disable();
        }
    }
}

impl<'d, T: Instance> CcPhy<'d, T> {
    /// Sets the pull-up/pull-down resistor values exposed on the CC pins.
    pub fn set_pull(&mut self, cc_pull: CcPull) {
        modify_cr::<T>(|w| {
            w.set_anamode(if cc_pull == CcPull::Sink {
                Anamode::Sink
            } else {
                Anamode::Source
            });
            w.set_anasubmode(match cc_pull {
                CcPull::SourceDefaultUsb => 1,
                CcPull::Source1_5A => 2,
                CcPull::Source3_0A => 3,
                _ => 0,
            });
            w.set_ccenable(if cc_pull == CcPull::Disabled {
                Ccenable::Disabled
            } else {
                Ccenable::Both
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

        // Software trim according to RM0456, p. 3480/3462
        #[cfg(stm32u5)]
        T::REGS.cfgr3().modify(|w| match cc_pull {
            CcPull::Source1_5A => {
                let trim_1a5_cc1 = unsafe { *(0x0BFA_07A7 as *const u8) & 0xF };
                let trim_1a5_cc2 = unsafe { *(0x0BFA_07A8 as *const u8) & 0xF };

                w.set_trim_cc1_rp(trim_1a5_cc1);
                w.set_trim_cc2_rp(trim_1a5_cc2);
            }
            _ => {
                let trim_3a0_cc1 = unsafe { *(0x0BFA_0545 as *const u8) & 0xF };
                let trim_3a0_cc2 = unsafe { *(0x0BFA_0547 as *const u8) & 0xF };

                w.set_trim_cc1_rp(trim_3a0_cc1);
                w.set_trim_cc2_rp(trim_3a0_cc2);
            }
        });

        // Disable dead-battery pull-down resistors which are enabled by default on boot.
        critical_section::with(|cs| {
            init(
                cs,
                #[cfg(not(stm32n6))]
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
    ///
    /// This also marks the current state as seen: [`wait_for_vstate_change`](Self::wait_for_vstate_change)
    /// returns once the state changes after this call.
    pub fn vstate(&self) -> (CcVState, CcVState) {
        // Clear the event flags before reading the state, so that a change after the read sets
        // them again instead of being lost.
        T::REGS.icr().write(|w| {
            w.set_typecevt1cf(true);
            w.set_typecevt2cf(true);
        });
        let sr = T::REGS.sr().read();
        (sr.typec_vstate_cc1(), sr.typec_vstate_cc2())
    }

    /// Waits for a change in voltage state on either CC line.
    ///
    /// Returns the current voltage state once the hardware has reported a change on CC1 or CC2
    /// since the state was last read, by [`vstate`](Self::vstate) or by this function. Changes
    /// are latched: one that happened before this call makes it return immediately, so a loop like
    ///
    /// ```ignore
    /// let (mut cc1, _) = cc_phy.vstate();
    /// while cc1 != wanted {
    ///     (cc1, _) = cc_phy.wait_for_vstate_change().await;
    /// }
    /// ```
    ///
    /// can't miss a change. The returned state can be equal to the one seen before, for example
    /// if the line changed and changed back.
    pub async fn wait_for_vstate_change(&mut self) -> (CcVState, CcVState) {
        let _on_drop = DropGuard::new((), |()| self.enable_cc_interrupts(false));
        poll_fn(|cx| {
            let sr = T::REGS.sr().read();
            if sr.typecevt1() || sr.typecevt2() {
                // Clears the flags, then reads the state.
                Poll::Ready(self.vstate())
            } else {
                T::state().cc_waker.register(cx.waker());
                self.enable_cc_interrupts(true);
                Poll::Pending
            }
        })
        .await
    }

    fn enable_cc_interrupts(&self, enable: bool) {
        if enable {
            T::REGS.imr().set_bits(|w| {
                w.set_typecevt1ie(true);
                w.set_typecevt2ie(true);
            });
        } else {
            T::REGS.imr().clear_bits(|w| {
                w.set_typecevt1ie(false);
                w.set_typecevt2ie(false);
            });
        }
    }
}

/// Start of Packet.
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

impl Sop {
    fn value(&self) -> u32 {
        match self {
            Sop::Sop => 0b10001_11000_11000_11000,
            Sop::SopPrime => 0b00110_00110_11000_11000,
            Sop::SopDoublePrime => 0b00110_11000_00110_11000,
            Sop::SopPrimeDebug => 0b00110_11001_11001_11000,
            Sop::SopDoublePrimeDebug => 0b10001_00110_11001_11000,
        }
    }
}

/// Receive Error.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RxError {
    /// Incorrect CRC or truncated message (a line becoming static before EOP is met).
    Crc,

    /// The provided buffer was too small for the received message, or messages were lost: a
    /// message arrived while the previous one still waited for [`PdPhy::receive`], or bytes of a
    /// message were lost.
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

    /// Hard Reset received during transmission.
    HardReset,
}

/// Size of the largest message: header, extended header and 260 data bytes of an unchunked
/// extended message.
const MAX_MESSAGE_LEN: usize = 264;

/// Size of an RX buffer: one byte more than the largest message.
///
/// An over-long frame fills the buffer and leaves its next byte in RXDR, which the DMA then puts
/// at the start of the next buffer. Thanks to the spare byte, that byte always shows up in the
/// byte count of the next message, even for a message of the largest size, which is then dropped
/// instead of being returned shifted by one byte.
const RX_BUFFER_LEN: usize = MAX_MESSAGE_LEN + 1;

/// Buffers for received PD messages, given to [`Ucpd::split_pd_phy`] or
/// [`IdlePdPhy::activate`].
///
/// Holds two messages of the largest size: while one waits for [`PdPhy::receive`], the next one
/// is received into the other.
pub struct RxBuffers {
    bufs: [[u8; RX_BUFFER_LEN]; 2],
}

impl RxBuffers {
    /// Creates the buffers.
    pub const fn new() -> Self {
        Self {
            bufs: [[0; RX_BUFFER_LEN]; 2],
        }
    }
}

impl Default for RxBuffers {
    fn default() -> Self {
        Self::new()
    }
}

/// Power Delivery (PD) PHY.
pub struct PdPhy<'d, T: Instance> {
    _lifetime: PhantomData<&'d mut T>,
    rx_dma: ChannelAndRequest<'d>,
    tx_dma: ChannelAndRequest<'d>,
}

impl<'d, T: Instance> Drop for PdPhy<'d, T> {
    fn drop(&mut self) {
        self.stop_reception();
        // Check if the CcPhy part was dropped already.
        if !T::state().drop_not_ready.clear() {
            T::REGS.cfgr1().write(|w| w.set_ucpden(false));
            rcc::disable::<T>();
            T::Interrupt::disable();
        }
    }
}

impl<'d, T: Instance> PdPhy<'d, T> {
    /// Starts receiving on the given CC line, into `rx_buffers`.
    ///
    /// From now on, the interrupt owns the RX DMA: at the end of every message it restarts the
    /// DMA at the start of a buffer, so that the DMA is always ready for the next message.
    ///
    /// # Safety
    ///
    /// `rx_buffers` must stay borrowed until reception stops.
    unsafe fn start_reception(&mut self, cc_sel: CcSel, rx_buffers: &mut RxBuffers) {
        let r = T::REGS;

        // `stop_reception` takes the DMA back before `'d` ends, and before `rx_buffers` is released.
        let mut dma = RxDmaState::new(&self.rx_dma, rx_buffers, r.rxdr().as_ptr() as *mut u8);
        dma.receive_into(0);
        critical_section::with(|cs| {
            *T::state().rx.borrow_ref_mut(cs) = RxState {
                dma: Some(dma),
                ..RxState::new()
            }
        });

        r.icr().write(|w| {
            w.set_rxhrstdetcf(true);
            w.set_rxmsgendcf(true);
            w.set_rxovrcf(true);
        });
        r.imr().set_bits(|w| {
            w.set_rxmsgendie(true);
            w.set_rxhrstdetie(true);
        });
        modify_cr::<T>(|w| {
            w.set_phyccsel(cc_sel);
            w.set_phyrxen(true);
        });
    }

    /// Stops receiving, if reception is on.
    fn stop_reception(&mut self) {
        let r = T::REGS;

        modify_cr::<T>(|w| w.set_phyrxen(false));
        r.imr().clear_bits(|w| {
            w.set_rxmsgendie(false);
            w.set_rxhrstdetie(false);
        });

        let Some(mut dma) = critical_section::with(|cs| T::state().rx.borrow_ref_mut(cs).dma.take()) else {
            return;
        };

        // A byte left in RXDR leaves a DMA request pending, which neither reading RXDR nor
        // clearing RXNE cancels: the next reception would start with it. Let the DMA take it,
        // restarted first in case the current buffer is full.
        dma.receive_into(0);
        while r.sr().read().rxne() {}

        // Dropping `dma` stops the DMA.
    }

    /// Receives a PD message into the provided buffer.
    ///
    /// Returns the number of received bytes or an error.
    pub async fn receive(&mut self, buf: &mut [u8]) -> Result<usize, RxError> {
        self.receive_with_sop(buf).await.map(|(_sop, size)| size)
    }

    /// Receives SOP and a PD message into the provided buffer.
    ///
    /// Messages are received in the background, so a message that arrived before this call is
    /// returned right away. One message can wait for this call; one that arrives while another one
    /// is still waiting is dropped, and reported as [`RxError::Overrun`]. A received Hard Reset
    /// discards the waiting message, and is reported as [`RxError::HardReset`].
    ///
    /// Returns the start of packet type and number of received bytes or an error.
    pub async fn receive_with_sop(&mut self, buf: &mut [u8]) -> Result<(Sop, usize), RxError> {
        poll_fn(|cx| {
            T::state().rx_waker.register(cx.waker());
            match critical_section::with(|cs| T::state().rx.borrow_ref_mut(cs).take_next()) {
                None => Poll::Pending,
                Some(Err(error)) => Poll::Ready(Err(error)),
                Some(Ok(msg)) => Poll::Ready(Self::take_message(msg, buf)),
            }
        })
        .await
    }

    /// Copies the waiting message into `buf`, and hands its buffer back to the DMA.
    fn take_message(msg: RxMessage, buf: &mut [u8]) -> Result<(Sop, usize), RxError> {
        let result = if msg.len > buf.len() {
            Err(RxError::Overrun)
        } else {
            // The DMA wrote the message before the interrupt recorded it.
            fence(Ordering::Acquire);
            // SAFETY: the buffers stay borrowed while reception is on, and the DMA doesn't write
            // into the message's buffer until it is handed back below.
            buf[..msg.len].copy_from_slice(unsafe { slice::from_raw_parts(msg.data, msg.len) });
            msg.sop().map(|sop| (sop, msg.len))
        };

        critical_section::with(|cs| T::state().rx.borrow_ref_mut(cs).other = OtherBuffer::Free);
        result
    }

    /// Transmits an SOP PD message.
    pub async fn transmit(&mut self, buf: &[u8]) -> Result<(), TxError> {
        self.transmit_with_sop(Sop::Sop, buf).await
    }

    /// Transmits a PD message with a given SOP.
    pub async fn transmit_with_sop(&mut self, sop: Sop, buf: &[u8]) -> Result<(), TxError> {
        let r = T::REGS;

        // Only a Hard Reset received from now on ends this transmission.
        T::state().tx_hard_reset.store(false, Ordering::Relaxed);

        // When a previous transmission was dropped before it had finished it
        // might still be running because there is no way to abort an ongoing
        // message transmission. Wait for it to finish but ignore errors.
        if r.cr().read().txsend()
            && let Err(TxError::HardReset) = Self::wait_tx_done().await
        {
            return Err(TxError::HardReset);
        }

        // Clear the TX interrupt flags.
        T::REGS.icr().write(|w| {
            w.set_txmsgdisccf(true);
            w.set_txmsgsentcf(true);
        });

        r.tx_ordsetr().write(|w| w.set_txordset(sop.value()));

        // Start the DMA and let it do its thing in the background.
        let _dma = unsafe {
            self.tx_dma
                .write(buf, r.txdr().as_ptr() as *mut u8, TransferOptions::default())
        };

        // Configure and start the transmission.
        r.tx_payszr().write(|w| w.set_txpaysz(buf.len() as _));
        modify_cr::<T>(|w| {
            w.set_txmode(Txmode::Packet);
            w.set_txsend(true);
        });

        Self::wait_tx_done().await
    }

    async fn wait_tx_done() -> Result<(), TxError> {
        let _on_drop = DropGuard::new((), |()| Self::enable_tx_interrupts(false));
        poll_fn(|cx| {
            let r = T::REGS;
            let sr = r.sr().read();
            if T::state().tx_hard_reset.clear() {
                Poll::Ready(Err(TxError::HardReset))
            } else if sr.txmsgdisc() {
                Poll::Ready(Err(TxError::Discarded))
            } else if sr.txmsgsent() {
                Poll::Ready(Ok(()))
            } else {
                T::state().tx_waker.register(cx.waker());
                Self::enable_tx_interrupts(true);
                Poll::Pending
            }
        })
        .await
    }

    fn enable_tx_interrupts(enable: bool) {
        if enable {
            T::REGS.imr().set_bits(|w| {
                w.set_txmsgdiscie(true);
                w.set_txmsgsentie(true);
            });
        } else {
            T::REGS.imr().clear_bits(|w| {
                w.set_txmsgdiscie(false);
                w.set_txmsgsentie(false);
            });
        }
    }

    /// Transmit a hard reset.
    pub async fn transmit_hardreset(&mut self) -> Result<(), TxError> {
        // Only a Hard Reset received from now on ends this transmission.
        T::state().tx_hard_reset.store(false, Ordering::Relaxed);

        // Clear the hardreset interrupt flags.
        T::REGS.icr().write(|w| {
            w.set_hrstdisccf(true);
            w.set_hrstsentcf(true);
        });

        // Trigger hard reset transmission.
        modify_cr::<T>(|w| {
            w.set_txhrst(true);
        });

        let _on_drop = DropGuard::new((), |()| self.enable_hardreset_interrupts(false));
        poll_fn(|cx| {
            let r = T::REGS;
            let sr = r.sr().read();
            if T::state().tx_hard_reset.clear() {
                Poll::Ready(Err(TxError::HardReset))
            } else if sr.hrstdisc() {
                Poll::Ready(Err(TxError::Discarded))
            } else if sr.hrstsent() {
                Poll::Ready(Ok(()))
            } else {
                T::state().tx_waker.register(cx.waker());
                self.enable_hardreset_interrupts(true);
                Poll::Pending
            }
        })
        .await
    }

    fn enable_hardreset_interrupts(&self, enable: bool) {
        if enable {
            T::REGS.imr().set_bits(|w| {
                w.set_hrstdiscie(true);
                w.set_hrstsentie(true);
            });
        } else {
            T::REGS.imr().clear_bits(|w| {
                w.set_hrstdiscie(false);
                w.set_hrstsentie(false);
            });
        }
    }
}

/// Power Delivery (PD) PHY with PD communication disabled, created by
/// [`Ucpd::split_idle_pd_phy`].
pub struct IdlePdPhy<'d, T: Instance> {
    pd_phy: PdPhy<'d, T>,
}

impl<'d, T: Instance> IdlePdPhy<'d, T> {
    /// Enables PD communication on the given CC line.
    ///
    /// PD messages are received into `rx_buffers` in the background, so that messages arriving
    /// between two calls of [`PdPhy::receive`] are not lost. PD reception is disabled again when
    /// the returned [`ActivePdPhy`] is dropped.
    pub fn activate<'a>(&'a mut self, cc_sel: CcSel, rx_buffers: &'a mut RxBuffers) -> ActivePdPhy<'a, 'd, T> {
        // SAFETY: `rx_buffers` is borrowed for as long as the returned `ActivePdPhy`, and dropping
        // it stops reception.
        unsafe { self.pd_phy.start_reception(cc_sel, rx_buffers) };
        ActivePdPhy {
            pd_phy: &mut self.pd_phy,
        }
    }
}

/// Power Delivery (PD) PHY with PD communication enabled, created by [`IdlePdPhy::activate`].
///
/// Derefs to [`PdPhy`] for receiving and transmitting messages. Dropping it disables PD
/// reception.
pub struct ActivePdPhy<'a, 'd, T: Instance> {
    pd_phy: &'a mut PdPhy<'d, T>,
}

impl<'a, 'd, T: Instance> Drop for ActivePdPhy<'a, 'd, T> {
    fn drop(&mut self) {
        self.pd_phy.stop_reception();
    }
}

impl<'a, 'd, T: Instance> Deref for ActivePdPhy<'a, 'd, T> {
    type Target = PdPhy<'d, T>;

    fn deref(&self) -> &Self::Target {
        self.pd_phy
    }
}

impl<'a, 'd, T: Instance> DerefMut for ActivePdPhy<'a, 'd, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.pd_phy
    }
}

/// Modifies CR.
///
/// CR is written by both the CC part and the PD part, which may run at different priorities, so
/// the read-modify-write runs in a critical section to keep them from overwriting each other.
///
/// TXSEND, TXHRST and FRSTX are commands that the hardware clears once it acts on them. A value
/// read while one of them was still set must not be written back, as that would issue the command
/// again (e.g. send a message twice), so they are written as 0 unless `f` sets them.
fn modify_cr<T: Instance>(f: impl FnOnce(&mut crate::pac::ucpd::regs::Cr)) {
    critical_section::with(|_| {
        T::REGS.cr().modify(|w| {
            w.set_txsend(false);
            w.set_txhrst(false);
            w.set_frstx(false);
            f(w);
        });
    });
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::REGS;
        let state = T::state();
        let sr = r.sr().read();

        if sr.typecevt1() || sr.typecevt2() {
            // Leave the event flags set, `wait_for_vstate_change` clears them. This way a change is
            // not lost if the state has changed back by the time the task runs.
            r.imr().clear_bits(|w| {
                w.set_typecevt1ie(false);
                w.set_typecevt2ie(false);
            });
            state.cc_waker.wake();
        }

        if sr.rxmsgend() {
            critical_section::with(|cs| state.rx.borrow_ref_mut(cs).on_message_end(r, sr));
            state.rx_waker.wake();
        }

        // After RXMSGEND, so that it also discards a message that ended before it.
        if sr.rxhrstdet() {
            r.icr().write(|w| w.set_rxhrstdetcf(true));
            // A received Hard Reset is reported to both a reception and a transmission. Only the
            // interrupt consumes the flag, so neither can hide it from the other.
            critical_section::with(|cs| state.rx.borrow_ref_mut(cs).on_hard_reset());
            state.tx_hard_reset.store(true, Ordering::Relaxed);
            state.rx_waker.wake();
            state.tx_waker.wake();
        }

        if sr.txmsgdisc() || sr.txmsgsent() {
            r.imr().clear_bits(|w| {
                w.set_txmsgdiscie(false);
                w.set_txmsgsentie(false);
            });
            state.tx_waker.wake();
        }

        if sr.hrstdisc() || sr.hrstsent() {
            r.imr().clear_bits(|w| {
                w.set_hrstdiscie(false);
                w.set_hrstsentie(false);
            });
            state.tx_waker.wake();
        }

        // Wake the tasks to clear and re-enabled interrupts.
    }
}

struct State {
    // CcPhy and PdPhy can be used from different tasks at the same time, and an AtomicWaker only
    // holds one waker, so each wait has its own.
    cc_waker: AtomicWaker,
    rx_waker: AtomicWaker,
    tx_waker: AtomicWaker,
    // Inverted logic for a default state of 0 so that the data goes into the .bss section.
    drop_not_ready: AtomicBool,
    rx: Mutex<RefCell<RxState>>,
    /// A Hard Reset was received since the current transmission started.
    tx_hard_reset: AtomicBool,
}

impl State {
    pub const fn new() -> Self {
        Self {
            cc_waker: AtomicWaker::new(),
            rx_waker: AtomicWaker::new(),
            tx_waker: AtomicWaker::new(),
            drop_not_ready: AtomicBool::new(false),
            rx: Mutex::new(RefCell::new(RxState::new())),
            tx_hard_reset: AtomicBool::new(false),
        }
    }
}

/// Reception state, shared by the interrupt and [`PdPhy::receive_with_sop`].
///
/// The DMA receives into one of the two [`RxBuffers`]. At the end of a message, the interrupt
/// keeps it for `receive`, and restarts the DMA at the start of the other buffer. While the other
/// buffer is taken, the DMA stays in its buffer: further messages are dropped, and reported as
/// lost.
struct RxState {
    /// Set while reception is on.
    dma: Option<RxDmaState>,
    /// The buffer the DMA receives into.
    receiving: usize,
    /// The buffer the DMA doesn't receive into.
    other: OtherBuffer,
    /// An error to report with the next `receive`.
    error: Option<RxError>,
}

// SAFETY: the pointers into the RX buffers are only used by the interrupt and by the `PdPhy` that
// started reception, which keeps the buffers borrowed until it stops reception.
unsafe impl Send for RxState {}

/// The buffer the DMA doesn't receive into.
#[derive(Clone, Copy)]
enum OtherBuffer {
    /// The DMA can switch to it at the end of a message.
    Free,
    /// Holds a message that waits for `receive`.
    Ready(RxMessage),
    /// `receive` is copying a message out of it.
    Copying,
}

impl RxState {
    const fn new() -> Self {
        Self {
            dma: None,
            receiving: 0,
            other: OtherBuffer::Free,
            error: None,
        }
    }

    /// Takes what the next `receive` returns: an error, else the waiting message.
    ///
    /// The message's buffer stays taken until `receive` has copied the message out, so that the
    /// DMA doesn't reuse it meanwhile.
    fn take_next(&mut self) -> Option<Result<RxMessage, RxError>> {
        if let Some(error) = self.error.take() {
            return Some(Err(error));
        }
        match self.other {
            OtherBuffer::Ready(msg) => {
                self.other = OtherBuffer::Copying;
                Some(Ok(msg))
            }
            OtherBuffer::Free | OtherBuffer::Copying => None,
        }
    }

    /// Records a received message, and restarts the DMA for the next one.
    ///
    /// RX_PAYSZR, RX_ORDSETR and the error flags describe the message until the next one starts,
    /// at least ~300 us after this one ended (inter-frame gap, preamble and SOP).
    fn on_message_end(&mut self, r: crate::pac::ucpd::Ucpd, sr: crate::pac::ucpd::regs::Sr) {
        let len = r.rx_payszr().read().rxpaysz() as usize;
        let sop = r.rx_ordsetr().read().rxordset();
        r.icr().write(|w| {
            w.set_rxmsgendcf(true);
            w.set_rxovrcf(true);
        });

        let Some(dma) = &mut self.dma else {
            return;
        };

        let result = if dma.received() != len || len > MAX_MESSAGE_LEN || sr.rxovr() {
            // The DMA must have received exactly the bytes the UCPD counted. Its last byte reached
            // RXDR before the CRC and EOP, at least ~150 us ago, so the DMA has long taken it. If
            // the counts differ, bytes were lost, a byte was left over from an over-long frame, or
            // the interrupt was so late that the next message has started.
            Err(RxError::Overrun)
        } else if sr.rxerr() {
            Err(RxError::Crc)
        } else if !matches!(self.other, OtherBuffer::Free) {
            // The other buffer is still taken: keep its message, and drop this one.
            Err(RxError::Overrun)
        } else {
            Ok(())
        };

        match result {
            Ok(()) => {
                // Keep the message, and receive the next one into the other buffer.
                self.other = OtherBuffer::Ready(RxMessage {
                    data: dma.buffer(self.receiving) as *const u8,
                    len,
                    ordset: sop,
                });
                self.receiving ^= 1;
            }
            Err(error) => {
                // Drop the message: its buffer is reused for the next one.
                self.error.get_or_insert(error);
            }
        }
        dma.receive_into(self.receiving);
    }

    /// Discards what was received before the Hard Reset, and records it for the next `receive`.
    ///
    /// A message that `receive` is copying out already isn't discarded: it is returned before
    /// the Hard Reset, and its buffer stays taken until the copy is done.
    fn on_hard_reset(&mut self) {
        if let OtherBuffer::Ready(_) = self.other {
            self.other = OtherBuffer::Free;
        }
        self.error = Some(RxError::HardReset);
        // Also discard a message that the Hard Reset cut short.
        if let Some(dma) = &mut self.dma {
            dma.receive_into(self.receiving);
        }
    }
}

/// A received message that waits for `receive`.
#[derive(Clone, Copy)]
struct RxMessage {
    /// Its bytes, at the start of an RX buffer.
    data: *const u8,
    len: usize,
    ordset: Rxordset,
}

impl RxMessage {
    fn sop(&self) -> Result<Sop, RxError> {
        match self.ordset {
            Rxordset::Sop => Ok(Sop::Sop),
            Rxordset::SopPrime => Ok(Sop::SopPrime),
            Rxordset::SopDoublePrime => Ok(Sop::SopDoublePrime),
            Rxordset::SopPrimeDebug => Ok(Sop::SopPrimeDebug),
            Rxordset::SopDoublePrimeDebug => Ok(Sop::SopDoublePrimeDebug),
            Rxordset::CableReset => Err(RxError::HardReset),
            // Extension headers are not supported
            _ => unreachable!(),
        }
    }
}

/// The RX DMA, which belongs to the interrupt while reception is on.
struct RxDmaState {
    channel: ChannelAndRequest<'static>,
    transfer: Option<Transfer<'static>>,
    buffers: *mut RxBuffers,
    rxdr: *mut u8,
}

impl RxDmaState {
    /// Creates the RX DMA state, without starting the DMA.
    ///
    /// # Safety
    ///
    /// `channel` and `buffers` must outlive the returned value.
    unsafe fn new(channel: &ChannelAndRequest<'_>, buffers: *mut RxBuffers, rxdr: *mut u8) -> Self {
        Self {
            channel: mem::transmute::<ChannelAndRequest<'_>, ChannelAndRequest<'static>>(channel.clone_unchecked()),
            transfer: None,
            buffers,
            rxdr,
        }
    }

    /// The RX buffer with the given index.
    fn buffer(&self, index: usize) -> *mut [u8; RX_BUFFER_LEN] {
        // SAFETY: `buffers` outlives `self`, see `new`.
        unsafe { &raw mut (*self.buffers).bufs[index] }
    }

    /// Restarts the DMA at the start of buffer `index`.
    fn receive_into(&mut self, index: usize) {
        // Dropping the transfer stops it.
        self.transfer = None;
        let buf: *mut [u8] = self.buffer(index);
        // SAFETY: `channel` and `buffers` outlive `self`, see `new`, and the transfer is stopped
        // before `self` is dropped.
        self.transfer = Some(unsafe {
            self.channel
                .clone_unchecked()
                .read_raw(self.rxdr, buf, TransferOptions::default())
                .unchecked_extend_lifetime()
        });
    }

    /// Number of bytes the DMA has received into its buffer.
    fn received(&self) -> usize {
        self.transfer
            .as_ref()
            .map_or(0, |t| RX_BUFFER_LEN - t.get_remaining_transfers() as usize)
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
