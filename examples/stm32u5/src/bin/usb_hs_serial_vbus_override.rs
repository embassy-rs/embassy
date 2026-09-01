#![no_std]
#![no_main]

use defmt::{panic, *};
use defmt_rtt as _; // global logger
use embassy_executor::Spawner;
use embassy_futures::join::join3;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::Pull;
use embassy_stm32::usb::{Driver, Instance};
use embassy_stm32::{Config, bind_interrupts, interrupt, peripherals, usb};
use embassy_usb::Builder;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use panic_probe as _;
use stm32_metapac as pac;

bind_interrupts!(struct Irqs {
    OTG_HS => usb::InterruptHandler<peripherals::USB_OTG_HS>;
    EXTI13 => exti::InterruptHandler<interrupt::typelevel::EXTI13>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        use embassy_stm32::time::Hertz;
        config.rcc.hse = Some(Hse {
            freq: Hertz(16_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div2,   // HSE / 2 = 8MHz
            mul: PllMul::Mul60,        // 8MHz * 60 = 480MHz
            divr: Some(PllDiv::Div3),  // 480MHz / 3 = 160MHz (sys_ck)
            divq: Some(PllDiv::Div10), // 480MHz / 10 = 48MHz (USB)
            divp: Some(PllDiv::Div15), // 480MHz / 15 = 32MHz (USBOTG)
        });
        config.rcc.mux.otghssel = mux::Otghssel::Pll1P;
        config.rcc.voltage_range = VoltageScale::Range1;
        config.rcc.sys = Sysclk::Pll1R;
    }

    let p = embassy_stm32::init(config);

    // Create the driver, from the HAL.
    let mut ep_out_buffer = [0u8; 1024];
    let mut config = embassy_stm32::usb::Config::default();
    // Do not enable vbus_detection. This is a safe default that works in all boards.
    // However, if your USB device is self-powered (can stay powered on if USB is unplugged), you need
    // to enable vbus_detection to comply with the USB spec. If you enable it, the board
    // has to support it or USB won't work at all. See docs on `vbus_detection` for details.
    config.vbus_detection = false;
    // If the board does not support vbus_detection, the valid signal can be overridden with a
    // digital input. Enable vbus_valid_override and set the vbvaloval and bvaloval bits.
    config.vbus_valid_override = true;
    let driver = Driver::new_hs(p.USB_OTG_HS, Irqs, p.PA12, p.PA11, &mut ep_out_buffer, config);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-serial example");
    config.serial_number = Some("12345678");

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // Create classes on the builder.
    // High-speed bulk endpoints must have a max packet size of 512 bytes.
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 512);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Do stuff with the class!
    let echo_fut = async {
        loop {
            class.wait_connection().await;
            info!("Connected");
            let _ = echo(&mut class).await;
            info!("Disconnected");
        }
    };

    // Create the VBUS interrupt input
    let mut vbus = ExtiInput::new(p.PC13, p.EXTI13, Pull::None, Irqs);

    // Run the VBUS override
    let vbus_fut = async {
        loop {
            vbus.wait_for_any_edge().await;
            let value = vbus.is_high();

            pac::USB_OTG_HS.gccfg_v3().modify(|w| {
                // VBUS valid override value
                w.set_vbvaloval(value);
            });

            pac::USB_OTG_HS.gotgctl().modify(|w| {
                // B-session valid override value
                w.set_bvaloen(true);
                w.set_bvaloval(value);
            });
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join3(usb_fut, echo_fut, vbus_fut).await;
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo<'d, T: Instance + 'd>(class: &mut CdcAcmClass<'d, Driver<'d, T>>) -> Result<(), Disconnected> {
    let mut buf = [0; 512];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
    }
}
