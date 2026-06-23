//! USB plain-text CDC monitor (`if00`).
//!
//! `defmt` logs go via RTT — visible in the same terminal as `cargo run` / `probe-rs run`
//! when flashing over SWD. Use `./usb-monitor.sh text` for ASCII milestones only.

use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::Peri;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

static TEXT_LINES: Channel<CriticalSectionRawMutex, &'static str, 16> = Channel::new();

static TEXT_STATE: StaticCell<State> = StaticCell::new();
static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

const PACKET: u16 = 64;

fn usb_config() -> Config<'static> {
    let mut c = Config::new(0xc0de, 0xcafe);
    c.manufacturer = Some("Embassy");
    c.product = Some("RP2350-Touch-LCD-7");
    c.serial_number = Some("rp2350-lcd7");
    c.max_power = 100;
    c.max_packet_size_0 = 64;
    c
}

/// Plain-text line on the text CDC. Safe from any task.
pub fn line(msg: &'static str) {
    let _ = TEXT_LINES.try_send(msg);
}

pub fn spawn(spawner: &Spawner, usb: Peri<'static, USB>) {
    spawner.spawn(unwrap!(run(usb)));
}

async fn write_text(sender: &mut embassy_usb::class::cdc_acm::Sender<'static, Driver<'static, USB>>, s: &str) {
    let bytes = s.as_bytes();
    let mut off = 0;
    while off < bytes.len() {
        let end = (off + 63).min(bytes.len());
        if sender.write_packet(&bytes[off..end]).await.is_err() {
            return;
        }
        off = end;
    }
    let _ = sender.write_packet(b"\r\n").await;
}

async fn text_port(mut sender: embassy_usb::class::cdc_acm::Sender<'static, Driver<'static, USB>>) {
    loop {
        sender.wait_connection().await;
        write_text(&mut sender, "text port ready — defmt via probe-rs RTT (cargo run)").await;
        loop {
            while let Ok(msg) = TEXT_LINES.try_receive() {
                write_text(&mut sender, msg).await;
            }
            match sender.write_packet(b".\r\n").await {
                Ok(()) => {}
                Err(EndpointError::Disabled) => break,
                Err(EndpointError::BufferOverflow) => {}
            }
            Timer::after_secs(2).await;
        }
    }
}

#[embassy_executor::task]
async fn run(usb: Peri<'static, USB>) -> ! {
    let driver = Driver::new(usb, Irqs);

    let mut builder = Builder::new(
        driver,
        usb_config(),
        CONFIG_DESC.init([0; 256]),
        BOS_DESC.init([0; 256]),
        &mut [],
        CONTROL_BUF.init([0; 64]),
    );

    let text_state = TEXT_STATE.init(State::new());
    let text_class = CdcAcmClass::new(&mut builder, text_state, PACKET);
    let mut usb = builder.build();
    let (text_tx, _) = text_class.split();

    join(usb.run(), text_port(text_tx)).await;
    defmt::panic!("USB monitor exited");
}
