// The v1c ethernet driver was ported to embassy from the awesome stm32-eth project (https://github.com/stm32-rs/stm32-eth).

mod rx_desc;
mod tx_desc;

use core::marker::PhantomData;
use core::sync::atomic::{fence, Ordering};

use embassy_hal_internal::{into_ref, PeripheralRef};
use stm32_metapac::eth::vals::{Apcs, Cr, Dm, DmaomrSr, Fes, Ftf, Ifg, MbProgress, Mw, Pbl, Rsf, St, Tsf};

pub(crate) use self::rx_desc::{RDes, RDesRing};
pub(crate) use self::tx_desc::{TDes, TDesRing};
use super::*;
use crate::gpio::sealed::{AFType, Pin as __GpioPin};
use crate::gpio::AnyPin;
use crate::interrupt::InterruptExt;
#[cfg(eth_v1a)]
use crate::pac::AFIO;
#[cfg(any(eth_v1b, eth_v1c))]
use crate::pac::SYSCFG;
use crate::pac::{ETH, RCC};
use crate::rcc::sealed::RccPeripheral;
use crate::{interrupt, Peripheral};

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
pub struct Ethernet<'d, T: Instance, P: PHY> {
    _peri: PeripheralRef<'d, T>,
    pub(crate) tx: TDesRing<'d>,
    pub(crate) rx: RDesRing<'d>,

    pins: [PeripheralRef<'d, AnyPin>; 9],
    pub(crate) phy: P,
    pub(crate) station_management: EthernetStationManagement<T>,
    pub(crate) mac_addr: [u8; 6],
}

#[cfg(eth_v1a)]
macro_rules! config_in_pins {
    ($($pin:ident),*) => {
        critical_section::with(|_| {
            $(
                // TODO properly create a set_as_input function
                $pin.set_as_af($pin.af_num(), AFType::Input);
            )*
        })
    }
}

#[cfg(eth_v1a)]
macro_rules! config_af_pins {
    ($($pin:ident),*) => {
        critical_section::with(|_| {
            $(
                // We are lucky here, this configures to max speed (50MHz)
                $pin.set_as_af($pin.af_num(), AFType::OutputPushPull);
            )*
        })
    };
}

#[cfg(any(eth_v1b, eth_v1c))]
macro_rules! config_pins {
    ($($pin:ident),*) => {
        critical_section::with(|_| {
            $(
                $pin.set_as_af($pin.af_num(), AFType::OutputPushPull);
                $pin.set_speed(crate::gpio::Speed::VeryHigh);
            )*
        })
    };
}

