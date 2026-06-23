//! PIO RGB scan-out for the gen4-RP2350-70CT 7" panel (port of the 4D Systems
//! Graphics4D `rgb70.pio` / Waveshare LVGL C path).
//!
//! Drives DE/HSYNC/VSYNC/PCLK plus 16-bit RGB data from PSRAM framebuffers through
//! PIO1 (sync) and PIO2 (pixel stream + DMA). Sync inputs are GPIO18-21 and the
//! 16 RGB565 data lines are GPIO22-37 (DATA0 = blue LSB).

use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use defmt::{info, warn};
use embassy_rp::clocks::clk_sys_freq;
use embassy_rp::dma::ChannelInstance;
use embassy_rp::gpio::{Drive, Pull, SlewRate};
use embassy_rp::interrupt::typelevel::{Binding, Handler, Interrupt};
use embassy_rp::pio::program::pio_file;
use embassy_rp::pio::{Config, Direction, FifoJoin, Pio, PioPin};
use embassy_rp::{Peri, bind_interrupts, pac, peripherals};
use fixed::FixedU32;
use fixed::types::extra::U8;
use static_cell::StaticCell;

use crate::board::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

// gen4-RP2350-70CT timing-SM clock. The vendor `Graphics4D` drives the
// HSYNC/VSYNC/DE generators at sys_clk / 36 MHz and the RGB pixel SM at
// `sync_div / 7 * 2.7` (see `rgb70.pio` init). `LCD_CLK_FREQ` (25 MHz) is the
// resulting nominal panel pixel clock.
const SYNC_PIO_FREQ: u32 = 36_000_000;
pub const TRANSFER_LINES: usize = 60;
const TRANSFER_SIZE: usize = DISPLAY_WIDTH * TRANSFER_LINES;
const TRANSFER_INDEX_MAX: u16 = (DISPLAY_WIDTH * DISPLAY_HEIGHT / TRANSFER_SIZE) as u16;
const PIO2_RGB_SM: u8 = 0;

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
            flush_disp: ptr::null_mut(),
        }
    }
}

static mut SCANOUT: ScanOutState = ScanOutState::empty();
static SCANOUT_READY: AtomicBool = AtomicBool::new(false);
static SCANOUT_PIO1: StaticCell<Pio<'static, peripherals::PIO1>> = StaticCell::new();
static SCANOUT_PIO2: StaticCell<Pio<'static, peripherals::PIO2>> = StaticCell::new();

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
    FixedU32::from_num(sys_clk as f32 / SYNC_PIO_FREQ as f32)
}

fn rgb_freq_divider() -> FixedU32<U8> {
    let sync_div = clk_sys_freq() as f32 / SYNC_PIO_FREQ as f32;
    FixedU32::from_num(sync_div / 7.0 * 2.7)
}

fn set_gpio_base(pio: pac::pio::Pio) {
    pio.gpiobase().write(|w| w.set_gpiobase(true));
}

fn apply_wait_pin_map(pio: pac::pio::Pio, sm: usize) {
    let sm = pio.sm(sm);
    // gpio_base=16 → PIO pin index = GPIO - 16. gen4 sync pins:
    //   DE=GPIO18→2, VSYNC=GPIO19→3, HSYNC=GPIO20→4, PCLK=GPIO21→5.
    // Preserve OUT_BASE/OUT_COUNT from Config::set_out_pins; overwriting the
    // full PINCTRL register here would move RGB data away from GPIO22..37.
    sm.pinctrl().modify(|w| w.set_in_base(0));
    sm.shiftctrl().modify(|w| w.set_in_count(8));
}

fn transfer_buffer_for(state: &ScanOutState, index: u16) -> *mut u16 {
    if index & 1 == 0 {
        state.transfer_buffer1
    } else {
        state.transfer_buffer2
    }
}

fn copy_transfer_chunk(state: &ScanOutState, index: u16) {
    let chunk = index as usize * state.transfer_size;
    let transfer_src = unsafe { state.active_framebuffer.add(chunk) };
    let transfer_dst = transfer_buffer_for(state, index);
    unsafe {
        ptr::copy_nonoverlapping(transfer_src, transfer_dst, state.transfer_size);
    }
}

