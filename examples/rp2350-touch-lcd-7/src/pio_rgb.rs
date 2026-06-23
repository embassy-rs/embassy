//! PIO RGB scan-out for the ST7262 panel (Waveshare `pio_rgb.c` / LVGL C port).
//!
//! Drives DE/HSYNC/VSYNC/PCLK plus 16-bit RGB data from PSRAM framebuffers through
//! PIO1 (sync) and PIO2 (pixel stream + DMA), matching `RP2350-Touch-7-Exp`.

use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use defmt::{info, warn};
use static_cell::StaticCell;
use embassy_rp::clocks::clk_sys_freq;
use embassy_rp::gpio::{Drive, Pull, SlewRate};
use embassy_rp::interrupt::typelevel::{Binding, Handler, Interrupt};
use embassy_rp::dma::ChannelInstance;
use embassy_rp::pio::program::pio_file;
use embassy_rp::pio::{Config, Direction, FifoJoin, Pio, PioPin};
use embassy_rp::{bind_interrupts, pac, peripherals, Peri};
use fixed::types::extra::U8;
use fixed::FixedU32;

use crate::board::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

const PCLK_FREQ: u32 = 16_000_000;
const TRANSFER_LINES: usize = 80;
const TRANSFER_SIZE: usize = DISPLAY_WIDTH * TRANSFER_LINES;
const TRANSFER_INDEX_MAX: u16 = (DISPLAY_WIDTH * DISPLAY_HEIGHT / TRANSFER_SIZE) as u16;
const PIO2_RGB_SM: u8 = 1;

bind_interrupts!(pub struct ScanOutIrqs {
    PIO1_IRQ_0 => embassy_rp::pio::InterruptHandler<peripherals::PIO1>;
    PIO2_IRQ_0 => embassy_rp::pio::InterruptHandler<peripherals::PIO2>;
    DMA_IRQ_0 => ScanOutDmaHandler;
});

struct ScanOutState {
    transfer_size: usize,
    framebuffer1: *mut u16,
    framebuffer2: *mut u16,
    transfer_buffer1: *mut u16,
    transfer_buffer2: *mut u16,
    active_framebuffer: *mut u16,
    change_framebuffer_flag: bool,
    transfer_index: u16,
    dma_channel: u8,
    flush_pending: bool,
    #[cfg(feature = "oxivgl")]
    flush_disp: *mut oxivgl_sys::lv_display_t,
}

impl ScanOutState {
    const fn empty() -> Self {
        Self {
            transfer_size: 0,
            framebuffer1: ptr::null_mut(),
            framebuffer2: ptr::null_mut(),
            transfer_buffer1: ptr::null_mut(),
            transfer_buffer2: ptr::null_mut(),
            active_framebuffer: ptr::null_mut(),
            change_framebuffer_flag: false,
            transfer_index: 0,
            dma_channel: 0,
            flush_pending: false,
            #[cfg(feature = "oxivgl")]
            flush_disp: ptr::null_mut(),
        }
    }
}

static mut SCANOUT: ScanOutState = ScanOutState::empty();
static SCANOUT_READY: AtomicBool = AtomicBool::new(false);
static SCANOUT_PIO1: StaticCell<Pio<'static, peripherals::PIO1>> = StaticCell::new();
static SCANOUT_PIO2: StaticCell<Pio<'static, peripherals::PIO2>> = StaticCell::new();

#[cfg(feature = "oxivgl")]
static FLUSH_DISP: AtomicPtr<oxivgl_sys::lv_display_t> = AtomicPtr::new(ptr::null_mut());

#[inline(always)]
unsafe fn scanout_mut() -> *mut ScanOutState {
    core::ptr::addr_of_mut!(SCANOUT)
}

#[inline(always)]
unsafe fn scanout_ref() -> *const ScanOutState {
    core::ptr::addr_of!(SCANOUT)
}

static FB0: AtomicPtr<u16> = AtomicPtr::new(ptr::null_mut());
static FB1: AtomicPtr<u16> = AtomicPtr::new(ptr::null_mut());

/// DMA completion ISR — reprograms the RGB pixel stream (see Waveshare `pio_rgb.c`).
pub struct ScanOutDmaHandler;

