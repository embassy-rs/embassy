use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, Peripheral};
use embassy_usb_driver::{EndpointAddress, EndpointAllocError, EndpointType, Event, Unsupported};
use embassy_usb_synopsys_otg::otg_v1::vals::Dspd;
use embassy_usb_synopsys_otg::otg_v1::Otg;
pub use embassy_usb_synopsys_otg::Config;
use embassy_usb_synopsys_otg::{
    on_interrupt as on_interrupt_impl, Bus as OtgBus, ControlPipe, Driver as OtgDriver, Endpoint, In, OtgInstance, Out,
    PhyType, State,
};

use crate::gpio::AFType;
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;
use crate::rcc::{self, RccPeripheral};

const MAX_EP_COUNT: usize = 9;

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        trace!("irq");
        let r = T::regs();
        let state = T::state();

        let setup_late_cnak = quirk_setup_late_cnak(r);

        on_interrupt_impl(r, state, T::ENDPOINT_COUNT, setup_late_cnak);
    }
}

macro_rules! config_ulpi_pins {
    ($($pin:ident),*) => {
        into_ref!($($pin),*);
        critical_section::with(|_| {
            $(
                $pin.set_as_af($pin.af_num(), AFType::OutputPushPull);
                #[cfg(gpio_v2)]
                $pin.set_speed(crate::gpio::Speed::VeryHigh);
            )*
        })
    };
}

// From `synopsys-usb-otg` crate:
// This calculation doesn't correspond to one in a Reference Manual.
// In fact, the required number of words is higher than indicated in RM.
// The following numbers are pessimistic and were figured out empirically.
const RX_FIFO_EXTRA_SIZE_WORDS: u16 = 30;

/// USB driver.
pub struct Driver<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    inner: OtgDriver<'d, MAX_EP_COUNT>,
}

impl<'d, T: Instance> Driver<'d, T> {
    /// Initializes USB OTG peripheral with internal Full-Speed PHY.
    ///
    /// # Arguments
    ///
    /// * `ep_out_buffer` - An internal buffer used to temporarily store received packets.
    /// Must be large enough to fit all OUT endpoint max packet sizes.
    /// Endpoint allocation will fail if it is too small.
    pub fn new_fs(
        _peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        dp: impl Peripheral<P = impl DpPin<T>> + 'd,
        dm: impl Peripheral<P = impl DmPin<T>> + 'd,
        ep_out_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        into_ref!(dp, dm);

        dp.set_as_af(dp.af_num(), AFType::OutputPushPull);
        dm.set_as_af(dm.af_num(), AFType::OutputPushPull);

        let regs = T::regs();

        let instance = OtgInstance {
            regs,
            state: T::state(),
            fifo_depth_words: T::FIFO_DEPTH_WORDS,
            extra_rx_fifo_words: RX_FIFO_EXTRA_SIZE_WORDS,
            endpoint_count: T::ENDPOINT_COUNT,
            phy_type: PhyType::InternalFullSpeed,
            quirk_setup_late_cnak: quirk_setup_late_cnak(regs),
            calculate_trdt_fn: calculate_trdt::<T>,
        };

        Self {
            inner: OtgDriver::new(ep_out_buffer, instance, config),
            phantom: PhantomData,
        }
    }

