//! PIO RGB scan-out for the gen4-RP2350-70CT panel (`rgb70.pio` / Graphics4D port).
//!
//! Continuous DMA from a single PSRAM RGB565 framebuffer into the PIO2 pixel
//! stream, with HSYNC/VSYNC/DE generated on PIO1 — matching the reference C
//! firmware in `gen4_rp2350_lvgl`.

use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use defmt::{info, warn};
use embassy_rp::clocks::clk_sys_freq;
use embassy_rp::dma::ChannelInstance;
use embassy_rp::gpio::{Drive, Pull, SlewRate};
use embassy_rp::interrupt::typelevel::{Binding, Handler, Interrupt};
use embassy_rp::pio::program::pio_file;
use embassy_rp::pio::{Config, Direction, FifoJoin, Pio, PioPin};
use embassy_rp::{bind_interrupts, pac, peripherals, Peri};
use fixed::types::extra::U8;
use fixed::FixedU32;
use static_cell::StaticCell;

use crate::board::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

/// Effective RGB dot clock target used by the gen4 70" PIO programs.
const PCLK_TARGET_HZ: f32 = 36_000_000.0;
/// Match `BOUNCE_BUFFER_LINES` in `Graphics4D.cpp` (gen4_rp2350_lvgl).
const BOUNCE_BUFFER_LINES: usize = 60;
const BOUNCE_BUFFER_PIXELS: usize = DISPLAY_WIDTH * BOUNCE_BUFFER_LINES;
const BOUNCE_BUFFER_COUNT: u16 = (DISPLAY_HEIGHT / BOUNCE_BUFFER_LINES) as u16;
const PIO2_RGB_SM: u8 = 0;

bind_interrupts!(pub struct ScanOutIrqs {
    PIO1_IRQ_0 => embassy_rp::pio::InterruptHandler<peripherals::PIO1>;
    PIO2_IRQ_0 => embassy_rp::pio::InterruptHandler<peripherals::PIO2>;
    DMA_IRQ_1 => ScanOutDmaHandler;
});

struct ScanOutState {
    framebuffer: *mut u16,
    transfer_index: u16,
    dma_channel_a: u8,
    dma_channel_b: u8,
    /// Dedicated channel formerly used for the PSRAM→bounce refill DMA. The
    /// refill is now a cache-coherent CPU copy in the ISR, so this is reserved
    /// but unused (kept so DMA_CH2 stays claimed for scan-out's exclusive use).
    #[allow(dead_code)]
    dma_channel_copy: u8,
}

impl ScanOutState {
    const fn empty() -> Self {
        Self {
            framebuffer: ptr::null_mut(),
            transfer_index: 0,
            dma_channel_a: 0,
            dma_channel_b: 0,
            dma_channel_copy: 0,
        }
    }
}

static mut SCANOUT: ScanOutState = ScanOutState::empty();
static SCANOUT_READY: AtomicBool = AtomicBool::new(false);

/// Pending framebuffer pointer requested via [`present`]. `0` means "no swap
/// pending". The DMA ISR consumes it at a frame boundary so the swap is atomic
/// with respect to the displayed image (no tearing).
static PENDING_FB: AtomicU32 = AtomicU32::new(0);
static SCANOUT_PIO1: StaticCell<Pio<'static, peripherals::PIO1>> = StaticCell::new();
static SCANOUT_PIO2: StaticCell<Pio<'static, peripherals::PIO2>> = StaticCell::new();

/// Force 4-byte alignment so the bounce buffers can be filled with 32-bit
/// accesses (two RGB565 pixels per word). This halves the number of PSRAM
/// read transactions performed by the refill in the DMA ISR.
#[repr(align(4))]
struct BounceBuffer([u16; BOUNCE_BUFFER_PIXELS]);

static mut BOUNCE0: BounceBuffer = BounceBuffer([0; BOUNCE_BUFFER_PIXELS]);
static mut BOUNCE1: BounceBuffer = BounceBuffer([0; BOUNCE_BUFFER_PIXELS]);

#[inline(always)]
unsafe fn scanout_mut() -> *mut ScanOutState {
    core::ptr::addr_of_mut!(SCANOUT)
}

pub struct ScanOutDmaHandler;

