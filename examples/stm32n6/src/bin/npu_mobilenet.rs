//! STM32N6 Neural-ART (ATON) NPU bring-up with a **real** ST Edge AI network.
//!
//! This example runs genuine ATON microcode produced by ST Edge AI for the
//! public `STM32N6-GettingStarted-ImageClassification` project (MobileNetV1-0.25,
//! 96x96x3 uint8 input, 5 int8 logits). The single hardware epoch runs entirely
//! on the NPU; the CPU only writes the input tensor, runs the network's trailing
//! *software* epochs, and reads back the logits.
//!
//! # Model assets
//!
//! ST publishes the network (blob + encoded weights) under SLA0044, so it is not
//! vendored here. `build.rs` imports it from a checkout of the public repo:
//!
//! ```text
//! git clone --depth 1 https://github.com/STMicroelectronics/STM32N6-GettingStarted-ImageClassification
//! STM32N6_GETTINGSTARTED_MODEL_DIR=$PWD/STM32N6-GettingStarted-ImageClassification/Model/NUCLEO-N657X0-Q \
//!     cargo build --features npu-model --bin npu_mobilenet
//! ```
//!
//! # What the compiled model expects (fixed, absolute addresses)
//!
//! The network was compiled with `--enable-virtual-mem-pools` in "absolute mode",
//! so every buffer lives at a hard-coded address and the EC binary carries no
//! relocation table (`LL_ATON_EC_Network_Init_network()` is a no-op). From the
//! generated `network.c`:
//!
//! * input  `Input_0_out_0`      -> `0x342E_0000`, 96*96*3 uint8
//! * logits `Gemm_130` output    -> `0x342E_03F0`, 5 int8   (Softmax input)
//! * weights                     -> `0x7038_0000` in external xSPI2 flash (207.7 KB)
//! * activations                 -> AXISRAM3..6 (`0x3420_0000`..), powered by RAMCFG
//!
//! # Prerequisites
//!
//! 1. `network_data.xSPI2.bin` (from the same ST repo) must be programmed at
//!    absolute flash address `0x7038_0000`, exactly as ST's own README describes
//!    (`network_data.hex` + STM32CubeProgrammer). The example verifies this at
//!    run time and bails out with a clear message otherwise.
//! 2. The board must be in development mode so this image is loaded to RAM.
//!
//! # Where `embedded-nn` fits
//!
//! ST's compiled network is not purely hardware: it ends with two *software*
//! epoch blocks (Softmax, then Dequantize) that the Cortex-M55 executes once the
//! NPU epoch completes. This example reproduces them with `embedded-nn` kernels,
//! exposed by the driver as `embassy_stm32::npu::epoch`
//! ([`SoftmaxEpoch`] -> `embedded_nn::softmax::softmax_s8`, [`DequantizeEpoch`]
//! -> `embedded_nn::support::dequantize_s8_to_f32`, [`ArgMaxEpoch`]), so the CPU
//! half of the pipeline is genuinely `embedded-nn` rather than hand-rolled.
//!
//! The input is a fixed, public-domain Mexican marigold (see `npu-model/`),
//! embedded as raw `RGB8`. A flower is used deliberately: the model is a
//! 101-way flower classifier (`STAI_NETWORK_OUT_1_SIZE` in `stai_network.h`),
//! so an in-distribution image is required for the logits to carry any signal.
//!
//! A hardware-independent cross-check hook exists ([`GOLDEN_LOGITS`]) but is
//! currently unpopulated: see its doc comment for why the reference this
//! example originally shipped doesn't apply to the model obtained by
//! following this crate's own setup instructions.
//!
//! # Boards
//!
//! Select with `--features board-dk` (default) or `--features board-nucleo`.
//! Both boards route xSPI2 to the same XSPIM Port 2 pins; they differ in the
//! power supply topology and the size of the on-board Macronix NOR flash.

#![no_std]
#![no_main]

use aligned::{A8, Aligned};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::mode::Blocking;
use embassy_stm32::npu::epoch::{ArgMaxEpoch, DequantizeEpoch, SoftmaxEpoch, SoftwareKernel};
use embassy_stm32::npu::{self, cache};
use embassy_stm32::rcc::{
    CpuClk, IcConfig, Icint, Icsel, Pll, Plldivm, Pllpdiv, Pllsel, SupplyConfig, SysClk, XspiClkSrc,
};
use embassy_stm32::rif::{RifMaster, RifMasterAttributes, RifPeripheral, RifPeripheralAttributes};
use embassy_stm32::xspi::{
    AddressSize, ChipSelectHighTime, DummyCycles, FIFOThresholdLevel, MemorySize, MemoryType, TransferConfig, WrapSize,
    Xspi, XspiWidth,
};
use embassy_stm32::{bind_interrupts, pac, peripherals};
use panic_probe as _;