    /// Initializes USB OTG peripheral with external High-Speed PHY.
    ///
    /// # Arguments
    ///
    /// * `ep_out_buffer` - An internal buffer used to temporarily store received packets.
    /// Must be large enough to fit all OUT endpoint max packet sizes.
    /// Endpoint allocation will fail if it is too small.
    pub fn new_hs_ulpi(
        _peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        ulpi_clk: impl Peripheral<P = impl UlpiClkPin<T>> + 'd,
        ulpi_dir: impl Peripheral<P = impl UlpiDirPin<T>> + 'd,
        ulpi_nxt: impl Peripheral<P = impl UlpiNxtPin<T>> + 'd,
        ulpi_stp: impl Peripheral<P = impl UlpiStpPin<T>> + 'd,
        ulpi_d0: impl Peripheral<P = impl UlpiD0Pin<T>> + 'd,
        ulpi_d1: impl Peripheral<P = impl UlpiD1Pin<T>> + 'd,
        ulpi_d2: impl Peripheral<P = impl UlpiD2Pin<T>> + 'd,
        ulpi_d3: impl Peripheral<P = impl UlpiD3Pin<T>> + 'd,
        ulpi_d4: impl Peripheral<P = impl UlpiD4Pin<T>> + 'd,
        ulpi_d5: impl Peripheral<P = impl UlpiD5Pin<T>> + 'd,
        ulpi_d6: impl Peripheral<P = impl UlpiD6Pin<T>> + 'd,
        ulpi_d7: impl Peripheral<P = impl UlpiD7Pin<T>> + 'd,
        ep_out_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        assert!(T::HIGH_SPEED == true, "Peripheral is not capable of high-speed USB");

        config_ulpi_pins!(
            ulpi_clk, ulpi_dir, ulpi_nxt, ulpi_stp, ulpi_d0, ulpi_d1, ulpi_d2, ulpi_d3, ulpi_d4, ulpi_d5, ulpi_d6,
            ulpi_d7
        );

        let regs = T::regs();

        let instance = OtgInstance {
            regs: T::regs(),
            state: T::state(),
            fifo_depth_words: T::FIFO_DEPTH_WORDS,
            extra_rx_fifo_words: RX_FIFO_EXTRA_SIZE_WORDS,
            endpoint_count: T::ENDPOINT_COUNT,
            phy_type: PhyType::ExternalHighSpeed,
            quirk_setup_late_cnak: quirk_setup_late_cnak(regs),
            calculate_trdt_fn: calculate_trdt::<T>,
        };

        Self {
            inner: OtgDriver::new(ep_out_buffer, instance, config),
            phantom: PhantomData,
        }
    }
}

impl<'d, T: Instance> embassy_usb_driver::Driver<'d> for Driver<'d, T> {
    type EndpointOut = Endpoint<'d, Out>;
    type EndpointIn = Endpoint<'d, In>;
    type ControlPipe = ControlPipe<'d>;
    type Bus = Bus<'d, T>;

    fn alloc_endpoint_in(
        &mut self,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointIn, EndpointAllocError> {
        self.inner.alloc_endpoint_in(ep_type, max_packet_size, interval_ms)
    }

    fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointOut, EndpointAllocError> {
        self.inner.alloc_endpoint_out(ep_type, max_packet_size, interval_ms)
    }

    fn start(self, control_max_packet_size: u16) -> (Self::Bus, Self::ControlPipe) {
        let (bus, cp) = self.inner.start(control_max_packet_size);

        (
            Bus {
                phantom: PhantomData,
                inner: bus,
                inited: false,
            },
            cp,
        )
    }
}

/// USB bus.
pub struct Bus<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    inner: OtgBus<'d, MAX_EP_COUNT>,
    inited: bool,
}

impl<'d, T: Instance> Bus<'d, T> {
    fn init(&mut self) {
        super::common_init::<T>();

        // Enable ULPI clock if external PHY is used
        let phy_type = self.inner.phy_type();
        let _ulpien = !phy_type.internal();

        #[cfg(any(stm32f2, stm32f4, stm32f7))]
        if T::HIGH_SPEED {
            critical_section::with(|_| {
                let rcc = crate::pac::RCC;
                rcc.ahb1enr().modify(|w| w.set_usb_otg_hsulpien(_ulpien));
                rcc.ahb1lpenr().modify(|w| w.set_usb_otg_hsulpilpen(_ulpien));
            });
        }

        #[cfg(stm32h7)]
        critical_section::with(|_| {
            let rcc = crate::pac::RCC;
            if T::HIGH_SPEED {
                rcc.ahb1enr().modify(|w| w.set_usb_otg_hs_ulpien(_ulpien));
                rcc.ahb1lpenr().modify(|w| w.set_usb_otg_hs_ulpilpen(_ulpien));
            } else {
                rcc.ahb1enr().modify(|w| w.set_usb_otg_fs_ulpien(_ulpien));
                rcc.ahb1lpenr().modify(|w| w.set_usb_otg_fs_ulpilpen(_ulpien));
            }
        });

        let r = T::regs();
        let core_id = r.cid().read().0;
        trace!("Core id {:08x}", core_id);

        // Wait for AHB ready.
        while !r.grstctl().read().ahbidl() {}

        // Configure as device.
        self.inner.configure_as_device();

        // Configuring Vbus sense and SOF output
        match core_id {
            0x0000_1200 | 0x0000_1100 => self.inner.config_v1(),
            0x0000_2000 | 0x0000_2100 | 0x0000_2300 | 0x0000_3000 | 0x0000_3100 => self.inner.config_v2v3(),
            _ => unimplemented!("Unknown USB core id {:X}", core_id),
        }
    }

