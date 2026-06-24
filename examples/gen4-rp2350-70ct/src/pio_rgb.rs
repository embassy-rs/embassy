//! PIO RGB scan-out for the gen4-RP2350-70CT panel (`rgb70.pio` / Graphics4D port).
//!
//! Continuous DMA from a single PSRAM RGB565 framebuffer into the PIO2 pixel
//! stream, with HSYNC/VSYNC/DE generated on PIO1 — matching the reference C
//! firmware in `gen4_rp2350_lvgl`.

use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

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
    dma_channel: u8,
}

impl ScanOutState {
    const fn empty() -> Self {
        Self {
            framebuffer: ptr::null_mut(),
            transfer_index: 0,
            dma_channel: 0,
        }
    }
}

static mut SCANOUT: ScanOutState = ScanOutState::empty();
static SCANOUT_READY: AtomicBool = AtomicBool::new(false);
static SCANOUT_PIO1: StaticCell<Pio<'static, peripherals::PIO1>> = StaticCell::new();
static SCANOUT_PIO2: StaticCell<Pio<'static, peripherals::PIO2>> = StaticCell::new();

static mut BOUNCE0: [u16; BOUNCE_BUFFER_PIXELS] = [0; BOUNCE_BUFFER_PIXELS];
static mut BOUNCE1: [u16; BOUNCE_BUFFER_PIXELS] = [0; BOUNCE_BUFFER_PIXELS];

#[inline(always)]
unsafe fn scanout_mut() -> *mut ScanOutState {
    core::ptr::addr_of_mut!(SCANOUT)
}

pub struct ScanOutDmaHandler;

impl Handler<embassy_rp::interrupt::typelevel::DMA_IRQ_1> for ScanOutDmaHandler {
    unsafe fn on_interrupt() {
        let state = unsafe { &mut *scanout_mut() };
        let ch = state.dma_channel as usize;
        let ints = pac::DMA.ints(1).read();
        if ints & (1 << ch) == 0 {
            return;
        }
        pac::DMA.ints(1).write_value(1 << ch);
        dma_complete_handler(state);
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
fn apply_wait_pin_map(pio: pac::pio::Pio, sm: usize) {
    let sm = pio.sm(sm);
    sm.pinctrl().write(|w| w.set_in_base(16));
    sm.shiftctrl().modify(|w| w.set_in_count(8));
}

fn rgb_dma_treq(sm: u8) -> pac::dma::vals::TreqSel {
    pac::dma::vals::TreqSel::from(2 * 8 + sm)
}

fn dma_start_read(channel: u8, rgb_sm: u8, src: *const u16, count: usize) {
    let ch = pac::DMA.ch(channel as _);
    ch.read_addr().write_value(src as u32);
    ch.write_addr()
        .write_value(pac::PIO2.txf(rgb_sm as _).as_ptr() as u32);
    ch.trans_count().write(|w| {
        w.set_mode(0.into());
        w.set_count(count as u32);
    });
    ch.ctrl_trig().write(|w| {
        w.set_treq_sel(rgb_dma_treq(rgb_sm));
        w.set_data_size(pac::dma::vals::DataSize::SizeHalfword);
        w.set_incr_read(true);
        w.set_incr_write(false);
        w.set_chain_to(channel);
        w.set_bswap(false);
        w.set_en(true);
    });
}

#[inline(always)]
unsafe fn bounce0_ptr() -> *mut u16 {
    core::ptr::addr_of_mut!(BOUNCE0).cast::<u16>()
}

#[inline(always)]
unsafe fn bounce1_ptr() -> *mut u16 {
    core::ptr::addr_of_mut!(BOUNCE1).cast::<u16>()
}

fn dma_complete_handler(state: &mut ScanOutState) {
    if state.framebuffer.is_null() {
        return;
    }

    let next_index = (state.transfer_index + 1) % BOUNCE_BUFFER_COUNT;
    let chunk = next_index as usize * BOUNCE_BUFFER_PIXELS;
    let transfer_src = unsafe { state.framebuffer.add(chunk) };

    let dma_buf = if next_index % 2 == 0 {
        unsafe { bounce0_ptr() }
    } else {
        unsafe { bounce1_ptr() }
    };

    unsafe {
        ptr::copy_nonoverlapping(transfer_src, dma_buf, BOUNCE_BUFFER_PIXELS);
    }
    dma_start_read(state.dma_channel, PIO2_RGB_SM, dma_buf, BOUNCE_BUFFER_PIXELS);
    state.transfer_index = next_index;
}

fn prime_scanout_dma(state: &mut ScanOutState) {
    unsafe {
        ptr::copy_nonoverlapping(state.framebuffer, bounce0_ptr(), BOUNCE_BUFFER_PIXELS);
        ptr::copy_nonoverlapping(
            state.framebuffer.add(BOUNCE_BUFFER_PIXELS),
            bounce1_ptr(),
            BOUNCE_BUFFER_PIXELS,
        );
    }
    state.transfer_index = 0;
    dma_start_read(
        state.dma_channel,
        PIO2_RGB_SM,
        unsafe { bounce0_ptr() },
        BOUNCE_BUFFER_PIXELS,
    );
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
    _dma: Peri<'static, peripherals::DMA_CH0>,
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
    state.dma_channel = peripherals::DMA_CH0::number();

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

    pac::PIO2.ctrl().modify(|w| w.set_sm_enable(w.sm_enable() | 0b0001));
    pac::PIO1.ctrl().modify(|w| w.set_sm_enable(w.sm_enable() | 0b0111));

    SCANOUT_PIO1.init(pio1_dev);
    SCANOUT_PIO2.init(pio2_dev);

    let ch = peripherals::DMA_CH0::number();
    pac::DMA.inte(1).write(|w| *w = 1 << ch);
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