// ── Board selection ─────────────────────────────────────────────────────────

#[cfg(all(feature = "board-dk", feature = "board-nucleo"))]
compile_error!("select exactly one of `board-dk` / `board-nucleo`");

#[cfg(not(any(feature = "board-dk", feature = "board-nucleo")))]
compile_error!("select one of `board-dk` / `board-nucleo`");

// ST's own board bring-up: the DK is powered from the external SMPS, the Nucleo
// from the internal one; the DK board wires a 128 MiB part on xSPI2, the Nucleo
// a 64 MiB one. See Application/<board>/Src/main.c in the GettingStarted repo.
#[cfg(feature = "board-dk")]
const SUPPLY: SupplyConfig = SupplyConfig::External;
#[cfg(feature = "board-nucleo")]
const SUPPLY: SupplyConfig = SupplyConfig::Smps;

#[cfg(feature = "board-dk")]
const NOR_SIZE: MemorySize = MemorySize::_128MiB;
#[cfg(feature = "board-nucleo")]
const NOR_SIZE: MemorySize = MemorySize::_64MiB;

// ── Network geometry (from the public generated `network.c`) ────────────────

/// Input tensor address inside AXISRAM5 (NPURAM5).
const INPUT_ADDR: u32 = 0x342E_0000;
/// `1 x 96 x 96 x 3` uint8.
const INPUT_LEN: usize = 96 * 96 * 3;
/// int8 logits produced by the (single) hardware epoch.
const LOGITS_ADDR: u32 = 0x342E_03F0;
/// `STAI_NETWORK_OUT_1_SIZE` in `stai_network.h`: this is a 101-way flower
/// classifier (likely Oxford 102 Flowers minus one unused class), not the
/// 5-class model this example's golden reference was computed against — see
/// [`GOLDEN_LOGITS`].
const NUM_CLASSES: usize = 101;
/// Where `network_data.xSPI2.bin` must have been programmed.
const WEIGHTS_ADDR: u32 = 0x7038_0000;

// Parameters of ST's trailing software epoch blocks, verbatim from the generated
// `network.c` (`Softmax_835` / `Dequantize_837`; `softmax_integer1_sw_info` /
// `dequantizelinear2_sw_info`).
/// TFLite fixed-point softmax multiplier / shift / clamp (`sw_info`).
const SOFTMAX_MULT: i32 = 1_378_335_488;
const SOFTMAX_LEFT_SHIFT: i32 = 23;
const SOFTMAX_DIFF_MIN: i32 = -248;
/// Byte offsets, inside `network_data.xSPI2.bin`, of the Dequantize f32 scale
/// and int32 zero-point (`Dequantize_837.is` / `.izp`).
const DEQ_SCALE_OFFSET: u32 = 8_283_360;
const DEQ_ZP_OFFSET: u32 = 8_283_408;

/// Fixed network input: a public-domain Mexican marigold (`Tagetes`), raw `RGB8`
/// 96x96x3, byte-identical to the image the `embedded-nn` reference is computed
/// from. See `npu-model/README.md` for provenance.
///
/// A flower is deliberately chosen because this model is a (101-way) flower
/// classifier; an out-of-distribution image such as the Mona Lisa produces
/// near-uniform logits and makes a useless regression vector.
static INPUT_IMAGE: &[u8] = include_bytes!("../../npu-model/mexican_marigold_96x96.rgb");

/// Expected pre-softmax logits for `INPUT_IMAGE`, from `embedded-nn`'s host
/// interpreter — `None` here, deliberately.
///
/// ST's `STM32N6-GettingStarted-ImageClassification` repo ships two
/// different, unversioned models under the same demo: `Model/
/// mobilenet_v1_0.25_96_tfs_int8.tflite` (confirmed via `embedded-nn-tflite`:
/// input `[1,96,96,3]` uint8, output `[1,5]` float32 — a genuine 5-class
/// model, matching this slot's original reference `[-37, -27, -3, 8, 0]`
/// and the "daisy/dandelion/rose/sunflower/tulip" README description) and
/// the actually-compiled `Model/STM32N6570-DK/network_ecblobs.h` +
/// `network_data.xSPI2.bin` that `build.rs` imports and this example runs
/// on the NPU, which is a **101**-class model (`STAI_NETWORK_OUT_1_SIZE` in
/// `stai_network.h`). They are not the same network, so the 5-value
/// reference cannot validate what actually runs here; comparing them was
/// producing meaningless golden-check failures, not a sign of a NPU/driver
/// bug. No 101-class `.tflite` is published in that repo to regenerate a
/// real reference from.
const GOLDEN_LOGITS: Option<[i8; NUM_CLASSES]> = None;

