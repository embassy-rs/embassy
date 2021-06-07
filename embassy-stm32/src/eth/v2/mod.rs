use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::{fence, Ordering};

use embassy::util::{AtomicWaker, Unborrow};
use embassy_extras::peripheral::{PeripheralMutex, PeripheralState};
use embassy_extras::unborrow;
use embassy_net::MTU;

use crate::gpio::sealed::Pin as __GpioPin;
use crate::gpio::AnyPin;
use crate::gpio::Pin as GpioPin;
use crate::interrupt::Interrupt;
use crate::pac::gpio::vals::Ospeedr;
use crate::pac::ETH;
use crate::peripherals;

mod descriptors;
use descriptors::DescriptorRing;

/// Station Management Interface (SMI) on an ethernet PHY
pub trait StationManagement {
    /// Read a register over SMI.
    fn smi_read(&mut self, reg: u8) -> u16;
    /// Write a register over SMI.
    fn smi_write(&mut self, reg: u8, val: u16);
}

/// Traits for an Ethernet PHY
pub trait PHY {
    /// Reset PHY and wait for it to come out of reset.
    fn phy_reset(&mut self);
    /// PHY initialisation.
    fn phy_init(&mut self);
}

pub struct Ethernet<'d, T: Instance, const TX: usize, const RX: usize> {
    state: PeripheralMutex<Inner<'d, T, TX, RX>>,
    pins: [AnyPin; 9],
}

impl<'d, T: Instance, const TX: usize, const RX: usize> Ethernet<'d, T, TX, RX> {
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
        mac_addr: [u8; 6],
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

        Self { state, pins }
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
    }
}

impl<'d, T: Instance, const TX: usize, const RX: usize> Drop for Ethernet<'d, T, TX, RX> {
    fn drop(&mut self) {
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
