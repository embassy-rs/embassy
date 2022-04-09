#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _; // global logger
use panic_probe as _;

use defmt::{info, unwrap};
use defmt_rtt as _; // global logger
use embassy::interrupt::InterruptExt;
use futures::pin_mut;
use panic_probe as _; // print out panic messages

use embassy::executor::Spawner;
use embassy::io::{AsyncBufReadExt, AsyncWriteExt};
use embassy_stm32::pac::pwr::vals::Usv;
use embassy_stm32::pac::{PWR, RCC};
use embassy_stm32::rcc::{ClockSrc, PLLClkDiv, PLLMul, PLLSource, PLLSrcDiv};
use embassy_stm32::usb_otg::{State, Usb, UsbBus, UsbOtg, UsbSerial};
use embassy_stm32::{interrupt, Config, Peripherals};
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};

static mut EP_MEMORY: [u32; 2048] = [0; 2048];

// USB requires at least 48 MHz clock
fn config() -> Config {
    let mut config = Config::default();
    // set up a 80Mhz clock
    config.rcc.mux = ClockSrc::PLL(
        PLLSource::HSI16,
        PLLClkDiv::Div2,
        PLLSrcDiv::Div2,
        PLLMul::Mul20,
        None,
    );
    // enable HSI48 clock for USB
    config.rcc.hsi48 = true;
    config
}

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, p: Peripherals) {
    // Enable PWR peripheral
    unsafe { RCC.apb1enr1().modify(|w| w.set_pwren(true)) };
    unsafe { PWR.cr2().modify(|w| w.set_usv(Usv::VALID)) }

    let mut rx_buffer = [0u8; 64];
    // we send back input + cr + lf
    let mut tx_buffer = [0u8; 66];

    let peri = UsbOtg::new_fs(p.USB_OTG_FS, p.PA12, p.PA11);
    let usb_bus = UsbBus::new(peri, unsafe { &mut EP_MEMORY });

    let serial = UsbSerial::new(&usb_bus, &mut rx_buffer, &mut tx_buffer);

    let device = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(0x02)
        .build();

    let irq = interrupt::take!(OTG_FS);
    irq.set_priority(interrupt::Priority::P3);

    let mut state = State::new();
    let usb = unsafe { Usb::new(&mut state, device, serial, irq) };
    pin_mut!(usb);

    let (mut reader, mut writer) = usb.as_ref().take_serial_0();

    info!("usb initialized!");

    unwrap!(
        writer
            .write_all(b"\r\nInput returned upper cased on CR+LF\r\n")
            .await
    );

    let mut buf = [0u8; 64];
    loop {
        let mut n = 0;

        async {
            loop {
                let char = unwrap!(reader.read_byte().await);

                if char == b'\r' || char == b'\n' {
                    break;
                }

                buf[n] = char;
                n += 1;

                // stop if we're out of room
                if n == buf.len() {
                    break;
                }
            }
        }
        .await;

        if n > 0 {
            for char in buf[..n].iter_mut() {
                // upper case
                if 0x61 <= *char && *char <= 0x7a {
                    *char &= !0x20;
                }
            }
            unwrap!(writer.write_all(&buf[..n]).await);
            unwrap!(writer.write_all(b"\r\n").await);
            unwrap!(writer.flush().await);
        }
    }
}
