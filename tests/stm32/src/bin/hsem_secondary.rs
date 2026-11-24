// required-features: stm32h755zi

//! Dual-core boot handshake regression test (issue #7609 follow-up).
//!
//! The secondary core (CM4) must observe the primary core's `init_primary` from
//! across the cores. The RCC enable registers and the HSEM status registers are
//! banked per core on this part, so what matters is:
//!
//! - the CM4 can enable its own HSEM clock (each core gates its own peripherals;
//!   the primary core cannot do it for this core), and
//! - the channel 1 notification level (HSEM ISF bit 0 in this core's status
//!   bank: `get_hsem(1)` is channel 1, stored as index 0) is visible to this
//!   core after `init_primary` unlocked the channel.
//!
//! The CM4 side runs as whichever image the CM4 boots after the post-flash
//! reset: the tiny one embedded at 0x08100000 (BOOT_ADD1), or this image's
//! `cm4_role` (BOOT_ADD0 = 0x08000000 on this board), dispatched on CPUID.
//!
//! Shared SRAM4 words (volatile u32):
//!   +4  status: bit 0 = saw HSEM clock enable, bit 1 = saw notification
//!   +8  done (= 0xD04ED04E when status == 3)
//!   +12 heartbeat: bumped every poll iteration
//!   +16 raw AHB4ENR (0x580244E0) as the CM4 reads it
//!   +20 raw HSEM ISR bank 1 (0x58026518) as the CM4 reads it
//!   +52 count of poll iterations that saw the notification bit
//!   +28 command: primary writes 0xC0DEC0DE to trigger the visibility probe
//!   +32 probe-done (= 0xD04ED04E)
//!   +36..+52 probe slots (see `cm4_role`)
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;

use common::*;
use embassy_executor::Spawner;

teleprobe_meta::timeout!(30);

/// D3 SRAM4: shared with the CM4, always powered in run mode.
const SRAM4: u32 = 0x3800_0000;
/// Magic values, so stale SRAM4 contents from previous runs never pass for
/// real handshake words.
const MAGIC_CMD: u32 = 0xC0DE_C0DE;
const MAGIC_DONE: u32 = 0xD04E_D04E;