impl Handler<embassy_rp::interrupt::typelevel::DMA_IRQ_1> for ScanOutDmaHandler {
    unsafe fn on_interrupt() {
        let state = unsafe { &mut *scanout_mut() };
        let ints = pac::DMA.ints(1).read();
        let ch_a = state.dma_channel_a as usize;
        let ch_b = state.dma_channel_b as usize;
        if ints & (1 << ch_a) != 0 {
            pac::DMA.ints(1).write_value(1 << ch_a);
            dma_complete_handler(state, state.dma_channel_a);
        }
        if ints & (1 << ch_b) != 0 {
            pac::DMA.ints(1).write_value(1 << ch_b);
            dma_complete_handler(state, state.dma_channel_b);
        }
    }
}

fn pio_sync_divider() -> FixedU32<U8> {
    let sys_clk = clk_sys_freq() as f32;
    FixedU32::from_num(sys_clk / PCLK_TARGET_HZ)
}

fn pio_rgb_divider() -> FixedU32<U8> {
    let sync = clk_sys_freq() as f32 / PCLK_TARGET_HZ;
    FixedU32::from_num(sync / 7.0 * 2.7)
}

fn set_gpio_base(pio: pac::pio::Pio) {
    pio.gpiobase().write(|w| w.set_gpiobase(true));
}

/// Map GPIO18..21 (DE/VSYNC/HSYNC/PCLK) to PIO IN pins 2..5.
///
/// With `gpiobase = 16` the state machine's pin indices are already relative to
/// GPIO16, so the IN base must stay at 0 (matching the Graphics4D reference,
/// which never calls `sm_config_set_in_pins`). This makes `wait pin 2/3/5`
/// resolve to DE (GPIO18), VSYNC (GPIO19) and PCLK (GPIO21). Using a base of 16
/// here would shift the wait pins to GPIO32+, so the RGB SM would stall forever
/// and never clock out a single pixel (blank panel).
fn apply_wait_pin_map(pio: pac::pio::Pio, sm: usize) {
    let sm = pio.sm(sm);
    sm.pinctrl().modify(|w| w.set_in_base(0));
}

fn rgb_dma_treq(sm: u8) -> pac::dma::vals::TreqSel {
    pac::dma::vals::TreqSel::from(2 * 8 + sm)
}

/// Write READ_ADDR, WRITE_ADDR and TRANS_COUNT for a DMA channel.
#[inline(always)]
#[unsafe(link_section = ".data.ram_func")]
fn dma_set_addrs(channel: u8, rgb_sm: u8, src: *const u16, count: usize) {
    let ch = pac::DMA.ch(channel as _);
    ch.read_addr().write_value(src as u32);
    ch.write_addr()
        .write_value(pac::PIO2.txf(rgb_sm as _).as_ptr() as u32);
    ch.trans_count().write(|w| {
        w.set_mode(0.into());
        w.set_count(count as u32);
    });
}

/// Apply the common CTRL fields for scan-out DMA via a closure.
#[inline(always)]
#[unsafe(link_section = ".data.ram_func")]
fn write_ctrl_fields(w: &mut pac::dma::regs::Ctrl, chain_to: u8, rgb_sm: u8) {
    w.set_treq_sel(rgb_dma_treq(rgb_sm));
    w.set_data_size(pac::dma::vals::DataSize::SizeHalfword);
    w.set_incr_read(true);
    w.set_incr_write(false);
    w.set_chain_to(chain_to);
    w.set_bswap(false);
    // Mark the scan-out channels as high priority so the DMA arbiter always
    // services the panel pixel stream ahead of any low-priority traffic.
    w.set_high_priority(true);
    w.set_en(true);
}

/// Configure a DMA channel for scan-out and trigger it immediately.
/// Used only for the initial start in `prime_scanout_dma`.
#[inline(never)]
#[unsafe(link_section = ".data.ram_func")]
fn dma_start(channel: u8, chain_to: u8, rgb_sm: u8, src: *const u16, count: usize) {
    dma_set_addrs(channel, rgb_sm, src, count);
    // Writing CTRL_TRIG triggers the transfer.
    pac::DMA.ch(channel as _).ctrl_trig().write(|w| {
        write_ctrl_fields(w, chain_to, rgb_sm);
    });
}