impl Handler<embassy_rp::interrupt::typelevel::DMA_IRQ_0> for ScanOutDmaHandler {
    unsafe fn on_interrupt() {
        let state = unsafe { &mut *scanout_mut() };
        let ch = state.dma_channel as usize;
        let ints = pac::DMA.ints(0).read();
        if ints & (1 << ch) == 0 {
            return;
        }
        pac::DMA.ints(0).write_value(1 << ch);
        dma_complete_handler(state);
    }
}

fn pio_freq_divider() -> FixedU32<U8> {
    let sys_clk = clk_sys_freq();
    let div = sys_clk as f32 / (PCLK_FREQ as f32 * 2.0);
    FixedU32::from_num(div)
}

fn set_gpio_base(pio: pac::pio::Pio) {
    pio.gpiobase().write(|w| w.set_gpiobase(true));
}

fn apply_wait_pin_map(pio: pac::pio::Pio, sm: usize) {
    let sm = pio.sm(sm);
    // IN pin 4..7 → GPIO 20..23 (DE/VSYNC/HSYNC/PCLK) with gpio_base=16.
    sm.pinctrl().write(|w| w.set_in_base(16));
    sm.shiftctrl().modify(|w| w.set_in_count(8));
}

fn prime_scanout_dma(state: &mut ScanOutState) {
    unsafe {
        ptr::copy_nonoverlapping(
            state.active_framebuffer,
            state.transfer_buffer1,
            state.transfer_size,
        );
    }
    state.transfer_index = 0;
    dma_start_read(
        state.dma_channel,
        PIO2_RGB_SM,
        state.transfer_buffer1,
        state.transfer_size,
    );
}

