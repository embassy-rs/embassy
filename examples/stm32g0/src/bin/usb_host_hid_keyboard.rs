#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{AfType, Level, Output, OutputType, Speed};
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::time::{mhz, Hertz};
use embassy_stm32::usb::USBHostDriver;
use embassy_stm32::{bind_interrupts, pac, peripherals, usb, Config};
use embassy_time::Timer;
use embassy_usb::host::{USBDescriptor, UsbHost};
use heapless::Vec;
use {defmt_rtt as _, panic_probe as _};

pub use crate::pac::rcc::vals::Mcosel;

bind_interrupts!(struct Irqs {
    USB_UCPD1_2 => usb::USBHostInterruptHandler<peripherals::USB>;
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[derive(defmt::Format)]
struct HIDDescriptor {
    len: u8,
    descriptor_type: u8,
    bcd_hid: u16,
    country_code: u8,
    num_descriptors: u8,

    // num_descriptors determines how many pairs of descriptor_typeI/descriptor_lengthI follow.
    descriptor_type0: u8,
    descriptor_length0: u16,
}

impl USBDescriptor for HIDDescriptor {
    const SIZE: usize = 9; // only valid for 1 descriptor

    const DESC_TYPE: u8 = 33;

    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }

        Ok(Self {
            len: bytes[0],
            descriptor_type: bytes[1],
            bcd_hid: u16::from_le_bytes([bytes[2], bytes[3]]),
            country_code: bytes[4],
            num_descriptors: bytes[5],
            descriptor_type0: bytes[6],
            descriptor_length0: u16::from_le_bytes([bytes[7], bytes[8]]),
        })
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: mhz(8),
            mode: HseMode::Bypass,
        });

        config.rcc.pll = Some(
            // fVCO = fPLLIN × (N / M) = 8 MHz × (16 / 1) = 128 MHz
            // • fPLLP = fVCO / P
            // • fPLLQ = fVCO / Q
            // • fPLLR = fVCO / R
            // N = mul
            // M = prediv
            // PLLRCLK => system clock
            // PLLQCLK => USB
            // PLLPCLK => unused
            // Maximum VCO frequency is 344 MHz. For Range 1 (default)
            // 2.66 < PLL / M < 16
            // M = 2 => 8 / 2 = 4
            // N = 30 => 4 * 30 = 120
            // fVCO = 8Mhz / 2 * 60 = 240MHz
            // PLLR = 240MHz / 4 = 60MHz
            // PLLQ = 240MHz / 5 = 48MHz
            Pll {
                source: PllSource::HSE, // 8 Mhz
                prediv: PllPreDiv::DIV2,
                mul: PllMul::MUL60,
                divp: None,
                divq: Some(PllQDiv::DIV5), // PLLQ should be 48MHz
                divr: Some(PllRDiv::DIV4),
            },
        );
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true });
        config.rcc.mux.usbsel = mux::Usbsel::PLL1_Q;
    }

    let p = embassy_stm32::init(config);

    // configure clock out
    pac::RCC
        .cfgr()
        .modify(|w: &mut pac::rcc::regs::Cfgr| w.set_mco1sel(Mcosel::PLL1_Q));
    // configure pin for clock out
    let mut mco = embassy_stm32::gpio::Flex::new(p.PA9);
    mco.set_as_af_unchecked(0, AfType::output(OutputType::PushPull, Speed::High));

    let mut led = Output::new(p.PA5, Level::High, Speed::Low);

    let mut enable = Output::new(p.PC8, Level::High, Speed::Low);
    enable.set_high();

    Timer::after_millis(1000).await;

    // i2c
    // SCL: PB8
    // SDA: PB9
    let mut i2c = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        Irqs,
        p.DMA1_CH1,
        p.DMA1_CH2,
        Hertz(100_000),
        Default::default(),
    );

    let i2c_address: u8 = 0x68 >> 1; //0b00110_100; // 7 bits address 0110 10x

    // We have to turn on the GDP switches to enable power from SOURCE
    let reg = 0;
    let value: u8 = 0b00011100;

    i2c.write(i2c_address, &[reg, value]).await.unwrap();

    let reg = 0x1;
    let mut read_buf = [255u8; 1];
    i2c.write_read(i2c_address, &[reg], &mut read_buf).await.unwrap();

    info!("Value 1: {:02X}", read_buf[0]);

    // Create the driver, from the HAL.
    let mut driver = USBHostDriver::new(p.USB, Irqs, p.PA12, p.PA11);

    info!("Start USB driver");
    driver.start();
    let mut host = UsbHost::new(driver);

    // loop {
    host.wait_for_device().await;

    let (_device_desc, cfg_desc) = host.enumerate().await.unwrap();

    info!("Found {} interfaces", cfg_desc.num_interfaces);

    let interface0 = cfg_desc.parse_interface(0).unwrap();
    info!("Interface 0: {:?}", interface0);

    // let interface1 = cfg_desc.parse_interface(1).unwrap();
    // info!("Interface 1: {:?}", interface1);

    let endpoints = interface0.parse_endpoints::<4>();

    info!("Endpoints: {:?}", endpoints[0]);

    if hid_keyboard::is_compatible(&interface0) {
        let hid_desc: HIDDescriptor = interface0.parse_class_descriptor().unwrap();

        info!("HID descriptor: {:?}", hid_desc);

        // Request report descriptor
        let mut buffer = [0u8; 128];

        let request_buffer = &mut buffer[..hid_desc.descriptor_length0 as usize];
        host.interface_request_descriptor_bytes::<ReportDescriptor>(interface0.interface_number, request_buffer)
            .await
            .unwrap();

        if let Ok(mut keyboard) = hid_keyboard::HIDKeyboardBuilder::new(host).build(&cfg_desc).await {
            info!("Keyboard initialized");
            loop {
                keyboard.listen().await;
            }
        }
        // let mut host = keyboard.release();
    } else {
        host.wait_for_device_disconnect().await;
    }

    info!("Bye");
    loop {
        led.set_high();
        Timer::after_millis(900).await;

        led.set_low();
        Timer::after_millis(900).await;
    }
}

