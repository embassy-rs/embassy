mod descriptors;

use core::marker::PhantomData;
use core::sync::atomic::{fence, Ordering};

use embassy_hal_internal::{into_ref, PeripheralRef};

pub(crate) use self::descriptors::{RDes, RDesRing, TDes, TDesRing};
use super::*;
use crate::gpio::sealed::{AFType, Pin as _};
use crate::gpio::{AnyPin, Speed};
use crate::interrupt::InterruptExt;
use crate::pac::ETH;
use crate::rcc::sealed::RccPeripheral;
use crate::{interrupt, Peripheral};

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
pub struct Ethernet<'d, T: Instance, P: PHY> {
    _peri: PeripheralRef<'d, T>,
    pub(crate) tx: TDesRing<'d>,
    pub(crate) rx: RDesRing<'d>,
    pins: Pins<'d>,
    pub(crate) phy: P,
    pub(crate) station_management: EthernetStationManagement<T>,
    pub(crate) mac_addr: [u8; 6],
}

/// Pins of ethernet driver.
enum Pins<'d> {
    Rmii([PeripheralRef<'d, AnyPin>; 9]),
    Mii([PeripheralRef<'d, AnyPin>; 14]),
}

macro_rules! config_pins {
    ($($pin:ident),*) => {
        critical_section::with(|_| {
            $(
                $pin.set_as_af($pin.af_num(), AFType::OutputPushPull);
                $pin.set_speed(Speed::VeryHigh);
            )*
        })
    };
}

impl<'d, T: Instance, P: PHY> Ethernet<'d, T, P> {
    /// Create a new RMII ethernet driver using 9 pins.
    pub fn new<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: impl Peripheral<P = T> + 'd,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
        ref_clk: impl Peripheral<P = impl RefClkPin<T>> + 'd,
        mdio: impl Peripheral<P = impl MDIOPin<T>> + 'd,
        mdc: impl Peripheral<P = impl MDCPin<T>> + 'd,
        crs: impl Peripheral<P = impl CRSPin<T>> + 'd,
        rx_d0: impl Peripheral<P = impl RXD0Pin<T>> + 'd,
        rx_d1: impl Peripheral<P = impl RXD1Pin<T>> + 'd,
        tx_d0: impl Peripheral<P = impl TXD0Pin<T>> + 'd,
        tx_d1: impl Peripheral<P = impl TXD1Pin<T>> + 'd,
        tx_en: impl Peripheral<P = impl TXEnPin<T>> + 'd,
        phy: P,
        mac_addr: [u8; 6],
    ) -> Self {
        // Enable the necessary Clocks
        #[cfg(not(rcc_h5))]
        critical_section::with(|_| {
            crate::pac::RCC.ahb1enr().modify(|w| {
                w.set_eth1macen(true);
                w.set_eth1txen(true);
                w.set_eth1rxen(true);
            });

            crate::pac::SYSCFG.pmcr().modify(|w| w.set_epis(0b100));
        });

        #[cfg(rcc_h5)]
        critical_section::with(|_| {
            crate::pac::RCC.apb3enr().modify(|w| w.set_sbsen(true));

            crate::pac::RCC.ahb1enr().modify(|w| {
                w.set_ethen(true);
                w.set_ethtxen(true);
                w.set_ethrxen(true);
            });

            crate::pac::SYSCFG
                .pmcr()
                .modify(|w| w.set_eth_sel_phy(crate::pac::syscfg::vals::EthSelPhy::B_0X4));
        });

        into_ref!(ref_clk, mdio, mdc, crs, rx_d0, rx_d1, tx_d0, tx_d1, tx_en);
        config_pins!(ref_clk, mdio, mdc, crs, rx_d0, rx_d1, tx_d0, tx_d1, tx_en);

        let pins = Pins::Rmii([
            ref_clk.map_into(),
            mdio.map_into(),
            mdc.map_into(),
            crs.map_into(),
            rx_d0.map_into(),
            rx_d1.map_into(),
            tx_d0.map_into(),
            tx_d1.map_into(),
            tx_en.map_into(),
        ]);

        Self::new_inner(queue, peri, irq, pins, phy, mac_addr)
    }

    /// Create a new MII ethernet driver using 14 pins.
    pub fn new_mii<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: impl Peripheral<P = T> + 'd,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
        rx_clk: impl Peripheral<P = impl RXClkPin<T>> + 'd,
        tx_clk: impl Peripheral<P = impl TXClkPin<T>> + 'd,
        mdio: impl Peripheral<P = impl MDIOPin<T>> + 'd,
        mdc: impl Peripheral<P = impl MDCPin<T>> + 'd,
        rxdv: impl Peripheral<P = impl RXDVPin<T>> + 'd,
        rx_d0: impl Peripheral<P = impl RXD0Pin<T>> + 'd,
        rx_d1: impl Peripheral<P = impl RXD1Pin<T>> + 'd,
        rx_d2: impl Peripheral<P = impl RXD2Pin<T>> + 'd,
        rx_d3: impl Peripheral<P = impl RXD3Pin<T>> + 'd,
        tx_d0: impl Peripheral<P = impl TXD0Pin<T>> + 'd,
        tx_d1: impl Peripheral<P = impl TXD1Pin<T>> + 'd,
        tx_d2: impl Peripheral<P = impl TXD2Pin<T>> + 'd,
        tx_d3: impl Peripheral<P = impl TXD3Pin<T>> + 'd,
        tx_en: impl Peripheral<P = impl TXEnPin<T>> + 'd,
        phy: P,
        mac_addr: [u8; 6],
    ) -> Self {
        // Enable necessary clocks.
        #[cfg(not(rcc_h5))]
        critical_section::with(|_| {
            crate::pac::RCC.ahb1enr().modify(|w| {
                w.set_eth1macen(true);
                w.set_eth1txen(true);
                w.set_eth1rxen(true);
            });

            crate::pac::SYSCFG.pmcr().modify(|w| w.set_epis(0b000));
        });

        #[cfg(rcc_h5)]
        critical_section::with(|_| {
            crate::pac::RCC.apb3enr().modify(|w| w.set_sbsen(true));

            crate::pac::RCC.ahb1enr().modify(|w| {
                w.set_ethen(true);
                w.set_ethtxen(true);
                w.set_ethrxen(true);
            });

            // TODO: This is for RMII - what would MII need here?
            crate::pac::SYSCFG
                .pmcr()
                .modify(|w| w.set_eth_sel_phy(crate::pac::syscfg::vals::EthSelPhy::B_0X4));
        });

        into_ref!(rx_clk, tx_clk, mdio, mdc, rxdv, rx_d0, rx_d1, rx_d2, rx_d3, tx_d0, tx_d1, tx_d2, tx_d3, tx_en);
        config_pins!(rx_clk, tx_clk, mdio, mdc, rxdv, rx_d0, rx_d1, rx_d2, rx_d3, tx_d0, tx_d1, tx_d2, tx_d3, tx_en);

        let pins = Pins::Mii([
            rx_clk.map_into(),
            tx_clk.map_into(),
            mdio.map_into(),
            mdc.map_into(),
            rxdv.map_into(),
            rx_d0.map_into(),
            rx_d1.map_into(),
            rx_d2.map_into(),
            rx_d3.map_into(),
            tx_d0.map_into(),
            tx_d1.map_into(),
            tx_d2.map_into(),
            tx_d3.map_into(),
            tx_en.map_into(),
        ]);

        Self::new_inner(queue, peri, irq, pins, phy, mac_addr)
    }

    fn new_inner<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
        pins: Pins<'d>,
        phy: P,
        mac_addr: [u8; 6],
    ) -> Self {
        let dma = ETH.ethernet_dma();
        let mac = ETH.ethernet_mac();
        let mtl = ETH.ethernet_mtl();

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

        let hclk = <T as RccPeripheral>::frequency();
        let hclk_mhz = hclk.0 / 1_000_000;

        // Set the MDC clock frequency in the range 1MHz - 2.5MHz
        let clock_range = match hclk_mhz {
            0..=34 => 2,    // Divide by 16
            35..=59 => 3,   // Divide by 26
            60..=99 => 0,   // Divide by 42
            100..=149 => 1, // Divide by 62
            150..=249 => 4, // Divide by 102
            250..=310 => 5, // Divide by 124
            _ => {
                panic!("HCLK results in MDC clock > 2.5MHz even for the highest CSR clock divider")
            }
        };

        let mut this = Self {
            _peri: peri.into_ref(),
            tx: TDesRing::new(&mut queue.tx_desc, &mut queue.tx_buf),
            rx: RDesRing::new(&mut queue.rx_desc, &mut queue.rx_buf),
            pins,
            phy,
            station_management: EthernetStationManagement {
                peri: PhantomData,
                clock_range: clock_range,
            },
            mac_addr,
        };

        fence(Ordering::SeqCst);

        let mac = ETH.ethernet_mac();
        let mtl = ETH.ethernet_mtl();
        let dma = ETH.ethernet_dma();

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

        this.phy.phy_reset(&mut this.station_management);
        this.phy.phy_init(&mut this.station_management);

        interrupt::ETH.unpend();
        unsafe { interrupt::ETH.enable() };

        this
    }
}