/// Configure a DMA channel for scan-out WITHOUT triggering it.
/// The channel will be started later by hardware chaining from the other channel.
/// We write READ_ADDR, WRITE_ADDR, TRANS_COUNT normally, then use the AL1_CTRL
/// alias register (offset 0x10 from channel base) which sets CTRL without triggering.
#[inline(never)]
#[unsafe(link_section = ".data.ram_func")]
fn dma_configure(channel: u8, chain_to: u8, rgb_sm: u8, src: *const u16, count: usize) {
    dma_set_addrs(channel, rgb_sm, src, count);
    // Build the CTRL value using PAC types, then write via AL1_CTRL to avoid trigger.
    let mut ctrl_val = pac::dma::regs::Ctrl::default();
    write_ctrl_fields(&mut ctrl_val, chain_to, rgb_sm);
    // AL1_CTRL is at offset 0x10 from the channel base (0x5000_0000 + ch * 0x40).
    let al1_ctrl_addr = (0x5000_0000u32) + (channel as u32) * 0x40 + 0x10;
    unsafe {
        core::ptr::write_volatile(al1_ctrl_addr as *mut u32, ctrl_val.0);
    }
}

#[inline(always)]
unsafe fn bounce0_ptr() -> *mut u16 {
    core::ptr::addr_of_mut!(BOUNCE0.0).cast::<u16>()
}

#[inline(always)]
unsafe fn bounce1_ptr() -> *mut u16 {
    core::ptr::addr_of_mut!(BOUNCE1.0).cast::<u16>()
}

/// Start an unpaced memory-to-memory DMA that refills a bounce buffer from the
/// PSRAM framebuffer.
///
/// History of this path:
/// 1. Originally a CPU `read_volatile` loop inside the DMA ISR — thousands of
///    individual QMI reads, starving the PIO TX FIFO.
/// 2. An attempt to read through the **XIP cache** via the hardware XIP stream
///    engine (`TreqSel::XipStream`, draining `XIP_AUX.stream()` into the bounce
///    buffer). This removed the flicker but produced **corrupt pixels** ("snow")
///    on this board — the streamed words did not match the framebuffer contents
///    (cache-coherency / address-aliasing mismatch between the CPU's writes and
///    the stream engine's reads on the QMI/PSRAM mapping). Reverted.
///
/// Current version: a plain unpaced (`TreqSel::Permanent`, `incr_read = true`)
/// memory-to-memory burst that copies the framebuffer chunk straight into the
/// bounce buffer. This reads exactly the bytes the CPU wrote, so the image is
/// always correct. Combined with the other anti-flicker fixes (single-FB
/// partial render, phase-locked sync-SM start, DMA bus priority) the residual
/// bus contention is no longer enough to cause a visible roll/flicker.
///
/// `src` (PSRAM framebuffer) and `dst` (bounce buffer) are 4-byte aligned and
/// `count` (pixels) is even, so `count / 2` is the number of 32-bit words.
/// `chain_to == channel` disables chaining (one-shot transfer).
#[allow(dead_code)]
#[inline(always)]
#[unsafe(link_section = ".data.ram_func")]
fn dma_refill_start(channel: u8, src: *const u16, dst: *mut u16, count: usize) {
    let words = (count / 2) as u32;

    let ch = pac::DMA.ch(channel as _);
    ch.read_addr().write_value(src as u32);
    ch.write_addr().write_value(dst as u32);
    ch.trans_count().write(|w| {
        w.set_mode(0.into());
        w.set_count(words);
    });
    ch.ctrl_trig().write(|w| {
        // Unpaced memory-to-memory: run as fast as the DMA can move data.
        w.set_treq_sel(pac::dma::vals::TreqSel::Permanent);
        w.set_data_size(pac::dma::vals::DataSize::SizeWord);
        // Both source and destination increment through their buffers.
        w.set_incr_read(true);
        w.set_incr_write(true);
        // Chaining to itself means "no chain" — this is a one-shot transfer.
        w.set_chain_to(channel);
        w.set_bswap(false);
        // High priority so the refill out-arbitrates the CPU render and the
        // bounce buffer is always ready before scan-out reaches it.
        w.set_high_priority(true);
        w.set_en(true);
    });
}