struct ReportDescriptor {}

impl USBDescriptor for ReportDescriptor {
    const SIZE: usize = 0; //

    const DESC_TYPE: u8 = 34;

    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        debug!("Report descriptor: {:?}", bytes);
        Ok(Self {})
    }
}

mod hid_keyboard {

    use embassy_usb::host::{ConfigurationDescriptor, InterfaceDescriptor, UsbHost};
    use embassy_usb_driver::host::{ChannelIn, USBHostDriverTrait};

    use super::*;

    pub fn is_compatible(desc: &InterfaceDescriptor) -> bool {
        desc.interface_class == 3 && desc.interface_subclass == 1 && desc.interface_protocol == 1
    }

    pub struct HIDBootKeyboard<D>
    where
        D: USBHostDriverTrait,
    {
        host: UsbHost<D>,
        channel: D::ChannelIn,
    }

    impl<D> HIDBootKeyboard<D>
    where
        D: USBHostDriverTrait,
    {
        pub async fn listen(&mut self) {
            let mut buffer = [0u8; 8];

            if let Ok(_l) = self.channel.read(&mut buffer[..]).await {
                let keycodes = parse_payload(&buffer);

                for keycode in keycodes {
                    let chr = keycode_to_ascii(keycode);
                    info!("Ascii: {:?}", chr);
                }
            }
        }

        #[allow(unused)]
        pub fn release(self) -> UsbHost<D> {
            self.host
        }
    }

    pub struct HIDKeyboardBuilder<D>
    where
        D: USBHostDriverTrait,
    {
        host: UsbHost<D>,
    }

    impl<D: USBHostDriverTrait> HIDKeyboardBuilder<D> {
        pub fn new(host: UsbHost<D>) -> Self {
            Self { host }
        }

        pub async fn build(mut self, cfg_desc: &ConfigurationDescriptor) -> Result<HIDBootKeyboard<D>, UsbHost<D>> {
            // Since we know this is a boot keyboard we don't need to parse the report descriptor.

            // We still need an Endpoint Descriptor.
            // claim endpoint

            debug!("Initializing HID keyboard");

            let mut boot_hid_interface = None;
            for i in 0..cfg_desc.num_interfaces {
                let interface = cfg_desc.parse_interface(i as usize).unwrap();
                if is_compatible(&interface) {
                    boot_hid_interface = Some(interface);
                }
            }

            let Some(interface) = boot_hid_interface else {
                error!("No HID interface found");
                return Err(self.host);
            };

            const SET_PROTOCOL: u8 = 0x0B;
            const BOOT_PROTOCOL: u16 = 0x0000;
            self.host
                .class_request_out(SET_PROTOCOL, BOOT_PROTOCOL, interface.interface_number as u16, &mut [])
                .await
                .unwrap();

            const SET_IDLE: u8 = 0x0A;
            self.host
                .class_request_out(SET_IDLE, 0, interface.interface_number as u16, &mut [])
                .await
                .unwrap();

            let endpoints = interface.parse_endpoints::<1>();

            if endpoints.len() != 1 {
                error!("Wrong number of endpoints");
                return Err(self.host);
            }

            let channel = self.host.claim_endpoint(&endpoints[0]).unwrap();

            Ok(HIDBootKeyboard {
                host: self.host,
                channel: channel,
            })
        }
    }

    fn parse_payload(buffer: &[u8; 8]) -> Vec<u8, 6> {
        let _modifier = buffer[0];
        // byte 1 is reserved

        let key_codes = buffer[2..]
            .iter()
            .filter(|b| **b != 0)
            .map(|b| *b)
            .collect::<Vec<u8, 6>>();

        return key_codes;
    }

    const SCANCODE_MAP: [char; 57] = [
        ' ',  // 0 Reserved (no event)
        ' ',  // 1
        ' ',  // 2
        ' ',  // 3
        'a',  // 4
        'b',  // 5
        'c',  // 6
        'd',  // 7
        'e',  // 8
        'f',  // 9
        'g',  // 10
        'h',  // 11
        'i',  // 12
        'j',  // 13
        'k',  // 14
        'l',  // 15
        'm',  // 16
        'n',  // 17
        'o',  // 18
        'p',  // 19
        'q',  // 20
        'r',  // 21
        's',  // 22
        't',  // 23
        'u',  // 24
        'v',  // 25
        'w',  // 26
        'x',  // 27
        'y',  // 28
        'z',  // 29
        '1',  // 30
        '2',  // 31
        '3',  // 32
        '4',  // 33
        '5',  // 34
        '6',  // 35
        '7',  // 36
        '8',  // 37
        '9',  // 38
        '0',  // 39
        '\n', // 40 enter
        ' ',  // 41 escape
        ' ',  // 42 backspace
        '\t', // 43 tab
        ' ',  // 44 space
        '-',  // 45 minus
        '=',  // 46 equal
        '[',  // 47 left bracket
        ']',  // 48 right bracket
        '\\', // 49 backslash
        '#',  // 50 pound
        ';',  // 51 semicolon
        '\'', // 52 apostrophe
        '`',  // 53 grave accent
        ',',  // 54 comma
        '.',  // 55 period
        '/',  // 56 slash
    ];

    fn keycode_to_ascii(keycode: u8) -> char {
        if keycode < SCANCODE_MAP.len() as u8 {
            SCANCODE_MAP[keycode as usize]
        } else {
            0 as char
        }
    }
}
