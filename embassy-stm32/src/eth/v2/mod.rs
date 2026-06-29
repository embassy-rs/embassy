mod descriptors;
#[cfg(feature = "ptp")]
mod ptp;

use core::sync::atomic::{Ordering, fence};

pub(crate) use descriptors::{RDes, RDesRing, TDes, TDesRing};
use embassy_hal_internal::Peri;
#[cfg(feature = "ptp")]
pub use ptp::{PtpClock, PtpClockConfig, PtpSubsecondIncrement, PtpTimeProvider};
#[cfg(eth_v2)]
use stm32_metapac::syscfg::vals::EthSelPhy;

use super::*;
use crate::gpio::{AfType, Flex, OutputType, Speed};
use crate::interrupt;
use crate::interrupt::InterruptExt;
#[cfg(eth_v2)]
use crate::pac::ETH;
#[cfg(eth_v2a)]
use crate::pac::ETH1 as ETH;
use crate::rcc::WakeGuard;

// The two MACs sit behind different interrupt lines.
#[cfg(eth_v2)]
type EthTypelevel = interrupt::typelevel::ETH;
#[cfg(eth_v2a)]
type EthTypelevel = interrupt::typelevel::ETH1;

/// Access a per-channel DMA/MTL register at channel 0.
///
/// On eth_v2a the DMA/MTL channel registers are arrays (the MAC has multiple
/// channels); on eth_v2 they are plain registers.
macro_rules! ch0 {
    ($regs:expr, $reg:ident) => {{
        #[cfg(eth_v2)]
        {
            $regs.$reg()
        }
        #[cfg(eth_v2a)]
        {
            $regs.$reg(0)
        }
    }};
}

/// Interrupt handler.
pub struct InterruptHandler {}

impl interrupt::typelevel::Handler<EthTypelevel> for InterruptHandler {
    unsafe fn on_interrupt() {
        WAKER.wake();

        // TODO: Check and clear more flags
        let dma = ETH.ethernet_dma();

        ch0!(dma, dmacsr).modify(|w| {
            w.set_ti(true);
            w.set_ri(true);
            w.set_nis(true);
        });
        // Delay two peripheral's clock
        ch0!(dma, dmacsr).read();
        ch0!(dma, dmacsr).read();
    }
}

/// Ethernet driver.
pub struct Ethernet<'d, T: Instance, P: Phy> {
    _peri: Peri<'d, T>,
    _wake_guard: WakeGuard,
    pub(crate) link_state: LinkState,
    pub(crate) tx: TDesRing<'d>,
    pub(crate) rx: RDesRing<'d>,
    _pins: Pins<'d>,
    pub(crate) phy: P,
    pub(crate) mac_addr: [u8; 6],
    #[cfg(feature = "ptp")]
    ptp_clock_taken: bool,
}