/// Allowed per-element error against `GOLDEN_LOGITS`, once one exists again.
///
/// The NPU and `embedded-nn` are independent implementations: ATON's
/// convolution accumulation order and rounding differ from the interpreter's,
/// so the result is not bit-exact by construction. `embedded-nn` vs. the
/// official TFLite runtime measured <= 1 LSB across every input tested; 2
/// leaves a unit of headroom for ATON's own ordering.
const GOLDEN_TOLERANCE: i32 = 2;

/// xSPI2 memory-mapped window base.
const XSPI2_MM_BASE: u32 = 0x7000_0000;

/// Raw ATON blob, decoded from `_ec_blob_network_1[]` (48 320 bytes).
///
/// Real, non-eliminable content (~300 KB) that must live outside the
/// boot-critical FLASH region — see memory.x / npu_link.x. `#[link_section]`
/// on a `&[u8]` binding only relocates the 8-byte fat pointer, not the bytes
/// it points to, so this goes through an explicitly `[u8; N]`-typed static
/// (with `N` computed by evaluating the same `include_bytes!` a second time,
/// which the compiler resolves to the same content) instead.
/// Length of the imported blob, shared with [`BLOB_STORAGE_LEN`] so the two
/// buffers always match exactly (AXISRAM1, where both live, is only 624 KiB
/// — see memory.x — leaving no room for a rounded-up guess on top of a full
/// second copy).
const EC_BLOB_LEN: usize = include_bytes!(concat!(env!("OUT_DIR"), "/npu/ec_blob.bin")).len();
#[unsafe(link_section = ".npu_blob")]
static EC_BLOB_ARRAY: [u8; EC_BLOB_LEN] = *include_bytes!(concat!(env!("OUT_DIR"), "/npu/ec_blob.bin"));
static EC_BLOB: &[u8] = &EC_BLOB_ARRAY;
/// Encoded network parameters, only used to verify the external flash content.
static WEIGHTS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/npu/weights.bin"));

#[cfg(not(npu_model_assets))]
compile_error!(
    "no NPU model imported: set STM32N6_GETTINGSTARTED_MODEL_DIR to the `Model/NUCLEO-N657X0-Q` \
     directory of https://github.com/STMicroelectronics/STM32N6-GettingStarted-ImageClassification \
     (see build.rs)"
);

bind_interrupts!(struct Irqs {
    NPU0 => npu::InterruptHandler<peripherals::NPU>;
});

/// Writable, 8-byte aligned copy of the blob. The epoch controller reads a blob
/// once per epoch, so a single static buffer, filled once in `main()`, is enough.
///
/// Sized to exactly match [`EC_BLOB_LEN`] (see there for why); the original
/// 64 KiB guess was too small for a real ST Edge AI network and always
/// failed the length check below.
const BLOB_STORAGE_LEN: usize = EC_BLOB_LEN;
/// Real content once written (~300 KB): same NPUMEM placement as EC_BLOB_ARRAY,
/// and for the same reason — see memory.x / npu_link.x. `.npu_uninit` is
/// NOLOAD, so unlike ordinary `.bss` it is *not* zeroed by the Reset handler
/// before AXISRAM3 is clocked on — `main()` overwrites the whole used prefix
/// itself via `copy_from_slice` below, so the initial content never matters.
///
/// This can't be a `StaticCell`: its `used` guard flag needs a real zero at
/// boot to read as "empty", which NOLOAD doesn't give it (its `false`
/// compile-time initializer is never actually written to memory), so the
/// very first `.init()` call would find `used` full of NOLOAD garbage and
/// panic as "already full". A plain `static mut`, touched exactly once in
/// `main()`, has no such guard to desync.
#[unsafe(link_section = ".npu_uninit")]
static mut BLOB_STORAGE: Aligned<A8, [u8; BLOB_STORAGE_LEN]> = Aligned([0u8; BLOB_STORAGE_LEN]);

