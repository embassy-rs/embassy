use core::marker::PhantomData;
use core::sync::atomic::{fence, Ordering};
use core::task::Waker;

use embassy::util::Unborrow;
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::peripheral::{PeripheralMutex, PeripheralState, StateStorage};
use embassy_hal_common::unborrow;
use embassy_net::{Device, DeviceCapabilities, LinkState, PacketBuf, MTU};

use crate::gpio::sealed::Pin as _;
use crate::gpio::{sealed::AFType, AnyPin, Speed};
use crate::pac::{ETH, RCC, SYSCFG};

mod descriptors;
use super::*;
use descriptors::DescriptorRing;

pub struct State<'d, T: Instance, const TX: usize, const RX: usize>(
    StateStorage<Inner<'d, T, TX, RX>>,
);
impl<'d, T: Instance, const TX: usize, const RX: usize> State<'d, T, TX, RX> {
    pub fn new() -> Self {
        Self(StateStorage::new())
    }
}
pub struct Ethernet<'d, T: Instance, P: PHY, const TX: usize, const RX: usize> {
    state: PeripheralMutex<'d, Inner<'d, T, TX, RX>>,
    pins: [AnyPin; 9],
    _phy: P,
    clock_range: u8,
    phy_addr: u8,
    mac_addr: [u8; 6],
}

macro_rules! config_pins {
    ($($pin:ident),*) => {
        // NOTE(unsafe) Exclusive access to the registers
        critical_section::with(|_| unsafe {
            $(
                $pin.set_as_af($pin.af_num(), AFType::OutputPushPull);
                $pin.set_speed(Speed::VeryHigh);
            )*
        })
    };
}

impl<'d, T: Instance, P: PHY, const TX: usize, const RX: usize> Ethernet<'d, T, P, TX, RX> {
    /// safety: the returned instance is not leak-safe
    pub unsafe fn new(
        state: &'d mut State<'d, T, TX, RX>,
        peri: impl Unborrow<Target = T> + 'd,
        interrupt: impl Unborrow<Target = crate::interrupt::ETH> + 'd,
        ref_clk: impl Unborrow<Target = impl RefClkPin<T>> + 'd,
        mdio: impl Unborrow<Target = impl MDIOPin<T>> + 'd,
        mdc: impl Unborrow<Target = impl MDCPin<T>> + 'd,
        crs: impl Unborrow<Target = impl CRSPin<T>> + 'd,
        rx_d0: impl Unborrow<Target = impl RXD0Pin<T>> + 'd,
        rx_d1: impl Unborrow<Target = impl RXD1Pin<T>> + 'd,
        tx_d0: impl Unborrow<Target = impl TXD0Pin<T>> + 'd,
        tx_d1: impl Unborrow<Target = impl TXD1Pin<T>> + 'd,
        tx_en: impl Unborrow<Target = impl TXEnPin<T>> + 'd,
        phy: P,
        mac_addr: [u8; 6],
        phy_addr: u8,
    ) -> Self {
        unborrow!(interrupt, ref_clk, mdio, mdc, crs, rx_d0, rx_d1, tx_d0, tx_d1, tx_en);

        // Enable the necessary Clocks
        // NOTE(unsafe) We have exclusive access to the registers
        critical_section::with(|_| {
            RCC.apb4enr().modify(|w| w.set_syscfgen(true));
            RCC.ahb1enr().modify(|w| {
                w.set_eth1macen(true);
                w.set_eth1txen(true);
                w.set_eth1rxen(true);
            });

            // RMII
            SYSCFG.pmcr().modify(|w| w.set_epis(0b100));
        });

        config_pins!(ref_clk, mdio, mdc, crs, rx_d0, rx_d1, tx_d0, tx_d1, tx_en);

        // NOTE(unsafe) We are ourselves not leak-safe.
        let state = PeripheralMutex::new_unchecked(interrupt, &mut state.0, || Inner::new(peri));

        // NOTE(unsafe) We have exclusive access to the registers
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

        mtl.mtlrx_qomr().modify(|w| w.set_rsf(true));
        mtl.mtltx_qomr().modify(|w| w.set_tsf(true));

        dma.dmactx_cr().modify(|w| w.set_txpbl(1)); // 32 ?
        dma.dmacrx_cr().modify(|w| {
            w.set_rxpbl(1); // 32 ?
            w.set_rbsz(MTU as u16);
        });

        // NOTE(unsafe) We got the peripheral singleton, which means that `rcc::init` was called
        let hclk = crate::rcc::get_freqs().ahb1;
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

        let pins = [
            ref_clk.degrade(),
            mdio.degrade(),
            mdc.degrade(),
            crs.degrade(),
            rx_d0.degrade(),
            rx_d1.degrade(),
            tx_d0.degrade(),
            tx_d1.degrade(),
            tx_en.degrade(),
        ];

        let mut this = Self {
            state,
            pins,
            _phy: phy,
            clock_range,
            phy_addr,
            mac_addr,
        };

        this.state.with(|s| {
            s.desc_ring.init();

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
        });
        P::phy_reset(&mut this);
        P::phy_init(&mut this);

        this
    }
}