/// Pins of ethernet driver.
enum Pins<'d> {
    #[allow(unused)]
    Rmii([Flex<'d>; 7]),
    #[cfg(eth_v2)]
    #[allow(unused)]
    Mii([Flex<'d>; 12]),
    #[cfg(eth_v2a)]
    #[allow(unused)]
    Rgmii([Flex<'d>; 13]),
}

macro_rules! config_pins {
    ($($pin:ident),*) => {
        config_pins!(@speed Speed::VeryHigh; $($pin),*)
    };
    (@speed $speed:expr; $($pin:ident),*) => {
        critical_section::with(|_| {
            $(
                // TODO: shouldn't some pins be configured as inputs?
                set_as_af!($pin, AfType::output(OutputType::PushPull, $speed));
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
    #[allow(clippy::too_many_arguments)]
    pub fn new<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<EthTypelevel, InterruptHandler> + 'd,
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
    #[cfg(eth_v2)]
    pub fn new_mii<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<EthTypelevel, InterruptHandler> + 'd,
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

#[cfg(eth_v2a)]
impl<'d, T: Instance, SMA: sma::Instance> Ethernet<'d, T, GenericPhy<Sma<'d, SMA>>> {
    /// Create a new RGMII ethernet driver using 13 pins.
    ///
    /// The MAC and PHY are fixed at 100 Mbit/s full duplex, so the link partner
    /// must support it.
    ///
    /// This function uses a [`GenericPhy::new_auto`] as PHY, created using the
    /// provided [`SMA`](sma::Instance), and MDIO and MDC pins.
    ///
    /// See [`Ethernet::new_rgmii_with_phy`] for creating an RGMII ethernet
    /// driver with a non-standard PHY.
    #[allow(clippy::too_many_arguments)]
    pub fn new_rgmii<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<EthTypelevel, InterruptHandler> + 'd,
        gtx_clk: Peri<'d, impl RGMIIGTXClkPin<T>>,
        tx_ctl: Peri<'d, impl RGMIITXCtlPin<T>>,
        tx_d0: Peri<'d, impl RGMIITXD0Pin<T>>,
        tx_d1: Peri<'d, impl RGMIITXD1Pin<T>>,
        tx_d2: Peri<'d, impl RGMIITXD2Pin<T>>,
        tx_d3: Peri<'d, impl RGMIITXD3Pin<T>>,
        rx_clk: Peri<'d, impl RGMIIRXClkPin<T>>,
        rx_ctl: Peri<'d, impl RGMIIRXCtlPin<T>>,
        rx_d0: Peri<'d, impl RGMIIRXD0Pin<T>>,
        rx_d1: Peri<'d, impl RGMIIRXD1Pin<T>>,
        rx_d2: Peri<'d, impl RGMIIRXD2Pin<T>>,
        rx_d3: Peri<'d, impl RGMIIRXD3Pin<T>>,
        clk125: Peri<'d, impl RGMIICLK125Pin<T>>,
        mac_addr: [u8; 6],
        sma: Peri<'d, SMA>,
        mdio: Peri<'d, impl MDIOPin<SMA>>,
        mdc: Peri<'d, impl MDCPin<SMA>>,
    ) -> Self {
        let sma = Sma::new(sma, mdio, mdc);
        let phy = GenericPhy::new_auto(sma);

        Self::new_rgmii_with_phy(
            queue, peri, irq, gtx_clk, tx_ctl, tx_d0, tx_d1, tx_d2, tx_d3, rx_clk, rx_ctl, rx_d0, rx_d1, rx_d2, rx_d3,
            clk125, mac_addr, phy,
        )
    }
}

impl<'d, T: Instance, P: Phy> Ethernet<'d, T, P> {
    /// Create a new RMII ethernet driver using 7 pins.
    pub fn new_with_phy<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<EthTypelevel, InterruptHandler> + 'd,
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
            Flex::new(ref_clk),
            Flex::new(crs),
            Flex::new(rx_d0),
            Flex::new(rx_d1),
            Flex::new(tx_d0),
            Flex::new(tx_d1),
            Flex::new(tx_en),
        ]);

        Self::new_inner(
            queue,
            peri,
            irq,
            pins,
            phy,
            mac_addr,
            #[cfg(eth_v2)]
            EthSelPhy::Rmii,
        )
    }

    /// Create a new MII ethernet driver using 12 pins.
    #[cfg(eth_v2)]
    pub fn new_mii_with_phy<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<EthTypelevel, InterruptHandler> + 'd,
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
            Flex::new(rx_clk),
            Flex::new(tx_clk),
            Flex::new(rxdv),
            Flex::new(rx_d0),
            Flex::new(rx_d1),
            Flex::new(rx_d2),
            Flex::new(rx_d3),
            Flex::new(tx_d0),
            Flex::new(tx_d1),
            Flex::new(tx_d2),
            Flex::new(tx_d3),
            Flex::new(tx_en),
        ]);

        Self::new_inner(queue, peri, irq, pins, phy, mac_addr, EthSelPhy::MiiGmii)
    }

    /// Create a new RGMII ethernet driver using 13 pins.
    #[cfg(eth_v2a)]
    #[allow(clippy::too_many_arguments)]
    pub fn new_rgmii_with_phy<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<EthTypelevel, InterruptHandler> + 'd,
        gtx_clk: Peri<'d, impl RGMIIGTXClkPin<T>>,
        tx_ctl: Peri<'d, impl RGMIITXCtlPin<T>>,
        tx_d0: Peri<'d, impl RGMIITXD0Pin<T>>,
        tx_d1: Peri<'d, impl RGMIITXD1Pin<T>>,
        tx_d2: Peri<'d, impl RGMIITXD2Pin<T>>,
        tx_d3: Peri<'d, impl RGMIITXD3Pin<T>>,
        rx_clk: Peri<'d, impl RGMIIRXClkPin<T>>,
        rx_ctl: Peri<'d, impl RGMIIRXCtlPin<T>>,
        rx_d0: Peri<'d, impl RGMIIRXD0Pin<T>>,
        rx_d1: Peri<'d, impl RGMIIRXD1Pin<T>>,
        rx_d2: Peri<'d, impl RGMIIRXD2Pin<T>>,
        rx_d3: Peri<'d, impl RGMIIRXD3Pin<T>>,
        clk125: Peri<'d, impl RGMIICLK125Pin<T>>,
        mac_addr: [u8; 6],
        phy: P,
    ) -> Self {
        // ST's example configures every ETH pin at VERY_HIGH speed
        config_pins!(
            tx_ctl, tx_d0, tx_d1, tx_d2, tx_d3, rx_clk, rx_ctl, rx_d0, rx_d1, rx_d2, rx_d3, clk125
        );
        // GTX_CLK uses MEDIUM speed in ST's example
        config_pins!(@speed Speed::Medium; gtx_clk);

        let pins = Pins::Rgmii([
            Flex::new(gtx_clk),
            Flex::new(tx_ctl),
            Flex::new(tx_d0),
            Flex::new(tx_d1),
            Flex::new(tx_d2),
            Flex::new(tx_d3),
            Flex::new(rx_clk),
            Flex::new(rx_ctl),
            Flex::new(rx_d0),
            Flex::new(rx_d1),
            Flex::new(rx_d2),
            Flex::new(rx_d3),
            Flex::new(clk125),
        ]);

        Self::new_inner(queue, peri, irq, pins, phy, mac_addr)
    }

    fn new_inner<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<EthTypelevel, InterruptHandler> + 'd,
        pins: Pins<'d>,
        phy: P,
        mac_addr: [u8; 6],
        #[cfg(eth_v2)] eth_sel_phy: EthSelPhy,
    ) -> Self {
        // Enable the necessary clocks
        #[cfg(eth_v2)]
        critical_section::with(|_| {
            crate::pac::RCC.ahb1enr().modify(|w| {
                w.set_ethen(true);
                w.set_ethtxen(true);
                w.set_ethrxen(true);
            });

            crate::pac::SYSCFG.pmcr().modify(|w| w.set_eth_sel_phy(eth_sel_phy));
        });
        #[cfg(eth_v2a)]
        critical_section::with(|_| {
            crate::pac::RCC.ahb5enr().modify(|w| {
                w.set_eth1en(true);
                w.set_eth1macen(true);
                w.set_eth1txen(true);
                w.set_eth1rxen(true);
            });

            let eth1sel: u8 = match pins {
                Pins::Rmii(_) => 0b100,
                Pins::Rgmii(_) => 0b001,
            };

            // Select the PHY interface (ETH1SEL: 0b000 = MII, 0b001 = RGMII,
            // 0b100 = RMII). The ETH1 kernel clock (ETH1CLKSEL = HCLK) and the
            // TX/RX reference clock sources (ETH1GTXCLKSEL/ETH1REFCLKSEL = external,
            // i.e. the GTX clock comes from the PHY's CLK125 output) are left at
            // their reset values.
            crate::pac::RCC.ccipr2().modify(|w| w.set_eth1sel(eth1sel));
        });

        let dma = T::regs().ethernet_dma();
        let mac = T::regs().ethernet_mac();
        let mtl = T::regs().ethernet_mtl();

        // Reset and wait
        dma.dmamr().modify(|w| w.set_swr(true));
        while dma.dmamr().read().swr() {}

        // Program the 1µs tick counter, used as a time base for internal MAC timeouts.
        #[cfg(eth_v2a)]
        {
            let hclk = unsafe { crate::rcc::get_freqs().hclk1.to_hertz() };
            let hclk_mhz = unwrap!(hclk, "ETH requires HCLK to be enabled, but it was not.").0 / 1_000_000;
            mac.mac1ustcr().modify(|w| w.set_tic_1us_cntr(hclk_mhz as u16 - 1));
        }

        mac.maccr().modify(|w| {
            w.set_ipg(0b000); // 96 bit times
            w.set_acs(true);
            #[cfg(eth_v2a)]
            w.set_ps(true);
            w.set_fes(true);
            w.set_dm(true);
            // Enable RX IP header / payload checksum offload (COE). Requires RX
            // store-and-forward (set below). TX insertion is requested per-frame
            // via TDES3.CIC; together with the driver `Capabilities` this lets
            // smoltcp skip software checksums.
            #[cfg(eth_v2a)]
            w.set_ipc(true);
            // TODO: Carrier sense ? ECRSFD
        });

        // Enable RX queue 0 for generic (non-AV) traffic.
        #[cfg(eth_v2a)]
        mac.macrxqc0r().modify(|w| w.set_rxq0en(0b10));

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

        mac.macq_tx_fcr().modify(|w| w.set_pt(0x100));

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

        ch0!(mtl, mtl_rx_qomr).modify(|w| {
            w.set_rsf(true);
            // 4096-byte RX queue
            #[cfg(eth_v2a)]
            w.set_rqs(15);
        });
        ch0!(mtl, mtl_tx_qomr).modify(|w| {
            w.set_tsf(true);
            // Like the RX queues, TX queues reset disabled on the multi-queue
            // eth_v2a MAC. 0b10 = enabled for generic (non-AV) traffic.
            #[cfg(eth_v2a)]
            w.set_txqen(0b10);
            // 2048-byte TX queue
            #[cfg(eth_v2a)]
            w.set_tqs(7);
        });

        // Map RX queue 0 to DMA channel 0 (the reset default, set explicitly).
        #[cfg(eth_v2a)]
        mtl.mtlrxqdmamr().modify(|w| w.set_q0mdmach(false));

        #[cfg(eth_v2a)]
        dma.dmasbmr().modify(|w| {
            w.set_aal(true);
            w.set_blen4(true);
            w.set_fb(true);
            w.set_rd_osr_lmt(3);
            w.set_wr_osr_lmt(3);
        });

        ch0!(dma, dmac_tx_cr).modify(|w| {
            #[cfg(eth_v2)]
            w.set_txpbl(1); // 32 ?
            #[cfg(eth_v2a)]
            w.set_txpbl(32);
        });
        ch0!(dma, dmac_rx_cr).modify(|w| {
            #[cfg(eth_v2)]
            w.set_rxpbl(1); // 32 ?
            #[cfg(eth_v2a)]
            w.set_rxpbl(32);
            w.set_rbsz(RX_BUFFER_SIZE as u16);
        });

        let (tx_state, rx_state) = queue.packet_state.split();

        let mut this = Self {
            _peri: peri,
            _wake_guard: T::RCC_INFO.wake_guard(),
            tx: TDesRing::new(&mut queue.tx_desc, &mut queue.tx_buf, tx_state),
            rx: RDesRing::new(&mut queue.rx_desc, &mut queue.rx_buf, rx_state),
            _pins: pins,
            phy,
            mac_addr,
            link_state: LinkState::Down,
            #[cfg(feature = "ptp")]
            ptp_clock_taken: false,
        };

        fence(Ordering::SeqCst);

        let mac = T::regs().ethernet_mac();
        let mtl = T::regs().ethernet_mtl();
        let dma = T::regs().ethernet_dma();

        mac.maccr().modify(|w| {
            w.set_re(true);
            w.set_te(true);
        });
        ch0!(mtl, mtl_tx_qomr).modify(|w| w.set_ftq(true));

        ch0!(dma, dmac_tx_cr).modify(|w| w.set_st(true));
        ch0!(dma, dmac_rx_cr).modify(|w| w.set_sr(true));

        // Enable interrupts
        ch0!(dma, dmacier).modify(|w| {
            w.set_nie(true);
            w.set_rie(true);
            w.set_tie(true);
        });

        this.phy.phy_reset();
        this.phy.phy_init();

        #[cfg(eth_v2)]
        {
            interrupt::ETH.unpend();
            unsafe { interrupt::ETH.enable() };
        }
        #[cfg(eth_v2a)]
        {
            interrupt::ETH1.unpend();
            unsafe { interrupt::ETH1.enable() };
        }

        this
    }

    /// Start the Ethernet MAC PTP clock.
    #[cfg(feature = "ptp")]
    pub fn start_ptp(&mut self, config: PtpClockConfig) -> PtpClock<T> {
        if self.ptp_clock_taken {
            panic!("Ethernet PTP clock already started");
        }

        let clock = PtpClock::start(config);
        self.ptp_clock_taken = true;
        clock
    }
}

impl<'d, T: Instance, P: Phy> Drop for Ethernet<'d, T, P> {
    fn drop(&mut self) {
        let dma = T::regs().ethernet_dma();
        let mac = T::regs().ethernet_mac();
        let mtl = T::regs().ethernet_mtl();

        // Disable the TX DMA and wait for any previous transmissions to be completed
        ch0!(dma, dmac_tx_cr).modify(|w| w.set_st(false));
        while {
            let txqueue = ch0!(mtl, mtl_tx_qdr).read();
            txqueue.trcsts() == 0b01 || txqueue.txqsts()
        } {}

        // Disable MAC transmitter and receiver
        mac.maccr().modify(|w| {
            w.set_re(false);
            w.set_te(false);
        });

        // Wait for previous receiver transfers to be completed and then disable the RX DMA
        while {
            let rxqueue = ch0!(mtl, mtl_rx_qdr).read();
            rxqueue.rxqsts() != 0b00 || rxqueue.prxq() != 0
        } {}
        ch0!(dma, dmac_rx_cr).modify(|w| w.set_sr(false));
    }
}