/// DMA completion handler. With chaining the *next* transfer is already running
/// on the other channel, so we only need to reconfigure the just-finished
/// channel for the transfer *after* the one currently in flight, then refill
/// the bounce buffer that was just consumed.
#[inline(never)]
#[unsafe(link_section = ".data.ram_func")]
fn dma_complete_handler(state: &mut ScanOutState, finished_ch: u8) {
    if state.framebuffer.is_null() {
        return;
    }

    // `transfer_index` is the chunk that just finished sending.
    // The chained channel is already sending chunk `transfer_index + 1`.
    // We must prepare chunk `transfer_index + 2` on the finished channel.
    let current_index = state.transfer_index;
    let next_index = (current_index + 1) % BOUNCE_BUFFER_COUNT;
    let future_index = (current_index + 2) % BOUNCE_BUFFER_COUNT;

    // Apply a pending framebuffer swap exactly at the frame boundary: when the
    // chunk we are about to refill is chunk 0, the whole upcoming displayed
    // frame (chunks 0..N-1) will be read from the new buffer, so the swap is
    // tear-free. Done before computing `src` below.
    if future_index == 0 {
        let pending = PENDING_FB.load(Ordering::Acquire);
        if pending != 0 {
            state.framebuffer = pending as *mut u16;
            PENDING_FB.store(0, Ordering::Release);
        }
    }

    // Determine which channel is the other (currently active) one.
    let other_ch = if finished_ch == state.dma_channel_a {
        state.dma_channel_b
    } else {
        state.dma_channel_a
    };

    // Reconfigure the finished channel for the future transfer, chained from
    // the currently-active channel so it starts automatically.
    let future_buf = if future_index % 2 == 0 {
        unsafe { bounce0_ptr() }
    } else {
        unsafe { bounce1_ptr() }
    };
    dma_configure(finished_ch, other_ch, PIO2_RGB_SM, future_buf, BOUNCE_BUFFER_PIXELS);

    // Refill the bounce buffer that was just consumed (current_index) with the
    // data for `future_index`.
    let free_buf = if current_index % 2 == 0 {
        unsafe { bounce0_ptr() }
    } else {
        unsafe { bounce1_ptr() }
    };
    let src = unsafe { state.framebuffer.add(future_index as usize * BOUNCE_BUFFER_PIXELS) };
    // Refill via CPU copy through the **cached** XIP/PSRAM window — exactly the
    // working C reference default (`allowDMAfill = false`, plain `memcpy`).
    //
    // Why CPU copy and not a DMA / XIP-stream engine here:
    // * An unpaced memory-to-memory DMA produced a correct image but kept
    //   issuing a raw QMI burst that competed with the CPU full-frame render on
    //   the shared bus → residual roll/flicker.
    // * The hardware XIP-stream engine removed the flicker but read PSRAM raw,
    //   bypassing the cache, so it saw the CPU's not-yet-flushed framebuffer
    //   writes as stale → corrupt "snow" pixels.
    // A `copy_nonoverlapping` reads the framebuffer through the same XIP cache
    // the CPU wrote it with, so it is always coherent (correct image) *and* the
    // cache coalesces the sequential reads into far fewer QMI transactions than
    // an uncached burst (low bus load → no flicker). The copy is a single
    // bounce chunk and runs while the two chained scan-out channels keep
    // streaming, with two chunks of slack ahead of consumption.
    unsafe {
        ptr::copy_nonoverlapping(src, free_buf, BOUNCE_BUFFER_PIXELS);
    }
    state.transfer_index = next_index;
}

fn prime_scanout_dma(state: &mut ScanOutState) {
    let ch_a = state.dma_channel_a;
    let ch_b = state.dma_channel_b;
    unsafe {
        ptr::copy_nonoverlapping(state.framebuffer, bounce0_ptr(), BOUNCE_BUFFER_PIXELS);
        ptr::copy_nonoverlapping(
            state.framebuffer.add(BOUNCE_BUFFER_PIXELS),
            bounce1_ptr(),
            BOUNCE_BUFFER_PIXELS,
        );
    }
    state.transfer_index = 0;

    // Pre-configure channel B for chunk 1 (bounce1), chained from A — do NOT trigger.
    dma_configure(ch_b, ch_a, PIO2_RGB_SM, unsafe { bounce1_ptr() }, BOUNCE_BUFFER_PIXELS);
    // Start channel A for chunk 0 (bounce0), chained to B — this one triggers.
    dma_start(ch_a, ch_b, PIO2_RGB_SM, unsafe { bounce0_ptr() }, BOUNCE_BUFFER_PIXELS);
}

