#![no_std]
#![no_main]

#[path = "common/helper_functions.rs"]
mod helper_functions;

#[path = "common/api.rs"]
mod api;

#[path = "common/usb_driver.rs"]
mod usb_driver;

use api::*;
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::{Flex, Input, Level, Output, Pull, Speed};
use embassy_stm32::peripherals::USB_OTG_HS as UsbOtgHs;
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::Driver as Stm32UsbDriver;
use embassy_stm32::{bind_interrupts, usb};
use embassy_time::{Duration, Timer};
use embassy_usb::msos::{CompatibleIdFeatureDescriptor, PropertyData, RegistryPropertyFeatureDescriptor};
use embassy_usb::{Builder, Config as USBConfig};
use panic_probe as _;
use static_cell::StaticCell;
use usb_driver::*;

const DEBUG_DURING_SLEEP: bool = true;

pub const VID: u16 = 0x1993;
pub const PID: u16 = 0xBEEF;

static EP_OUT_BUF: StaticCell<[u8; 1024]> = StaticCell::new();
static CMD_BUF: StaticCell<[u8; 128]> = StaticCell::new();

bind_interrupts!(pub struct Irqs {
    USB_OTG_HS => usb::InterruptHandler<UsbOtgHs>;
    EXTI9 => exti::InterruptHandler<embassy_stm32::interrupt::typelevel::EXTI9>;
});