    fn disable(&mut self) {
        T::Interrupt::disable();

        rcc::disable::<T>();
        self.inited = false;

        #[cfg(stm32l4)]
        crate::pac::PWR.cr2().modify(|w| w.set_usv(false));
        // Cannot disable PWR, because other peripherals might be using it
    }
}

impl<'d, T: Instance> embassy_usb_driver::Bus for Bus<'d, T> {
    async fn poll(&mut self) -> Event {
        if !self.inited {
            self.init();
            self.inited = true;
        }

        self.inner.poll().await
    }

    fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool) {
        self.inner.endpoint_set_stalled(ep_addr, stalled)
    }

    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool {
        self.inner.endpoint_is_stalled(ep_addr)
    }

    fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool) {
        self.inner.endpoint_set_enabled(ep_addr, enabled)
    }

    async fn enable(&mut self) {
        self.inner.enable().await
    }

    async fn disable(&mut self) {
        // NOTE: inner call is a no-op
        self.inner.disable().await
    }

    async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
        self.inner.remote_wakeup().await
    }
}

impl<'d, T: Instance> Drop for Bus<'d, T> {
    fn drop(&mut self) {
        Bus::disable(self);
    }
}

trait SealedInstance {
    const HIGH_SPEED: bool;
    const FIFO_DEPTH_WORDS: u16;
    const ENDPOINT_COUNT: usize;

    fn regs() -> Otg;
    fn state() -> &'static State<{ MAX_EP_COUNT }>;
}

/// USB instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + RccPeripheral + 'static {
    /// Interrupt for this USB instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

// Internal PHY pins
pin_trait!(DpPin, Instance);
pin_trait!(DmPin, Instance);

// External PHY pins
pin_trait!(UlpiClkPin, Instance);
pin_trait!(UlpiDirPin, Instance);
pin_trait!(UlpiNxtPin, Instance);
pin_trait!(UlpiStpPin, Instance);
pin_trait!(UlpiD0Pin, Instance);
pin_trait!(UlpiD1Pin, Instance);
pin_trait!(UlpiD2Pin, Instance);
pin_trait!(UlpiD3Pin, Instance);
pin_trait!(UlpiD4Pin, Instance);
pin_trait!(UlpiD5Pin, Instance);
pin_trait!(UlpiD6Pin, Instance);
pin_trait!(UlpiD7Pin, Instance);