/// Request a tear-free swap to a freshly rendered framebuffer.
///
/// The pointer is latched and the DMA ISR switches the scan-out source to it at
/// the next frame boundary. Use [`swap_in_progress`] to wait until the swap has
/// been consumed before reusing the previous front buffer as the next back
/// buffer.
pub fn present(fb: *mut u16) {
    PENDING_FB.store(fb as u32, Ordering::Release);
}

/// Returns `true` while a [`present`] request has not yet been applied by the
/// scan-out ISR (i.e. the previous front buffer is still being displayed).
pub fn swap_in_progress() -> bool {
    PENDING_FB.load(Ordering::Acquire) != 0
}

/// Register the PSRAM scan-out framebuffer (800×480 RGB565).
pub fn bind_framebuffer(fb: *mut u16) {
    if SCANOUT_READY.load(Ordering::Acquire) {
        let state = unsafe { &mut *scanout_mut() };
        state.framebuffer = fb;
    }
}

/// Start continuous PIO RGB scan-out.
#[allow(clippy::too_many_arguments)]
pub fn init_scanout(
    pio1: Peri<'static, peripherals::PIO1>,
    pio2: Peri<'static, peripherals::PIO2>,
    _dma_a: Peri<'static, peripherals::DMA_CH0>,
    _dma_b: Peri<'static, peripherals::DMA_CH1>,
    _dma_c: Peri<'static, peripherals::DMA_CH2>,
    _irq: impl Binding<embassy_rp::interrupt::typelevel::DMA_IRQ_1, ScanOutDmaHandler> + 'static,
    de: Peri<'static, impl PioPin>,
    vsync: Peri<'static, impl PioPin>,
    hsync: Peri<'static, impl PioPin>,
    pclk: Peri<'static, impl PioPin>,
    data0: Peri<'static, impl PioPin>,
    data1: Peri<'static, impl PioPin>,
    data2: Peri<'static, impl PioPin>,
    data3: Peri<'static, impl PioPin>,
    data4: Peri<'static, impl PioPin>,
    data5: Peri<'static, impl PioPin>,
    data6: Peri<'static, impl PioPin>,
    data7: Peri<'static, impl PioPin>,
    data8: Peri<'static, impl PioPin>,
    data9: Peri<'static, impl PioPin>,
    data10: Peri<'static, impl PioPin>,
    data11: Peri<'static, impl PioPin>,
    data12: Peri<'static, impl PioPin>,
    data13: Peri<'static, impl PioPin>,
    data14: Peri<'static, impl PioPin>,
    data15: Peri<'static, impl PioPin>,
    framebuffer: *mut u16,
) {
    if SCANOUT_READY.swap(true, Ordering::AcqRel) {
        return;
    }
    if framebuffer.is_null() {
        SCANOUT_READY.store(false, Ordering::Release);
        warn!("PIO RGB: framebuffer is null");
        return;
    }

    let state = unsafe { &mut *scanout_mut() };
    state.framebuffer = framebuffer;
    state.dma_channel_a = peripherals::DMA_CH0::number();
    state.dma_channel_b = peripherals::DMA_CH1::number();
    state.dma_channel_copy = peripherals::DMA_CH2::number();

    let sync_div = pio_sync_divider();
    let rgb_div = pio_rgb_divider();

    let mut pio1_dev = Pio::new(pio1, ScanOutIrqs);
    let mut pio2_dev = Pio::new(pio2, ScanOutIrqs);

    set_gpio_base(pac::PIO1);
    set_gpio_base(pac::PIO2);

    let hsync_file = pio_file!("pio/rgb70.pio", select_program("hsync"), options(max_program_size = 64));
    let vsync_file = pio_file!("pio/rgb70.pio", select_program("vsync"), options(max_program_size = 64));
    let rgbframe_file =
        pio_file!("pio/rgb70.pio", select_program("RGBframe"), options(max_program_size = 64));
    let rgb_file = pio_file!("pio/rgb70.pio", select_program("rgb"), options(max_program_size = 64));

    let hsync_loaded = pio1_dev.common.load_program(&hsync_file.program);
    let vsync_loaded = pio1_dev.common.load_program(&vsync_file.program);
    let rgbframe_loaded = pio1_dev.common.load_program(&rgbframe_file.program);
    let rgb_loaded = pio2_dev.common.load_program(&rgb_file.program);

    let pin_hsync = pio1_dev.common.make_pio_pin(hsync);
    let pin_pclk = pio1_dev.common.make_pio_pin(pclk);
    let pin_vsync = pio1_dev.common.make_pio_pin(vsync);
    let pin_de = pio1_dev.common.make_pio_pin(de);

    let mut d0 = pio2_dev.common.make_pio_pin(data0);
    let mut d1 = pio2_dev.common.make_pio_pin(data1);
    let mut d2 = pio2_dev.common.make_pio_pin(data2);
    let mut d3 = pio2_dev.common.make_pio_pin(data3);
    let mut d4 = pio2_dev.common.make_pio_pin(data4);
    let mut d5 = pio2_dev.common.make_pio_pin(data5);
    let mut d6 = pio2_dev.common.make_pio_pin(data6);
    let mut d7 = pio2_dev.common.make_pio_pin(data7);
    let mut d8 = pio2_dev.common.make_pio_pin(data8);
    let mut d9 = pio2_dev.common.make_pio_pin(data9);
    let mut d10 = pio2_dev.common.make_pio_pin(data10);
    let mut d11 = pio2_dev.common.make_pio_pin(data11);
    let mut d12 = pio2_dev.common.make_pio_pin(data12);
    let mut d13 = pio2_dev.common.make_pio_pin(data13);
    let mut d14 = pio2_dev.common.make_pio_pin(data14);
    let mut d15 = pio2_dev.common.make_pio_pin(data15);
    for pin in [
        &mut d0, &mut d1, &mut d2, &mut d3, &mut d4, &mut d5, &mut d6, &mut d7, &mut d8, &mut d9,
        &mut d10, &mut d11, &mut d12, &mut d13, &mut d14, &mut d15,
    ] {
        pin.set_pull(Pull::Up);
        pin.set_slew_rate(SlewRate::Fast);
        pin.set_drive_strength(Drive::_12mA);
    }
    let data_pins = [
        &d0, &d1, &d2, &d3, &d4, &d5, &d6, &d7, &d8, &d9, &d10, &d11, &d12, &d13, &d14, &d15,
    ];

    let mut hsync_cfg = Config::default();
    hsync_cfg.fifo_join = FifoJoin::TxOnly;
    hsync_cfg.clock_divider = sync_div;
    hsync_cfg.use_program(&hsync_loaded, &[&pin_hsync, &pin_pclk]);

    let mut vsync_cfg = Config::default();
    vsync_cfg.fifo_join = FifoJoin::TxOnly;
    vsync_cfg.clock_divider = sync_div;
    vsync_cfg.use_program(&vsync_loaded, &[&pin_vsync]);

    let mut rgbframe_cfg = Config::default();
    rgbframe_cfg.fifo_join = FifoJoin::TxOnly;
    rgbframe_cfg.clock_divider = sync_div;
    rgbframe_cfg.use_program(&rgbframe_loaded, &[&pin_de]);

    let mut rgb_cfg = Config::default();
    rgb_cfg.fifo_join = FifoJoin::TxOnly;
    rgb_cfg.clock_divider = rgb_div;
    rgb_cfg.use_program(&rgb_loaded, &[]);
    rgb_cfg.set_out_pins(&data_pins);

    pio1_dev.sm0.set_config(&hsync_cfg);
    pio1_dev.sm1.set_config(&vsync_cfg);
    pio1_dev.sm2.set_config(&rgbframe_cfg);
    pio2_dev.sm0.set_config(&rgb_cfg);
    apply_wait_pin_map(pac::PIO2, 0);

    pio1_dev.sm0.set_pin_dirs(Direction::Out, &[&pin_hsync, &pin_pclk]);
    pio1_dev.sm1.set_pin_dirs(Direction::Out, &[&pin_vsync]);
    pio1_dev.sm2.set_pin_dirs(Direction::Out, &[&pin_de]);
    pio2_dev.sm0.set_pin_dirs(Direction::Out, &data_pins);

    // Match Graphics4D preload values for the 70CT panel.
    pio1_dev.sm0.tx().push(DISPLAY_WIDTH as u32 - 1);
    pio1_dev.sm1.tx().push(DISPLAY_HEIGHT as u32 - 1);
    pio1_dev.sm2.tx().push((DISPLAY_WIDTH * 2) as u32 - 1);
    pio2_dev.sm0.tx().push(DISPLAY_WIDTH as u32 - 1);

    // Enable the RGB pixel SM first; it stalls on `wait pin` until the sync
    // SMs drive DE/VSYNC/PCLK, so its exact start cycle is uncritical.
    pac::PIO2.ctrl().modify(|w| w.set_sm_enable(w.sm_enable() | 0b0001));
    // Start the three PIO1 sync SMs (HSYNC/VSYNC/DE) **phase-locked**.
    //
    // They all run from the same fractional clock divider (sys/36 MHz ≈ 6.39).
    // Simply OR-ing `sm_enable` leaves each SM's fractional divider at whatever
    // phase it happened to reach since `sm_init`, so HSYNC, VSYNC and DE edges
    // drift against each other by sub-pixel fractions every line → the panel
    // sees jittering sync timing → vertical roll / flicker.
    //
    // The Graphics4D reference avoids this with `pio_enable_sm_mask_in_sync`,
    // which restarts the clock dividers of all SMs in the mask **in the same
    // write** as the enable, aligning their fractional phases. Mirror that here
    // by setting CLKDIV_RESTART and SM_ENABLE for the three SMs together.
    pac::PIO1.ctrl().modify(|w| {
        w.set_clkdiv_restart(w.clkdiv_restart() | 0b0111);
        w.set_sm_enable(w.sm_enable() | 0b0111);
    });

    SCANOUT_PIO1.init(pio1_dev);
    SCANOUT_PIO2.init(pio2_dev);

    // Give the DMA masters higher bus-arbitration priority than the two
    // processors. Without this the CPU full-frame render into the *other*
    // PSRAM framebuffer can starve the scan-out refill DMA on the shared QMI
    // bus, leaving a bounce buffer half-filled when scan-out reaches it →
    // PIO TX FIFO underflow → vertical roll / flicker.
    pac::BUSCTRL.bus_priority().write(|w| {
        w.set_dma_r(true);
        w.set_dma_w(true);
    });

    let ch_a = peripherals::DMA_CH0::number();
    let ch_b = peripherals::DMA_CH1::number();
    pac::DMA.inte(1).write(|w| *w = (1 << ch_a) | (1 << ch_b));
    unsafe {
        embassy_rp::interrupt::typelevel::DMA_IRQ_1::enable();
    }

    prime_scanout_dma(state);
    info!(
        "gen4 PIO RGB scan-out started ({}x{} @ ~{} MHz)",
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        (PCLK_TARGET_HZ / 1_000_000.0) as u32
    );
}

