use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::{fence, Ordering};
use core::task::Waker;

use embassy::util::{AtomicWaker, Unborrow};
use embassy_hal_common::peripheral::{PeripheralMutex, PeripheralState};
use embassy_hal_common::unborrow;
use embassy_net::{Device, DeviceCapabilities, LinkState, PacketBuf, MTU};

use crate::gpio::sealed::Pin as __GpioPin;
use crate::gpio::AnyPin;
use crate::gpio::Pin as GpioPin;
use crate::pac::gpio::vals::Ospeedr;
use crate::pac::{ETH, RCC, SYSCFG};
use crate::peripherals;

mod descriptors;
use super::{StationManagement, PHY};
use descriptors::DescriptorRing;

pub struct Ethernet<'d, P: PHY, const TX: usize, const RX: usize> {
    state: PeripheralMutex<Inner<'d, TX, RX>>,
    pins: [AnyPin; 9],
    _phy: P,
    clock_range: u8,
    phy_addr: u8,
    mac_addr: [u8; 6],
}

impl<'d, P: PHY, const TX: usize, const RX: usize> Ethernet<'d, P, TX, RX> {
    pub fn new(
        peri: impl Unborrow<Target = peripherals::ETH> + 'd,
        interrupt: impl Unborrow<Target = crate::interrupt::ETH> + 'd,
        ref_clk: impl Unborrow<Target = impl RefClkPin> + 'd,
        mdio: impl Unborrow<Target = impl MDIOPin> + 'd,
        mdc: impl Unborrow<Target = impl MDCPin> + 'd,
        crs: impl Unborrow<Target = impl CRSPin> + 'd,
        rx_d0: impl Unborrow<Target = impl RXD0Pin> + 'd,
        rx_d1: impl Unborrow<Target = impl RXD1Pin> + 'd,
        tx_d0: impl Unborrow<Target = impl TXD0Pin> + 'd,
        tx_d1: impl Unborrow<Target = impl TXD1Pin> + 'd,
        tx_en: impl Unborrow<Target = impl TXEnPin> + 'd,
        phy: P,
        mac_addr: [u8; 6],
        phy_addr: u8,
    ) -> Self {
        unborrow!(interrupt, ref_clk, mdio, mdc, crs, rx_d0, rx_d1, tx_d0, tx_d1, tx_en);

        // Enable the necessary Clocks
        // NOTE(unsafe) We have exclusive access to the registers
        critical_section::with(|_| unsafe {
            RCC.apb4enr().modify(|w| w.set_syscfgen(true));
            RCC.ahb1enr().modify(|w| {
                w.set_eth1macen(true);
                w.set_eth1txen(true);
                w.set_eth1rxen(true);
            });

            // RMII
            SYSCFG.pmcr().modify(|w| w.set_epis(0b100));
        });

        ref_clk.configure();
        mdio.configure();
        mdc.configure();
        crs.configure();
        rx_d0.configure();
        rx_d1.configure();
        tx_d0.configure();
        tx_d1.configure();
        tx_en.configure();

        let inner = Inner::new(peri);
        let state = PeripheralMutex::new(inner, interrupt);

        // NOTE(unsafe) We have exclusive access to the registers
        unsafe {
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

            mac.maca0lr().write(|w| {
                w.set_addrlo(
                    u32::from(mac_addr[0])
                        | (u32::from(mac_addr[1]) << 8)
                        | (u32::from(mac_addr[2]) << 16)
                        | (u32::from(mac_addr[3]) << 24),
                )
            });
            mac.maca0hr()
                .modify(|w| w.set_addrhi(u16::from(mac_addr[4]) | (u16::from(mac_addr[5]) << 8)));

            mac.macpfr().modify(|w| w.set_saf(true));
            mac.macqtx_fcr().modify(|w| w.set_pt(0x100));

            mtl.mtlrx_qomr().modify(|w| w.set_rsf(true));
            mtl.mtltx_qomr().modify(|w| w.set_tsf(true));

            dma.dmactx_cr().modify(|w| w.set_txpbl(1)); // 32 ?
            dma.dmacrx_cr().modify(|w| {
                w.set_rxpbl(1); // 32 ?
                w.set_rbsz(MTU as u16);
            });
        }

        // NOTE(unsafe) We got the peripheral singleton, which means that `rcc::init` was called
        let hclk = unsafe { crate::rcc::get_freqs().ahb1 };
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

        Self {
            state,
            pins,
            _phy: phy,
            clock_range,
            phy_addr,
            mac_addr,
        }
    }