pub enum TaskState {
    PRESLEEP,
    SLEEP,
    PREAWAKE,
    AWAKE,
}

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    info!("Hello from STM32WBA6 (65RI) low-power example using USB!");

    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;

        // Enable HSE (32 MHz external crystal) - REQUIRED for BLE radio
        config.rcc.hse = Some(Hse {
            prescaler: HsePrescaler::Div1,
            trim: Some(0x0C),
        });
        // Enable LSE (32.768 kHz external crystal) - REQUIRED for BLE radio sleep timer
        config.rcc.ls = LsConfig {
            rtc: RtcClockSource::Lse,
            lsi: false,
            lse: Some(LseConfig {
                frequency: Hertz(32_768),
                mode: LseMode::Oscillator(LseDrive::MediumLow),
                peripherals_clocked: true,
            }),
        };
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div1,   // PLLM = 1 → HSI / 1 = 16 MHz
            mul: PllMul::Mul30,        // PLLN = 30 → 16 MHz * 30 = 480 MHz VCO
            divr: Some(PllDiv::Div5),  // PLLR = 5 → 96 MHz (Sysclk)
            divq: Some(PllDiv::Div10), // PLLQ = 10 → 48 MHz
            divp: Some(PllDiv::Div30), // PLLP = 30 → 16 MHz (USB_OTG_HS)
            frac: Some(0),             // Fractional part (disabled)
        });

        config.rcc.ahb_pre = AHBPrescaler::Div1;
        config.rcc.apb1_pre = APBPrescaler::Div1;
        config.rcc.apb2_pre = APBPrescaler::Div1;
        config.rcc.apb7_pre = APBPrescaler::Div1;
        config.rcc.ahb5_pre = AHB5Prescaler::Div4;

        config.rcc.voltage_scale = VoltageScale::Range1;
        config.rcc.mux.otghssel = mux::Otghssel::Pll1P;
        config.rcc.mux.lptim2sel = mux::Lptim2sel::Hsi;
        config.rcc.mux.rngsel = mux::Rngsel::Hsi;
        config.rcc.sys = Sysclk::Pll1R;

        config.enable_debug_during_sleep = DEBUG_DURING_SLEEP;
        config.min_stop_pause = embassy_time::Duration::from_millis(10);
    }

    let mut p = embassy_stm32::init(config);

    info!("initializing unused GPIOs for minimum current draw ...");
    let _gpio_pd5 = Output::new(p.PD5, Level::Low, Speed::Low);
    let _gpio_pb10 = Output::new(p.PB10, Level::High, Speed::Low);
    let _gpio_pa6 = Input::new(p.PA6, Pull::Up);
    let _gpio_pe1 = Output::new(p.PE1, Level::High, Speed::VeryHigh);
    let _gpio_pe3 = Output::new(p.PE3, Level::High, Speed::VeryHigh);
    let _gpio_pe0 = Output::new(p.PE0, Level::Low, Speed::VeryHigh);
    let _gpio_pd14 = Output::new(p.PD14, Level::Low, Speed::VeryHigh);
    let mut flex_pd8 = Flex::new(p.PD8);
    flex_pd8.set_as_analog();
    let _gpio_ph3 = Output::new(p.PH3, Level::Low, Speed::Low);

    let _power_rail = Output::new(p.PB15, Level::Low, Speed::Low);
    let mut vbus_sns = ExtiInput::new(p.PD9, p.EXTI9, Pull::None, Irqs);

    let mut drv_cfg = embassy_stm32::usb::Config::default();
    drv_cfg.vbus_detection = true; // TODO: Make true later, we are battery powered

    let ep_out = EP_OUT_BUF.init([0u8; 1024]);
    let cmd_buf = CMD_BUF.init([0u8; 128]);

    Timer::after(Duration::from_millis(5000)).await;

    info!("usb: initializing usb stack");
    let usb_driver = Stm32UsbDriver::new_hs(
        p.USB_OTG_HS.reborrow(),
        Irqs,
        p.PD6.reborrow(), // HS_DM
        p.PD7.reborrow(), // HS_DP
        ep_out,
        drv_cfg,
    );

    let mut config_descriptor = [0u8; 256];
    let mut bos_descriptor = [0u8; 256];
    let mut msos_descriptor = [0u8; 256];
    let mut control_buf = [0u8; 64];

    let mut usb_config = USBConfig::new(VID, PID);
    usb_config.manufacturer = Some("MANU");
    usb_config.product = Some("PROD");
    usb_config.serial_number = Some("SERIAL");
    usb_config.composite_with_iads = false;
    usb_config.device_class = 0x00;
    usb_config.device_sub_class = 0x00;
    usb_config.device_protocol = 0x00;

    let mut usb_builder = Builder::new(
        usb_driver,
        usb_config.clone(),
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );

    // Add MS OS 2.0 descriptor so Windows automatically uses WinUSB without Zadig.
    // Device-level features are required for non-composite devices (composite_with_iads = false).
    usb_builder.msos_descriptor(0x0600_0000, 0x20);
    usb_builder.msos_feature(CompatibleIdFeatureDescriptor::new("WINUSB", ""));
    usb_builder.msos_feature(RegistryPropertyFeatureDescriptor::new(
        "DeviceInterfaceGUIDs",
        PropertyData::RegMultiSz(&["{00700007-0007-4007-A007-00000000010C}"]),
    ));

    // Interface: Vendor-Specific class (0xFF) so no OS kernel driver claims the
    // interface (macOS's IOUSBMassStorageClass grabs class 0x08 before userspace).
    let mut func = usb_builder.function(0xFF, 0xFF, 0xFF);
    let mut iface = func.interface();
    let mut alt = iface.alt_setting(0xFF, 0xFF, 0xFF, None);
    let ep_out = alt.endpoint_bulk_out(None, 512); // host → device
    let ep_in = alt.endpoint_bulk_in(None, 512); // device → host
    drop(alt);
    drop(iface);
    drop(func);

    let mut usb_device = usb_builder.build();
    let bulk_rw: BulkReaderWriter<'_, Stm32UsbDriver<'_, UsbOtgHs>> = BulkReaderWriter::new(ep_out, ep_in);

    let mut api_handler = ApiHandler::new(bulk_rw, cmd_buf);

    let mut task_state = TaskState::SLEEP;

    loop {
        match task_state {
            TaskState::PRESLEEP => task_state = TaskState::SLEEP,
            TaskState::SLEEP => {
                // TODO: add "USB connected at boot" case
                vbus_sns.wait_for_rising_edge().await;
                info!("VBUS rising edge detected!!");
                task_state = TaskState::PREAWAKE;
            }
            TaskState::PREAWAKE => task_state = TaskState::AWAKE,
            TaskState::AWAKE => {
                let usb_fut = usb_device.run();
                let cli_fut = async {
                    info!("USB: waiting for connections");
                    api_handler.serial.wait_connection().await;
                    loop {
                        info!("usb: receiving commands ...");
                        match api_handler.receive().await {
                            Ok(true) => {
                                info!("usb: received {} bytes", api_handler.get_rec_len());
                            }
                            Ok(false) => {
                                info!("Command not ready or invalid");
                            }
                            Err(UsbIoError::Disconnected) => {
                                warn!("USB disconnected - pausing handler");
                                break;
                            }
                            Err(e) => {
                                error!("Receive error: {:?}", e);
                                break;
                            }
                        }
                    }
                    info!("USB: done receiving");
                };

                match select(usb_fut, cli_fut).await {
                    Either::First(_) => {
                        warn!("USB device task exited unexpectedly, resetting USB stack...");
                    }
                    Either::Second(_) => {
                        warn!("USB disconnected, waiting for new connection...");
                    }
                }
                usb_device.disable().await;
                task_state = TaskState::PRESLEEP;
            }
        }
    }
}