/// RAMCFG instance base for AXISRAM3..6 (non-secure alias).
const RAMCFG_AXI_BASE: u32 = 0x4202_3000;
const RAMCFG_AXISRAM3: u32 = RAMCFG_AXI_BASE + 0x0100;
const RAMCFG_AXISRAM4: u32 = RAMCFG_AXI_BASE + 0x0180;
const RAMCFG_AXISRAM5: u32 = RAMCFG_AXI_BASE + 0x0200;
const RAMCFG_AXISRAM6: u32 = RAMCFG_AXI_BASE + 0x0280;
/// `RAMCFG_CR.SRAMSD`: set = AXISRAM powered down. Cleared for NPU RAMs.
const RAMCFG_CR_SRAMSD: u32 = 1 << 20;

fn enable_all_sram() {
    // System RAM clocks (RCC.MEMENR)...
    pac::RCC.memenr().modify(|w| {
        w.set_axisram1en(true);
        w.set_axisram2en(true);
        w.set_axisram3en(true);
        w.set_axisram4en(true);
        w.set_axisram5en(true);
        w.set_axisram6en(true);
        w.set_ahbsram1en(true);
        w.set_ahbsram2en(true);
        w.set_bkpsramen(true);
    });

    // RAMCFG's own AHB2 bus clock, without which its registers bus-fault.
    pac::RCC.ahb2enr().modify(|w| w.set_ramcfgen(true));

    // ...and power the NPU RAMs out of shutdown. AXISRAM3..6 are the memories
    // the compiled activation pools live in; HAL_RAMCFG_EnableAXISRAM() is just
    // a clear of RAMCFG.CR.SRAMSD.
    for reg in [RAMCFG_AXISRAM3, RAMCFG_AXISRAM4, RAMCFG_AXISRAM5, RAMCFG_AXISRAM6] {
        unsafe {
            let v = core::ptr::read_volatile(reg as *const u32);
            core::ptr::write_volatile(reg as *mut u32, v & !RAMCFG_CR_SRAMSD);
        }
    }
}

/// Grant the ATON NPU bus master + peripheral secure/privileged access (ST's
/// `Security_Config()`), so it can reach the internal RAMs and xSPI2.
fn configure_npu_rif() {
    RifMaster::Npu.set_attributes(&RifMasterAttributes::new(1, true, true));
    RifPeripheral::Npu.set_attributes(&RifPeripheralAttributes::new(true, true));
}

/// Bring xSPI2 up in memory-mapped Octal DTR mode so the NPU can fetch the
/// weights from external flash at real bandwidth. Returns the driver, which
/// must stay alive for the mapping to remain valid.
///
/// A single-lane `FASTREAD` at a deliberately slow bus clock used to sit
/// here instead. That's what "avoids depending on board-specific latency
/// tuning" bought: correctness with no bring-up risk, at the cost of ~9.5 s
/// per inference regardless of CPU/NPU clock (see git history) — streaming
/// this model's ~7.9 MB of weights over a single, non-DTR line dominates
/// everything else. This instead reproduces ST's own validated
/// `stm32n6570_discovery_xspi.c` config: Octal DTR (8-bit-wide, double
/// transfer rate) reads, with the flash's real per-board dummy-cycle count
/// (`Application/STM32N6570-DK/Inc/mx66uw1g45g_conf.h`, which overrides the
/// component driver's generic defaults).
fn enable_xspi2_memory_mapped(p: &mut embassy_stm32::Peripherals) -> Xspi<'static, peripherals::XSPI2, Blocking> {
    let config = embassy_stm32::xspi::Config {
        fifo_threshold: FIFOThresholdLevel::_4Bytes,
        memory_type: MemoryType::Macronix,
        delay_hold_quarter_cycle: true,
        device_size: NOR_SIZE,
        chip_select_high_time: ChipSelectHighTime::_2Cycle,
        free_running_clock: false,
        clock_mode: false,
        wrap_size: WrapSize::None,
        // ST brings the flash up at a conservative divider for the mode-switch
        // handshake below, then drops to undivided (0) for the fast path —
        // same ratio here (prescaler 3, ~1/4) against our slower HCLK5 kernel
        // clock than ST's board (see `xspi2sel` above), so if anything this
        // leaves *more* timing margin during the switch than ST's own config.
        clock_prescaler: 3,
        sample_shifting: true,
        chip_select_boundary: 0,
        max_transfer: 0,
        refresh: 0,
    };

    let mut xspi = Xspi::new_blocking_xspi_dqs(
        unsafe { p.XSPI2.clone_unchecked() },
        unsafe { p.PN6.clone_unchecked() },
        unsafe { p.PN2.clone_unchecked() },
        unsafe { p.PN3.clone_unchecked() },
        unsafe { p.PN4.clone_unchecked() },
        unsafe { p.PN5.clone_unchecked() },
        unsafe { p.PN8.clone_unchecked() },
        unsafe { p.PN9.clone_unchecked() },
        unsafe { p.PN10.clone_unchecked() },
        unsafe { p.PN11.clone_unchecked() },
        unsafe { p.PN1.clone_unchecked() },
        unsafe { p.PN0.clone_unchecked() }, // DQS: GPION pin 0, AF9 on the DK
        config,
    );

    switch_flash_to_octal_dtr(&mut xspi);

    // Undivided kernel clock for the fast path, mirroring ST dropping to
    // prescaler 0 once the switch is confirmed.
    xspi.set_clock_prescaler(0);

    // Octal DTR read (`MX66UW1G45G_OCTA_READ_DTR_CMD`): 16-bit DTR
    // instruction, 32-bit DTR address, 8-line DTR data, DQS-sampled.
    // `DUMMY_CYCLES_READ_OCTAL_DTR` = 10 is the DK-specific override, not the
    // component driver's generic default of 6.
    let read_config = TransferConfig {
        iwidth: XspiWidth::OCTO,
        isize: AddressSize::_16bit,
        idtr: true,
        adwidth: XspiWidth::OCTO,
        adsize: AddressSize::_32bit,
        addtr: true,
        dwidth: XspiWidth::OCTO,
        ddtr: true,
        instruction: Some(0xEE11),
        dummy: DummyCycles::_10,
        dqse: true,
        ..Default::default()
    };
    // Octal page program (`MX66UW1G45G_OCTA_PAGE_PROG_CMD`) — never actually
    // exercised (this example only reads), kept for API symmetry and so a
    // real write wouldn't silently corrupt data with the wrong protocol.
    let write_config = TransferConfig {
        iwidth: XspiWidth::OCTO,
        isize: AddressSize::_16bit,
        idtr: true,
        adwidth: XspiWidth::OCTO,
        adsize: AddressSize::_32bit,
        addtr: true,
        dwidth: XspiWidth::OCTO,
        ddtr: true,
        instruction: Some(0x12ED),
        dummy: DummyCycles::_0,
        ..Default::default()
    };
    xspi.enable_memory_mapped_mode(read_config, write_config).unwrap();
    xspi
}

