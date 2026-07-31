//! Raw [`embassy_usb_driver::Bus`] lifecycle for both LPC55 USB controllers.
//!
//! This test covers reversible disable, endpoint cancellation, invalid endpoint
//! addresses, final controller teardown, main-SRAM lifetime, and USB1 SRAM
//! ownership while both controllers run.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nxp::config::MainClock;
use embassy_nxp::interrupt::typelevel::Interrupt as _;
use embassy_nxp::usb::{Bus, Driver, Endpoint, Instance, InterruptHandler, Memory, Out};
use embassy_nxp::{bind_interrupts, pac, peripherals};
use embassy_usb::driver::{
    Bus as _, Direction, Driver as _, Endpoint as _, EndpointAddress, EndpointError, EndpointOut as _, EndpointType,
};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB0 => InterruptHandler<peripherals::USB0>;
    USB1 => InterruptHandler<peripherals::USBHSD>;
});

/// PDRUNCFG0 power-down bits, mirroring `PDEN_USBFSPHY` / `PDEN_USBHSPHY` in
/// `embassy-nxp/src/usb/lpc55.rs`. The bit is *set* when the block is off.
const PDEN_USBFSPHY: u32 = 1 << 11;
const PDEN_USBHSPHY: u32 = 1 << 12;

const USB1_SRAM_ADDR: u32 = 0x4010_0000;

/// Exercise the reversible lifecycle while the controller remains powered.
///
/// `regs` is the register block for this instance. `assert_powered_up` checks
/// that ordinary disable does not perform final hardware teardown.
fn ensure(condition: bool, name: &'static str, failure: &'static str) {
    defmt::assert!(condition, "{}: {}", name, failure);
}

