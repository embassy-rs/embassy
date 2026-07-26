#![no_std]

use core::slice;

use cortex_m::Peripherals;
use cortex_m::peripheral::MPU;
use defmt::info;
use embassy_stm32::dsihost::panel::DsiPanel;
use embassy_stm32::fmc::Fmc;
use embassy_stm32::peripherals::FMC;
use embassy_time::Delay;
use embedded_alloc::LlffHeap as Heap;
use static_cell::StaticCell;
use stm32_fmc::Sdram;
use stm32_fmc::devices::is42s32800g_6::Is42s32800g;

use crate::framebuffer::Framebuffer;
use crate::glass::Glass;
use crate::mpu::{ATTR_WRITE_BACK, ATTR_WRITE_THROUGH, mpu_region};

extern crate alloc;

pub mod framebuffer;
pub mod glass;
mod mpu;
pub mod ui;

const SDRAM_BASE: usize = 0xD000_0000;
const SDRAM_SIZE: usize = 32 * 1024 * 1024;

// 16MB graphics region, configured in MPU as write through
const GFX_START: usize = SDRAM_BASE;
const GFX_SIZE: usize = 16 * 1024 * 1024;

// 16MB heap region, configured in MPU as write back
const HEAP_START: usize = SDRAM_BASE + GFX_SIZE;
const HEAP_SIZE: usize = 16 * 1024 * 1024;

const FB_PIXELS: usize = Glass::ACTIVE_WIDTH as usize * Glass::ACTIVE_HEIGHT as usize;
const FB_BYTES: usize = FB_PIXELS * core::mem::size_of::<u32>();

const FB0_START: usize = GFX_START;
const FB1_START: usize = align_up(FB0_START + FB_BYTES, 64);

// Remaining graphics memory can be a texture arena (DMA2D usage)
/*
const TEX_START: usize = align_up(FB1_START + FB_BYTES, 64);
const TEX_SIZE: usize = GFX_START + GFX_SIZE - TEX_START;
*/

const MPU_REGION_GFX: u32 = 0;
const MPU_REGION_HEAP: u32 = 1;

const fn align_up(x: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (x + (align - 1)) & !(align - 1)
}

static SDRAM: StaticCell<&mut [u32]> = StaticCell::new();
pub static FB0: StaticCell<Framebuffer> = StaticCell::new();
pub static FB1: StaticCell<Framebuffer> = StaticCell::new();

#[global_allocator]
pub static HEAP: Heap = Heap::empty();

pub struct Buffers {
    pub fb0: &'static mut Framebuffer,
    pub fb1: &'static mut Framebuffer,
}

/// Initialize SDRAM
/// * Splits into graphics and heap regions
/// * Configures MPU for write through on graphics region, and write back on heap region
/// * Allocates two framebuffers in graphics region
///
/// Returns [`Buffers`] with static mut references to [`FrameBuf`]
pub fn init_sdram(mut cp: Peripherals, mut sdram: Sdram<Fmc<'_, FMC>, Is42s32800g>) -> Buffers {
    let base = SDRAM.init_with(|| unsafe {
        let ram_ptr: *mut u32 = sdram.init(&mut Delay) as *mut _;
        slice::from_raw_parts_mut(ram_ptr, SDRAM_SIZE / core::mem::size_of::<u32>())
    });

    let sdram_base = base.as_mut_ptr() as usize;
    info!("SDRAM mapped to {:08x}", sdram_base);

    assert_eq!(sdram_base, SDRAM_BASE, "unexpected SDRAM base");
    assert!(HEAP_START <= SDRAM_BASE + SDRAM_SIZE, "SDRAM too small");

    let fb0 = FB0.init_with(|| unsafe { Framebuffer::new(FB0_START as *mut u32, FB_PIXELS) });
    let fb1 = FB1.init_with(|| unsafe { Framebuffer::new(FB1_START as *mut u32, FB_PIXELS) });

    unsafe {
        HEAP.init(HEAP_START, HEAP_SIZE);
    }

    info!(
        "GFX  : {:08x}..{:08x} ({} MiB)",
        GFX_START,
        GFX_START + GFX_SIZE,
        GFX_SIZE / 1024 / 1024
    );
    info!(
        "Heap : {:08x}..{:08x} ({} MiB)",
        HEAP_START,
        HEAP_START + HEAP_SIZE,
        HEAP_SIZE / 1024 / 1024
    );

    info!("FB0  : {:08x} {} bytes", FB0_START, FB_BYTES);
    info!("FB1  : {:08x} {} bytes", FB1_START, FB_BYTES);

    cp.SCB.disable_icache();
    cp.SCB.disable_dcache(&mut cp.CPUID);

    sdram_cache_init(&mut cp.MPU);

    cp.SCB.enable_icache();
    cp.SCB.enable_dcache(&mut cp.CPUID);

    cortex_m::asm::dsb();
    cortex_m::asm::isb();

    Buffers { fb0, fb1 }
}

pub fn sdram_cache_init(mpu: &mut MPU) {
    unsafe {
        mpu.ctrl.write(0);

        mpu_region(mpu, MPU_REGION_GFX, GFX_START, GFX_SIZE, ATTR_WRITE_THROUGH);
        mpu_region(mpu, MPU_REGION_HEAP, HEAP_START, HEAP_SIZE, ATTR_WRITE_BACK);

        const MPU_ENABLE: u32 = 0x01;
        const MPU_DEFAULT_MMAP_FOR_PRIVILEGED: u32 = 0x04;

        mpu.ctrl.write(MPU_DEFAULT_MMAP_FOR_PRIVILEGED | MPU_ENABLE);
    }
}
