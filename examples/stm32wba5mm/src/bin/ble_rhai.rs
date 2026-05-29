//! BLE Rhai Interpreter Demo
//!
//! Receives Rhai scripts over BLE NUS (Nordic UART Service). After 500 ms of
//! idle time the accumulated input is dispatched to a dedicated eval task.
//! The eval task runs the Rhai engine and sends the result back over a channel;
//! the BLE task then notifies the client without ever blocking on eval.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐   ┌──────────────────────────────┐
//! │  main / BLE task                    │   │  eval_task                   │
//! │                                     │   │                              │
//! │  select3(                           │   │  owns: Engine, LED pin (PA1) │
//! │    ble.read_event(),         ──────────→│  SCRIPT_CHAN.receive()       │
//! │    RESULT_CHAN.receive(),    ←──────────│  engine.eval()               │
//! │    Timer::after_millis(500),    │   │   │  LED_STATE → set_level()     │
//! │  )                              │   │   │  RESULT_CHAN.send()          │
//! │                                 │   │   └──────────────────────────────┘
//! │  on timeout  → SCRIPT_CHAN.send()       (eval_pending flag prevents
//! │  on result   → gatt.notify()             double-dispatch)
//! │  on BLE event → handle connect /
//! │                 disconnect / RX data
//! └─────────────────────────────────────┘
//! ```
//!
//! BLE events are never blocked while Rhai evaluates. Both channels have
//! depth 1, so the eval task processes one script at a time.
//!
//! ## Build
//! cargo build --release --bin ble_rhai --features scripting

#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;

use embedded_alloc::LlffHeap as Heap;

use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, Either3, select, select3};
use embassy_stm32::aes::{self, Aes};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::{AES as AesPeriph, PKA as PkaPeriph};
use embassy_stm32::pka::{self, Pka};
use embassy_stm32::rcc;
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{Config, bind_interrupts, peripherals};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_stm32_wpan::bluetooth::HCI;
use embassy_stm32_wpan::bluetooth::gap::{AdvData, AdvParams, AdvType, GapEvent};
use embassy_stm32_wpan::bluetooth::gap::types::OwnAddressType;
use embassy_stm32_wpan::bluetooth::gatt::{
    CccdValue, CharProperties, GattEventMask, SecurityPermissions, ServiceType, Uuid,
    is_cccd_handle, is_value_handle,
};
use embassy_stm32_wpan::{HighInterruptHandler, LowInterruptHandler, Platform, new_platform};
use rhai::{Dynamic, Engine, packages::BasicMathPackage, packages::CorePackage, packages::Package};
use static_cell::StaticCell;
use stm32wb_hci::Event;
use stm32wb_hci::vendor::event::{AttExchangeMtuResponse, VendorEvent};
use {defmt_rtt as _, panic_probe as _};

#[global_allocator]
static HEAP: Heap = Heap::empty();

const HEAP_SIZE: usize = 48 * 1024;

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<peripherals::RNG>;
    AES => aes::InterruptHandler<AesPeriph>;
    PKA => pka::InterruptHandler<PkaPeriph>;
    RADIO => HighInterruptHandler;
    HASH => LowInterruptHandler;
});

// Nordic UART Service (NUS) UUIDs
const NUS_SERVICE_UUID: [u8; 16] = [
    0x9E, 0xCA, 0xDC, 0x24, 0x0E, 0xE5, 0xA9, 0xE0, 0x93, 0xF3, 0xA3, 0xB5, 0x01, 0x00, 0x40, 0x6E,
];
const NUS_RX_CHAR_UUID: [u8; 16] = [
    0x9E, 0xCA, 0xDC, 0x24, 0x0E, 0xE5, 0xA9, 0xE0, 0x93, 0xF3, 0xA3, 0xB5, 0x02, 0x00, 0x40, 0x6E,
];
const NUS_TX_CHAR_UUID: [u8; 16] = [
    0x9E, 0xCA, 0xDC, 0x24, 0x0E, 0xE5, 0xA9, 0xE0, 0x93, 0xF3, 0xA3, 0xB5, 0x03, 0x00, 0x40, 0x6E,
];