unsafe impl<'d, T: Instance, P: PHY, const TX: usize, const RX: usize> StationManagement
    for Ethernet<'d, T, P, TX, RX>
{
    fn smi_read(&mut self, reg: u8) -> u16 {
        // NOTE(unsafe) These registers aren't used in the interrupt and we have `&mut self`
        unsafe {
            let mac = ETH.ethernet_mac();

            mac.macmdioar().modify(|w| {
                w.set_pa(self.phy_addr);
                w.set_rda(reg);
                w.set_goc(0b11); // read
                w.set_cr(self.clock_range);
                w.set_mb(true);
            });
            while mac.macmdioar().read().mb() {}
            mac.macmdiodr().read().md()
        }
    }

    fn smi_write(&mut self, reg: u8, val: u16) {
        // NOTE(unsafe) These registers aren't used in the interrupt and we have `&mut self`
        unsafe {
            let mac = ETH.ethernet_mac();

            mac.macmdiodr().write(|w| w.set_md(val));
            mac.macmdioar().modify(|w| {
                w.set_pa(self.phy_addr);
                w.set_rda(reg);
                w.set_goc(0b01); // write
                w.set_cr(self.clock_range);
                w.set_mb(true);
            });
            while mac.macmdioar().read().mb() {}
        }
    }
}

impl<'d, T: Instance, P: PHY, const TX: usize, const RX: usize> Device
    for Ethernet<'d, T, P, TX, RX>
{
    fn is_transmit_ready(&mut self) -> bool {
        self.state.with(|s| s.desc_ring.tx.available())
    }

    fn transmit(&mut self, pkt: PacketBuf) {
        self.state.with(|s| unwrap!(s.desc_ring.tx.transmit(pkt)));
    }

    fn receive(&mut self) -> Option<PacketBuf> {
        self.state.with(|s| s.desc_ring.rx.pop_packet())
    }

    fn register_waker(&mut self, waker: &Waker) {
        WAKER.register(waker);
    }

    fn capabilities(&mut self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = MTU;
        caps.max_burst_size = Some(TX.min(RX));
        caps
    }

    fn link_state(&mut self) -> LinkState {
        if P::poll_link(self) {
            LinkState::Up
        } else {
            LinkState::Down
        }
    }

    fn ethernet_address(&mut self) -> [u8; 6] {
        self.mac_addr
    }
}

impl<'d, T: Instance, P: PHY, const TX: usize, const RX: usize> Drop
    for Ethernet<'d, T, P, TX, RX>
{
    fn drop(&mut self) {
        // NOTE(unsafe) We have `&mut self` and the interrupt doesn't use this registers
        unsafe {
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
        }

        // NOTE(unsafe) Exclusive access to the regs
        critical_section::with(|_| unsafe {
            for pin in self.pins.iter_mut() {
                pin.set_as_disconnected();
            }
        })
    }
}

//----------------------------------------------------------------------

struct Inner<'d, T: Instance, const TX: usize, const RX: usize> {
    _peri: PhantomData<&'d mut T>,
    desc_ring: DescriptorRing<TX, RX>,
}

impl<'d, T: Instance, const TX: usize, const RX: usize> Inner<'d, T, TX, RX> {
    pub fn new(_peri: impl Unborrow<Target = T> + 'd) -> Self {
        Self {
            _peri: PhantomData,
            desc_ring: DescriptorRing::new(),
        }
    }
}

impl<'d, T: Instance, const TX: usize, const RX: usize> PeripheralState for Inner<'d, T, TX, RX> {
    type Interrupt = crate::interrupt::ETH;

    fn on_interrupt(&mut self) {
        unwrap!(self.desc_ring.tx.on_interrupt());
        self.desc_ring.rx.on_interrupt();

        WAKER.wake();

        // TODO: Check and clear more flags
        unsafe {
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
}

static WAKER: AtomicWaker = AtomicWaker::new();
