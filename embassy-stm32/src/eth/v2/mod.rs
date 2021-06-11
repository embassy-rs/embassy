use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::{fence, Ordering};
use core::task::Waker;

use embassy::util::{AtomicWaker, Unborrow};
use embassy_extras::peripheral::{PeripheralMutex, PeripheralState};
use embassy_extras::unborrow;
use embassy_net::{Device, DeviceCapabilities, LinkState, PacketBuf, MTU};

use crate::gpio::sealed::Pin as __GpioPin;
use crate::gpio::AnyPin;
use crate::gpio::Pin as GpioPin;
use crate::interrupt::Interrupt;
use crate::pac::gpio::vals::Ospeedr;
use crate::pac::ETH;
use crate::peripherals;
use crate::time::Hertz;

mod descriptors;
use super::{StationManagement, PHY};
use descriptors::DescriptorRing;

pub struct Ethernet<'d, T: Instance, P: PHY, const TX: usize, const RX: usize> {
    state: PeripheralMutex<Inner<'d, T, TX, RX>>,
    pins: [AnyPin; 9],
    _phy: P,
    clock_range: u8,
    phy_addr: u8,
    mac_addr: [u8; 6],
}

impl<'d, T: Instance, P: PHY, const TX: usize, const RX: usize> Ethernet<'d, T, P, TX, RX> {
    pub fn new(
        peri: impl Unborrow<Target = T> + 'd,
        interrupt: impl Unborrow<Target = T::Interrupt> + 'd,
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
        hclk: Hertz,
        phy_addr: u8,
    ) -> Self {
        unborrow!(interrupt, ref_clk, mdio, mdc, crs, rx_d0, rx_d1, tx_d0, tx_d1, tx_en);

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

            // 200 MHz ?
            mac.mac1ustcr().modify(|w| w.set_tic_1us_cntr(200 - 1));

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

            // TODO: Enable filtering once we get the basics working
            mac.macpfr().modify(|w| w.set_ra(true));
            mac.macqtx_fcr().modify(|w| w.set_pt(0x100));

            mtl.mtlrx_qomr().modify(|w| w.set_rsf(true));
            mtl.mtltx_qomr().modify(|w| w.set_tsf(true));

            // TODO: Address aligned beats plus fixed burst ?
            dma.dmactx_cr().modify(|w| w.set_txpbl(1)); // 32 ?
            dma.dmacrx_cr().modify(|w| {
                w.set_rxpbl(1); // 32 ?
                w.set_rbsz(MTU as u16);
            });
        }

        // Set the MDC clock frequency in the range 1MHz - 2.5MHz
        let hclk_mhz = hclk.0 / 1_000_000;
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
        let mutex = unsafe { Pin::new_unchecked(&mut this.state) };