const MAX_DATA_LEN: usize = 244;
const INPUT_BUF_SIZE: usize = 1024;
const RESULT_BUF_SIZE: usize = 512;
const PRINT_BUF_SIZE: usize = 1024;

// BLE task → eval task: raw script bytes
static SCRIPT_CHAN: Channel<CriticalSectionRawMutex, heapless::Vec<u8, INPUT_BUF_SIZE>, 1> =
    Channel::new();

// eval task → BLE task: result bytes (print output + eval result)
static RESULT_CHAN: Channel<CriticalSectionRawMutex, heapless::Vec<u8, RESULT_BUF_SIZE>, 1> =
    Channel::new();

// Rhai print()/debug() output captured during eval, flushed into RESULT_CHAN message
static PRINT_BUF: Mutex<CriticalSectionRawMutex, RefCell<heapless::Vec<u8, PRINT_BUF_SIZE>>> =
    Mutex::new(RefCell::new(heapless::Vec::new()));

// LED state bridge between the Rhai led() closure and the Output pin owned by eval_task
static LED_STATE: AtomicBool = AtomicBool::new(false);

static LED_CELL: StaticCell<Output<'static>> = StaticCell::new();

// ---------------------------------------------------------------------------
// BLE platform tasks
// ---------------------------------------------------------------------------

#[embassy_executor::task]
async fn rng_runner_task(platform: &'static Platform) {
    platform.run_rng().await
}

#[embassy_executor::task]
async fn ble_runner_task(platform: &'static Platform) {
    platform.run_ble().await
}

// ---------------------------------------------------------------------------
// Eval task — owns the Rhai Engine and the LED pin
// ---------------------------------------------------------------------------

#[embassy_executor::task]
async fn eval_task(led: &'static mut Output<'static>) {
    let mut engine = Engine::new_raw();
    BasicMathPackage::new().register_into_engine(&mut engine);
    CorePackage::new().register_into_engine(&mut engine);

    engine.register_fn("led", |state: bool| {
        LED_STATE.store(state, Ordering::Relaxed);
    });

    engine.register_fn("timestamp", || {
        embassy_time::Instant::now().as_ticks() as i64
    });

    engine.on_print(|s| {
        PRINT_BUF.lock(|buf| {
            let mut buf = buf.borrow_mut();
            for &b in s.as_bytes() { let _ = buf.push(b); }
            let _ = buf.push(b'\r');
            let _ = buf.push(b'\n');
        });
    });
    engine.on_debug(|s, src, pos| {
        let msg = if let Some(src) = src {
            format!("[{}@{:?}] {}", src, pos, s)
        } else {
            format!("{}", s)
        };
        PRINT_BUF.lock(|buf| {
            let mut buf = buf.borrow_mut();
            for &b in msg.as_bytes() { let _ = buf.push(b); }
            let _ = buf.push(b'\r');
            let _ = buf.push(b'\n');
        });
    });

    info!("eval_task ready");

    loop {
        let script_bytes = SCRIPT_CHAN.receive().await;

        let mut result_buf: heapless::Vec<u8, RESULT_BUF_SIZE> = heapless::Vec::new();

        if let Ok(script) = core::str::from_utf8(&script_bytes) {
            info!("eval: {} bytes\n{}", script_bytes.len(), script);

            let eval_result = match engine.eval::<Dynamic>(script) {
                Ok(result) => {
                    let type_name = result.type_name();
                    let is_string = result.is_string();
                    info!("eval ok: type={} is_string={}", type_name, if is_string { "yes" } else { "no" });
                    let value = if is_string {
                        result.into_string().unwrap_or_default()
                    } else {
                        format!("{}", result)
                    };
                    let reply = format!("{}\r\n", value);
                    info!("reply: {}", format!("{:?}", reply).as_str());
                    reply
                }
                Err(e) => {
                    let s = format!("{}", e);
                    warn!("eval err: {}", s.as_str());
                    format!("err: {}\r\n", e)
                }
            };

            // Apply LED state set by the led() Rhai function
            led.set_level(if LED_STATE.load(Ordering::Relaxed) { Level::High } else { Level::Low });

            // Collect print() output then eval result into result_buf
            PRINT_BUF.lock(|buf| {
                let mut b = buf.borrow_mut();
                for &byte in b.iter() {
                    if result_buf.len() < RESULT_BUF_SIZE { let _ = result_buf.push(byte); }
                }
                b.clear();
            });
            for &byte in eval_result.as_bytes() {
                if result_buf.len() < RESULT_BUF_SIZE { let _ = result_buf.push(byte); }
            }
        }

        RESULT_CHAN.send(result_buf).await;
    }
}

