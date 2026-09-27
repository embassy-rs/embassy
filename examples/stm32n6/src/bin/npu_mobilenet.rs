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
//! embedded as raw `RGB8`. A flower is used deliberately: the model classifies
//! `daisy, dandelion, rose, sunflower, tulip`, so an in-distribution image is
//! required for the logits to carry any signal.
//!
//! A hardware-independent cross-check is wired in: the same quantized model's
//! public `.tflite` is executed on `embedded-nn`'s host interpreter and the NPU
//! logits must match it within [`GOLDEN_TOLERANCE`]. That reference was itself
//! verified against Google's official TFLite runtime.
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
use embassy_stm32::rcc::{IcConfig, Icint, Icsel, SupplyConfig, XspiClkSrc};
use embassy_stm32::rif::{RifMaster, RifMasterAttributes, RifPeripheral, RifPeripheralAttributes};
use embassy_stm32::xspi::{
    AddressSize, ChipSelectHighTime, DummyCycles, FIFOThresholdLevel, MemorySize, MemoryType, TransferConfig, WrapSize,
    Xspi, XspiWidth,
};
use embassy_stm32::{bind_interrupts, pac, peripherals};
use panic_probe as _;
use static_cell::StaticCell;

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
const NUM_CLASSES: usize = 5;
/// Where `network_data.xSPI2.bin` must have been programmed.
const WEIGHTS_ADDR: u32 = 0x7038_0000;

// Parameters of ST's trailing software epoch blocks, verbatim from the generated
// `network.c` (`Softmax_133` / `Dequantize_135`).
/// TFLite fixed-point softmax multiplier / shift / clamp (`sw_info`).
const SOFTMAX_MULT: i32 = 1_968_914_048;
const SOFTMAX_LEFT_SHIFT: i32 = 23;
const SOFTMAX_DIFF_MIN: i32 = -248;
/// Byte offsets, inside `network_data.xSPI2.bin`, of the Dequantize f32 scale
/// and int32 zero-point (`Dequantize_135.is` / `.izp`).
const DEQ_SCALE_OFFSET: u32 = 212_608;
const DEQ_ZP_OFFSET: u32 = 212_656;

/// Fixed network input: a public-domain Mexican marigold (`Tagetes`), raw `RGB8`
/// 96x96x3, byte-identical to the image the `embedded-nn` reference is computed
/// from. See `npu-model/README.md` for provenance.
///
/// A flower is deliberately chosen because this model classifies five flower
/// classes (`daisy, dandelion, rose, sunflower, tulip`); an out-of-distribution
/// image such as the Mona Lisa produces near-uniform logits and makes a useless
/// regression vector.
static INPUT_IMAGE: &[u8] = include_bytes!("../../npu-model/mexican_marigold_96x96.rgb");

/// Expected pre-softmax logits for `INPUT_IMAGE`, from `embedded-nn`'s host
/// interpreter.
///
/// The reference is itself validated against Google's official TFLite runtime
/// (`ai-edge-litert`) on this exact model and input: both produce
/// `[-37, -27, -3, 8, 0]` bit-for-bit, and across a set of seven images
/// (in-distribution flowers plus black/grey/white) `embedded-nn` agrees with
/// the official runtime to within one LSB on the logits and picks the same
/// class every time.
///
/// Regenerate with:
/// ```text
/// cargo run -p embedded-nn-tflite --example stai_golden -- \
///     Model/mobilenet_v1_0.25_96_tfs_int8.tflite npu-model/mexican_marigold_96x96.rgb
/// ```
const GOLDEN_LOGITS: Option<[i8; NUM_CLASSES]> = Some([-37, -27, -3, 8, 0]);

/// Allowed per-element error against `GOLDEN_LOGITS`.
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
static EC_BLOB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/npu/ec_blob.bin"));
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
/// once per epoch, so a `StaticCell`-backed buffer is enough.
static BLOB_STORAGE: StaticCell<Aligned<A8, [u8; 65536]>> = StaticCell::new();

/// RAMCFG instance base for AXISRAM3..6 (non-secure alias).
const RAMCFG_AXI_BASE: u32 = 0x4202_0000;
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

/// Bring xSPI2 up in memory-mapped mode so the NPU can fetch the weights from
/// external flash. Returns the driver, which must stay alive for the mapping to
/// remain valid.
///
/// The read/write configurations mirror `ll_aton`'s expectations: a plain
/// single-lane `FASTREAD` (0x0C) with 8 dummy cycles. This is slower than the
/// OPI/DTR mode ST uses but avoids depending on board-specific latency tuning.
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
        clock_prescaler: 8, // 64 MHz / 8 = 8 MHz, conservative
        sample_shifting: true,
        chip_select_boundary: 0,
        max_transfer: 0,
        refresh: 0,
    };

    let mut xspi = Xspi::new_blocking_xspi(
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
        config,
    );

    // FASTREAD (4-byte address) with 8 dummy cycles — same transaction the
    // `xspi_flash` example validates on the DK.
    let read_config = TransferConfig {
        iwidth: XspiWidth::SING,
        isize: AddressSize::_8bit,
        adwidth: XspiWidth::SING,
        adsize: AddressSize::_32bit,
        dwidth: XspiWidth::SING,
        instruction: Some(0x0C),
        dummy: DummyCycles::_8,
        ..Default::default()
    };
    let write_config = TransferConfig {
        iwidth: XspiWidth::SING,
        isize: AddressSize::_8bit,
        adwidth: XspiWidth::SING,
        adsize: AddressSize::_32bit,
        dwidth: XspiWidth::SING,
        instruction: Some(0x12), // PAGE PROGRAM (4-byte address)
        dummy: DummyCycles::_0,
        ..Default::default()
    };
    xspi.enable_memory_mapped_mode(read_config, write_config).unwrap();
    xspi
}

fn mm_read_u32(addr: u32) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}

/// Read a little-endian `f32` through the xSPI2 memory-mapped window.
fn mm_read_f32(addr: u32) -> f32 {
    f32::from_bits(mm_read_u32(addr))
}

/// Read a little-endian `i32` through the xSPI2 memory-mapped window.
fn mm_read_i32(addr: u32) -> i32 {
    mm_read_u32(addr) as i32
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("============================================================");
    info!("  STM32N6 Neural-ART (ATON) NPU bring-up — real ST Edge AI model");
    info!("============================================================");

    let mut config = embassy_stm32::Config::default();
    config.rcc.supply_config = SUPPLY;
    // xSPI2 kernel clock from IC4. The `xspi_flash` example drives PLL2 in
    // bypass mode, i.e. IC4 = HSI = 64 MHz, which both boards accept.
    config.rcc.ic4 = Some(IcConfig {
        source: Icsel::Pll2,
        divider: Icint::from_bits(0),
    });
    config.rcc.mux.xspi2sel = XspiClkSrc::Ic4;
    config.rcc.vddio3_1v8 = true;
    let mut p = embassy_stm32::init(config);

    // Cycle counter for the inference-latency report.
    let mut core = cortex_m::Peripherals::take().unwrap();
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
    if EC_BLOB.len() > 65536 || EC_BLOB.len() % 8 != 0 {
        error!("unexpected blob length {}", EC_BLOB.len());
        loop {
            cortex_m::asm::wfi();
        }
    }
    let storage = BLOB_STORAGE.init(Aligned([0u8; 65536]));
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
    let deq_zp = mm_read_i32(WEIGHTS_ADDR + DEQ_ZP_OFFSET);
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
