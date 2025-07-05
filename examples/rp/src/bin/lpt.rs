#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]

use assign_resources::assign_resources;
use const_format::formatcp;
use core::fmt::Write;
use core::str;
use cyw43::JoinOptions;
use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};
use defmt::{error, info, unwrap, Format};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_net::dns::DnsSocket;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_net::{Config, StackResources};
use embassy_rp::bind_interrupts;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Level, Output};
use embassy_rp::peripherals;
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_rp::Peri;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Delay, Timer};
use embedded_graphics::{
    mono_font::MonoTextStyle,
    prelude::*,
    primitives::PrimitiveStyle,
    text::{Alignment, LineHeight, Text, TextStyleBuilder},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use epd_waveshare::{epd3in7::*, prelude::*};
use heapless::String;
use panic_probe as _;
use profont::*;
use reqwless::client::{HttpClient, TlsConfig, TlsVerify};
use reqwless::request::Method;
use serde::Deserialize;
use serde_json_core::de::from_slice;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

const TFL_API_FIELD_SHORT_STR_SIZE: usize = 32;
const TFL_API_FIELD_STR_SIZE: usize = 64;
const TFL_API_FIELD_LONG_STR_SIZE: usize = 128;

#[derive(Deserialize, Debug, Format)]
#[serde(rename_all = "camelCase")]
struct TflApiPredictionTiming {
    #[serde(rename = "$type")]
    _type: String<TFL_API_FIELD_LONG_STR_SIZE>,
    countdown_server_adjustment: String<TFL_API_FIELD_STR_SIZE>,
    source: String<TFL_API_FIELD_STR_SIZE>,
    insert: String<TFL_API_FIELD_STR_SIZE>,
    read: String<TFL_API_FIELD_STR_SIZE>,
    sent: String<TFL_API_FIELD_STR_SIZE>,
    received: String<TFL_API_FIELD_STR_SIZE>,
}

#[derive(Deserialize, Debug, Format)]
#[serde(rename_all = "camelCase")]
struct TflApiPreciction {
    #[serde(rename = "$type")]
    _type: String<TFL_API_FIELD_LONG_STR_SIZE>,
    id: String<TFL_API_FIELD_STR_SIZE>,
    operation_type: u8,
    vehicle_id: String<TFL_API_FIELD_STR_SIZE>,
    naptan_id: String<TFL_API_FIELD_STR_SIZE>,
    station_name: String<TFL_API_FIELD_STR_SIZE>,
    line_id: String<TFL_API_FIELD_STR_SIZE>,
    line_name: String<TFL_API_FIELD_STR_SIZE>,
    platform_name: String<TFL_API_FIELD_STR_SIZE>,
    direction: String<TFL_API_FIELD_SHORT_STR_SIZE>,
    bearing: String<TFL_API_FIELD_STR_SIZE>,
    destination_naptan_id: String<TFL_API_FIELD_STR_SIZE>,
    destination_name: String<TFL_API_FIELD_STR_SIZE>,
    timestamp: String<TFL_API_FIELD_STR_SIZE>,
    time_to_station: u32,
    current_location: String<TFL_API_FIELD_LONG_STR_SIZE>,
    towards: String<TFL_API_FIELD_STR_SIZE>,
    expected_arrival: String<TFL_API_FIELD_STR_SIZE>,
    time_to_live: String<TFL_API_FIELD_STR_SIZE>,
    mode_name: String<TFL_API_FIELD_STR_SIZE>,
    timing: TflApiPredictionTiming,
}

#[embassy_executor::task(pool_size = 1)]
async fn cyw43_task(runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>) -> ! {
    runner.run().await
}

#[embassy_executor::task(pool_size = 1)]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

assign_resources! {
    display_resources: DisplayResources {
        spi1: SPI1,
        pin_9: PIN_9,
        pin_8: PIN_8,
        pin_13: PIN_13,
        pin_12: PIN_12,
        pin_11: PIN_11,
        pin_10: PIN_10,
    }
}

fn insert_linebreaks_inplace<const N: usize>(s: &mut heapless::String<N>, max_line_len: usize) {
    let mut i = 0;
    while i < s.len() {
        // Find the end of the current line (either '\n' or end of string)
        let line_end = match s[i..].find('\n') {
            Some(rel) => i + rel,
            None => s.len(),
        };
        // Only insert a break if the line is too long
        if line_end - i > max_line_len {
            // Look for the last space before the limit
            let mut break_pos = None;
            for j in (i..i + max_line_len).rev() {
                if j < s.len() && s.as_bytes()[j] == b' ' {
                    break_pos = Some(j);
                    break;
                }
            }
            let insert_at = break_pos.unwrap_or(i + max_line_len);
            if s.len() < s.capacity() {
                let mut tail = heapless::String::<N>::new();
                let _ = tail.push_str(&s[insert_at..]);
                let _ = s.truncate(insert_at);
                let _ = s.push('\n');
                let _ = s.push_str(&tail);
                // Move i to after the inserted line break
                i = insert_at + 1;
            } else {
                break;
            }
        } else {
            // Move i to the next line (after '\n' if present)
            i = if line_end < s.len() { line_end + 1 } else { s.len() };
        }
    }
}

fn extract_first_json_object(body: &[u8]) -> Option<&[u8]> {
    let mut start = None;
    let mut end = None;
    let mut brace_count = 0;

    for (i, &b) in body.iter().enumerate() {
        if b == b'{' {
            if start.is_none() {
                start = Some(i);
            }
            brace_count += 1;
        }
        if b == b'}' && start.is_some() {
            brace_count -= 1;
            if brace_count == 0 {
                end = Some(i);
                break;
            }
        }
    }

    match (start, end) {
        (Some(s), Some(e)) => Some(&body[s..=e]), // inclusive of the closing brace
        _ => None,
    }
}

static DISPLAY_TASK_DATA_CHANNEL: Channel<ThreadModeRawMutex, TflApiPreciction, 1> = Channel::new();

#[embassy_executor::task(pool_size = 1)]
async fn update_display_with_predictions(r: DisplayResources) {
    info!("Display: Initialising display...");

    // Setup display pins and SPI bus
    let pin_reset: Output<'_> = Output::new(r.pin_12, Level::Low);
    let pin_cs = Output::new(r.pin_9, Level::High);
    let pin_data_cmd: Output<'_> = Output::new(r.pin_8, Level::Low);
    let pin_spi_sclk = r.pin_10;
    let pin_spi_mosi = r.pin_11;
    let pin_busy = Input::new(r.pin_13, embassy_rp::gpio::Pull::None);

    let mut display_config = spi::Config::default();
    const DISPLAY_FREQ: u32 = 16_000_000;
    display_config.frequency = DISPLAY_FREQ;
    display_config.phase = spi::Phase::CaptureOnFirstTransition;
    display_config.polarity = spi::Polarity::IdleLow;

    let spi_bus = Spi::new_blocking_txonly(r.spi1, pin_spi_sclk, pin_spi_mosi, display_config);
    let mut spi_device = ExclusiveDevice::new(spi_bus, pin_cs, Delay);

    // // Setup the EPD driver and create a display buffer to draw on, specific for this ePaper
    let mut epd_driver = EPD3in7::new(&mut spi_device, pin_busy, pin_data_cmd, pin_reset, &mut Delay, None)
        .expect("Display: eink initalize error"); // Force unwrap, as there is nothing that can be done if this errors out

    // Create a Display buffer to draw on, specific for this ePaper
    let mut display = Display3in7::default();

    // Landscape mode, USB plug to the right
    display.set_rotation(DisplayRotation::Rotate270);

    // Change the background from the default black to white
    let _ = display
        .bounding_box()
        .into_styled(PrimitiveStyle::with_fill(Color::White))
        .draw(&mut display);

    display.clear(Color::White).ok();

    // Render splash drawing
    let character_style = MonoTextStyle::new(&PROFONT_24_POINT, Color::Black);
    let text_style = TextStyleBuilder::new().alignment(Alignment::Center).build();
    let position = display.bounding_box().center();
    Text::with_text_style("its3mile/london-pi-tube", position, character_style, text_style)
        .draw(&mut display)
        .expect("Failed create text in display buffer");

    epd_driver
        .update_and_display_frame(&mut spi_device, &mut display.buffer(), &mut Delay)
        .expect("Display: Failed to update display with splash");

    info!("Display: Display is up!");

    loop {
        info!("Display: waiting for data on channel");
        let prediction = DISPLAY_TASK_DATA_CHANNEL.receive().await;
        info!("Display: received data on channel");
        // Prepare the display message
        let mut message: String<
            { TFL_API_FIELD_SHORT_STR_SIZE + 5 * TFL_API_FIELD_STR_SIZE + TFL_API_FIELD_LONG_STR_SIZE },
        > = String::new();
        if (prediction.time_to_station as f32 / 60.0) < 1.0 {
            let _ = write!(
                &mut message,
                "{}\n{} Line {}\nTo: {}\nArriving in less than a minute\nCurrently {}",
                prediction.station_name,
                prediction.line_name,
                prediction.platform_name,
                prediction.destination_name,
                prediction.current_location
            );
        } else {
            let minutes_to_station = prediction.time_to_station / 60;
            let _ = write!(
                &mut message,
                "{}\n{} Line {}\nTo: {}\nArriving in {} minutes\nCurrently {}",
                prediction.station_name,
                prediction.line_name,
                prediction.platform_name,
                prediction.destination_name,
                minutes_to_station,
                prediction.current_location
            );
        }

        info!("Display: Updating display with text: {}", message);
        display.clear(Color::White).ok();
        insert_linebreaks_inplace(&mut message, 40);
        let character_style = MonoTextStyle::new(&PROFONT_18_POINT, Color::Black);
        let text_style = TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .line_height(LineHeight::Percent(125))
            .build();
        let position = display.bounding_box().top_left + Point::new(10, 25);
        Text::with_text_style(&message, position, character_style, text_style)
            .draw(&mut display)
            .expect("Failed create text in display buffer");
        epd_driver
            .update_and_display_frame(&mut spi_device, &mut display.buffer(), &mut Delay)
            .expect("Failed to update display with prediction");
        info!("Display: Display updated with prediction...going to sleep");
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals: embassy_rp::Peripherals = embassy_rp::init(Default::default());

    let fw = include_bytes!("../../../../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../../../../cyw43-firmware/43439A0_clm.bin");
    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs download 43439A0.bin --binary-format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs download 43439A0_clm.bin --binary-format bin --chip RP2040 --base-address 0x10140000
    // let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 230321) };
    // let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };

    // Spawn the task to update the display with predictions
    let r = split_resources!(peripherals);
    unwrap!(spawner.spawn(update_display_with_predictions(r.display_resources)));

    // Setup the CYW43 Wifi chip
    let pwr = Output::new(peripherals.PIN_23, Level::Low);
    let cs = Output::new(peripherals.PIN_25, Level::High);
    let mut pio = Pio::new(peripherals.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        peripherals.PIN_24,
        peripherals.PIN_29,
        peripherals.DMA_CH0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::Performance)
        .await;

    let config = Config::dhcpv4(Default::default());

    // Generate random seed
    let mut rng: RoscRng = RoscRng;
    let seed = rng.next_u64();

    // Init network stack
    static RESOURCES: StaticCell<StackResources<5>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(net_device, config, RESOURCES.init(StackResources::new()), seed);

    unwrap!(spawner.spawn(net_task(runner)));
    const WIFI_NETWORK: &'static str = env!("WIFI_NETWORK");
    const WIFI_PASSWORD: &'static str = env!("WIFI_PASSWORD");
    loop {
        match control
            .join(WIFI_NETWORK, JoinOptions::new(WIFI_PASSWORD.as_bytes()))
            .await
        {
            Ok(_) => break,
            Err(err) => {
                info!("join failed with status={}", err.status);
            }
        }
    }

    // Wait for DHCP, not necessary when using static IP
    info!("waiting for DHCP...");
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("DHCP is now up!");

    info!("waiting for link up...");
    while !stack.is_link_up() {
        Timer::after_millis(500).await;
    }
    info!("Link is up!");

    info!("waiting for stack to be up...");
    stack.wait_config_up().await;
    info!("Stack is up!");

    // define the URL for the TFL API request
    const TFL_API_PRIMARY_KEY: &'static str = env!("TFL_API_PRIMARY_KEY");
    const TFL_STOPCODE_PARAM: &'static str = env!("TFL_STOPCODE_PARAM");
    const HTTP_PROXY: &'static str = env!("HTTP_PROXY");
    let url = formatcp!("{HTTP_PROXY}/StopPoint/{TFL_STOPCODE_PARAM}/Arrivals?api_key={TFL_API_PRIMARY_KEY}");

    loop {
        // Create the HTTP client and DNS client
        let mut rx_buffer: [u8; 8192] = [0u8; 8192];
        let mut tls_read_buffer = [0; 16640];
        let mut tls_write_buffer = [0; 16640];

        let client_state = TcpClientState::<1, 1024, 1024>::new();
        let tcp_client = TcpClient::new(stack, &client_state);
        let dns_client = DnsSocket::new(stack);
        let tls_config = TlsConfig::new(seed, &mut tls_read_buffer, &mut tls_write_buffer, TlsVerify::None);

        let mut http_client = HttpClient::new_with_tls(&tcp_client, &dns_client, tls_config);

        // Sleep for a while before the starting requests
        // This also gives other tasks a chance to initialise and run
        info!("Waiting for 5 seconds before making the request...");
        let query_delay_secs: u64 = option_env!("QUERY_DELAY").and_then(|s| s.parse().ok()).unwrap_or(30);
        Timer::after_secs(query_delay_secs).await;

        // Make the HTTP request to the TFL API
        info!("connecting to {}", &url);

        // 1. Make HTTP request
        let mut request = match http_client.request(Method::GET, &url).await {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to make HTTP request: {}", e);
                continue;
            }
        };

        // 2. Send HTTP request
        let response = match request.send(&mut rx_buffer).await {
            Ok(resp) => resp,
            Err(e) => {
                error!("Failed to send HTTP request: {}", e);
                continue;
            }
        };

        // 3. Read response body
        let mut body = match response.body().read_to_end().await {
            Ok(body) => body,
            Err(_) => {
                error!("Failed to read response body");
                continue;
            }
        };

        // 4. Process JSON objects in body
        let mut searching = true;
        while searching {
            if let Some(json_object) = extract_first_json_object(&body) {
                match from_slice::<TflApiPreciction>(&json_object) {
                    Ok((prediction, used)) => {
                        if prediction.direction == "outbound" {
                            info!("Used {} bytes from the response body", used);
                            searching = false;
                            info!("Sending preduction to display task data channel");
                            DISPLAY_TASK_DATA_CHANNEL.send(prediction).await;
                            info!("Sent body to display task data channel");
                        } else {
                            body = &mut body[used..];
                        }
                    }
                    Err(e) => {
                        error!("Failed to deserialise JSON: {}", e);
                        error!("JSON: {}", str::from_utf8(json_object).unwrap_or("Invalid UTF-8"));
                        searching = false;
                    }
                }
            } else {
                error!("Could not extract JSON object from body");
                searching = false;
            }
        }
    }
}
