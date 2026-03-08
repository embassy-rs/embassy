// The v1c ethernet driver was ported to embassy from the awesome stm32-eth project (https://github.com/stm32-rs/stm32-eth).

mod rx_desc;
mod tx_desc;

use core::sync::atomic::{Ordering, fence};

use embassy_hal_internal::Peri;
use stm32_metapac::eth::vals::{Apcs, Dm, DmaomrSr, Fes, Ftf, Ifg, Pbl, Rsf, St, Tsf};

pub(crate) use self::rx_desc::{RDes, RDesRing};
pub(crate) use self::tx_desc::{TDes, TDesRing};
use super::*;
#[cfg(eth_v1a)]
use crate::gpio::Pull;
use crate::gpio::{AfType, Flex, OutputType, Speed};
use crate::interrupt;
use crate::interrupt::InterruptExt;
#[cfg(eth_v1a)]
use crate::pac::AFIO;
#[cfg(any(eth_v1b, eth_v1c))]
use crate::pac::SYSCFG;
use crate::pac::{ETH, RCC};

/// Interrupt handler.
pub struct InterruptHandler {}

impl interrupt::typelevel::Handler<interrupt::typelevel::ETH> for InterruptHandler {
    unsafe fn on_interrupt() {
        WAKER.wake();

        // TODO: Check and clear more flags
        let dma = ETH.ethernet_dma();

        dma.dmasr().modify(|w| {
            w.set_ts(true);
            w.set_rs(true);
            w.set_nis(true);
        });
        // Delay two peripheral's clock
        dma.dmasr().read();
        dma.dmasr().read();
    }
}

/// Ethernet driver.
pub struct Ethernet<'d, T: Instance, P: Phy> {
    _peri: Peri<'d, T>,
    pub(crate) tx: TDesRing<'d>,
    pub(crate) rx: RDesRing<'d>,

    _pins: Pins<'d>,
    pub(crate) phy: P,
    pub(crate) mac_addr: [u8; 6],
}