async fn check<T: Instance>(
    bus: &mut Bus<'_, T>,
    bulk: &mut Endpoint<'_, T, Out>,
    iso: &Endpoint<'_, T, Out>,
    name: &'static str,
    ep_list_base: u32,
    regs: pac::usbhsd::Usbhsd,
    assert_powered_up: impl Fn(),
) {
    let bulk_addr = bulk.info().addr;
    let iso_addr = iso.info().addr;

    bus.enable().await;
    let dcs = regs.devcmdstat().read();
    ensure(dcs.dcon(), name, "enable did not set DCON");
    ensure(dcs.dev_en(), name, "enable did not set DEV_EN");
    assert_powered_up();

    bus.endpoint_set_enabled(bulk_addr, true);
    bus.endpoint_set_stalled(bulk_addr, true);
    ensure(bus.endpoint_is_stalled(bulk_addr), name, "bulk endpoint not reported stalled");
    bus.endpoint_set_stalled(bulk_addr, false);
    ensure(!bus.endpoint_is_stalled(bulk_addr), name, "bulk endpoint remained stalled");

    bus.endpoint_set_enabled(iso_addr, true);
    bus.endpoint_set_stalled(iso_addr, true);
    ensure(!bus.endpoint_is_stalled(iso_addr), name, "isochronous endpoint reported stalled");

    for direction in [Direction::Out, Direction::In] {
        let invalid = EndpointAddress::from_parts(127, direction);
        bus.endpoint_set_enabled(invalid, true);
        bus.endpoint_set_stalled(invalid, true);
        ensure(!bus.endpoint_is_stalled(invalid), name, "invalid endpoint reported stalled");
    }

    let mut packet = [0; 512];
    let (read_result, ()) = join(bulk.read(&mut packet), bus.disable()).await;
    ensure(
        matches!(read_result, Err(EndpointError::Disabled)),
        name,
        "disable did not cancel a pending endpoint read",
    );
    ensure(regs.devcmdstat().read().dev_en(), name, "reversible disable cleared DEV_EN");
    let disabled_dcs = regs.devcmdstat().read();
    defmt::assert_eq!(
        disabled_dcs.dcon(),
        !disabled_dcs.vbus_debounced(),
        "{}: disable did not disconnect or enter attach-armed state",
        name
    );
    assert_powered_up();

    bus.enable().await;
    defmt::assert_eq!(
        regs.epliststart().read().0,
        ep_list_base,
        "{}: enable did not restore EPLISTSTART",
        name
    );
    let dcs = regs.devcmdstat().read();
    ensure(dcs.dev_en(), name, "second enable did not set DEV_EN");
    ensure(dcs.dcon(), name, "second enable did not set DCON");
    ensure(
        regs.inten().read().dev_int_en(),
        name,
        "second enable did not restore device interrupts",
    );

    ensure(bus.force_reset().is_ok(), name, "force_reset reported Unsupported");
    ensure(
        regs.devcmdstat().read().dcon(),
        name,
        "force_reset left the device disconnected",
    );
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_nxp::config::Config::default();
    config.main_clock = MainClock::FroHf96;
    let p = embassy_nxp::init(config);

    let fs_regs = unsafe { pac::usbhsd::Usbhsd::from_ptr(pac::USB0.as_ptr()) };
    let mut fs_mem = [0u8; 4096];
    let fs_memory = Memory::buffer(&mut fs_mem);
    let fs_base = fs_memory.base();
    let mut fs_driver = Driver::<peripherals::USB0>::new(p.USB0, Irqs, p.PIO0_22, fs_memory);
    let mut fs_bulk = fs_driver.alloc_endpoint_out(EndpointType::Bulk, None, 64, 0).unwrap();
    let fs_iso = fs_driver
        .alloc_endpoint_out(EndpointType::Isochronous, None, 1023, 1)
        .unwrap();
    let (mut fs_bus, fs_control) = fs_driver.start(64);

    let mut hs_driver = Driver::<peripherals::USBHSD>::new(p.USBHSD, Irqs, Memory::usb1_sram());
    let mut hs_bulk = hs_driver.alloc_endpoint_out(EndpointType::Bulk, None, 512, 0).unwrap();
    let hs_iso = hs_driver
        .alloc_endpoint_out(EndpointType::Isochronous, None, 1023, 1)
        .unwrap();
    let (mut hs_bus, hs_control) = hs_driver.start(64);

    check(&mut fs_bus, &mut fs_bulk, &fs_iso, "USB0", fs_base, fs_regs, || {
        defmt::assert!(pac::SYSCON.ahbclkctrl1().read().usb0_dev());
        defmt::assert_eq!(pac::PMC.pdruncfg0().read().0 & PDEN_USBFSPHY, 0);
        defmt::assert!(embassy_nxp::interrupt::typelevel::USB0::is_enabled());
    })
    .await;
    check(
        &mut hs_bus,
        &mut hs_bulk,
        &hs_iso,
        "USB1",
        USB1_SRAM_ADDR,
        pac::USBHSD,
        || {
            let clk = pac::SYSCON.ahbclkctrl2().read();
            defmt::assert!(clk.usb1_dev());
            defmt::assert!(clk.usb1_phy());
            defmt::assert_eq!(pac::PMC.pdruncfg0().read().0 & PDEN_USBHSPHY, 0);
            defmt::assert!(embassy_nxp::interrupt::typelevel::USB1::is_enabled());
        },
    )
    .await;

    drop(hs_bulk);
    drop(hs_iso);
    drop(hs_control);
    drop(hs_bus);
    let hs_clk = pac::SYSCON.ahbclkctrl2().read();
    defmt::assert!(!hs_clk.usb1_dev(), "USB1 device clock remained on after Bus drop");
    defmt::assert!(!hs_clk.usb1_phy(), "USB1 PHY clock remained on after Bus drop");
    ensure(
        pac::PMC.pdruncfg0().read().0 & PDEN_USBHSPHY != 0,
        "USB1",
        "PHY remained powered after Bus drop",
    );
    ensure(
        !embassy_nxp::interrupt::typelevel::USB1::is_enabled(),
        "USB1",
        "interrupt remained enabled after Bus drop",
    );
    let hs_dcs = pac::USBHSD.devcmdstat().read();
    defmt::assert!(!hs_dcs.dcon() && !hs_dcs.dev_en(), "USB1 remained live after Bus drop");

    ensure(
        !pac::SYSCON.ahbclkctrl2().read().usb1_ram(),
        "USB1",
        "last SRAM lease did not gate the RAM clock",
    );
    defmt::assert_eq!(
        fs_regs.epliststart().read().0,
        fs_base,
        "USB1 teardown corrupted USB0 EPLISTSTART"
    );
    fs_bus.endpoint_set_stalled(fs_bulk.info().addr, true);
    defmt::assert!(fs_bus.endpoint_is_stalled(fs_bulk.info().addr));
    fs_bus.endpoint_set_stalled(fs_bulk.info().addr, false);

    drop(fs_bulk);
    drop(fs_iso);
    drop(fs_control);
    drop(fs_bus);
    defmt::assert!(!pac::SYSCON.ahbclkctrl1().read().usb0_dev());
    defmt::assert!(pac::PMC.pdruncfg0().read().0 & PDEN_USBFSPHY != 0);
    ensure(
        !embassy_nxp::interrupt::typelevel::USB0::is_enabled(),
        "USB0",
        "interrupt remained enabled after Bus drop",
    );
    let fs_dcs = fs_regs.devcmdstat().read();
    defmt::assert!(!fs_dcs.dcon() && !fs_dcs.dev_en(), "USB0 remained live after Bus drop");
    ensure(
        !pac::SYSCON.ahbclkctrl2().read().usb1_ram(),
        "USB0",
        "last USB1 SRAM lease did not gate the RAM clock",
    );

    let reacquired = Memory::usb1_sram();
    defmt::assert!(pac::SYSCON.ahbclkctrl2().read().usb1_ram());
    drop(reacquired);
    defmt::assert!(!pac::SYSCON.ahbclkctrl2().read().usb1_ram());

    fs_mem.fill(0xa5);
    defmt::assert!(fs_mem.iter().all(|byte| *byte == 0xa5));

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
