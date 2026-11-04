#![no_std]
#![no_main]

//! Give a dedicated flash-data region an explicit MBC read/write policy.
//!
//! `mbc-data.x` reserves whole MBC blocks for SETTINGS, separate from code,
//! constants and flash load images. No flash programming occurs at runtime.
//! On A2 the 16 KiB region occupies two 8 KiB MBC blocks. Slot5 must not already
//! serve IFR or other main-flash blocks: the driver rejects such a change.

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::{Duration, Timer};
use hal::glikey::Glikey;
use hal::mbc::{Mbc, Permission, PermissionSet, Policy, PolicySlot, RegionConfig, SlotConfig, WritableSlot};
use panic_probe as _;

const DATA_SIZE: usize = 16 * 1024;

// A whole, aligned region prevents a permission change from sharing a block
// with code, vectors, unrelated constants, or .data's flash initialization image.
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

    // Construct GLIKEY explicitly; it can later serve other protected peripherals.
    let mut glikey = Glikey::new(p.GLIKEY0).expect("GLIKEY setup failed");

    // Main flash starts at zero on this board, so this address is also its byte
    // offset. Obtain the actual linker placement, rather than guessing an offset.
    let start = core::ptr::addr_of!(SETTINGS) as u32;
    let regions = [RegionConfig {
        range: start..start + DATA_SIZE as u32,
        slot: WritableSlot::Slot5,
    }];
    let policy = Policy {
        slots: SLOTS,
        regions: &regions,
    };

    // SAFETY: the linker isolates SETTINGS in whole blocks, and its reads remain
    // allowed throughout. No code, vectors, policy slices or recovery resources
    // are in this region. Preflight rejects existing users of the changed slot.
    // There are no concurrent MBC writers or bus masters modifying its registers.
    let mbc = unsafe { Mbc::new_with_policy(p.MBC0, &mut glikey, &policy) }.expect("MBC policy application failed");

    let geometry = mbc.geometry();
    defmt::info!(
        "Main flash: {} bytes, {}-byte MBC blocks",
        geometry.flash_size,
        geometry.block_size
    );
    defmt::info!("Settings at {=u32:#x}: {:?}", start, mbc.policy(PolicySlot::Slot5));

    // GLIKEY's mutable borrow has ended and its grant was explicitly closed.
    // Changing this policy later uses unsafe mbc.apply_policy(&mut glikey, ...),
    // with the same intermediate-state safety obligations. Nothing is locked.
    loop {
        // SAFETY: SETTINGS is initialized flash data and its read access remains
        // enabled. Volatile demonstrates a real access instead of constant folding.
        let revision = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(SETTINGS).cast::<u8>()) };
        defmt::info!("Settings revision: {=u8:#x}", revision);
        Timer::after(Duration::from_secs(5)).await;
    }
}