    pub fn init(self: Pin<&mut Self>) {
        // NOTE(unsafe) We won't move this
        let this = unsafe { self.get_unchecked_mut() };
        let mut mutex = unsafe { Pin::new_unchecked(&mut this.state) };
        // SAFETY: The lifetime of `Inner` is only due to `PhantomData`; it isn't actually referencing any data with that lifetime.
        unsafe { mutex.as_mut().register_interrupt_unchecked() }

        mutex.with(|s| {
            s.desc_ring.init();

            fence(Ordering::SeqCst);

            unsafe {
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
            }
        });
        P::phy_reset(this);
        P::phy_init(this);
    }
}

unsafe impl<'d, P: PHY, const TX: usize, const RX: usize> StationManagement
    for Ethernet<'d, P, TX, RX>
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

impl<'d, P: PHY, const TX: usize, const RX: usize> Device for Pin<&mut Ethernet<'d, P, TX, RX>> {
    fn is_transmit_ready(&mut self) -> bool {
        // NOTE(unsafe) We won't move out of self
        let this = unsafe { self.as_mut().get_unchecked_mut() };
        let mutex = unsafe { Pin::new_unchecked(&mut this.state) };

        mutex.with(|s| s.desc_ring.tx.available())
    }

    fn transmit(&mut self, pkt: PacketBuf) {
        // NOTE(unsafe) We won't move out of self
        let this = unsafe { self.as_mut().get_unchecked_mut() };
        let mutex = unsafe { Pin::new_unchecked(&mut this.state) };

        mutex.with(|s| unwrap!(s.desc_ring.tx.transmit(pkt)));
    }

    fn receive(&mut self) -> Option<PacketBuf> {
        // NOTE(unsafe) We won't move out of self
        let this = unsafe { self.as_mut().get_unchecked_mut() };
        let mutex = unsafe { Pin::new_unchecked(&mut this.state) };

        mutex.with(|s| s.desc_ring.rx.pop_packet())
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
        // NOTE(unsafe) We won't move out of self
        let this = unsafe { self.as_mut().get_unchecked_mut() };

        if P::poll_link(this) {
            LinkState::Up
        } else {
            LinkState::Down
        }
    }

    fn ethernet_address(&mut self) -> [u8; 6] {
        // NOTE(unsafe) We won't move out of self
        let this = unsafe { self.as_mut().get_unchecked_mut() };

        this.mac_addr
    }
}

impl<'d, P: PHY, const TX: usize, const RX: usize> Drop for Ethernet<'d, P, TX, RX> {
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

        for pin in self.pins.iter_mut() {
            // NOTE(unsafe) Exclusive access to the regs
            critical_section::with(|_| unsafe {
                pin.set_as_analog();
                pin.block()
                    .ospeedr()
                    .modify(|w| w.set_ospeedr(pin.pin() as usize, Ospeedr::LOWSPEED));
            })
        }
    }
}

//----------------------------------------------------------------------

struct Inner<'d, const TX: usize, const RX: usize> {
    _peri: PhantomData<&'d mut peripherals::ETH>,
    desc_ring: DescriptorRing<TX, RX>,
}

