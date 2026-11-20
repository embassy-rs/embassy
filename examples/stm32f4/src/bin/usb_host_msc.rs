//! USB host mass storage example on STM32F429I-DISCO.
//!
//! The micro-AB connector (CN6) is wired to USB_OTG_HS, which runs at full speed
//! through its internal PHY. VBUS for the attached device comes from a power switch
//! whose enable is PC4, active low.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::HostDriver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_time::Timer;
use embassy_usb::class::msc::host::MscDevice;
use embassy_usb::driver::host::DeviceEvent;
use embassy_usb::host::{BusRoute, BusState};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    OTG_HS => usb::HostInterruptHandler<peripherals::USB_OTG_HS>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("USB host MSC example");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll_src = PllSource::Hse;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul168,
            divp: Some(PllPDiv::Div2), // 168 MHz sysclk
            divq: Some(PllQDiv::Div7), // 48 MHz USB clock
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::Div1;
        config.rcc.apb1_pre = APBPrescaler::Div4;
        config.rcc.apb2_pre = APBPrescaler::Div2;
        config.rcc.sys = Sysclk::Pll1P;
        config.rcc.mux.clk48sel = mux::Clk48sel::Pll1Q;
    }
    let p = embassy_stm32::init(config);

    // Turn on VBUS for CN6.
    let _vbus = Output::new(p.PC4, Level::Low, Speed::Low);

    let driver = HostDriver::new_fs_host(p.USB_OTG_HS, p.PB15, p.PB14, Irqs);

    static BUS_STATE: BusState = BusState::new();
    let (mut bus_ctrl, bus) = embassy_usb::host::bus(driver, &BUS_STATE);
    info!("USB host initialized, waiting for device...");

    loop {
        let speed = bus_ctrl.wait_for_connection().await;
        info!("Device connected at speed {:?}", speed);

        let mut config_buf = [0u8; 256];
        let (enum_info, config_len) = match bus.enumerate(BusRoute::Direct(speed), &mut config_buf).await {
            Ok(r) => r,
            Err(e) => {
                error!("Enumeration failed: {:?}", e);
                continue;
            }
        };
        info!(
            "Enumerated: VID={:04x} PID={:04x} addr={}",
            enum_info.device_desc.vendor_id, enum_info.device_desc.product_id, enum_info.device_address
        );

        let device = match MscDevice::new(&bus, &enum_info, &config_buf[..config_len]).await {
            Ok(d) => d,
            Err(e) => {
                error!("MSC init failed: {:?}", e);
                continue;
            }
        };
        info!("MSC device with {} LUN(s)", device.num_luns());

        let mut lun = unwrap!(device.lun(0));

        let mut inq = [0u8; 36];
        match lun.inquiry(&mut inq).await {
            Ok(info) => info!(
                "INQUIRY: {:?} removable={} vendor={:a} product={:a} rev={:a}",
                info.peripheral, info.removable, info.vendor, info.product, info.revision
            ),
            Err(e) => {
                error!("INQUIRY failed: {:?}", e);
                continue;
            }
        }

        // Freshly plugged drives may report "not ready" for a short while.
        for _ in 0..10 {
            match lun.test_unit_ready().await {
                Ok(true) => break,
                Ok(false) => {}
                Err(e) => warn!("TEST UNIT READY failed: {:?}", e),
            }
            Timer::after_millis(100).await;
        }

        let cap = match lun.capacity().await {
            Ok(c) => c,
            Err(e) => {
                error!("READ CAPACITY failed: {:?}", e);
                continue;
            }
        };
        info!(
            "Capacity: {} blocks of {} bytes ({} MiB)",
            cap.block_count,
            cap.block_size,
            cap.block_count * cap.block_size as u64 / (1024 * 1024)
        );

        let mut block = [0u8; 512];
        match lun.read_blocks(0, &mut block).await {
            Ok(()) => {
                info!("Block 0: {:02x}", block);
                info!("Boot signature: {:02x} {:02x}", block[510], block[511]);
            }
            Err(e) => error!("READ(10) failed: {:?}", e),
        }

        info!("Waiting for disconnect...");
        while bus_ctrl.wait_for_device_event().await != DeviceEvent::Disconnected {}
        info!("Device disconnected");
    }
}
