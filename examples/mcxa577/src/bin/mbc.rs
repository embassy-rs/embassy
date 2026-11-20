#![no_std]
#![no_main]

//! Give a dedicated flash-data region an explicit MBC read/write policy.
//!
//! `mbc-data.x` reserves one 16 KiB MBC block for SETTINGS, separate from code,
//! constants and flash load images. No flash programming occurs at runtime.
//! Slot5 must not already serve IFR or other main-flash blocks: preflight rejects
//! such a change. The A5 manual has conflicting execute/NSE descriptions; this
//! example teaches register configuration, not verified execution isolation.

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::{Duration, Timer};
use hal::glikey::Glikey;
use hal::mbc::{Mbc, Permission, PermissionSet, Policy, PolicySlot, RegionConfig, SlotConfig, WritableSlot};
use panic_probe as _;

const DATA_SIZE: usize = 16 * 1024;

// Whole-block allocation keeps unrelated memory out of the affected region.
#[repr(C, align(16384))]
struct FlashSettings([u8; DATA_SIZE]);

#[used]
#[unsafe(link_section = ".mbc_data")]
static SETTINGS: FlashSettings = FlashSettings([0x42; DATA_SIZE]);

// Const policies use from_array; runtime code can also use
// `Permission::Read | Permission::Write`. This configures access, not flash data.
const SLOTS: &[SlotConfig] = &[SlotConfig {
    slot: WritableSlot::Slot5,
    permissions: PermissionSet::from_array([Permission::Read, Permission::Write]),
}];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());
    let mut glikey = Glikey::new(p.GLIKEY0).expect("GLIKEY setup failed");

    // Main flash starts at zero on this board. Use actual linker placement as
    // the main-flash-relative offset rather than choosing an arbitrary address.
    let start = core::ptr::addr_of!(SETTINGS) as u32;
    let regions = [RegionConfig {
        range: start..start + DATA_SIZE as u32,
        slot: WritableSlot::Slot5,
    }];
    let policy = Policy {
        slots: SLOTS,
        regions: &regions,
    };

    // SAFETY: the linker isolates SETTINGS in a whole block, and its reads stay
    // allowed. Code, vectors, constants, policy slices and recovery resources are
    // elsewhere. Preflight rejects existing users of the changed slot. There
    // are no competing MBC register writers, including other bus masters.
    let mbc = unsafe { Mbc::new_with_policy(p.MBC0, &mut glikey, &policy) }.expect("MBC policy application failed");

    let geometry = mbc.geometry();
    defmt::info!(
        "Main flash: {} bytes, {}-byte MBC blocks",
        geometry.flash_size,
        geometry.block_size
    );
    defmt::info!("Settings at {=u32:#x}: {:?}", start, mbc.policy(PolicySlot::Slot5));

    // The policy and GLIKEY borrows are not retained by MBC. GLIKEY's grant was
    // explicitly closed, so the same controller can serve another peripheral.
    // Later changes use unsafe mbc.apply_policy(&mut glikey, &new_policy).
    loop {
        // SAFETY: initialized flash data remains readable after this policy.
        let revision = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(SETTINGS).cast::<u8>()) };
        defmt::info!("Settings revision: {=u8:#x}", revision);
        Timer::after(Duration::from_secs(5)).await;
    }
}