/// Switch the MX66UW1G45G from its default single-lane SPI (1-1-1) mode into
/// Octal DTR ("DOPI"), mirroring ST's `XSPI_NOR_EnterDOPIMode` /
/// `MX66UW1G45G_WriteCfg2Register` (`stm32n6570_discovery_xspi.c` +
/// `mx66uw1g45g.c`). Must run before `enable_memory_mapped_mode`, at the
/// conservative clock the caller already configured — these are one-shot
/// register writes over the *old* single-lane protocol, not the fast path.
fn switch_flash_to_octal_dtr(xspi: &mut Xspi<'static, peripherals::XSPI2, Blocking>) {
    const WRITE_ENABLE: u32 = 0x06;
    const WRITE_CFG2: u32 = 0x72;
    const READ_CFG2_OCTAL_DTR: u32 = 0x718E; // MX66UW1G45G_OCTA_READ_CFG_REG2_CMD
    /// Dummy-cycle-count select register (`MX66UW1G45G_CR2_REG3_ADDR`).
    const CR2_REG3_ADDR: u32 = 0x0000_0300;
    /// "20 dummy cycles" preset (`MX66UW1G45G_CR2_DC_20_CYCLES`) — the value
    /// ST's own board bring-up writes here; the count that actually governs
    /// read timing at runtime is `DUMMY_CYCLES_READ_OCTAL_DTR` above.
    const CR2_DC_20_CYCLES: u8 = 0x00;
    /// Protocol-select register (`MX66UW1G45G_CR2_REG1_ADDR`).
    const CR2_REG1_ADDR: u32 = 0x0000_0000;
    /// Octal DTR ("DOPI") protocol select bit (`MX66UW1G45G_CR2_DOPI`).
    const CR2_DOPI: u8 = 0x02;

    let write_enable = TransferConfig {
        iwidth: XspiWidth::SING,
        isize: AddressSize::_8bit,
        instruction: Some(WRITE_ENABLE),
        ..Default::default()
    };
    let write_cfg2 = |addr: u32| TransferConfig {
        iwidth: XspiWidth::SING,
        isize: AddressSize::_8bit,
        instruction: Some(WRITE_CFG2),
        adwidth: XspiWidth::SING,
        adsize: AddressSize::_32bit,
        address: Some(addr),
        dwidth: XspiWidth::SING,
        ..Default::default()
    };

    // Set the dummy-cycle-count register, then switch protocol — both need
    // their own preceding Write Enable, matching the flash's write-latch
    // semantics (it self-clears after one write-type command).
    debug!("octal DTR switch: write enable #1");
    xspi.blocking_command(&write_enable).unwrap();
    debug!("octal DTR switch: set dummy-cycle-count register");
    xspi.blocking_write(&[CR2_DC_20_CYCLES], write_cfg2(CR2_REG3_ADDR))
        .unwrap();
    debug!("octal DTR switch: write enable #2");
    xspi.blocking_command(&write_enable).unwrap();
    debug!("octal DTR switch: set DOPI protocol bit");
    xspi.blocking_write(&[CR2_DOPI], write_cfg2(CR2_REG1_ADDR)).unwrap();

    // ST waits `MX66UW1G45G_WRITE_REG_MAX_TIME` (40 ms, the datasheet's max
    // register-write time) before touching the flash again. `embassy_time`
    // isn't worth pulling into this one-shot bring-up path; a cycle-count
    // spin at the CPU's configured 800 MHz covers the same 40 ms with room
    // to spare.
    debug!("octal DTR switch: waiting for flash to apply");
    cortex_m::asm::delay(40_000_000);

    // Read back CR2 register 1 in the *new* Octal DTR protocol to confirm
    // the switch actually took, mirroring ST's own post-switch check.
    debug!("octal DTR switch: verifying via Octal DTR readback");
    let read_cfg2_dtr = TransferConfig {
        iwidth: XspiWidth::OCTO,
        isize: AddressSize::_16bit,
        idtr: true,
        instruction: Some(READ_CFG2_OCTAL_DTR),
        adwidth: XspiWidth::OCTO,
        adsize: AddressSize::_32bit,
        addtr: true,
        address: Some(CR2_REG1_ADDR),
        dwidth: XspiWidth::OCTO,
        ddtr: true,
        dummy: DummyCycles::_5, // DUMMY_CYCLES_REG_OCTAL_DTR
        dqse: true,
        ..Default::default()
    };
    let mut reg = [0u8; 2]; // DTR reads transfer in pairs; only reg[0] is meaningful.
    xspi.blocking_read(&mut reg, read_cfg2_dtr).unwrap();
    if reg[0] != CR2_DOPI {
        error!(
            "flash did not switch to Octal DTR mode (CR2 register 1 read back 0x{:02x}, expected 0x{:02x})",
            reg[0], CR2_DOPI
        );
        loop {
            cortex_m::asm::wfi();
        }
    }
}