fn prime_scanout_dma(state: &mut ScanOutState) {
    copy_transfer_chunk(state, 0);
    copy_transfer_chunk(state, 1);
    state.transfer_index = 0;
    dma_start_read(
        state.dma_channel,
        PIO2_RGB_SM,
        transfer_buffer_for(state, 0),
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
    ch.write_addr().write_value(pac::PIO2.txf(rgb_sm as _).as_ptr() as u32);
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
    let dma_buf = transfer_buffer_for(state, next_index);
    dma_start_read(state.dma_channel, PIO2_RGB_SM, dma_buf, state.transfer_size);
    state.transfer_index = next_index;

    if state.change_framebuffer_flag && state.transfer_index == TRANSFER_INDEX_MAX - 1 {
        state.change_framebuffer_flag = false;
        state.active_framebuffer = if state.active_framebuffer == state.framebuffer1 {
            state.framebuffer2
        } else {
            state.framebuffer1
        };
        if state.flush_pending && !state.flush_disp.is_null() {
            state.flush_pending = false;
            FLUSH_DISP.store(state.flush_disp, Ordering::Release);
            state.flush_disp = ptr::null_mut();
        }
    }

    let fill_index = (state.transfer_index + 1) % TRANSFER_INDEX_MAX;
    copy_transfer_chunk(state, fill_index);
}

/// Call from the LVGL task after DMA scan-out completes (not from the DMA ISR).
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

/// Register PSRAM/SRAM DMA staging buffers (`width × TRANSFER_LINES` RGB565 pixels each).
pub fn bind_transfer_buffers(tb0: *mut u16, tb1: *mut u16) {
    let state = unsafe { &mut *scanout_mut() };
    state.transfer_buffer1 = tb0;
    state.transfer_buffer2 = tb1;
}

/// LVGL flush completion: swap the scanned-out framebuffer when the DMA frame finishes.
pub fn request_swap(flush_disp: *mut oxivgl_sys::lv_display_t) {
    if !SCANOUT_READY.load(Ordering::Acquire) {
        return;
    }
    let state = unsafe { &mut *scanout_mut() };
    state.change_framebuffer_flag = true;
    state.flush_pending = !flush_disp.is_null();
    state.flush_disp = flush_disp;
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

    let hsync_file = pio_file!(
        "pio/pio_rgb.pio",
        select_program("hsync"),
        options(max_program_size = 64)
    );
    let vsync_file = pio_file!(
        "pio/pio_rgb.pio",
        select_program("vsync"),
        options(max_program_size = 64)
    );
    let de_file = pio_file!(
        "pio/pio_rgb.pio",
        select_program("rgb_frame"),
        options(max_program_size = 64)
    );
    let rgb_file = pio_file!("pio/pio_rgb.pio", select_program("rgb"), options(max_program_size = 64));

    let hsync_loaded = pio1_dev.common.load_program(&hsync_file.program);
    let vsync_loaded = pio1_dev.common.load_program(&vsync_file.program);
    let de_loaded = pio1_dev.common.load_program(&de_file.program);
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
        &mut d0, &mut d1, &mut d2, &mut d3, &mut d4, &mut d5, &mut d6, &mut d7, &mut d8, &mut d9, &mut d10, &mut d11,
        &mut d12, &mut d13, &mut d14, &mut d15,
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
    vsync_cfg.clock_divider = pio_freq;
    vsync_cfg.use_program(&vsync_loaded, &[&pin_vsync]);
    vsync_cfg.set_set_pins(&[&pin_vsync]);

    let mut de_cfg = Config::default();
    de_cfg.fifo_join = FifoJoin::TxOnly;
    de_cfg.clock_divider = pio_freq;
    de_cfg.use_program(&de_loaded, &[&pin_de]);

    let mut rgb_cfg = Config::default();
    rgb_cfg.fifo_join = FifoJoin::TxOnly;
    rgb_cfg.clock_divider = rgb_freq_divider();
    rgb_cfg.use_program(&rgb_loaded, &[]);
    rgb_cfg.set_out_pins(&data_pins);

    // pio1: sm0=hsync(+pclk), sm1=vsync, sm2=DE; pio2: sm0=rgb pixel stream.
    pio1_dev.sm0.set_config(&hsync_cfg);
    pio1_dev.sm1.set_config(&vsync_cfg);
    pio1_dev.sm2.set_config(&de_cfg);
    pio2_dev.sm0.set_config(&rgb_cfg);
    apply_wait_pin_map(pac::PIO2, 0);

    pio1_dev.sm0.set_pin_dirs(Direction::Out, &[&pin_hsync, &pin_pclk]);
    pio1_dev.sm1.set_pin_dirs(Direction::Out, &[&pin_vsync]);
    pio1_dev.sm2.set_pin_dirs(Direction::Out, &[&pin_de]);
    pio2_dev.sm0.set_pin_dirs(Direction::Out, &data_pins);

    pio1_dev.sm0.tx().push(DISPLAY_WIDTH as u32 - 1);
    pio1_dev.sm1.tx().push(DISPLAY_HEIGHT as u32 - 1);
    pio1_dev.sm2.tx().push((DISPLAY_WIDTH as u32 * 2) - 1);
    pio2_dev.sm0.tx().push(DISPLAY_WIDTH as u32 - 1);

    // Park the RGB pixel SM on its pin waits first, then start the PIO1 sync
    // generators (hsync/vsync/DE) together (vendor `pio_enable_sm_mask_in_sync`).
    pac::PIO2.ctrl().modify(|w| w.set_sm_enable(w.sm_enable() | 0b0001));
    pac::PIO1.ctrl().modify(|w| w.set_sm_enable(w.sm_enable() | 0b0111));

    // Keep PIO peripherals alive — dropping `Pio` disables all state machines.
    SCANOUT_PIO1.init(pio1_dev);
    SCANOUT_PIO2.init(pio2_dev);

    let ch = peripherals::DMA_CH0::number();
    pac::DMA.inte(0).write(|w| *w = 1 << ch);
    unsafe { embassy_rp::interrupt::typelevel::DMA_IRQ_0::enable() };

    prime_scanout_dma(state);
    info!(
        "PIO RGB scan-out started ({}x{}, sync SM @ {} MHz)",
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        SYNC_PIO_FREQ / 1_000_000
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
            ptr::copy_nonoverlapping(src.as_ptr().add(src_off), back.add(dst_off).cast::<u8>(), row_bytes);
        }
    }
}