        mutex.with(|s, _| {
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
    for Pin<&mut Ethernet<'d, T, P, TX, RX>>
{
    fn is_transmit_ready(&mut self) -> bool {
        // NOTE(unsafe) We won't move out of self
        let this = unsafe { self.as_mut().get_unchecked_mut() };
        let mutex = unsafe { Pin::new_unchecked(&mut this.state) };

        mutex.with(|s, _| s.desc_ring.tx.available())
    }

    fn transmit(&mut self, pkt: PacketBuf) {
        // NOTE(unsafe) We won't move out of self
        let this = unsafe { self.as_mut().get_unchecked_mut() };
        let mutex = unsafe { Pin::new_unchecked(&mut this.state) };

        mutex.with(|s, _| unwrap!(s.desc_ring.tx.transmit(pkt)));
    }

    fn receive(&mut self) -> Option<PacketBuf> {
        // NOTE(unsafe) We won't move out of self
        let this = unsafe { self.as_mut().get_unchecked_mut() };
        let mutex = unsafe { Pin::new_unchecked(&mut this.state) };

        mutex.with(|s, _| s.desc_ring.rx.pop_packet())
    }

    fn register_waker(&mut self, waker: &Waker) {
        T::state().register(waker);
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
    type Interrupt = T::Interrupt;

    fn on_interrupt(&mut self) {
        unwrap!(self.desc_ring.tx.on_interrupt());
        self.desc_ring.rx.on_interrupt();

        T::state().wake();

        // TODO: Check and clear more flags
        unsafe {
            let dma = ETH.ethernet_dma();

            dma.dmacsr().modify(|w| {
                w.set_ti(false);
                w.set_ri(false);
            });
            // Delay two peripheral's clock
            dma.dmacsr().read();
            dma.dmacsr().read();
        }
    }
}

mod sealed {
    use super::*;

    pub trait Instance {
        type Interrupt: Interrupt;

        fn state() -> &'static AtomicWaker;
    }

    pub trait RefClkPin<T: Instance>: GpioPin {
        fn configure(&mut self);
    }

    pub trait MDIOPin<T: Instance>: GpioPin {
        fn configure(&mut self);
    }

    pub trait MDCPin<T: Instance>: GpioPin {
        fn configure(&mut self);
    }

    pub trait CRSPin<T: Instance>: GpioPin {
        fn configure(&mut self);
    }

    pub trait RXD0Pin<T: Instance>: GpioPin {
        fn configure(&mut self);
    }

    pub trait RXD1Pin<T: Instance>: GpioPin {
        fn configure(&mut self);
    }

    pub trait TXD0Pin<T: Instance>: GpioPin {
        fn configure(&mut self);
    }

    pub trait TXD1Pin<T: Instance>: GpioPin {
        fn configure(&mut self);
    }

    pub trait TXEnPin<T: Instance>: GpioPin {
        fn configure(&mut self);
    }
}

pub trait Instance: sealed::Instance + 'static {}

pub trait RefClkPin<T: Instance>: sealed::RefClkPin<T> + 'static {}

pub trait MDIOPin<T: Instance>: sealed::MDIOPin<T> + 'static {}

pub trait MDCPin<T: Instance>: sealed::MDCPin<T> + 'static {}

pub trait CRSPin<T: Instance>: sealed::CRSPin<T> + 'static {}

pub trait RXD0Pin<T: Instance>: sealed::RXD0Pin<T> + 'static {}

pub trait RXD1Pin<T: Instance>: sealed::RXD1Pin<T> + 'static {}

pub trait TXD0Pin<T: Instance>: sealed::TXD0Pin<T> + 'static {}

pub trait TXD1Pin<T: Instance>: sealed::TXD1Pin<T> + 'static {}

pub trait TXEnPin<T: Instance>: sealed::TXEnPin<T> + 'static {}

crate::pac::peripherals!(
    (eth, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::$inst;

            fn state() -> &'static AtomicWaker {
                static WAKER: AtomicWaker = AtomicWaker::new();
                &WAKER
            }
        }

        impl Instance for peripherals::$inst {}
    };
);

macro_rules! impl_pin {
    ($inst:ident, $pin:ident, $signal:ident, $af:expr) => {
        impl sealed::$signal<peripherals::$inst> for peripherals::$pin {
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

        impl $signal<peripherals::$inst> for peripherals::$pin {}
    };
}

crate::pac::peripheral_pins!(
    ($inst:ident, eth, ETH, $pin:ident, REF_CLK, $af:expr) =>  {
        impl_pin!($inst, $pin, RefClkPin, $af);
    };
    ($inst:ident, eth, ETH, $pin:ident, MDIO, $af:expr) =>  {
        impl_pin!($inst, $pin, MDIOPin, $af);
    };
    ($inst:ident, eth, ETH, $pin:ident, MDC, $af:expr) =>  {
        impl_pin!($inst, $pin, MDCPin, $af);
    };
    ($inst:ident, eth, ETH, $pin:ident, CRS_DV, $af:expr) =>  {
        impl_pin!($inst, $pin, CRSPin, $af);
    };
    ($inst:ident, eth, ETH, $pin:ident, RXD0, $af:expr) =>  {
        impl_pin!($inst, $pin, RXD0Pin, $af);
    };
    ($inst:ident, eth, ETH, $pin:ident, RXD1, $af:expr) =>  {
        impl_pin!($inst, $pin, RXD1Pin, $af);
    };
    ($inst:ident, eth, ETH, $pin:ident, TXD0, $af:expr) =>  {
        impl_pin!($inst, $pin, TXD0Pin, $af);
    };
    ($inst:ident, eth, ETH, $pin:ident, TXD1, $af:expr) =>  {
        impl_pin!($inst, $pin, TXD1Pin, $af);
    };
    ($inst:ident, eth, ETH, $pin:ident, TX_EN, $af:expr) =>  {
        impl_pin!($inst, $pin, TXEnPin, $af);
    };
);
