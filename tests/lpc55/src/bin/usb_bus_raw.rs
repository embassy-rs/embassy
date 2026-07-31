//! Raw [`embassy_usb_driver::Bus`] lifecycle for both LPC55 USB controllers.
//!
//! This covers the `Bus` methods `embassy-usb` never calls, so nothing else in
//! the suite can reach them: `force_reset` has no caller anywhere in
//! `embassy-usb`, and `disable` followed by a second `enable` (the `reinit`
//! path, which re-programs the controller after `power_down` reset the block)
//! is only reachable through `UsbDevice::disable`, which none of the examples
//! or tests use.
//!
//! Needs no host cable: the assertions are all on device-side registers and
//! command-list state, and the bus never comes up, so no host traffic is
//! involved. Soft-connect is asserted as a register bit, not as enumeration.
//!
//! The per-controller body is one generic function. That works because the
//! driver itself views USB0 through the `usbhsd` register map (see
//! `SealedInstance for USB0` in `embassy-nxp/src/usb/lpc55.rs`), so the two
//! controllers share a register type; only the clock-gate and PDRUNCFG
//! assertions differ, and those come in as a closure.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::config::MainClock;
use embassy_nxp::usb::{Driver, Instance, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, pac, peripherals};
use embassy_usb::driver::{Bus as _, Driver as _, Endpoint as _, EndpointType};
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

/// Drive one controller through the full `Bus` lifecycle.
///
/// `regs` is the ip3511 register block for this instance, `ep_list_base` the
/// endpoint-list address the driver was handed, and `assert_powered_down` the
/// instance-specific clock-gate/PDRUNCFG check run after `disable`.
async fn check<T: Instance>(
    mut driver: Driver<'_, T>,
    name: &'static str,
    mps: u16,
    ep_list_base: u32,
    regs: pac::usbhsd::Usbhsd,
    assert_powered_down: impl Fn(),
) {
    let bulk = driver.alloc_endpoint_out(EndpointType::Bulk, None, mps, 0).unwrap();
    let iso = driver
        .alloc_endpoint_out(EndpointType::Isochronous, None, 1023, 1)
        .unwrap();
    let bulk_addr = bulk.info().addr;
    let iso_addr = iso.info().addr;

    let (mut bus, _control) = driver.start(64);

    // Soft-connect and device-enable, the two bits `enable` owns.
    bus.enable().await;
    let dcs = regs.devcmdstat().read();
    defmt::assert!(dcs.dcon(), "{}: enable did not set DCON", name);
    defmt::assert!(dcs.dev_en(), "{}: enable did not set DEV_EN", name);

    // Stall round-trip on a bulk endpoint: CMD_S set, observed, then cleared.
    bus.endpoint_set_enabled(bulk_addr, true);
    bus.endpoint_set_stalled(bulk_addr, true);
    defmt::assert!(
        bus.endpoint_is_stalled(bulk_addr),
        "{}: bulk endpoint not reported stalled after set_stalled(true)",
        name
    );
    bus.endpoint_set_stalled(bulk_addr, false);
    defmt::assert!(
        !bus.endpoint_is_stalled(bulk_addr),
        "{}: bulk endpoint still reported stalled after set_stalled(false)",
        name
    );

    // Isochronous transfers have no handshake phase, so an iso endpoint can
    // never STALL; the driver deliberately never sets CMD_S on one, and
    // `endpoint_is_stalled` therefore always reports `false` for it.
    bus.endpoint_set_enabled(iso_addr, true);
    bus.endpoint_set_stalled(iso_addr, true);
    defmt::assert!(
        !bus.endpoint_is_stalled(iso_addr),
        "{}: isochronous endpoint reported stalled, but iso has no handshake phase",
        name
    );

    // Power the block down and check the gates the driver claims to close.
    bus.disable().await;
    assert_powered_down();

    // Second `enable`: `powered` is false, so this takes the `power_up` +
    // `reinit` path, which must re-program the controller from scratch.
    bus.enable().await;
    defmt::assert_eq!(
        regs.epliststart().read().0,
        ep_list_base,
        "{}: reinit did not restore EPLISTSTART",
        name
    );
    let dcs = regs.devcmdstat().read();
    defmt::assert!(dcs.dev_en(), "{}: reinit did not set DEV_EN", name);
    defmt::assert!(dcs.dcon(), "{}: enable after reinit did not set DCON", name);
    defmt::assert!(
        regs.inten().read().dev_int_en(),
        "{}: reinit did not re-enable the device interrupt",
        name
    );

    // `force_reset` drops DCON for a host-visible SE0 and busy-waits
    // `cortex_m::asm::delay(10_000_000)` (~100 ms at 96 MHz) before raising it
    // again, so the test visibly pauses here.
    defmt::assert!(bus.force_reset().is_ok(), "{}: force_reset reported Unsupported", name);
    defmt::assert!(
        regs.devcmdstat().read().dcon(),
        "{}: force_reset left the device disconnected",
        name
    );

    // Leave the controller powered down and disconnected. This test asserts
    // DCON but never runs a `UsbDevice`, so a controller left connected is a
    // device that pulls D+ up and then answers nothing: the host resets and
    // retries the port for as long as the core sits at the breakpoint, which
    // is enough to stop the next test's debug session from attaching.
    bus.disable().await;
    assert_powered_down();
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_nxp::config::Config::default();
    // The high-speed controller requires a system clock of at least 96 MHz.
    config.main_clock = MainClock::FroHf96;
    let p = embassy_nxp::init(config);

    // ---------------------------------------------------------------- USB1 HS
    let hs = Driver::<peripherals::USBHSD>::new(p.USBHSD, Irqs, Memory::usb1_sram());
    check(hs, "USB1", 512, USB1_SRAM_ADDR, pac::USBHSD, || {
        let clk = pac::SYSCON.ahbclkctrl2().read();
        defmt::assert!(!clk.usb1_dev(), "USB1: disable left the device clock on");
        defmt::assert!(!clk.usb1_phy(), "USB1: disable left the PHY clock on");
        defmt::assert!(
            pac::PMC.pdruncfg0().read().0 & PDEN_USBHSPHY != 0,
            "USB1: disable left the HS PHY powered"
        );
    })
    .await;

    // ---------------------------------------------------------------- USB0 FS
    // `Memory::usb1_sram` is already taken by the high-speed half of this test
    // and is never released, so USB0 gets a region in main SRAM.
    let mut ep_mem = [0u8; 4096];
    let mem = Memory::buffer(&mut ep_mem);
    let fs_base = mem.base();
    let fs = Driver::<peripherals::USB0>::new(p.USB0, Irqs, p.PIO0_22, mem);
    // Same ip3511 register map at the USB0 base; this is exactly how the driver
    // reaches USB0's registers.
    let fs_regs = unsafe { pac::usbhsd::Usbhsd::from_ptr(pac::USB0.as_ptr()) };
    check(fs, "USB0", 64, fs_base, fs_regs, || {
        defmt::assert!(
            !pac::SYSCON.ahbclkctrl1().read().usb0_dev(),
            "USB0: disable left the device clock on"
        );
        defmt::assert!(
            pac::PMC.pdruncfg0().read().0 & PDEN_USBFSPHY != 0,
            "USB0: disable left the FS PHY powered"
        );
    })
    .await;

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
