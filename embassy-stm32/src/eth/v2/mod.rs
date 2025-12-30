mod descriptors;

use core::sync::atomic::{Ordering, fence};

use embassy_hal_internal::Peri;
use stm32_metapac::syscfg::vals::EthSelPhy;

pub(crate) use self::descriptors::{RDes, RDesRing, TDes, TDesRing};
use super::*;
use crate::gpio::{AfType, AnyPin, OutputType, SealedPin as _, Speed};
use crate::interrupt;
use crate::interrupt::InterruptExt;
use crate::pac::ETH;

/// Interrupt handler.
pub struct InterruptHandler {}

impl interrupt::typelevel::Handler<interrupt::typelevel::ETH> for InterruptHandler {
    unsafe fn on_interrupt() {
        WAKER.wake();

        // TODO: Check and clear more flags
        let dma = ETH.ethernet_dma();

        dma.dmacsr().modify(|w| {
            w.set_ti(true);
            w.set_ri(true);
            w.set_nis(true);
        });
        // Delay two peripheral's clock
        dma.dmacsr().read();
        dma.dmacsr().read();
    }
}

/// Ethernet driver.
pub struct Ethernet<'d, T: Instance, P: Phy> {
    _peri: Peri<'d, T>,
    pub(crate) tx: TDesRing<'d>,
    pub(crate) rx: RDesRing<'d>,
    pins: Pins<'d>,
    pub(crate) phy: P,
    pub(crate) mac_addr: [u8; 6],
}

/// Pins of ethernet driver.
enum Pins<'d> {
    Rmii([Peri<'d, AnyPin>; 7]),
    Mii([Peri<'d, AnyPin>; 12]),
}

macro_rules! config_pins {
    ($($pin:ident),*) => {
        critical_section::with(|_| {
            $(
                // TODO: shouldn't some pins be configured as inputs?
                set_as_af!($pin, AfType::output(OutputType::PushPull, Speed::VeryHigh));
            )*
        })
    };
}

impl<'d, T: Instance, SMA: sma::Instance> Ethernet<'d, T, GenericPhy<Sma<'d, SMA>>> {
    /// Create a new RMII ethernet driver using 7 pins.
    ///
    /// This function uses a [`GenericPhy::new_auto`] as PHY, created using the
    /// provided [`SMA`](sma::Instance), and MDIO and MDC pins.
    ///
    /// See [`Ethernet::new_with_phy`] for creating an RMII ethernet
    /// river with a non-standard PHY.
    pub fn new<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
        ref_clk: Peri<'d, impl RefClkPin<T>>,
        crs: Peri<'d, impl CRSPin<T>>,
        rx_d0: Peri<'d, impl RXD0Pin<T>>,
        rx_d1: Peri<'d, impl RXD1Pin<T>>,
        tx_d0: Peri<'d, impl TXD0Pin<T>>,
        tx_d1: Peri<'d, impl TXD1Pin<T>>,
        tx_en: Peri<'d, impl TXEnPin<T>>,
        mac_addr: [u8; 6],
        sma: Peri<'d, SMA>,
        mdio: Peri<'d, impl MDIOPin<SMA>>,
        mdc: Peri<'d, impl MDCPin<SMA>>,
    ) -> Self {
        let sma = Sma::new(sma, mdio, mdc);
        let phy = GenericPhy::new_auto(sma);

        Self::new_with_phy(
            queue, peri, irq, ref_clk, crs, rx_d0, rx_d1, tx_d0, tx_d1, tx_en, mac_addr, phy,
        )
    }

    /// Create a new MII ethernet driver using 14 pins.
    ///
    /// This function uses a [`GenericPhy::new_auto`] as PHY, created using the
    /// provided [`SMA`](sma::Instance), and MDIO and MDC pins.
    ///
    /// See [`Ethernet::new_mii_with_phy`] for creating an RMII ethernet
    /// river with a non-standard PHY.
    pub fn new_mii<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
        rx_clk: Peri<'d, impl RXClkPin<T>>,
        tx_clk: Peri<'d, impl TXClkPin<T>>,
        rxdv: Peri<'d, impl RXDVPin<T>>,
        rx_d0: Peri<'d, impl RXD0Pin<T>>,
        rx_d1: Peri<'d, impl RXD1Pin<T>>,
        rx_d2: Peri<'d, impl RXD2Pin<T>>,
        rx_d3: Peri<'d, impl RXD3Pin<T>>,
        tx_d0: Peri<'d, impl TXD0Pin<T>>,
        tx_d1: Peri<'d, impl TXD1Pin<T>>,
        tx_d2: Peri<'d, impl TXD2Pin<T>>,
        tx_d3: Peri<'d, impl TXD3Pin<T>>,
        tx_en: Peri<'d, impl TXEnPin<T>>,
        mac_addr: [u8; 6],
        sma: Peri<'d, SMA>,
        mdio: Peri<'d, impl MDIOPin<SMA>>,
        mdc: Peri<'d, impl MDCPin<SMA>>,
    ) -> Self {
        let sma = Sma::new(sma, mdio, mdc);
        let phy = GenericPhy::new_auto(sma);

        Self::new_mii_with_phy(
            queue, peri, irq, rx_clk, tx_clk, rxdv, rx_d0, rx_d1, rx_d2, rx_d3, tx_d0, tx_d1, tx_d2, tx_d3, tx_en,
            mac_addr, phy,
        )
    }
}

