#![no_std]
#![no_main]

//! CDC-ACM echo over USB1_OTG_HS on the STM32N6570-DK.
//!
//! The N6's OTG cores are High-Speed only and drive an integrated UTMI+ PHY on
//! dedicated `OTG1_HSDP` / `OTG1_HSDM` balls, so the driver takes no D+/D- pins.

use defmt::{panic, *};
use defmt_rtt as _; // global logger
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::rcc::mux::Otgphysel;
use embassy_stm32::rcc::{IcConfig, Icint, Icsel, Pll, Plldivm, Pllpdiv, Pllsel, SupplyConfig};
use embassy_stm32::usb::{Driver, Instance};
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_usb::Builder;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB1_OTG_HS => usb::InterruptHandler<peripherals::USB1_OTG_HS>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let mut config = Config::default();
    {
        // DK uses external SMPS (UM3300 Tab.6); embassy default = internal SMPS hangs init() at VOSRDY.
        config.rcc.supply_config = SupplyConfig::External;

        // The HS PHY reference clock must be exactly 19.2, 20 or 24 MHz (RM0486 Rev 4,
        // USBPHYC_CR.FSEL, p. 3929). Derive 24 MHz from the HSI so the example does not
        // depend on the board's crystal:
        //
        //   hsi_ck   = 64 MHz                              (RM0486 Rev 4, §14.6.2)
        //   FREF     = 64 MHz / DIVM 4      =  16 MHz      (5..1200 MHz, integer mode, p. 435)
        //   FVCO     = 16 MHz * DIVN 60     = 960 MHz      (800..3200 MHz, p. 435)
        //   pll4_ck  = 960 MHz / 2 / 1      = 480 MHz      (DIVP1 = 2, DIVP2 = 1)
        //   ic15_ck  = 480 MHz / 20         =  24 MHz      (ICINT = Div20)
        //
        // PLL4 is the RM's "display, camera, FDCAN, and other peripherals" PLL (§14.6.5,
        // p. 435) and nothing else in this example uses it.
        config.rcc.pll4 = Some(Pll::Oscillator {
            source: Pllsel::Hsi,
            divm: Plldivm::Div4,
            fractional: 0,
            divn: 60,
            divp1: Pllpdiv::Div2,
            divp2: Pllpdiv::Div1,
        });
        config.rcc.ic15 = Some(IcConfig {
            source: Icsel::Pll4,
            divider: Icint::Div20,
        });
        config.rcc.mux.otgphy1sel = Otgphysel::Ic15;
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
    let driver = Driver::new_hs_dedicated_pins(p.USB1_OTG_HS, Irqs, &mut ep_out_buffer, config);

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

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, echo_fut).await;
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
