#![no_std]
#![no_main]
teleprobe_meta::target!(b"rpi-pico");

use cyw43::{A4, Aligned, JoinOptions, SpiBus, aligned_bytes};
use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use defmt::{panic, *};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_net::StackStorage;
use embassy_rp::dma::{self, Channel};
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::{bind_interrupts, rom_data};
use embassy_time::{Duration, with_timeout};
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>;
});

teleprobe_meta::timeout!(120);

// Test-only wifi network, no internet access!
const WIFI_NETWORK: &str = "EmbassyTestWPA2";
const WIFI_PASSWORD: &str = "V8YxhKt5CdIAJFud";

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<'static, SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>, cyw43::Cyw43439>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");
    let p = embassy_rp::init(Default::default());

    // needed for reading the firmware from flash via XIP.
    unsafe {
        rom_data::connect_internal_flash();
        rom_data::flash_exit_xip();
        rom_data::flash_flush_cache();
        rom_data::flash_enter_cmd_xip();
    }

    // Firmware now in ELF (see build.rs)
    macro_rules! flash_bytes {
        ($env:expr, $section:literal, $path:literal) => {{
            #[cfg(feature = "flash-fw")]
            {
                #[unsafe(link_section = $section)]
                static BYTES: Aligned<A4, [u8; include_bytes!($path).len()]> = Aligned(*include_bytes!($path));
                let bytes: &Aligned<A4, [u8]> = &BYTES;
                bytes
            }

            #[cfg(not(feature = "flash-fw"))]
            {
                unsafe {
                    core::mem::transmute::<_, &Aligned<A4, [u8]>>(core::slice::from_raw_parts(
                        parse_bin(env!($env)) as *const u8,
                        include_bytes!($path).len(),
                    ))
                }
            }
        }};
    }
    let fw = flash_bytes!("CYW43_FW", ".cyw43_fw", "../../../../cyw43-firmware/43439A0.bin");
    let clm = flash_bytes!("CYW43_CLM", ".cyw43_clm", "../../../../cyw43-firmware/43439A0_clm.bin");

    let nvram = aligned_bytes!("../../../../cyw43-firmware/nvram_rp2040.bin");

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        Channel::new(p.DMA_CH0, Irqs),
        Channel::new(p.DMA_CH1, Irqs),
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw, nvram).await;
    spawner.spawn(unwrap!(wifi_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    // Generate random seed
    let seed = 0x0123_4567_89ab_cdef; // chosen by fair dice roll. guarenteed to be random.

    // Init network stack
    static STACK: StaticCell<StackStorage> = StaticCell::new();
    let (stack, runner) = embassy_net::Stack::new(STACK.init(StackStorage::new()), seed);

    // Add the network interface to the stack.
    static DEVICE: StaticCell<cyw43::NetDriver<'static>> = StaticCell::new();
    let iface = unwrap!(stack.add_iface(DEVICE.init(net_device)));
    unwrap!(iface.set_dhcpv4(Some(Default::default())));

    spawner.spawn(unwrap!(net_task(runner)));

    info!("mac: {:02x}", control.address().await);
    let t0 = embassy_time::Instant::now();

    // A scan proves boot and firmware
    let mut networks = 0;
    let mut scanner = control.scan(Default::default()).await;
    while scanner.next().await.is_some() {
        networks += 1;
    }
    drop(scanner);

    info!("scan found {} networks in {} ms", networks, t0.elapsed().as_millis());
    if networks == 0 {
        panic!("scan found no networks");
    }

    // Connecting depends on the AP, so failing or hanging here skips perf test
    let join = control.join(WIFI_NETWORK, JoinOptions::new(WIFI_PASSWORD.as_bytes()));
    let connected = with_timeout(Duration::from_secs(10), join)
        .await
        .is_ok_and(|r| r.is_ok())
        && with_timeout(Duration::from_secs(10), iface.wait_config_up())
            .await
            .is_ok();

    if connected {
        perf_client::run(
            iface,
            perf_client::Expected {
                down_kbps: 100,
                up_kbps: 100,
                updown_kbps: 100,
            },
        )
        .await;
    } else {
        warn!("not connected, skipping perf");
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

#[cfg(not(feature = "flash-fw"))]
const fn parse_bin(s: &str) -> usize {
    let bytes = s.as_bytes();

    let mut i = 0;
    let mut value = 0usize;

    while i < bytes.len() {
        value <<= 1;

        match bytes[i] {
            b'0' => {}
            b'1' => value |= 1,
            _ => core::unreachable!(),
        }

        i += 1;
    }

    value
}