impl<'d, T: Instance, P: PHY> Ethernet<'d, T, P> {
    /// safety: the returned instance is not leak-safe
    pub fn new<const TX: usize, const RX: usize>(
        queue: &'d mut PacketQueue<TX, RX>,
        peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::ETH, InterruptHandler> + 'd,
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
        into_ref!(peri, ref_clk, mdio, mdc, crs, rx_d0, rx_d1, tx_d0, tx_d1, tx_en);

        // Enable the necessary Clocks
        #[cfg(eth_v1a)]
        critical_section::with(|_| {
            RCC.apb2enr().modify(|w| w.set_afioen(true));

            // Select RMII (Reduced Media Independent Interface)
            // Must be done prior to enabling peripheral clock
            AFIO.mapr().modify(|w| w.set_mii_rmii_sel(true));

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

            // RMII (Reduced Media Independent Interface)
            SYSCFG.pmc().modify(|w| w.set_mii_rmii_sel(true));
        });

        #[cfg(eth_v1a)]
        {
            config_in_pins!(ref_clk, rx_d0, rx_d1);
            config_af_pins!(mdio, mdc, tx_d0, tx_d1, tx_en);
        }

        #[cfg(any(eth_v1b, eth_v1c))]
        config_pins!(ref_clk, mdio, mdc, crs, rx_d0, rx_d1, tx_d0, tx_d1, tx_en);

        let dma = ETH.ethernet_dma();
        let mac = ETH.ethernet_mac();

        // Reset and wait
        dma.dmabmr().modify(|w| w.set_sr(true));
        while dma.dmabmr().read().sr() {}

        mac.maccr().modify(|w| {
            w.set_ifg(Ifg::IFG96); // inter frame gap 96 bit times
            w.set_apcs(Apcs::STRIP); // automatic padding and crc stripping
            w.set_fes(Fes::FES100); // fast ethernet speed
            w.set_dm(Dm::FULLDUPLEX); // full duplex
                                      // TODO: Carrier sense ? ECRSFD
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
            w.set_tsf(Tsf::STOREFORWARD);
            w.set_rsf(Rsf::STOREFORWARD);
        });

        dma.dmabmr().modify(|w| {
            w.set_pbl(Pbl::PBL32) // programmable burst length - 32 ?
        });

        // TODO MTU size setting not found for v1 ethernet, check if correct

        let hclk = <T as RccPeripheral>::frequency();
        let hclk_mhz = hclk.0 / 1_000_000;

        // Set the MDC clock frequency in the range 1MHz - 2.5MHz
        let clock_range = match hclk_mhz {
            0..=24 => panic!("Invalid HCLK frequency - should be at least 25 MHz."),
            25..=34 => Cr::CR_20_35,     // Divide by 16
            35..=59 => Cr::CR_35_60,     // Divide by 26
            60..=99 => Cr::CR_60_100,    // Divide by 42
            100..=149 => Cr::CR_100_150, // Divide by 62
            150..=216 => Cr::CR_150_168, // Divide by 102
            _ => {
                panic!("HCLK results in MDC clock > 2.5MHz even for the highest CSR clock divider")
            }
        };

        let pins = [
            ref_clk.map_into(),
            mdio.map_into(),
            mdc.map_into(),
            crs.map_into(),
            rx_d0.map_into(),
            rx_d1.map_into(),
            tx_d0.map_into(),
            tx_d1.map_into(),
            tx_en.map_into(),
        ];

        let mut this = Self {
            _peri: peri,
            pins,
            phy: phy,
            station_management: EthernetStationManagement {
                peri: PhantomData,
                clock_range: clock_range,
            },
            mac_addr,
            tx: TDesRing::new(&mut queue.tx_desc, &mut queue.tx_buf),
            rx: RDesRing::new(&mut queue.rx_desc, &mut queue.rx_buf),
        };

        fence(Ordering::SeqCst);

        let mac = ETH.ethernet_mac();
        let dma = ETH.ethernet_dma();

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

        this.phy.phy_reset(&mut this.station_management);
        this.phy.phy_init(&mut this.station_management);

        interrupt::ETH.unpend();
        unsafe { interrupt::ETH.enable() };

        this
    }
}

/// Ethernet station management interface.
pub struct EthernetStationManagement<T: Instance> {
    peri: PhantomData<T>,
    clock_range: Cr,
}

unsafe impl<T: Instance> StationManagement for EthernetStationManagement<T> {
    fn smi_read(&mut self, phy_addr: u8, reg: u8) -> u16 {
        let mac = ETH.ethernet_mac();

        mac.macmiiar().modify(|w| {
            w.set_pa(phy_addr);
            w.set_mr(reg);
            w.set_mw(Mw::READ); // read operation
            w.set_cr(self.clock_range);
            w.set_mb(MbProgress::BUSY); // indicate that operation is in progress
        });
        while mac.macmiiar().read().mb() == MbProgress::BUSY {}
        mac.macmiidr().read().md()
    }

    fn smi_write(&mut self, phy_addr: u8, reg: u8, val: u16) {
        let mac = ETH.ethernet_mac();

        mac.macmiidr().write(|w| w.set_md(val));
        mac.macmiiar().modify(|w| {
            w.set_pa(phy_addr);
            w.set_mr(reg);
            w.set_mw(Mw::WRITE); // write
            w.set_cr(self.clock_range);
            w.set_mb(MbProgress::BUSY);
        });
        while mac.macmiiar().read().mb() == MbProgress::BUSY {}
    }
}

impl<'d, T: Instance, P: PHY> Drop for Ethernet<'d, T, P> {
    fn drop(&mut self) {
        let dma = ETH.ethernet_dma();
        let mac = ETH.ethernet_mac();

        // Disable the TX DMA and wait for any previous transmissions to be completed
        dma.dmaomr().modify(|w| w.set_st(St::STOPPED));

        // Disable MAC transmitter and receiver
        mac.maccr().modify(|w| {
            w.set_re(false);
            w.set_te(false);
        });

        dma.dmaomr().modify(|w| w.set_sr(DmaomrSr::STOPPED));

        critical_section::with(|_| {
            for pin in self.pins.iter_mut() {
                pin.set_as_disconnected();
            }
        })
    }
}