impl<'d, const TX: usize, const RX: usize> Inner<'d, TX, RX> {
    pub fn new(_peri: impl Unborrow<Target = peripherals::ETH> + 'd) -> Self {
        Self {
            _peri: PhantomData,
            desc_ring: DescriptorRing::new(),
        }
    }
}

impl<'d, const TX: usize, const RX: usize> PeripheralState for Inner<'d, TX, RX> {
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

mod sealed {
    use super::*;

    pub trait RefClkPin: GpioPin {
        fn configure(&mut self);
    }

    pub trait MDIOPin: GpioPin {
        fn configure(&mut self);
    }

    pub trait MDCPin: GpioPin {
        fn configure(&mut self);
    }

    pub trait CRSPin: GpioPin {
        fn configure(&mut self);
    }

    pub trait RXD0Pin: GpioPin {
        fn configure(&mut self);
    }

    pub trait RXD1Pin: GpioPin {
        fn configure(&mut self);
    }

    pub trait TXD0Pin: GpioPin {
        fn configure(&mut self);
    }

    pub trait TXD1Pin: GpioPin {
        fn configure(&mut self);
    }

    pub trait TXEnPin: GpioPin {
        fn configure(&mut self);
    }
}

pub trait RefClkPin: sealed::RefClkPin + 'static {}

pub trait MDIOPin: sealed::MDIOPin + 'static {}

pub trait MDCPin: sealed::MDCPin + 'static {}

pub trait CRSPin: sealed::CRSPin + 'static {}

pub trait RXD0Pin: sealed::RXD0Pin + 'static {}

pub trait RXD1Pin: sealed::RXD1Pin + 'static {}

pub trait TXD0Pin: sealed::TXD0Pin + 'static {}

pub trait TXD1Pin: sealed::TXD1Pin + 'static {}

pub trait TXEnPin: sealed::TXEnPin + 'static {}

static WAKER: AtomicWaker = AtomicWaker::new();

macro_rules! impl_pin {
    ($pin:ident, $signal:ident, $af:expr) => {
        impl sealed::$signal for peripherals::$pin {
            fn configure(&mut self) {
                // NOTE(unsafe) Exclusive access to the registers
                critical_section::with(|_| unsafe {
                    self.set_as_af($af);
                    self.block()
                        .ospeedr()
                        .modify(|w| w.set_ospeedr(self.pin() as usize, Ospeedr::VERYHIGHSPEED));
                })
            }
        }

        impl $signal for peripherals::$pin {}
    };
}

crate::pac::peripheral_pins!(
    ($inst:ident, eth, ETH, $pin:ident, REF_CLK) => {
        impl_pin!($pin, RefClkPin, 11);
    };
    ($inst:ident, eth, ETH, $pin:ident, MDIO, $af:expr) => {
        impl_pin!($pin, MDIOPin, $af);
    };
    ($inst:ident, eth, ETH, $pin:ident, MDC, $af:expr) => {
        impl_pin!($pin, MDCPin, $af);
    };
    ($inst:ident, eth, ETH, $pin:ident, CRS_DV, $af:expr) => {
        impl_pin!($pin, CRSPin, $af);
    };
    ($inst:ident, eth, ETH, $pin:ident, RXD0, $af:expr) => {
        impl_pin!($pin, RXD0Pin, $af);
    };
    ($inst:ident, eth, ETH, $pin:ident, RXD1, $af:expr) => {
        impl_pin!($pin, RXD1Pin, $af);
    };
    ($inst:ident, eth, ETH, $pin:ident, TXD0, $af:expr) => {
        impl_pin!($pin, TXD0Pin, $af);
    };
    ($inst:ident, eth, ETH, $pin:ident, TXD1, $af:expr) => {
        impl_pin!($pin, TXD1Pin, $af);
    };
    ($inst:ident, eth, ETH, $pin:ident, TX_EN, $af:expr) => {
        impl_pin!($pin, TXEnPin, $af);
    };
);