impl<'d, T: Instance, P: Phy> Ethernet<'d, T, P> {
    /// Create a new RMII ethernet driver using 7 pins.
    pub fn new_with_phy<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
        ref_clk: Peri<'d, impl RefClkPin<T>>,
        crs: Peri<'d, impl CRSPin<T>>,
        rx_d0: Peri<'d, impl RXD0Pin<T>>,
        rx_d1: Peri<'d, impl RXD1Pin<T>>,
        tx_d0: Peri<'d, impl TXD0Pin<T>>,
        tx_d1: Peri<'d, impl TXD1Pin<T>>,
        tx_en: Peri<'d, impl TXEnPin<T>>,
        mac_addr: [u8; 6],
        phy: P,
    ) -> Self {
        config_pins!(ref_clk, crs, rx_d0, rx_d1, tx_d0, tx_d1, tx_en);

        let pins = Pins::Rmii([
            ref_clk.into(),
            crs.into(),
            rx_d0.into(),
            rx_d1.into(),
            tx_d0.into(),
            tx_d1.into(),
            tx_en.into(),
        ]);

        Self::new_inner(queue, peri, irq, pins, phy, mac_addr, EthSelPhy::RMII)
    }

    /// Create a new MII ethernet driver using 12 pins.
    pub fn new_mii_with_phy<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
        rx_clk: Peri<'d, impl RXClkPin<T>>,
        tx_clk: Peri<'d, impl TXClkPin<T>>,
        rxdv: Peri<'d, impl RXDVPin<T>>,
        rx_d0: Peri<'d, impl RXD0Pin<T>>,
        rx_d1: Peri<'d, impl RXD1Pin<T>>,
        rx_d2: Peri<'d, impl RXD2Pin<T>>,
        rx_d3: Peri<'d, impl RXD3Pin<T>>,
        tx_d0: Peri<'d, impl TXD0Pin<T>>,
        tx_d1: Peri<'d, impl TXD1Pin<T>>,
        tx_d2: Peri<'d, impl TXD2Pin<T>>,
        tx_d3: Peri<'d, impl TXD3Pin<T>>,
        tx_en: Peri<'d, impl TXEnPin<T>>,
        mac_addr: [u8; 6],
        phy: P,
    ) -> Self {
        config_pins!(
            rx_clk, tx_clk, rxdv, rx_d0, rx_d1, rx_d2, rx_d3, tx_d0, tx_d1, tx_d2, tx_d3, tx_en
        );

        let pins = Pins::Mii([
            rx_clk.into(),
            tx_clk.into(),
            rxdv.into(),
            rx_d0.into(),
            rx_d1.into(),
            rx_d2.into(),
            rx_d3.into(),
            tx_d0.into(),
            tx_d1.into(),
            tx_d2.into(),
            tx_d3.into(),
            tx_en.into(),
        ]);

        Self::new_inner(queue, peri, irq, pins, phy, mac_addr, EthSelPhy::MII_GMII)
    }

    fn new_inner<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
        pins: Pins<'d>,
        phy: P,
        mac_addr: [u8; 6],
        eth_sel_phy: EthSelPhy,
    ) -> Self {
        // Enable the necessary clocks
        critical_section::with(|_| {
            crate::pac::RCC.ahb1enr().modify(|w| {
                w.set_ethen(true);
                w.set_ethtxen(true);
                w.set_ethrxen(true);
            });

            crate::pac::SYSCFG.pmcr().modify(|w| w.set_eth_sel_phy(eth_sel_phy));
        });

        let dma = T::regs().ethernet_dma();
        let mac = T::regs().ethernet_mac();
        let mtl = T::regs().ethernet_mtl();

        // Reset and wait
        dma.dmamr().modify(|w| w.set_swr(true));
        while dma.dmamr().read().swr() {}

        mac.maccr().modify(|w| {
            w.set_ipg(0b000); // 96 bit times
            w.set_acs(true);
            w.set_fes(true);
            w.set_dm(true);
            // TODO: Carrier sense ? ECRSFD
        });

        // Disable multicast filter
        mac.macpfr().modify(|w| w.set_pm(true));

        // Note: Writing to LR triggers synchronisation of both LR and HR into the MAC core,
        // so the LR write must happen after the HR write.
        mac.maca0hr()
            .modify(|w| w.set_addrhi(u16::from(mac_addr[4]) | (u16::from(mac_addr[5]) << 8)));
        mac.maca0lr().write(|w| {
            w.set_addrlo(
                u32::from(mac_addr[0])
                    | (u32::from(mac_addr[1]) << 8)
                    | (u32::from(mac_addr[2]) << 16)
                    | (u32::from(mac_addr[3]) << 24),
            )
        });

        mac.macqtx_fcr().modify(|w| w.set_pt(0x100));

        // disable all MMC RX interrupts
        mac.mmc_rx_interrupt_mask().write(|w| {
            w.set_rxcrcerpim(true);
            w.set_rxalgnerpim(true);
            w.set_rxucgpim(true);
            w.set_rxlpiuscim(true);
            w.set_rxlpitrcim(true)
        });

        // disable all MMC TX interrupts
        mac.mmc_tx_interrupt_mask().write(|w| {
            w.set_txscolgpim(true);
            w.set_txmcolgpim(true);
            w.set_txgpktim(true);
            w.set_txlpiuscim(true);
            w.set_txlpitrcim(true);
        });

        mtl.mtlrx_qomr().modify(|w| w.set_rsf(true));
        mtl.mtltx_qomr().modify(|w| w.set_tsf(true));

        dma.dmactx_cr().modify(|w| w.set_txpbl(1)); // 32 ?
        dma.dmacrx_cr().modify(|w| {
            w.set_rxpbl(1); // 32 ?
            w.set_rbsz(RX_BUFFER_SIZE as u16);
        });

        let mut this = Self {
            _peri: peri,
            tx: TDesRing::new(&mut queue.tx_desc, &mut queue.tx_buf),
            rx: RDesRing::new(&mut queue.rx_desc, &mut queue.rx_buf),
            pins,
            phy,
            mac_addr,
        };

        fence(Ordering::SeqCst);

        let mac = T::regs().ethernet_mac();
        let mtl = T::regs().ethernet_mtl();
        let dma = T::regs().ethernet_dma();

        mac.maccr().modify(|w| {
            w.set_re(true);
            w.set_te(true);
        });
        mtl.mtltx_qomr().modify(|w| w.set_ftq(true));

        dma.dmactx_cr().modify(|w| w.set_st(true));
        dma.dmacrx_cr().modify(|w| w.set_sr(true));

        // Enable interrupts
        dma.dmacier().modify(|w| {
            w.set_nie(true);
            w.set_rie(true);
            w.set_tie(true);
        });

        this.phy.phy_reset();
        this.phy.phy_init();

        interrupt::ETH.unpend();
        unsafe { interrupt::ETH.enable() };

        this
    }
}