fn mm_read_u32(addr: u32) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}

/// Read a little-endian `f32` through the xSPI2 memory-mapped window.
fn mm_read_f32(addr: u32) -> f32 {
    f32::from_bits(mm_read_u32(addr))
}

/// Read a single signed byte through the xSPI2 memory-mapped window.
///
/// `sw_info.izp` (the Dequantize zero-point) is stored as one `int8_t` — ST's
/// own `ll_sw_integer.c` casts its pointer to `int8_t*` via
/// `AI_BUFFER_META_FLAG_ZEROPOINT_S8`, not a 32-bit int. Reading it as `i32`
/// zero-extends the raw byte instead of sign-extending it, e.g. turning a
/// true `-128` (`0x80`) into `+128`; fed into `scale * (val - zero_point)`
/// that flips every dequantized probability negative.
fn mm_read_i8(addr: u32) -> i32 {
    unsafe { core::ptr::read_volatile(addr as *const i8) as i32 }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("============================================================");
    info!("  STM32N6 Neural-ART (ATON) NPU bring-up — real ST Edge AI model");
    info!("============================================================");

    let mut config = embassy_stm32::Config::default();
    config.rcc.supply_config = SUPPLY;

    // PLL1: 800 MHz CPU, 200 MHz system bus (same config as `neochrom_lcd`,
    // hardware-validated on this board). Left at the HSI-bypass default
    // (64 MHz sys/CPU) this example's NPU epoch took ~9.6 s — the DWT cycle
    // counter's own 612M-cycle count divided almost exactly by 64 MHz — for
    // a MobileNetV1-0.25 96x96 inference that should be low-single-digit
    // milliseconds. AXISRAM1..6's AHB5 clock (which the NPU itself runs on,
    // via `hclk5`) derives from this same "sys" tree, so this also speeds up
    // the NPU's own compute, not just the CPU driving it.
    config.rcc.pll1 = Some(Pll::Oscillator {
        source: Pllsel::Hsi,
        divm: Plldivm::Div4,
        fractional: 0,
        divn: 50,
        divp1: Pllpdiv::Div1,
        divp2: Pllpdiv::Div1,
    });
    config.rcc.ic1 = Some(IcConfig {
        source: Icsel::Pll1,
        divider: Icint::Div1,
    });
    let sys_ic = IcConfig {
        source: Icsel::Pll1,
        divider: Icint::Div4,
    };
    config.rcc.ic2 = Some(sys_ic);
    config.rcc.ic6 = Some(sys_ic);
    config.rcc.ic11 = Some(sys_ic);
    config.rcc.cpu = CpuClk::Ic1;
    config.rcc.sys = SysClk::Ic2;

    // xSPI2 kernel clock = HCLK5 (100 MHz here, from the PLL1 config above),
    // matching ST's own `stm32n6570_discovery_xspi.c` ("XSPI2 kernel clock =
    // HCLK = 200 MHz" — ours is half that since `sys`->`hclk` divides by 2
    // here where ST's board config doesn't, but that only means *more*
    // timing margin on the Octal DTR read below, not less).
    config.rcc.mux.xspi2sel = XspiClkSrc::Hclk5;
    config.rcc.vddio3_1v8 = true;
    let mut p = embassy_stm32::init(config);

    // The STM32N6 boot ROM jumps to dev-mode RAM applications with PRIMASK
    // set (interrupts globally masked); nothing else here clears it. Without
    // this, the NPU0 completion interrupt `run_epoch_blob` awaits below can
    // never actually reach the CPU, and the epoch hangs forever. Same fix as
    // `neochrom.rs` / `neochrom_graphics.rs`.
    unsafe { cortex_m::interrupt::enable() };

    // Cycle counter for the inference-latency report.
    //
    // `embassy_stm32::init` already steals `cortex_m::Peripherals` on N6 (to
    // enable the FPU and set VTOR in `rcc::n6`), so `take()` would find the
    // singleton already consumed; steal it again instead.
    let mut core = unsafe { cortex_m::Peripherals::steal() };
    core.DCB.enable_trace();
    core.DWT.enable_cycle_counter();

    // 1. Memories the compiled activation pools live in.
    enable_all_sram();
    info!("AXISRAM1..6 clocked, AXISRAM3..6 out of shutdown");

    // 2. Bus-master isolation: let the NPU reach internal RAM and xSPI2.
    configure_npu_rif();
    info!("RIF: NPU master/peripheral set secure+privileged");

    // 3. External flash in memory-mapped mode (weights live there).
    let xspi = enable_xspi2_memory_mapped(&mut p);
    info!("xSPI2 memory-mapped at 0x{:08x}", XSPI2_MM_BASE);

    // Sanity-check that `network_data.xSPI2.bin` was programmed where the model
    // expects it, rather than letting the NPU silently read blank flash.
    let w0 = mm_read_u32(WEIGHTS_ADDR);
    let want = u32::from_le_bytes([WEIGHTS[0], WEIGHTS[1], WEIGHTS[2], WEIGHTS[3]]);
    if w0 != want {
        error!(
            "xSPI2 @0x{:08x} = 0x{:08x}, expected 0x{:08x} — program network_data.xSPI2.bin there first",
            WEIGHTS_ADDR, w0, want
        );
        loop {
            cortex_m::asm::wfi();
        }
    }
    info!("xSPI2 weights verified ({} bytes)", WEIGHTS.len());

    // 4. The NPU itself.
    let mut npu = npu::Npu::new(p.NPU, Irqs);
    cache::npu_cache_enable();
    info!("ATON initialised, CACHEAXI enabled");

    // 5. Stage the blob: copy it into aligned, (MCU-)cacheable RAM and make it
    //    visible to the NPU's bus master.
    if EC_BLOB.len() > BLOB_STORAGE_LEN || EC_BLOB.len() % 8 != 0 {
        error!("unexpected blob length {}", EC_BLOB.len());
        loop {
            cortex_m::asm::wfi();
        }
    }
    let storage = unsafe { &mut *core::ptr::addr_of_mut!(BLOB_STORAGE) };
    storage[..EC_BLOB.len()].copy_from_slice(EC_BLOB);
    cache::mcu_clean_range(storage.as_ptr() as u32, EC_BLOB.len() as u32);

    let blob: &[u64] = unsafe { core::slice::from_raw_parts(storage.as_ptr() as *const u64, EC_BLOB.len() / 8) };

    // 6. Feed the input tensor: a real, public-domain Mexican marigold. The NPU
    //    block has no image decoder, so the model's input buffer must already
    //    hold uint8 NHWC pixels (scale 1/127.5, zero-point 127). A real
    //    application would have DCMIPP/DMA2D write here instead.
    if INPUT_IMAGE.len() != INPUT_LEN {
        error!("expected a {} byte RGB8 input, got {}", INPUT_LEN, INPUT_IMAGE.len());
        loop {
            cortex_m::asm::wfi();
        }
    }
    unsafe {
        let input = core::slice::from_raw_parts_mut(INPUT_ADDR as *mut u8, INPUT_LEN);
        input.copy_from_slice(INPUT_IMAGE);
    }
    cache::mcu_clean_range(INPUT_ADDR, INPUT_LEN as u32);

    // 7. Run the single hardware epoch. The whole network executes on the NPU.
    info!("running NPU epoch ({} bytes of microcode)...", EC_BLOB.len());
    let t0 = cortex_m::peripheral::DWT::cycle_count();
    match npu.run_epoch_blob(blob).await {
        Ok(()) => {
            let cycles = cortex_m::peripheral::DWT::cycle_count().wrapping_sub(t0);
            info!("NPU epoch complete in {} cycles", cycles);
        }
        Err(e) => {
            error!("NPU epoch failed: {:?}", e);
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    // 8. Reproduce ST's software epochs on the CPU with `embedded-nn`.
    //
    //    ST's generated code ends the network with two *software* epoch blocks
    //    (`LL_ATON_End_EpochBlock_31/32`), i.e. the CPU finishes the graph once
    //    the NPU epoch is done:
    //
    //      Softmax   : 0x342E_03F0 (5 x i8)  -> 0x342E_0410 (5 x i8)
    //      Dequantize: 0x342E_0410 (5 x i8)  -> 0x342E_03F0 (5 x f32)
    //
    //    Their parameters are baked into the model image; the Softmax
    //    fixed-point triple is a compile-time constant while the Dequantize
    //    scale/zero-point are read straight out of xSPI2 (ST's `sw_info` stores
    //    *pointers* to them at the tail of `network_data.xSPI2.bin`).
    cache::mcu_invalidate_range(LOGITS_ADDR, NUM_CLASSES as u32);
    let logits = unsafe { core::slice::from_raw_parts(LOGITS_ADDR as *const i8, NUM_CLASSES) };

    // 9. Compare against the `embedded-nn` reference.
    match GOLDEN_LOGITS {
        Some(expected) => {
            let worst = logits
                .iter()
                .zip(expected.iter())
                .map(|(got, want)| (*got as i32 - *want as i32).abs())
                .max()
                .unwrap_or(0);
            if worst == 0 {
                info!("golden check: EXACT match with embedded-nn (logits {:?})", logits);
            } else if worst <= GOLDEN_TOLERANCE {
                info!(
                    "golden check: PASS, worst |delta| = {} (tolerance {})",
                    worst, GOLDEN_TOLERANCE
                );
            } else {
                warn!(
                    "golden check: MISMATCH, worst |delta| = {} > {} — NPU {:?} vs embedded-nn {:?}",
                    worst, GOLDEN_TOLERANCE, logits, expected
                );
            }
        }
        None => warn!("golden check: SKIPPED, NPU logits = {:?}", logits),
    }

    let mut sm = [0i8; NUM_CLASSES];
    SoftmaxEpoch::new_1d(logits, &mut sm, SOFTMAX_MULT, SOFTMAX_LEFT_SHIFT, SOFTMAX_DIFF_MIN)
        .run()
        .unwrap();

    // `sw_info.is` / `sw_info.izp` of `Dequantize_135` (offsets 212608 / 212656).
    let deq_scale = mm_read_f32(WEIGHTS_ADDR + DEQ_SCALE_OFFSET);
    let deq_zp = mm_read_i8(WEIGHTS_ADDR + DEQ_ZP_OFFSET);
    let mut probs = [0f32; NUM_CLASSES];
    DequantizeEpoch::new(&sm, &mut probs, deq_scale, deq_zp).run().unwrap();

    let mut best = 0usize;
    let mut best_score = 0i8;
    ArgMaxEpoch::new(logits, &mut best, &mut best_score).run().unwrap();

    info!("NPU logits (i8): {:?}", logits);
    info!("softmax   (i8): {:?}", sm);
    info!("probabilities : {:?}", probs);
    info!(
        "=> class {} (argmax i8 score {}), p(class) = {:?}",
        best, best_score, probs[best]
    );
    info!("============================================================");

    // Keep the xSPI2 mapping alive.
    core::mem::forget(xspi);

    loop {
        cortex_m::asm::wfi();
    }
}
