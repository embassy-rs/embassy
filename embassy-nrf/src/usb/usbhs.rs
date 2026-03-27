//! Universal Serial Bus (USB) driver for nRF54LM20A USBHS.

pub mod vbus_detect;

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_futures::select::{Either, select};
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver::{EndpointAddress, EndpointAllocError, EndpointType, Event, Unsupported};
pub use embassy_usb_synopsys_otg::Config;
use embassy_usb_synopsys_otg::otg_v1::Otg;
use embassy_usb_synopsys_otg::otg_v1::vals::Dspd;
use embassy_usb_synopsys_otg::{
    Bus as OtgBus, ControlPipe, Driver as OtgDriver, Endpoint, In, OtgInstance, Out, PhyType, State,
    on_interrupt as on_interrupt_impl,
};

use self::vbus_detect::VbusDetect;
use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, pac};

const MAX_EP_COUNT: usize = 16;
const RX_FIFO_EXTRA_SIZE_WORDS: u16 = 30;
const FIFO_DEPTH_WORDS: u16 = 3040;

static BUS_WAKER: AtomicWaker = AtomicWaker::new();

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        on_interrupt_impl(T::core_regs(), T::state(), MAX_EP_COUNT);
    }
}

/// USB driver.
pub struct Driver<'d, V: VbusDetect> {
    inner: OtgDriver<'d, MAX_EP_COUNT>,
    usb_regs: pac::usbhs::Usbhs,
    vbus_detect: V,
    _phantom: PhantomData<&'d ()>,
}

impl<'d, V: VbusDetect> Driver<'d, V> {
    /// Create a new USBHS driver.
    pub fn new<T: Instance>(
        _usb: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        vbus_detect: V,
        ep_out_buffer: &'d mut [u8],
        mut config: Config,
    ) -> Self {
        // LM20A VBUS events are handled via VREGUSB instead of the OTG core's session events.
        config.vbus_detection = false;

        let instance = OtgInstance {
            regs: T::core_regs(),
            state: T::state(),
            fifo_depth_words: FIFO_DEPTH_WORDS,
            endpoint_count: MAX_EP_COUNT,
            phy_type: PhyType::InternalHighSpeed,
            extra_rx_fifo_words: RX_FIFO_EXTRA_SIZE_WORDS,
            calculate_trdt_fn: calculate_trdt,
        };

        Self {
            inner: OtgDriver::new(ep_out_buffer, instance, config),
            usb_regs: T::usb_regs(),
            vbus_detect,
            _phantom: PhantomData,
        }
    }
}

impl<'d, V: VbusDetect + 'd> embassy_usb_driver::Driver<'d> for Driver<'d, V> {
    type EndpointOut = Endpoint<'d, Out>;
    type EndpointIn = Endpoint<'d, In>;
    type ControlPipe = ControlPipe<'d>;
    type Bus = Bus<'d, V>;

    fn alloc_endpoint_in(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointIn, EndpointAllocError> {
        self.inner
            .alloc_endpoint_in(ep_type, ep_addr, max_packet_size, interval_ms)
    }

    fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        ep_addr: Option<EndpointAddress>,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointOut, EndpointAllocError> {
        self.inner
            .alloc_endpoint_out(ep_type, ep_addr, max_packet_size, interval_ms)
    }

    fn start(self, control_max_packet_size: u16) -> (Self::Bus, Self::ControlPipe) {
        let (inner, cp) = self.inner.start(control_max_packet_size);
        (
            Bus {
                inner,
                usb_regs: self.usb_regs,
                vbus_detect: self.vbus_detect,
                power_present: false,
                enabled: false,
                core_ready: false,
                _phantom: PhantomData,
            },
            cp,
        )
    }
}

/// USB bus.
pub struct Bus<'d, V: VbusDetect> {
    inner: OtgBus<'d, MAX_EP_COUNT>,
    usb_regs: pac::usbhs::Usbhs,
    vbus_detect: V,
    power_present: bool,
    enabled: bool,
    core_ready: bool,
    _phantom: PhantomData<&'d ()>,
}

impl<'d, V: VbusDetect> Bus<'d, V> {
    fn hold_pullup_off(&self) {
        let phy = self.usb_regs.phy();

        phy.overridevalues().write(|w| {
            w.set_id(pac::usbhs::vals::Id::DEVICE);
        });
        phy.inputoverride().write(|w| {
            w.set_vbusvalid(true);
            w.set_id(true);
        });
    }

    fn release_vbus_override(&self) {
        self.usb_regs.phy().inputoverride().write(|w| {
            w.set_id(true);
        });
    }

    fn start_xo24m(&self) {
        if pac::CLOCK.pll24m().run().read().status() {
            return;
        }

        pac::CLOCK.events_xo24mstarted().write_value(0);
        pac::CLOCK.tasks_xo24mstart().write_value(1);
        while pac::CLOCK.events_xo24mstarted().read() == 0 {}
    }

    fn stop_xo24m(&self) {
        if !pac::CLOCK.pll24m().run().read().status() {
            return;
        }

        pac::CLOCK.tasks_xo24mstop().write_value(1);
    }
}