fn rgb_dma_treq(sm: u8) -> pac::dma::vals::TreqSel {
    // PIO2 instance number is 2.
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

fn dma_complete_handler(state: &mut ScanOutState) {
    if state.transfer_buffer1.is_null() || state.transfer_buffer2.is_null() {
        dma_start_read(
            state.dma_channel,
            PIO2_RGB_SM,
            state.active_framebuffer,
            DISPLAY_WIDTH * DISPLAY_HEIGHT,
        );
        return;
    }

    let next_index = (state.transfer_index + 1) % TRANSFER_INDEX_MAX;
    let chunk = next_index as usize * state.transfer_size;
    let transfer_src = unsafe { state.active_framebuffer.add(chunk) };

    let dma_buf = if next_index % 2 == 0 {
        state.transfer_buffer1
    } else {
        state.transfer_buffer2
    };

    unsafe {
        ptr::copy_nonoverlapping(transfer_src, dma_buf, state.transfer_size);
    }
    dma_start_read(state.dma_channel, PIO2_RGB_SM, dma_buf, state.transfer_size);
    state.transfer_index = next_index;

    if state.change_framebuffer_flag && state.transfer_index == TRANSFER_INDEX_MAX - 1 {
        state.change_framebuffer_flag = false;
        state.active_framebuffer = if state.active_framebuffer == state.framebuffer1 {
            state.framebuffer2
        } else {
            state.framebuffer1
        };
        #[cfg(feature = "oxivgl")]
        if state.flush_pending && !state.flush_disp.is_null() {
            state.flush_pending = false;
            FLUSH_DISP.store(state.flush_disp, Ordering::Release);
            state.flush_disp = ptr::null_mut();
        }
    }
}

/// Call from the LVGL task after DMA scan-out completes (not from the DMA ISR).
#[cfg(feature = "oxivgl")]
pub fn poll_flush_ready() {
    let disp = FLUSH_DISP.swap(ptr::null_mut(), Ordering::AcqRel);
    if !disp.is_null() {
        unsafe {
            oxivgl_sys::lv_display_flush_ready(disp);
        }
    }
}

/// Register two full-screen RGB565 buffers (typically in PSRAM).
pub fn bind_framebuffers(fb0: *mut u16, fb1: *mut u16) {
    FB0.store(fb0, Ordering::Release);
    FB1.store(fb1, Ordering::Release);
    if SCANOUT_READY.load(Ordering::Acquire) {
        let state = unsafe { &mut *scanout_mut() };
        state.framebuffer1 = fb0;
        state.framebuffer2 = fb1;
        state.active_framebuffer = fb0;
    }
}

/// Register PSRAM/SRAM DMA staging buffers (`width × 80` RGB565 pixels each).
pub fn bind_transfer_buffers(tb0: *mut u16, tb1: *mut u16) {
    let state = unsafe { &mut *scanout_mut() };
    state.transfer_buffer1 = tb0;
    state.transfer_buffer2 = tb1;
}

/// LVGL flush completion: swap the scanned-out framebuffer when the DMA frame finishes.
#[cfg(feature = "oxivgl")]
pub fn request_swap(flush_disp: *mut oxivgl_sys::lv_display_t) {
    if !SCANOUT_READY.load(Ordering::Acquire) {
        return;
    }
    let state = unsafe { &mut *scanout_mut() };
    state.change_framebuffer_flag = true;
    state.flush_pending = !flush_disp.is_null();
    state.flush_disp = flush_disp;
}

/// Request a framebuffer swap without LVGL (non-OxivGL builds).
#[cfg(not(feature = "oxivgl"))]
pub fn request_swap() {
    if !SCANOUT_READY.load(Ordering::Acquire) {
        return;
    }
    let state = unsafe { &mut *scanout_mut() };
    state.change_framebuffer_flag = true;
}

/// Pointer to the buffer LVGL should draw into (inactive framebuffer).
pub fn draw_ptr() -> *mut u16 {
    if !SCANOUT_READY.load(Ordering::Acquire) {
        return FB0.load(Ordering::Acquire);
    }
    let state = unsafe { &*scanout_ref() };
    if state.active_framebuffer == state.framebuffer1 {
        state.framebuffer2
    } else {
        state.framebuffer1
    }
}

/// Pointer to the buffer currently scanned out to the panel.
pub fn front_ptr() -> *mut u16 {
    if !SCANOUT_READY.load(Ordering::Acquire) {
        return FB0.load(Ordering::Acquire);
    }
    unsafe { (*scanout_ref()).active_framebuffer }
}

/// Back buffer for partial-render fallback (same as [`draw_ptr`]).
pub fn back_ptr() -> *mut u16 {
    draw_ptr()
}

/// Swap front/back after LVGL finished a frame (legacy name).
#[cfg(feature = "oxivgl")]
pub fn present_swap() {
    request_swap(ptr::null_mut());
}

/// Start PIO RGB scan-out (PIO1 sync + PIO2 pixel/DMA).
#[allow(clippy::too_many_arguments)]
pub fn init_scanout(
    pio1: Peri<'static, peripherals::PIO1>,
    pio2: Peri<'static, peripherals::PIO2>,
    _dma: Peri<'static, peripherals::DMA_CH0>,
    _irq: impl Binding<embassy_rp::interrupt::typelevel::DMA_IRQ_0, ScanOutDmaHandler> + 'static,
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
) {
    if SCANOUT_READY.swap(true, Ordering::AcqRel) {
        return;
    }

    let fb0 = FB0.load(Ordering::Acquire);
    let fb1 = FB1.load(Ordering::Acquire);
    if fb0.is_null() || fb1.is_null() {
        SCANOUT_READY.store(false, Ordering::Release);
        warn!("PIO RGB: framebuffers not bound");
        return;
    }

    let state = unsafe { &mut *scanout_mut() };
    state.transfer_size = TRANSFER_SIZE;
    state.framebuffer1 = fb0;
    state.framebuffer2 = fb1;
    state.active_framebuffer = fb0;
    state.dma_channel = peripherals::DMA_CH0::number();

    let pio_freq = pio_freq_divider();

    let mut pio1_dev = Pio::new(pio1, ScanOutIrqs);
    let mut pio2_dev = Pio::new(pio2, ScanOutIrqs);

    set_gpio_base(pac::PIO1);
    set_gpio_base(pac::PIO2);

    let hsync_file = pio_file!("pio/pio_rgb.pio", select_program("hsync"), options(max_program_size = 64));
    let vsync_file = pio_file!("pio/pio_rgb.pio", select_program("vsync"), options(max_program_size = 64));
    let rgb_de_file = pio_file!("pio/pio_rgb.pio", select_program("rgb_de"), options(max_program_size = 64));
    let rgb_file = pio_file!("pio/pio_rgb.pio", select_program("rgb"), options(max_program_size = 64));

    let hsync_loaded = pio1_dev.common.load_program(&hsync_file.program);
    let vsync_loaded = pio1_dev.common.load_program(&vsync_file.program);
    let rgb_de_loaded = pio2_dev.common.load_program(&rgb_de_file.program);
    let rgb_loaded = pio2_dev.common.load_program(&rgb_file.program);

    let pin_hsync = pio1_dev.common.make_pio_pin(hsync);
    let pin_pclk = pio1_dev.common.make_pio_pin(pclk);
    let pin_vsync = pio1_dev.common.make_pio_pin(vsync);
    let pin_de = pio2_dev.common.make_pio_pin(de);

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
        &mut d0, &mut d1, &mut d2, &mut d3, &mut d4, &mut d5, &mut d6, &mut d7, &mut d8, &mut d9, &mut d10,
        &mut d11, &mut d12, &mut d13, &mut d14, &mut d15,
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
    hsync_cfg.clock_divider = pio_freq;
    hsync_cfg.use_program(&hsync_loaded, &[&pin_hsync, &pin_pclk]);

    let mut vsync_cfg = Config::default();
    vsync_cfg.fifo_join = FifoJoin::TxOnly;
    vsync_cfg.use_program(&vsync_loaded, &[&pin_vsync]);

    let mut rgb_de_cfg = Config::default();
    rgb_de_cfg.fifo_join = FifoJoin::TxOnly;
    rgb_de_cfg.use_program(&rgb_de_loaded, &[&pin_de]);

    let mut rgb_cfg = Config::default();
    rgb_cfg.fifo_join = FifoJoin::TxOnly;
    rgb_cfg.clock_divider = pio_freq;
    rgb_cfg.use_program(&rgb_loaded, &[]);
    rgb_cfg.set_out_pins(&data_pins);

    pio1_dev.sm0.set_config(&hsync_cfg);
    pio1_dev.sm1.set_config(&vsync_cfg);
    pio2_dev.sm0.set_config(&rgb_de_cfg);
    pio2_dev.sm1.set_config(&rgb_cfg);
    apply_wait_pin_map(pac::PIO2, 0);
    apply_wait_pin_map(pac::PIO2, 1);

    pio1_dev.sm0.set_pin_dirs(Direction::Out, &[&pin_hsync, &pin_pclk]);
    pio1_dev.sm1.set_pin_dirs(Direction::Out, &[&pin_vsync]);
    pio2_dev.sm0.set_pin_dirs(Direction::Out, &[&pin_de]);
    pio2_dev.sm1.set_pin_dirs(Direction::Out, &data_pins);

    pio1_dev.sm0.tx().push(DISPLAY_WIDTH as u32 - 1);
    pio1_dev.sm1.tx().push(DISPLAY_HEIGHT as u32 - 1);
    pio2_dev.sm0.tx().push(DISPLAY_HEIGHT as u32 - 1);
    pio2_dev.sm1.tx().push(DISPLAY_WIDTH as u32 - 1);

    // Start all scan-out state machines together (Waveshare `pio_enable_sm_mask_in_sync`).
    pac::PIO1.ctrl().modify(|w| w.set_sm_enable(w.sm_enable() | 0b0011));
    pac::PIO2.ctrl().modify(|w| w.set_sm_enable(w.sm_enable() | 0b0011));

    // Keep PIO peripherals alive — dropping `Pio` disables all state machines.
    SCANOUT_PIO1.init(pio1_dev);
    SCANOUT_PIO2.init(pio2_dev);

    let ch = peripherals::DMA_CH0::number();
    pac::DMA.inte(0).write(|w| *w = 1 << ch);
    unsafe { embassy_rp::interrupt::typelevel::DMA_IRQ_0::enable() };

    prime_scanout_dma(state);
    info!(
        "PIO RGB scan-out started ({}x{} @ {} MHz pclk)",
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        PCLK_FREQ / 1_000_000
    );
}

/// Partial blit from LVGL flush into the back buffer (partial render fallback).
pub fn blit_rgb565(back: *mut u16, x1: i32, y1: i32, w: usize, h: usize, stride: usize, src: &[u8]) {
    if back.is_null() {
        return;
    }
    let row_bytes = w * 2;
    for row in 0..h {
        let y = y1 as usize + row;
        let dst_off = y * stride + x1 as usize;
        let src_off = row * row_bytes;
        if src_off + row_bytes > src.len() {
            break;
        }
        unsafe {
            ptr::copy_nonoverlapping(
                src.as_ptr().add(src_off),
                back.add(dst_off).cast::<u8>(),
                row_bytes,
            );
        }
    }
}
