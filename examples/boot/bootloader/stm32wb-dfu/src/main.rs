#![no_std]
#![no_main]

use core::cell::RefCell;

use cortex_m_rt::{entry, exception};
#[cfg(feature = "defmt")]
use defmt_rtt as _;
use embassy_boot_stm32::*;
use embassy_stm32::flash::{Flash, BANK1_REGION, WRITE_SIZE};
use embassy_stm32::rcc::WPAN_DEFAULT;
use embassy_stm32::usb::Driver;
use embassy_stm32::{bind_interrupts, peripherals, usb};
use embassy_sync::blocking_mutex::Mutex;
use embassy_usb::Builder;
use embassy_usb_dfu::consts::DfuAttributes;
use embassy_usb_dfu::{usb_dfu, Control, ResetImmediate};

bind_interrupts!(struct Irqs {
    USB_LP => usb::InterruptHandler<peripherals::USB>;
});

#[entry]
fn main() -> ! {
    let mut config = embassy_stm32::Config::default();
    config.rcc = WPAN_DEFAULT;
    let p = embassy_stm32::init(config);

    // Prevent a hard fault when accessing flash 'too early' after boot.
    #[cfg(feature = "defmt")]
    for _ in 0..10000000 {
        cortex_m::asm::nop();
    }

    let layout = Flash::new_blocking(p.FLASH).into_blocking_regions();
    let flash = Mutex::new(RefCell::new(layout.bank1_region));

    let config = BootLoaderConfig::from_linkerfile_blocking(&flash, &flash, &flash);
    let active_offset = config.active.offset();
    let bl = BootLoader::prepare::<_, _, _, 2048>(config);
    if bl.state == State::DfuDetach {
        let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Embassy");
        config.product = Some("USB-DFU Bootloader example");
        config.serial_number = Some("1235678");

        let fw_config = FirmwareUpdaterConfig::from_linkerfile_blocking(&flash, &flash);
        let mut buffer = AlignedBuffer([0; WRITE_SIZE]);
        let updater = BlockingFirmwareUpdater::new(fw_config, &mut buffer.0[..]);

        let mut device_descriptor = [0; 256];
        let mut config_descriptor = [0; 256];
        let mut bos_descriptor = [0; 256];
        let mut control_buf = [0; 4096];
        let mut state = Control::new(updater, DfuAttributes::CAN_DOWNLOAD);
        let mut builder = Builder::new(
            driver,
            config,
            &mut device_descriptor,
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut [],
            &mut control_buf,
        );

        usb_dfu::<_, _, _, ResetImmediate, 4096>(&mut builder, &mut state);

        let mut dev = builder.build();
        embassy_futures::block_on(dev.run());
    }

    unsafe { bl.load(BANK1_REGION.base + active_offset) }
}

#[no_mangle]
#[cfg_attr(target_os = "none", link_section = ".HardFault.user")]
unsafe extern "C" fn HardFault() {
    cortex_m::peripheral::SCB::sys_reset();
}

#[exception]
unsafe fn DefaultHandler(_: i16) -> ! {
    const SCB_ICSR: *const u32 = 0xE000_ED04 as *const u32;
    let irqn = core::ptr::read_volatile(SCB_ICSR) as u8 as i16 - 16;

    panic!("DefaultHandler #{:?}", irqn);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    cortex_m::asm::udf();
}