foreach_interrupt!(
    (USB_OTG_FS, otg, $block:ident, GLOBAL, $irq:ident) => {
        impl SealedInstance for crate::peripherals::USB_OTG_FS {
            const HIGH_SPEED: bool = false;

            cfg_if::cfg_if! {
                if #[cfg(stm32f1)] {
                    const FIFO_DEPTH_WORDS: u16 = 128;
                    const ENDPOINT_COUNT: usize = 8;
                } else if #[cfg(any(
                    stm32f2,
                    stm32f401,
                    stm32f405,
                    stm32f407,
                    stm32f411,
                    stm32f415,
                    stm32f417,
                    stm32f427,
                    stm32f429,
                    stm32f437,
                    stm32f439,
                ))] {
                    const FIFO_DEPTH_WORDS: u16 = 320;
                    const ENDPOINT_COUNT: usize = 4;
                } else if #[cfg(any(
                    stm32f412,
                    stm32f413,
                    stm32f423,
                    stm32f446,
                    stm32f469,
                    stm32f479,
                    stm32f7,
                    stm32l4,
                    stm32u5,
                ))] {
                    const FIFO_DEPTH_WORDS: u16 = 320;
                    const ENDPOINT_COUNT: usize = 6;
                } else if #[cfg(stm32g0x1)] {
                    const FIFO_DEPTH_WORDS: u16 = 512;
                    const ENDPOINT_COUNT: usize = 8;
                } else if #[cfg(any(stm32h7, stm32h7rs))] {
                    const FIFO_DEPTH_WORDS: u16 = 1024;
                    const ENDPOINT_COUNT: usize = 9;
                } else if #[cfg(stm32u5)] {
                    const FIFO_DEPTH_WORDS: u16 = 320;
                    const ENDPOINT_COUNT: usize = 6;
                } else {
                    compile_error!("USB_OTG_FS peripheral is not supported by this chip.");
                }
            }

            fn regs() -> Otg {
                unsafe { Otg::from_ptr(crate::pac::USB_OTG_FS.as_ptr()) }
            }

            fn state() -> &'static State<MAX_EP_COUNT> {
                static STATE: State<MAX_EP_COUNT> = State::new();
                &STATE
            }
        }

        impl Instance for crate::peripherals::USB_OTG_FS {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };

    (USB_OTG_HS, otg, $block:ident, GLOBAL, $irq:ident) => {
        impl SealedInstance for crate::peripherals::USB_OTG_HS {
            const HIGH_SPEED: bool = true;

            cfg_if::cfg_if! {
                if #[cfg(any(
                    stm32f2,
                    stm32f405,
                    stm32f407,
                    stm32f415,
                    stm32f417,
                    stm32f427,
                    stm32f429,
                    stm32f437,
                    stm32f439,
                ))] {
                    const FIFO_DEPTH_WORDS: u16 = 1024;
                    const ENDPOINT_COUNT: usize = 6;
                } else if #[cfg(any(
                    stm32f446,
                    stm32f469,
                    stm32f479,
                    stm32f7,
                    stm32h7, stm32h7rs,
                ))] {
                    const FIFO_DEPTH_WORDS: u16 = 1024;
                    const ENDPOINT_COUNT: usize = 9;
                } else if #[cfg(stm32u5)] {
                    const FIFO_DEPTH_WORDS: u16 = 1024;
                    const ENDPOINT_COUNT: usize = 9;
                } else {
                    compile_error!("USB_OTG_HS peripheral is not supported by this chip.");
                }
            }

            fn regs() -> Otg {
                // OTG HS registers are a superset of FS registers
                unsafe { Otg::from_ptr(crate::pac::USB_OTG_HS.as_ptr()) }
            }

            fn state() -> &'static State<MAX_EP_COUNT> {
                static STATE: State<MAX_EP_COUNT> = State::new();
                &STATE
            }
        }

        impl Instance for crate::peripherals::USB_OTG_HS {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
);

fn calculate_trdt<T: Instance>(speed: Dspd) -> u8 {
    let ahb_freq = T::frequency().0;
    match speed {
        Dspd::HIGH_SPEED => {
            // From RM0431 (F72xx), RM0090 (F429), RM0390 (F446)
            if ahb_freq >= 30_000_000 {
                0x9
            } else {
                panic!("AHB frequency is too low")
            }
        }
        Dspd::FULL_SPEED_EXTERNAL | Dspd::FULL_SPEED_INTERNAL => {
            // From RM0431 (F72xx), RM0090 (F429)
            match ahb_freq {
                0..=14_199_999 => panic!("AHB frequency is too low"),
                14_200_000..=14_999_999 => 0xF,
                15_000_000..=15_999_999 => 0xE,
                16_000_000..=17_199_999 => 0xD,
                17_200_000..=18_499_999 => 0xC,
                18_500_000..=19_999_999 => 0xB,
                20_000_000..=21_799_999 => 0xA,
                21_800_000..=23_999_999 => 0x9,
                24_000_000..=27_499_999 => 0x8,
                27_500_000..=31_999_999 => 0x7, // 27.7..32 in code from CubeIDE
                32_000_000..=u32::MAX => 0x6,
            }
        }
        _ => unimplemented!(),
    }
}

fn quirk_setup_late_cnak(r: Otg) -> bool {
    r.cid().read().0 & 0xf000 == 0x1000
}