/// Partial blit helper (LVGL flush copies into the PSRAM framebuffer).
pub fn blit_rgb565(fb: *mut u16, x1: i32, y1: i32, w: usize, h: usize, src: &[u8]) {
    if fb.is_null() {
        return;
    }
    let stride = DISPLAY_WIDTH;
    let row_bytes = w * 2;
    for row in 0..h {
        let y = y1 as usize + row;
        if y >= DISPLAY_HEIGHT {
            break;
        }
        let x = (x1 as usize).min(DISPLAY_WIDTH);
        let copy = row_bytes.min((DISPLAY_WIDTH - x) * 2);
        let dst_off = y * stride + x;
        let src_off = row * row_bytes;
        if src_off + copy > src.len() {
            break;
        }
        unsafe {
            ptr::copy_nonoverlapping(
                src.as_ptr().add(src_off),
                fb.add(dst_off).cast::<u8>(),
                copy,
            );
        }
    }
}

/// Fill the scan-out framebuffer with a solid colour (bring-up / prefill).
pub fn fill_framebuffer(fb: *mut u16, colour: u16) {
    if fb.is_null() {
        return;
    }
    unsafe {
        for i in 0..DISPLAY_WIDTH * DISPLAY_HEIGHT {
            *fb.add(i) = colour;
        }
    }
}