/// Ethernet SMI driver.
pub struct EthernetStationManagement<T: Instance> {
    peri: PhantomData<T>,
    clock_range: u8,
}

unsafe impl<T: Instance> StationManagement for EthernetStationManagement<T> {
    fn smi_read(&mut self, phy_addr: u8, reg: u8) -> u16 {
        let mac = ETH.ethernet_mac();

        mac.macmdioar().modify(|w| {
            w.set_pa(phy_addr);
            w.set_rda(reg);
            w.set_goc(0b11); // read
            w.set_cr(self.clock_range);
            w.set_mb(true);
        });
        while mac.macmdioar().read().mb() {}
        mac.macmdiodr().read().md()
    }

    fn smi_write(&mut self, phy_addr: u8, reg: u8, val: u16) {
        let mac = ETH.ethernet_mac();

        mac.macmdiodr().write(|w| w.set_md(val));
        mac.macmdioar().modify(|w| {
            w.set_pa(phy_addr);
            w.set_rda(reg);
            w.set_goc(0b01); // write
            w.set_cr(self.clock_range);
            w.set_mb(true);
        });
        while mac.macmdioar().read().mb() {}
    }
}

impl<'d, T: Instance, P: PHY> Drop for Ethernet<'d, T, P> {
    fn drop(&mut self) {
        let dma = ETH.ethernet_dma();
        let mac = ETH.ethernet_mac();
        let mtl = ETH.ethernet_mtl();

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