impl<'d, T: Instance, P: Phy> Drop for Ethernet<'d, T, P> {
    fn drop(&mut self) {
        let dma = T::regs().ethernet_dma();
        let mac = T::regs().ethernet_mac();
        let mtl = T::regs().ethernet_mtl();

        // Disable the TX DMA and wait for any previous transmissions to be completed
        dma.dmactx_cr().modify(|w| w.set_st(false));
        while {
            let txqueue = mtl.mtltx_qdr().read();
            txqueue.trcsts() == 0b01 || txqueue.txqsts()
        } {}

        // Disable MAC transmitter and receiver
        mac.maccr().modify(|w| {
            w.set_re(false);
            w.set_te(false);
        });

        // Wait for previous receiver transfers to be completed and then disable the RX DMA
        while {
            let rxqueue = mtl.mtlrx_qdr().read();
            rxqueue.rxqsts() != 0b00 || rxqueue.prxq() != 0
        } {}
        dma.dmacrx_cr().modify(|w| w.set_sr(false));

        critical_section::with(|_| {
            for pin in match self.pins {
                Pins::Rmii(ref mut pins) => pins.iter_mut(),
                Pins::Mii(ref mut pins) => pins.iter_mut(),
            } {
                pin.set_as_disconnected();
            }
        })
    }
}