// ---------------------------------------------------------------------------
// Main — BLE task
// ---------------------------------------------------------------------------

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    {
        use core::mem::MaybeUninit;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(core::ptr::addr_of!(HEAP_MEM) as usize, HEAP_SIZE) }
    }

    let mut config = Config::default();
    config.rcc = rcc::Config::new_wpan();
    let p = embassy_stm32::init(config);

    let led = LED_CELL.init(Output::new(p.PA1, Level::Low, Speed::Low));

    info!("BLE Rhai interpreter starting");

    let (platform, runtime) = new_platform!(
        Rng::new(p.RNG, Irqs),
        Aes::new_blocking(p.AES, Irqs),
        Pka::new_blocking(p.PKA, Irqs),
        8
    );

    spawner.spawn(rng_runner_task(platform).expect("spawn rng"));
    spawner.spawn(ble_runner_task(platform).expect("spawn ble"));

    let mut ble = HCI::new(platform, runtime, Irqs)
        .await
        .expect("BLE init failed");
    embassy_futures::yield_now().await;

    let mut gatt = ble.gatt_server();

    let service_handle = gatt
        .add_service(Uuid::from_u128_le(NUS_SERVICE_UUID), ServiceType::Primary, 10)
        .expect("add NUS service");

    let rx_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u128_le(NUS_RX_CHAR_UUID),
            MAX_DATA_LEN as u16,
            CharProperties::WRITE | CharProperties::WRITE_WITHOUT_RESPONSE,
            SecurityPermissions::NONE,
            GattEventMask::ATTRIBUTE_MODIFIED,
            0,
            true,
        )
        .expect("add RX char");

    let tx_char_handle = gatt
        .add_characteristic(
            service_handle,
            Uuid::from_u128_le(NUS_TX_CHAR_UUID),
            MAX_DATA_LEN as u16,
            CharProperties::NOTIFY,
            SecurityPermissions::NONE,
            GattEventMask::empty(),
            0,
            true,
        )
        .expect("add TX char");

    let mut adv_data = AdvData::new();
    adv_data.add_flags(0x06).unwrap();
    adv_data.add_name("RhaiShell").unwrap();

    let mut scan_rsp = AdvData::new();
    scan_rsp.add_service_uuid_128(&NUS_SERVICE_UUID).unwrap();

    let adv_params = AdvParams {
        interval_min: 0x0050,
        interval_max: 0x0064,
        adv_type: AdvType::ConnectableUndirected,
        own_addr_type: OwnAddressType::Random,
        ..AdvParams::default()
    };

    ble.start_advertising(adv_params.clone(), adv_data.clone(), Some(scan_rsp.clone()))
        .await
        .expect("start advertising");

    // Spawn eval_task only after BLE is fully initialised and advertising.
    // Spawning earlier would let Engine::new_raw() run heap allocations
    // concurrently with BLE stack init, which can corrupt BLE internal state
    // (null callback pointer → BusFault at 0x00000010).
    spawner.spawn(eval_task(led).expect("spawn eval"));

    info!("Advertising as 'RhaiShell' — connect and send Rhai expressions");

    let mut input_buf: heapless::Vec<u8, INPUT_BUF_SIZE> = heapless::Vec::new();
    let mut tx_notifications = false;
    let mut conn_handle: Option<u16> = None;
    let mut eval_pending = false;

    loop {
        // 3-way select when buffer is non-empty: BLE event | eval result | 500ms timeout
        // 2-way select otherwise: BLE event | eval result (in case disconnect triggered eval)
        enum Msg<E> {
            Ble(E),
            Result(heapless::Vec<u8, RESULT_BUF_SIZE>),
            Timeout,
        }

        let msg = if !input_buf.is_empty() {
            match select3(
                ble.read_event(),
                RESULT_CHAN.receive(),
                embassy_time::Timer::after_millis(500),
            ).await {
                Either3::First(ev)  => Msg::Ble(ev),
                Either3::Second(r)  => Msg::Result(r),
                Either3::Third(_)   => Msg::Timeout,
            }
        } else {
            match select(ble.read_event(), RESULT_CHAN.receive()).await {
                Either::First(ev)  => Msg::Ble(ev),
                Either::Second(r)  => Msg::Result(r),
            }
        };

        match msg {
            // ----------------------------------------------------------------
            // 500 ms idle — dispatch buffer to eval task
            // ----------------------------------------------------------------
            Msg::Timeout => {
                if !eval_pending {
                    let mut script_buf: heapless::Vec<u8, INPUT_BUF_SIZE> = heapless::Vec::new();
                    for &b in input_buf.iter() { let _ = script_buf.push(b); }
                    SCRIPT_CHAN.send(script_buf).await;
                    eval_pending = true;
                }
                input_buf.clear();
            }

            // ----------------------------------------------------------------
            // Eval result arrived — notify client
            // ----------------------------------------------------------------
            Msg::Result(result) => {
                eval_pending = false;
                info!("result ready, {} bytes", result.len());
                if let Some(conn) = conn_handle {
                    for chunk in result.chunks(MAX_DATA_LEN) {
                        let _ = gatt.notify(conn, service_handle, tx_char_handle, chunk);
                    }
                    let _ = gatt.notify(conn, service_handle, tx_char_handle, b"> ");
                }
            }

            // ----------------------------------------------------------------
            // BLE event
            // ----------------------------------------------------------------
            Msg::Ble(event) => {
                if let Some(gap_event) = ble.process_event(&event) {
                    match gap_event {
                        GapEvent::Connected(conn) => {
                            info!("Connected: 0x{:04X}", conn.handle.0);
                            conn_handle = Some(conn.handle.0);
                            tx_notifications = false;
                            input_buf.clear();
                            eval_pending = false;
                        }
                        GapEvent::Disconnected { handle, reason } => {
                            info!("Disconnected: 0x{:04X} reason=0x{:02X}", handle.0, reason);
                            // Flush remaining buffer to eval task on disconnect
                            if !input_buf.is_empty() && !eval_pending {
                                let mut script_buf: heapless::Vec<u8, INPUT_BUF_SIZE> = heapless::Vec::new();
                                for &b in input_buf.iter() { let _ = script_buf.push(b); }
                                SCRIPT_CHAN.send(script_buf).await;
                                eval_pending = true;
                                input_buf.clear();
                            }
                            conn_handle = None;
                            tx_notifications = false;
                            ble.start_advertising(
                                adv_params.clone(), adv_data.clone(), Some(scan_rsp.clone()),
                            ).await.expect("restart advertising");
                        }
                        _ => {}
                    }
                }

                match &event {
                    Event::Vendor(VendorEvent::GattAttributeModified(attr)) => {
                        if is_cccd_handle(tx_char_handle.0, attr.attr_handle.0) {
                            tx_notifications = CccdValue::from_bytes(attr.data()).notifications;
                            if tx_notifications {
                                if let Some(conn) = conn_handle {
                                    let _ = gatt.notify(conn, service_handle, tx_char_handle, b"> ");
                                }
                            }
                            info!("TX notifications {}", if tx_notifications { "on" } else { "off" });
                        } else if is_value_handle(rx_char_handle.0, attr.attr_handle.0) {
                            for &b in attr.data() {
                                if input_buf.len() < INPUT_BUF_SIZE {
                                    let _ = input_buf.push(b);
                                }
                            }
                            debug!("buffered {} bytes total", input_buf.len());
                        }
                    }

                    Event::Vendor(VendorEvent::AttExchangeMtuResponse(AttExchangeMtuResponse {
                        conn_handle: ch,
                        server_rx_mtu,
                    })) => {
                        if let Some(conn) = ble.get_connection_mut(*ch) {
                            conn.update_mtu(*server_rx_mtu as u16);
                        }
                    }

                    _ => {}
                }
            }
        }
    }
}
