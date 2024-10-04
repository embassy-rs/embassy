#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts, gpio, peripherals::USB};
use embassy_time::{Duration, Timer};
use embassy_usb::host::{Channel, ControlChannelExt, DeviceDescriptor, USBDescriptor, UsbDeviceRegistry, UsbHost};
use embassy_usb_driver::host::{channel, UsbHostDriver};
use gpio::{Level, Output};
use heapless::Vec;

use {defmt_rtt as _, panic_probe as _};

use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::host::InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialise Peripherals
    let p = embassy_rp::init(Default::default());

    // Create the driver, from the HAL.
    let mut driver = embassy_rp::usb::host::Driver::new(p.USB, Irqs);

    info!("Start USB driver");

    static HOST_REGISTRY: StaticCell<UsbDeviceRegistry<1>> = StaticCell::new();
    let registry = HOST_REGISTRY.init(UsbDeviceRegistry::new());

    static HOST: StaticCell<UsbHost<embassy_rp::usb::host::Driver<'static, USB>>> = StaticCell::new();
    let host = HOST.init(UsbHost::new(driver, registry));

    loop {
        let dev = host.poll().await.unwrap();
        
        info!("Found {} interfaces", dev.cfg_desc.num_interfaces);

        let interface0 = dev.cfg_desc.parse_interface(0).unwrap();
        info!("Interface 0: {:?}", interface0);

        // let interface1 = dev.cfg_desc.parse_interface(1).unwrap();
        // info!("Interface 1: {:?}", interface1);

        let endpoints = interface0.parse_endpoints::<4>();

        info!("Endpoints: {:?}", endpoints);

        if hid_keyboard::is_compatible(&interface0) {
            let hid_desc: hid_keyboard::HIDDescriptor = interface0.parse_class_descriptor().unwrap();
            info!("HID descriptor: {:?}", hid_desc);

            {
                // Request report descriptor
                let mut buffer = [0u8; 128];

                // Control channel should be unlocked after use 
                let mut cc = host.control_channel(dev.addr).await.unwrap();
            
                let request_buffer = &mut buffer[..hid_desc.descriptor_length0 as usize];
                cc.interface_request_descriptor_bytes::<ReportDescriptor>(
                    interface0.interface_number, 
                    request_buffer
                )
                .await
                .unwrap();
            }

            match hid_keyboard::HIDBootKeyboard::configure(dev, host).await {
                Ok(mut keyboard) => {
                    info!("Keyboard initialized");
                    spawner.must_spawn(keyboard_task(keyboard));
                },
                Err(e) => {
                    error!("Failed to configure keyboard: {}", e)
                }
            }
        } else {
            info!("Not a keyboard");
        }
    }
}

#[embassy_executor::task(pool_size = 2)]
async fn keyboard_task(mut keyboard: hid_keyboard::HIDBootKeyboard<'static, impl UsbHostDriver>) {
    loop {
        if let Err(e) = keyboard.listen().await {
            warn!("keyboard error: {}", e);
            return
        }
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

    use embassy_usb::host::{ConfigurationDescriptor, Device, Channel, InterfaceDescriptor, UsbHost};
    use embassy_usb_driver::host::{channel, HostError, UsbChannel};

    use super::*;

    pub fn is_compatible(desc: &InterfaceDescriptor) -> bool {
        desc.interface_class == 3 && desc.interface_subclass == 1 && desc.interface_protocol == 1
    }

    #[derive(defmt::Format)]
    pub struct HIDDescriptor {
        pub len: u8,
        pub descriptor_type: u8,
        pub bcd_hid: u16,
        pub country_code: u8,
        pub num_descriptors: u8,

        // num_descriptors determines how many pairs of descriptor_typeI/descriptor_lengthI follow.
        pub descriptor_type0: u8,
        pub descriptor_length0: u16,
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

    pub struct HIDBootKeyboard<'d, D>
    where
        D: UsbHostDriver,
    {
        channel: Channel<'d, D, channel::Interrupt, channel::In>,
    }

    impl<'d, D> HIDBootKeyboard<'d, D>
    where
        D: UsbHostDriver,
    {
        pub async fn listen(&mut self) -> Result<(), HostError> {
            let mut buffer = [0u8; 8];

            self.channel.request_in(&mut buffer[..]).await?;
            let keycodes = parse_payload(&buffer);

            for keycode in keycodes {
                let chr = keycode_to_ascii(keycode);
                info!("Ascii: {:?}", chr);
            }

            Ok(())
        }

        pub async fn configure(
            dev: Device,
            host: &'d UsbHost<'d, D>,
        ) -> Result<Self, HostError> {
            // Since we know this is a boot keyboard we don't need to parse the report descriptor.
            // We still need an Endpoint Descriptor.
            // claim endpoint

            debug!("Initializing HID keyboard");

            let mut boot_hid_interface = None;
            for i in 0..dev.cfg_desc.num_interfaces {
                let interface = dev.cfg_desc.parse_interface(i as usize).unwrap();
                if is_compatible(&interface) {
                    boot_hid_interface = Some(interface);
                }
            }

            let Some(interface) = boot_hid_interface else {
                error!("No HID interface found");
                return Err(HostError::Other("No HID Interface found"));
            };

            let mut cc = host.control_channel(dev.addr).await?;

            const SET_PROTOCOL: u8 = 0x0B;
            const BOOT_PROTOCOL: u16 = 0x0000;
            cc.class_request_out(
                SET_PROTOCOL, 
                BOOT_PROTOCOL, 
                interface.interface_number as u16, 
                &mut []
            )
            .await?;

            const SET_IDLE: u8 = 0x0A;
            cc.class_request_out(SET_IDLE, 0, interface.interface_number as u16, &mut [])
                .await?;

            let endpoints = interface.parse_endpoints::<1>();

            if endpoints.len() != 1 {
                error!("Wrong number of endpoints");
                return Err(HostError::Other("Wrong number of endpoints"));
            }

            // Test drop
            let channel = {
                let _channel0 = host.alloc_channel::<channel::Interrupt, channel::In>(dev.addr, &endpoints[0])?;
                let _channel1 = host.alloc_channel::<channel::Interrupt, channel::In>(dev.addr, &endpoints[0])?;

                host.alloc_channel(dev.addr, &endpoints[0])?
            };

            Ok(HIDBootKeyboard {
                channel,
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