impl<'d, V: VbusDetect> embassy_usb_driver::Bus for Bus<'d, V> {
    async fn enable(&mut self) {
        if self.enabled {
            return;
        }

        if self.vbus_detect.wait_power_ready().await.is_err() {
            return;
        }

        trace!("USBHS enable");
        self.start_xo24m();
        self.usb_regs.phy().clock().modify(|w| {
            w.set_fsel(pac::usbhs::vals::Fsel::CLOCK24000KHZ);
        });
        self.usb_regs.enable().write(|w| {
            w.set_core(true);
        });
        self.hold_pullup_off();
        self.usb_regs.enable().write(|w| {
            w.set_core(true);
            w.set_phy(true);
        });
        busy_wait_us(45);
        self.usb_regs.tasks_start().write_value(1);
        busy_wait_us(1);
        self.inner.core_soft_reset();
        self.inner.configure_as_device();
        self.release_vbus_override();
        self.inner.enable().await;
        self.enabled = true;
        self.core_ready = false;
    }

    async fn disable(&mut self) {
        if !self.enabled {
            return;
        }

        trace!("USBHS disable");
        self.inner.disable().await;
        crate::interrupt::typelevel::USBHS::disable();
        self.hold_pullup_off();
        self.usb_regs.enable().write(|w| {
            w.set_core(false);
            w.set_phy(false);
        });
        self.stop_xo24m();
        self.inner.deinit_device();
        self.enabled = false;
        self.core_ready = false;
    }

    async fn poll(&mut self) -> Event {
        if !self.power_present {
            let vbus_detect = &self.vbus_detect;
            poll_fn(|cx| {
                BUS_WAKER.register(cx.waker());
                if vbus_detect.is_usb_detected() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;

            trace!("USBHS power detected");
            self.power_present = true;
            return Event::PowerDetected;
        }

        if self.enabled && !self.core_ready {
            self.inner.init_device();
            crate::interrupt::typelevel::USBHS::unpend();
            unsafe { crate::interrupt::typelevel::USBHS::enable() };
            self.core_ready = true;
        }

        let vbus_detect = &self.vbus_detect;
        let inner = &mut self.inner;
        let remove_fut = poll_fn(|cx| {
            BUS_WAKER.register(cx.waker());
            if !vbus_detect.is_usb_detected() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        });

        match select(inner.poll(), remove_fut).await {
            Either::First(event) => event,
            Either::Second(()) => {
                trace!("USBHS power removed");
                self.power_present = false;
                Event::PowerRemoved
            }
        }
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

    async fn remote_wakeup(&mut self) -> Result<(), Unsupported> {
        self.inner.remote_wakeup().await
    }
}

impl<'d, V: VbusDetect> Drop for Bus<'d, V> {
    fn drop(&mut self) {
        crate::interrupt::typelevel::USBHS::disable();
        self.hold_pullup_off();
        self.usb_regs.enable().write(|w| {
            w.set_core(false);
            w.set_phy(false);
        });
        self.stop_xo24m();
    }
}

pub(crate) trait SealedInstance {
    fn usb_regs() -> pac::usbhs::Usbhs;
    fn core_regs() -> Otg;
    fn state() -> &'static State<MAX_EP_COUNT>;
}

/// USB peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

impl SealedInstance for crate::peripherals::USBHS {
    fn usb_regs() -> pac::usbhs::Usbhs {
        pac::USBHS
    }

    fn core_regs() -> Otg {
        unsafe { Otg::from_ptr(pac::USBHSCORE.as_ptr()) }
    }

    fn state() -> &'static State<MAX_EP_COUNT> {
        static STATE: State<MAX_EP_COUNT> = State::new();
        &STATE
    }
}

impl Instance for crate::peripherals::USBHS {
    type Interrupt = crate::interrupt::typelevel::USBHS;
}

fn current_hclk() -> u32 {
    match pac::OSCILLATORS.pll().currentfreq().read().currentfreq() {
        pac::oscillators::vals::Currentfreq::CK128M => 128_000_000,
        pac::oscillators::vals::Currentfreq::CK64M => 64_000_000,
        _ => unreachable!(),
    }
}

fn calculate_trdt(speed: Dspd) -> u8 {
    let ahb_freq = current_hclk();
    match speed {
        Dspd::HIGH_SPEED => {
            assert!(ahb_freq >= 30_000_000, "AHB frequency is too low for USBHS");
            0x9
        }
        Dspd::FULL_SPEED_EXTERNAL | Dspd::FULL_SPEED_INTERNAL => match ahb_freq {
            0..=14_199_999 => panic!("AHB frequency is too low"),
            14_200_000..=14_999_999 => 0xF,
            15_000_000..=15_999_999 => 0xE,
            16_000_000..=17_199_999 => 0xD,
            17_200_000..=18_499_999 => 0xC,
            18_500_000..=19_999_999 => 0xB,
            20_000_000..=21_799_999 => 0xA,
            21_800_000..=23_999_999 => 0x9,
            24_000_000..=27_499_999 => 0x8,
            27_500_000..=31_999_999 => 0x7,
            32_000_000..=u32::MAX => 0x6,
        },
        _ => unimplemented!(),
    }
}

fn busy_wait_us(us: u32) {
    #[cfg(feature = "time")]
    embassy_time::block_for(embassy_time::Duration::from_micros(us as u64));

    #[cfg(not(feature = "time"))]
    {
        // `asm::delay()` is a conservative lower-bound in CPU cycles, which is good enough for
        // minimum hardware settle times when a real-time driver is unavailable.
        let cycles = (current_hclk() / 1_000_000).saturating_mul(us);
        cortex_m::asm::delay(cycles);
    }
}
