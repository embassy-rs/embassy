mod descriptors;

use core::marker::PhantomData;
use core::sync::atomic::{fence, Ordering};

use embassy_hal_internal::Peri;
use stm32_metapac::syscfg::vals::EthSelPhy;

pub(crate) use self::descriptors::{RDes, RDesRing, TDes, TDesRing};
use super::*;
use crate::gpio::{AfType, AnyPin, OutputType, SealedPin as _, Speed};
use crate::interrupt;
use crate::interrupt::InterruptExt;
use crate::pac::ETH;
use crate::rcc::SealedRccPeripheral;

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
    pub(crate) station_management: EthernetStationManagement<T>,
    pub(crate) mac_addr: [u8; 6],
}

/// Pins of ethernet driver.
enum Pins<'d> {
    Rmii([Peri<'d, AnyPin>; 9]),
    Mii([Peri<'d, AnyPin>; 14]),
}

macro_rules! config_pins {
    ($($pin:ident),*) => {
        critical_section::with(|_| {
            $(
                // TODO: shouldn't some pins be configured as inputs?
                $pin.set_as_af($pin.af_num(), AfType::output(OutputType::PushPull, Speed::VeryHigh));
            )*
        })
    };
}

impl<'d, T: Instance, P: Phy> Ethernet<'d, T, P> {
    /// Create a new RMII ethernet driver using 9 pins.
    pub fn new<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
        ref_clk: Peri<'d, impl RefClkPin<T>>,
        mdio: Peri<'d, impl MDIOPin<T>>,
        mdc: Peri<'d, impl MDCPin<T>>,
        crs: Peri<'d, impl CRSPin<T>>,
        rx_d0: Peri<'d, impl RXD0Pin<T>>,
        rx_d1: Peri<'d, impl RXD1Pin<T>>,
        tx_d0: Peri<'d, impl TXD0Pin<T>>,
        tx_d1: Peri<'d, impl TXD1Pin<T>>,
        tx_en: Peri<'d, impl TXEnPin<T>>,
        phy: P,
        mac_addr: [u8; 6],
    ) -> Self {
        // Enable the necessary clocks
        critical_section::with(|_| {
            crate::pac::RCC.ahb1enr().modify(|w| {
                w.set_ethen(true);
                w.set_ethtxen(true);
                w.set_ethrxen(true);
            });

            crate::pac::SYSCFG.pmcr().modify(|w| w.set_eth_sel_phy(EthSelPhy::RMII));
        });

        config_pins!(ref_clk, mdio, mdc, crs, rx_d0, rx_d1, tx_d0, tx_d1, tx_en);

        let pins = Pins::Rmii([
            ref_clk.into(),
            mdio.into(),
            mdc.into(),
            crs.into(),
            rx_d0.into(),
            rx_d1.into(),
            tx_d0.into(),
            tx_d1.into(),
            tx_en.into(),
        ]);

        Self::new_inner(queue, peri, irq, pins, phy, mac_addr)
    }

    /// Create a new MII ethernet driver using 14 pins.
    pub fn new_mii<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
        rx_clk: Peri<'d, impl RXClkPin<T>>,
        tx_clk: Peri<'d, impl TXClkPin<T>>,
        mdio: Peri<'d, impl MDIOPin<T>>,
        mdc: Peri<'d, impl MDCPin<T>>,
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
        phy: P,
        mac_addr: [u8; 6],
    ) -> Self {
        // Enable the necessary clocks
        critical_section::with(|_| {
            crate::pac::RCC.ahb1enr().modify(|w| {
                w.set_ethen(true);
                w.set_ethtxen(true);
                w.set_ethrxen(true);
            });

            crate::pac::SYSCFG
                .pmcr()
                .modify(|w| w.set_eth_sel_phy(EthSelPhy::MII_GMII));
        });

        config_pins!(rx_clk, tx_clk, mdio, mdc, rxdv, rx_d0, rx_d1, rx_d2, rx_d3, tx_d0, tx_d1, tx_d2, tx_d3, tx_en);

        let pins = Pins::Mii([
            rx_clk.into(),
            tx_clk.into(),
            mdio.into(),
            mdc.into(),
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

        Self::new_inner(queue, peri, irq, pins, phy, mac_addr)
    }

    fn new_inner<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
        pins: Pins<'d>,
        phy: P,
        mac_addr: [u8; 6],
    ) -> Self {
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

        let hclk = <T as SealedRccPeripheral>::frequency();
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
            _peri: peri,
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

impl<T: Instance> StationManagement for EthernetStationManagement<T> {
    fn smi_read(&mut self, phy_addr: u8, reg: u8) -> u16 {
        let mac = T::regs().ethernet_mac();

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
        let mac = T::regs().ethernet_mac();

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