core::arch::global_asm!(
    r#"
    .syntax unified
    .thumb
    .section .cm4_image, "ax"

    // Vector table (16 words): the CM4 boots this image if its boot address
    // points here (BOOT_ADD1 = 0x08100000).
    .word 0x38000800        // initial SP (SRAM4, past the shared words)
    .word cm4_reset + 1     // Reset
    .word cm4_hang + 1      // NMI
    .word cm4_hang + 1      // HardFault
    .word cm4_hang + 1      // MemManage
    .word cm4_hang + 1      // BusFault
    .word cm4_hang + 1      // UsageFault
    .word 0
    .word 0
    .word 0
    .word 0
    .word cm4_hang + 1      // SVCall
    .word cm4_hang + 1      // DebugMon
    .word 0
    .word cm4_hang + 1      // PendSV
    .word cm4_hang + 1      // SysTick

cm4_reset:
    ldr   r0, =0x38000000   // shared words in SRAM4
    // Clear the handshake words: stale values from previous runs must never
    // pass for this run's commands/results.
    movs  r1, #0
    str   r1, [r0, #8]
    str   r1, [r0, #12]
    str   r1, [r0, #28]
    str   r1, [r0, #32]
    str   r1, [r0, #52]
    // Enable the HSEM clock in this core's gate register, like the fixed
    // try_init_secondary does: the RCC enable registers are banked per core,
    // so the primary core cannot enable this core's peripherals.
    ldr   r2, =0x580244E0
    ldr   r3, [r2]
    ldr   r1, =0x02000000
    orrs  r3, r1
    str   r3, [r2]
2:
    // heartbeat: prove this loop runs at all.
    ldr   r2, [r0, #12]
    adds  r2, r2, #1
    str   r2, [r0, #12]
    // status bit 0 = RCC AHB4ENR bit 25 (HSEM clock enable).
    ldr   r2, =0x580244E0
    ldr   r2, [r2]
    str   r2, [r0, #16]     // raw AHB4ENR as this core sees it
    lsrs  r1, r2, #25
    ands  r1, r1, #1
    str   r1, [r0, #4]      // publish partial status
    // Like try_init_secondary: do not touch the HSEM registers before its
    // clock is enabled, or the read faults on the bus.
    cmp   r1, #1
    bne   3f
    // status bit 1 = ISF bit 0 of this core's HSEM status bank (0x58026518):
    // get_hsem(1) is channel 1 = index 0. The flag is latched in the status
    // register of every core when the channel is unlocked.
    ldr   r2, =0x58026518
    ldr   r2, [r2]
    str   r2, [r0, #20]     // raw status bank as this core sees it
    ands  r3, r2, #1
    // count how many iterations saw the notification
    ldr   r2, [r0, #52]
    adds  r2, r2, r3
    str   r2, [r0, #52]
    lsls  r3, r3, #1
    orrs  r1, r1, r3
    str   r1, [r0, #4]      // publish current status (0..3)
    cmp   r1, #3
    bne   3f
    ldr   r1, =0xD04ED04E
    str   r1, [r0, #8]      // publish done
3:
    // Wait for the primary's command to run the visibility probe.
    ldr   r2, [r0, #28]
    ldr   r3, =0xC0DEC0DE
    cmp   r2, r3
    bne   2b

    // Probe: this core's view of the HSEM status banks and the RCC gate.
    ldr   r2, =0x58026518
    ldr   r3, [r2]
    str   r3, [r0, #40]
    ldr   r2, =0x58026508
    ldr   r3, [r2]
    str   r3, [r0, #36]
    ldr   r2, =0x5802650C
    ldr   r3, [r2]
    str   r3, [r0, #44]
    ldr   r2, =0x5802651C
    ldr   r3, [r2]
    str   r3, [r0, #48]
    ldr   r2, =0x580244E0
    ldr   r3, [r2]
    str   r3, [r0, #24]
    ldr   r1, =0xD04ED04E
    str   r1, [r0, #32]     // probe done
cm4_hang:
    wfi
    b     cm4_hang

    .ltorg
    "#
);

/// The CM4 role when the CM4 boots *this* image (BOOT_ADD0 = 0x08000000).
fn cm4_role() -> ! {
    unsafe { core::arch::asm!("msr msp, {0}", in(reg) 0x3800_0800u32) };

    let shared = SRAM4 as *mut u32;

    // Clear the handshake words: stale values from previous runs must never
    // pass for this run's commands/results.
    for slot in [2usize, 3, 7, 8, 13] {
        unsafe { shared.add(slot).write_volatile(0) };
    }

    // Enable the HSEM clock in this core's gate register, like the fixed
    // try_init_secondary does: the RCC enable registers are banked per core,
    // so the primary core cannot enable this core's peripherals.
    unsafe {
        let p = 0x5802_44E0u32 as *mut u32;
        p.write_volatile(p.read_volatile() | (1 << 25));
    }

    loop {
        unsafe {
            shared
                .add(3)
                .write_volatile(shared.add(3).read_volatile().wrapping_add(1));

            // status bit 0 = HSEM clock enable in this core's gate register.
            let ahb4enr = (0x5802_44E0u32 as *const u32).read_volatile();
            shared.add(4).write_volatile(ahb4enr);
            let is_enabled = (ahb4enr >> 25) & 1;
            shared.add(1).write_volatile(is_enabled);
            if is_enabled == 1 {
                // status bit 1 = ISF bit 0 of this core's HSEM status bank:
                // get_hsem(1) is channel 1 = index 0.
                let isr = (0x5802_6518u32 as *const u32).read_volatile();
                shared.add(5).write_volatile(isr);
                let seen = isr & 1;
                shared
                    .add(13)
                    .write_volatile(shared.add(13).read_volatile().wrapping_add(seen));
                let status = is_enabled | (seen << 1);
                shared.add(1).write_volatile(status);
                if status == 3 {
                    shared.add(2).write_volatile(MAGIC_DONE);
                }
            }

            if shared.add(7).read_volatile() == MAGIC_CMD {
                break;
            }
        }
    }

    // Probe: this core's view of the HSEM status banks and the RCC gate.
    unsafe {
        shared
            .add(10)
            .write_volatile((0x5802_6518u32 as *const u32).read_volatile()); // this core's bank
        shared
            .add(9)
            .write_volatile((0x5802_6508u32 as *const u32).read_volatile()); // other bank view
        shared
            .add(11)
            .write_volatile((0x5802_650Cu32 as *const u32).read_volatile()); // masked, this bank
        shared
            .add(12)
            .write_volatile((0x5802_651Cu32 as *const u32).read_volatile()); // masked, other bank
        shared
            .add(6)
            .write_volatile((0x5802_44E0u32 as *const u32).read_volatile()); // RCC gate
        shared.add(8).write_volatile(MAGIC_DONE);
    }

    loop {
        unsafe { core::arch::asm!("wfi") };
    }
}

#[embassy_executor::main(executor = "common::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    // If this is the secondary core (the CM4 boots this image on this board),
    // take the CM4 role and never come back. The CPUID part numbers are
    // 0xC24 (Cortex-M4) and 0xC27 (Cortex-M7).
    let cpuid = unsafe { (0xE000_ED00u32 as *const u32).read_volatile() };
    if (cpuid >> 4) & 0xfff == 0xc24 {
        cm4_role();
    }

    // Give the secondary core time to finish cortex-m-rt startup (which both
    // cores run on the same image) and reach its own stack before this core
    // mutates any shared state in init().
    for _ in 0..1_000_000 {
        cortex_m::asm::nop();
    }

    let _p: embassy_stm32::Peripherals = init();

    // `init()` ran `init_primary` on this (primary) core: it enabled this
    // core's HSEM clock gate and sent the channel 1 notification level. Wait
    // for the CM4 to observe both effects, like init_secondary's loop does.
    let shared = SRAM4 as *const u32;
    let mut status = 0u32;
    let mut done = false;
    for _ in 0..20_000_000 {
        unsafe {
            status = shared.add(1).read_volatile();
            done = shared.add(2).read_volatile() == MAGIC_DONE;
        }
        if done {
            break;
        }
        cortex_m::asm::nop();
    }

    // Heartbeat double sample BEFORE triggering the probe (the role stops
    // polling once the probe runs): the difference proves the role is alive.
    let hb1 = unsafe { shared.add(3).read_volatile() };
    for _ in 0..1_000_000 {
        cortex_m::asm::nop();
    }
    let hb2 = unsafe { shared.add(3).read_volatile() };

    // Trigger the visibility probe on the CM4 and wait for it.
    unsafe { (shared as *mut u32).add(7).write_volatile(MAGIC_CMD) };
    let mut probe_done = false;
    for _ in 0..20_000_000 {
        unsafe {
            probe_done = shared.add(8).read_volatile() == MAGIC_DONE;
        }
        if probe_done {
            break;
        }
        cortex_m::asm::nop();
    }

    let (ahb4enr, isr0, isr1) = unsafe {
        (
            (0x5802_44E0u32 as *const u32).read_volatile(),
            (0x5802_6508u32 as *const u32).read_volatile(),
            (0x5802_6518u32 as *const u32).read_volatile(),
        )
    };
    let (cm4_raw, cm4_isr, seen) = unsafe {
        (
            shared.add(4).read_volatile(),
            shared.add(5).read_volatile(),
            shared.add(13).read_volatile(),
        )
    };
    let (p_own, p_other, p_mine, p_moth, p_gate) = unsafe {
        (
            shared.add(10).read_volatile(),
            shared.add(9).read_volatile(),
            shared.add(11).read_volatile(),
            shared.add(12).read_volatile(),
            shared.add(6).read_volatile(),
        )
    };

    info!(
        "cm4 status {=u32:#b} done={=bool} heartbeat {=u32}->{=u32} seen={=u32} probe_done={=bool}",
        status, done, hb1, hb2, seen, probe_done
    );
    info!("cm4 polls: gate={=u32:#x} isr-bank={=u32:#x}", cm4_raw, cm4_isr);
    info!(
        "cm4 probe: isr(0x118)={=u32:#x} isr(0x108)={=u32:#x} misr(0x10c)={=u32:#x} misr(0x11c)={=u32:#x} gate={=u32:#x}",
        p_own, p_other, p_mine, p_moth, p_gate
    );
    info!(
        "primary: gate={=u32:#x} isr0={=u32:#x} isr1={=u32:#x}",
        ahb4enr, isr0, isr1
    );

    defmt::assert!(hb2 != hb1, "the CM4 role is not running");
    defmt::assert!(done, "the CM4 core never observed the primary core's init");
    defmt::assert!(status & 1 == 1, "the CM4 core did not see its HSEM clock enabled");
    defmt::assert!(status & 2 == 2, "the CM4 core did not see the notification level");

    info!("Test OK");
    cortex_m::asm::bkpt();
}
