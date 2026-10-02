//! GUD (Generic USB Display) example for the RP2350
//!
//! The board enumerates as a 1920x1080 RGB565 display to a Linux host running
//! the `gud` DRM driver (VID/PID 0x1209/0x4FB3).

#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver as UsbDriver, InterruptHandler as UsbInterruptHandler};
use embassy_usb::class::gud::{
    self, Config as GudConfig, ConnectorState, GudClass, GudConnector, GudEvent, PixelFormat, State,
};
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
});

/// 128-byte EDID with modes:
/// - 1920x1080@60Hz (preferred)
/// - 1680x1050@60Hz
/// - 1600x900@60Hz
/// - 1440x900@60Hz
/// - 1280x800@60Hz
/// - 1280x720@60Hz
/// - 1024x768@60Hz
/// - 800x600@60Hz
/// - 640x480@60Hz
const EDID: [u8; 128] = [
    0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x1A, 0x62, 0xD0, 0x01, 0x01, 0x00, 0x00, 0x00, 0x01, 0x23, 0x01,
    0x04, 0x80, 0x34, 0x1D, 0x78, 0x0A, 0xEE, 0x51, 0xA3, 0x54, 0x4C, 0x99, 0x26, 0x0F, 0x50, 0x54, 0x21, 0x04, 0x00,
    0xA9, 0x40, 0x81, 0x00, 0x81, 0x40, 0x95, 0x00, 0xB3, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02, 0x3A, 0x80,
    0x18, 0x71, 0x38, 0x2D, 0x40, 0x58, 0x2C, 0x45, 0x00, 0xA0, 0x5A, 0x00, 0x00, 0x00, 0x1E, 0x00, 0x00, 0x00, 0xFC,
    0x00, 0x45, 0x6D, 0x62, 0x61, 0x73, 0x73, 0x79, 0x20, 0x47, 0x55, 0x44, 0x0A, 0x20, 0x00, 0x00, 0x00, 0xFD, 0x00,
    0x38, 0x4B, 0x1E, 0x53, 0x11, 0x00, 0x0A, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x00, 0x00, 0x00, 0xFE, 0x00, 0x50,
    0x69, 0x63, 0x6F, 0x32, 0x57, 0x20, 0x47, 0x55, 0x44, 0x0A, 0x20, 0x20, 0x00, 0xBD,
];

static FORMATS: [PixelFormat; 1] = [PixelFormat::Rgb565];

static EDID_STORAGE: StaticCell<[u8; 128]> = StaticCell::new();
static CONNECTOR_STATE: StaticCell<ConnectorState<'static>> = StaticCell::new();
static CONNECTORS: StaticCell<[GudConnector<'static>; 1]> = StaticCell::new();

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    info!("GUD display example");

    let p = embassy_rp::init(Default::default());

    let driver = UsbDriver::new(p.USB, Irqs);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(gud::GUD_VENDOR_ID, gud::GUD_PRODUCT_ID);
    config.manufacturer = Some("Embassy");
    config.product = Some("Pico 2 W GUD display");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    let mut builder = {
        static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 512]> = StaticCell::new();

        embassy_usb::Builder::new(
            driver,
            config,
            CONFIG_DESCRIPTOR.init([0; 256]),
            BOS_DESCRIPTOR.init([0; 256]),
            &mut [], // no msos descriptors
            // >= 128 so the EDID is served un-truncated
            CONTROL_BUF.init([0; 512]),
        )
    };

    // Leave modes empty so the Linux driver uses the EDID mode list.
    // This shared handle can also be retained by a DDC/hotplug task independently of GudClass.
    let connector_state: &'static ConnectorState<'static> =
        CONNECTOR_STATE.init(ConnectorState::new(&mut [], EDID_STORAGE.init([0; 128])));
    connector_state
        .update(gud::CONNECTOR_STATUS_CONNECTED, &[], Some(&EDID))
        .unwrap();
    let connectors = CONNECTORS.init([GudConnector {
        connector_type: gud::CONNECTOR_TYPE_PANEL,
        flags: 0,
        state: connector_state,
        properties: &[],
        tv_mode_values: None,
    }]);

    let class = {
        static STATE: StaticCell<State> = StaticCell::new();
        let state = STATE.init(State::new());

        static GUD_CONFIG: StaticCell<GudConfig> = StaticCell::new();
        let gud_config = GUD_CONFIG.init(GudConfig {
            max_packet_size: 64, // full-speed bulk
            min_width: 640,
            max_width: 1920,
            min_height: 480,
            max_height: 1080,
            // Stream uncapped updates without allocating a framebuffer.
            max_buffer_size: 0,
            formats: &FORMATS,
            supported_rotations: 0,
            connectors,
            // The discard sink has no constraints.
            validate_mode: |_, _| true,
        });

        GudClass::new(&mut builder, state, gud_config)
    };

    let mut usb = builder.build();
    let usb_fut = usb.run();

    // Consume display events and stream pixel data one USB packet at a time.
    let mut class = class;
    let gud_fut = async {
        class.wait_connection().await;
        info!("USB configured, GUD host driver will probe");

        let mut chunk = [0u8; 64]; // One full-speed bulk packet.
        loop {
            match class.next_event().await {
                GudEvent::Buffer(info) => match class
                    .read_buffer(&info, &mut chunk, async |pkt| {
                        Ok::<_, core::convert::Infallible>(pkt.len())
                    })
                    .await
                {
                    Ok(bytes) => info!(
                        "buffer {}x{}+{}+{}: streamed {} bytes (discarded)",
                        info.width, info.height, info.x, info.y, bytes
                    ),
                    Err(e) => info!("bulk read failed: {:?}", e),
                },
                GudEvent::StateCheck => {
                    if let Some(state) = class.checked_state() {
                        info!(
                            "state check: {}x{} fmt={:?} conn={}",
                            state.mode.hdisplay, state.mode.vdisplay, state.format, state.connector
                        );
                    }
                }
                GudEvent::StateCommit => {
                    if let Some(state) = class.committed_state() {
                        info!(
                            "state commit: {}x{} fmt={:?} conn={}",
                            state.mode.hdisplay, state.mode.vdisplay, state.format, state.connector
                        );
                    }
                }
                ev => info!("gud event: {:?}", ev),
            }
        }
    };

    join(usb_fut, gud_fut).await;
}