/// Pins of ethernet driver.
enum Pins<'d> {
    #[allow(unused)]
    Rmii([Flex<'d>; 7]),
    #[allow(unused)]
    Mii([Flex<'d>; 12]),
}

#[cfg(eth_v1a)]
macro_rules! config_in_pins {
    ($($pin:ident),*) => {
        critical_section::with(|_| {
            $(
                // TODO properly create a set_as_input function
                set_as_af!($pin, AfType::input(Pull::None));
            )*
        })
    }
}

#[cfg(eth_v1a)]
macro_rules! config_af_pins {
    ($($pin:ident),*) => {
        critical_section::with(|_| {
            $(
                set_as_af!($pin, AfType::output(OutputType::PushPull, Speed::VeryHigh));
            )*
        })
    };
}

#[cfg(any(eth_v1b, eth_v1c))]
macro_rules! config_pins {
    ($($pin:ident),*) => {
        critical_section::with(|_| {
            $(
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
    ///
    /// safety: the returned instance is not leak-safe
    pub fn new<const TX: usize, const RX: usize, #[cfg(afio)] A>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
        ref_clk: Peri<'d, if_afio!(impl RefClkPin<T, A>)>,
        crs: Peri<'d, if_afio!(impl CRSPin<T, A>)>,
        rx_d0: Peri<'d, if_afio!(impl RXD0Pin<T, A>)>,
        rx_d1: Peri<'d, if_afio!(impl RXD1Pin<T, A>)>,
        tx_d0: Peri<'d, if_afio!(impl TXD0Pin<T, A>)>,
        tx_d1: Peri<'d, if_afio!(impl TXD1Pin<T, A>)>,
        tx_en: Peri<'d, if_afio!(impl TXEnPin<T, A>)>,
        mac_addr: [u8; 6],
        sma: Peri<'d, SMA>,
        mdio: Peri<'d, if_afio!(impl MDIOPin<SMA, A>)>,
        mdc: Peri<'d, if_afio!(impl MDCPin<SMA, A>)>,
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
    pub fn new_mii<const TX: usize, const RX: usize, #[cfg(afio)] A>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
        rx_clk: Peri<'d, if_afio!(impl RXClkPin<T, A>)>,
        tx_clk: Peri<'d, if_afio!(impl TXClkPin<T, A>)>,
        rxdv: Peri<'d, if_afio!(impl RXDVPin<T, A>)>,
        rx_d0: Peri<'d, if_afio!(impl RXD0Pin<T, A>)>,
        rx_d1: Peri<'d, if_afio!(impl RXD1Pin<T, A>)>,
        rx_d2: Peri<'d, if_afio!(impl RXD2Pin<T, A>)>,
        rx_d3: Peri<'d, if_afio!(impl RXD3Pin<T, A>)>,
        tx_d0: Peri<'d, if_afio!(impl TXD0Pin<T, A>)>,
        tx_d1: Peri<'d, if_afio!(impl TXD1Pin<T, A>)>,
        tx_d2: Peri<'d, if_afio!(impl TXD2Pin<T, A>)>,
        tx_d3: Peri<'d, if_afio!(impl TXD3Pin<T, A>)>,
        tx_en: Peri<'d, if_afio!(impl TXEnPin<T, A>)>,
        mac_addr: [u8; 6],
        sma: Peri<'d, SMA>,
        mdio: Peri<'d, if_afio!(impl MDIOPin<SMA, A>)>,
        mdc: Peri<'d, if_afio!(impl MDCPin<SMA, A>)>,
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
    /// safety: the returned instance is not leak-safe
    pub fn new_with_phy<const TX: usize, const RX: usize, #[cfg(afio)] A>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
        ref_clk: Peri<'d, if_afio!(impl RefClkPin<T, A>)>,
        crs: Peri<'d, if_afio!(impl CRSPin<T, A>)>,
        rx_d0: Peri<'d, if_afio!(impl RXD0Pin<T, A>)>,
        rx_d1: Peri<'d, if_afio!(impl RXD1Pin<T, A>)>,
        tx_d0: Peri<'d, if_afio!(impl TXD0Pin<T, A>)>,
        tx_d1: Peri<'d, if_afio!(impl TXD1Pin<T, A>)>,
        tx_en: Peri<'d, if_afio!(impl TXEnPin<T, A>)>,
        mac_addr: [u8; 6],
        phy: P,
    ) -> Self {
        #[cfg(eth_v1a)]
        {
            config_in_pins!(ref_clk, rx_d0, rx_d1);
            config_af_pins!(tx_d0, tx_d1, tx_en);
        }

        #[cfg(any(eth_v1b, eth_v1c))]
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

        Self::new_inner(queue, peri, irq, pins, phy, mac_addr, true)
    }

    fn new_inner<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
        pins: Pins<'d>,
        phy: P,
        mac_addr: [u8; 6],
        rmii_mii_sel: bool,
    ) -> Self {
        // Enable the necessary Clocks
        #[cfg(eth_v1a)]
        critical_section::with(|_| {
            RCC.apb2enr().modify(|w| w.set_afioen(true));

            // Select (R)MII (Reduced Media Independent Interface)
            // Must be done prior to enabling peripheral clock
            AFIO.mapr().modify(|w| {
                w.set_mii_rmii_sel(rmii_mii_sel);
                w.set_swj_cfg(crate::pac::afio::vals::SwjCfg::NO_OP);
            });

            RCC.ahbenr().modify(|w| {
                w.set_ethen(true);
                w.set_ethtxen(true);
                w.set_ethrxen(true);
            });
        });

        #[cfg(any(eth_v1b, eth_v1c))]
        critical_section::with(|_| {
            RCC.ahb1enr().modify(|w| {
                w.set_ethen(true);
                w.set_ethtxen(true);
                w.set_ethrxen(true);
            });

            // (R)MII ((Reduced) Media Independent Interface)
            SYSCFG.pmc().modify(|w| w.set_mii_rmii_sel(rmii_mii_sel));
        });

        let dma = T::regs().ethernet_dma();
        let mac = T::regs().ethernet_mac();

        // Reset and wait
        dma.dmabmr().modify(|w| w.set_sr(true));
        while dma.dmabmr().read().sr() {}

        mac.maccr().modify(|w| {
            w.set_ifg(Ifg::IFG96); // inter frame gap 96 bit times
            w.set_apcs(Apcs::STRIP); // automatic padding and crc stripping
            w.set_fes(Fes::FES100); // fast ethernet speed
            w.set_dm(Dm::FULL_DUPLEX); // full duplex
            // TODO: Carrier sense ? ECRSFD
        });

        // Set the mac to pass all multicast packets
        mac.macffr().modify(|w| {
            w.set_pam(true);
        });

        // Note: Writing to LR triggers synchronisation of both LR and HR into the MAC core,
        // so the LR write must happen after the HR write.
        mac.maca0hr()
            .modify(|w| w.set_maca0h(u16::from(mac_addr[4]) | (u16::from(mac_addr[5]) << 8)));
        mac.maca0lr().write(|w| {
            w.set_maca0l(
                u32::from(mac_addr[0])
                    | (u32::from(mac_addr[1]) << 8)
                    | (u32::from(mac_addr[2]) << 16)
                    | (u32::from(mac_addr[3]) << 24),
            )
        });

        // pause time
        mac.macfcr().modify(|w| w.set_pt(0x100));

        // Transfer and Forward, Receive and Forward
        dma.dmaomr().modify(|w| {
            w.set_tsf(Tsf::STORE_FORWARD);
            w.set_rsf(Rsf::STORE_FORWARD);
        });

        dma.dmabmr().modify(|w| {
            w.set_pbl(Pbl::PBL32) // programmable burst length - 32 ?
        });

        // TODO MTU size setting not found for v1 ethernet, check if correct

        let mut this = Self {
            _peri: peri,
            _pins: pins,
            phy: phy,
            mac_addr,
            tx: TDesRing::new(&mut queue.tx_desc, &mut queue.tx_buf),
            rx: RDesRing::new(&mut queue.rx_desc, &mut queue.rx_buf),
        };

        fence(Ordering::SeqCst);

        let mac = T::regs().ethernet_mac();
        let dma = T::regs().ethernet_dma();

        mac.maccr().modify(|w| {
            w.set_re(true);
            w.set_te(true);
        });
        dma.dmaomr().modify(|w| {
            w.set_ftf(Ftf::FLUSH); // flush transmit fifo (queue)
            w.set_st(St::STARTED); // start transmitting channel
            w.set_sr(DmaomrSr::STARTED); // start receiving channel
        });

        this.rx.demand_poll();

        // Enable interrupts
        dma.dmaier().modify(|w| {
            w.set_nise(true);
            w.set_rie(true);
            w.set_tie(true);
        });

        this.phy.phy_reset();
        this.phy.phy_init();

        interrupt::ETH.unpend();
        unsafe { interrupt::ETH.enable() };

        this
    }

    /// Create a new MII ethernet driver using 12 pins.
    pub fn new_mii_with_phy<const TX: usize, const RX: usize, #[cfg(afio)] A>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
        rx_clk: Peri<'d, if_afio!(impl RXClkPin<T, A>)>,
        tx_clk: Peri<'d, if_afio!(impl TXClkPin<T, A>)>,
        rxdv: Peri<'d, if_afio!(impl RXDVPin<T, A>)>,
        rx_d0: Peri<'d, if_afio!(impl RXD0Pin<T, A>)>,
        rx_d1: Peri<'d, if_afio!(impl RXD1Pin<T, A>)>,
        rx_d2: Peri<'d, if_afio!(impl RXD2Pin<T, A>)>,
        rx_d3: Peri<'d, if_afio!(impl RXD3Pin<T, A>)>,
        tx_d0: Peri<'d, if_afio!(impl TXD0Pin<T, A>)>,
        tx_d1: Peri<'d, if_afio!(impl TXD1Pin<T, A>)>,
        tx_d2: Peri<'d, if_afio!(impl TXD2Pin<T, A>)>,
        tx_d3: Peri<'d, if_afio!(impl TXD3Pin<T, A>)>,
        tx_en: Peri<'d, if_afio!(impl TXEnPin<T, A>)>,
        mac_addr: [u8; 6],
        phy: P,
    ) -> Self {
        #[cfg(eth_v1a)]
        {
            config_in_pins!(rx_clk, tx_clk, rx_d0, rx_d1, rx_d2, rx_d3, rxdv);
            config_af_pins!(tx_d0, tx_d1, tx_d2, tx_d3, tx_en);
        }

        #[cfg(any(eth_v1b, eth_v1c))]
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

        Self::new_inner(queue, peri, irq, pins, phy, mac_addr, false)
    }
}

impl<'d, T: Instance, P: Phy> Drop for Ethernet<'d, T, P> {
    fn drop(&mut self) {
        let dma = T::regs().ethernet_dma();
        let mac = T::regs().ethernet_mac();

        // Disable the TX DMA and wait for any previous transmissions to be completed
        dma.dmaomr().modify(|w| w.set_st(St::STOPPED));

        // Disable MAC transmitter and receiver
        mac.maccr().modify(|w| {
            w.set_re(false);
            w.set_te(false);
        });

        dma.dmaomr().modify(|w| w.set_sr(DmaomrSr::STOPPED));
    }
}
