#![no_std]
#![no_main]

//! Camera capture → SD card example for the STM32N6570-DK + MB1854.
//!
//! Streams the IMX335 → DCMIPP → LTDC pipeline (same as `bin/camera.rs`)
//! and on every press of the **Tamper** button (B4 / PE0) snapshots the
//! current frame into a `IMGNNNN.BMP` file on a FAT32-formatted microSD
//! card. The next-file index is recovered from the card on boot by
//! scanning the root directory, so reboots don't overwrite earlier saves.

#[path = "../imx335.rs"]
mod imx335;

use core::cell::RefCell;

use defmt::{error, info, unwrap};
use embassy_executor::Spawner;
use embassy_futures::block_on;
use embassy_futures::select::{Either3, select3};
use embassy_stm32::csi::{self, Csi, LaneCount};
use embassy_stm32::dcmipp::{
    self, BayerPattern, ChannelGains, Dcmipp, DownsizeConfig, InputSource, Pipe1, Pipe1Config,
    PixelFormat as DcmippPixelFormat,
};
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::i2c::I2c;
use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig, PixelFormat};
use embassy_stm32::peripherals::DCMIPP;
use embassy_stm32::rcc::mux::{Dcmippsel, Ltdcsel};
use embassy_stm32::rcc::{CpuClk, IcConfig, Icint, Icsel, Pll, Plldivm, Pllpdiv, Pllsel, SysClk};
use embassy_stm32::sdmmc::Sdmmc;
use embassy_stm32::sdmmc::sd::{Addressable, Card, CmdBlock, DataBlock, StorageDevice};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, interrupt, pac, peripherals};
use embassy_time::Timer;
use embedded_sdmmc::{Block, BlockCount, BlockDevice, BlockIdx, Mode, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use {defmt_rtt as _, panic_probe as _};

#[path = "../rk050hr18c.rs"]
mod rk050hr18c;
use rk050hr18c::{HEIGHT, LTDC_CONFIG, Rk050Hr18c, WIDTH};

use crate::imx335::Imx335;

bind_interrupts!(struct Irqs {
    LTDC_LO => ltdc::InterruptHandler<peripherals::LTDC>;
    CSI    => csi::InterruptHandler<peripherals::CSI>;
    DCMIPP => dcmipp::InterruptHandler<peripherals::DCMIPP>;
    EXTI13 => exti::InterruptHandler<interrupt::typelevel::EXTI13>;
    EXTI0  => exti::InterruptHandler<interrupt::typelevel::EXTI0>;
    SDMMC2 => embassy_stm32::sdmmc::InterruptHandler<peripherals::SDMMC2>;
});

const FB0_BASE: usize = 0x3410_0000;
const FB1_BASE: usize = 0x3420_0000;
const FB_PITCH_BYTES: u16 = WIDTH as u16 * 2;

const SENSOR_W: u16 = 2592;
const SENSOR_H: u16 = 1944;
const CSI_RATE_MBPS: u32 = 1600;

const STATIC_WB: ChannelGains = ChannelGains { r: 1.6, g: 1.0, b: 1.4 };

// Auto-exposure tuning. Brightness ∝ integration = (VMAX - shutter), so
// AE works in exposure-line space and converts back to a shutter value.
const VMAX_LINES: u32 = 4500;
const MIN_SHUTTER: u32 = 10;
const MAX_SHUTTER: u32 = 4400;
const MIN_GAIN_DB_X10: u16 = 0;
const MAX_GAIN_DB_X10: u16 = 360;
// Calibrated empirically — read the `ae:` log at a known-good exposure.
const TARGET_LUMA: u32 = 50_000;
// Run AE every N frames (≈30 Hz / N). The deadband around TARGET_LUMA
// suppresses I2C traffic at idle, so we can sample faster without thrash.
const AE_PERIOD_FRAMES: u32 = 3;

#[derive(Clone, Copy, defmt::Format)]
enum AutoMode {
    Off,
    Wb,
    Ae,
    Both,
}

impl AutoMode {
    fn next(self) -> Self {
        match self {
            Self::Off => Self::Wb,
            Self::Wb => Self::Ae,
            Self::Ae => Self::Both,
            Self::Both => Self::Off,
        }
    }
    fn wb(self) -> bool {
        matches!(self, Self::Wb | Self::Both)
    }
    fn ae(self) -> bool {
        matches!(self, Self::Ae | Self::Both)
    }
}

#[derive(Clone, Copy)]
struct Exposure {
    shutter: u32,
    gain_db_x10: u16,
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(rcc_config());
    info!("stm32n6 camera_to_sd starting");

    enable_all_sram();
    promote_axi_masters_to_secure();

    // ---- Camera sensor ----
    let mut i2c_cfg = embassy_stm32::i2c::Config::default();
    i2c_cfg.frequency = Hertz::khz(400);
    let i2c = I2c::new_blocking(p.I2C1, p.PH9, p.PC1, i2c_cfg);
    let pwr_en = Output::new(p.PC8, Level::Low, Speed::Low);
    let nrst = Output::new(p.PD2, Level::Low, Speed::Low);
    let mut cam = Imx335::new(i2c, pwr_en, nrst);
    unwrap!(cam.power_on().await);
    unwrap!(cam.init().await);
    unwrap!(cam.set_gain_db_x10(150));
    unwrap!(cam.set_shutter_lines(2000));
    info!("imx335: ready");

    // ---- CSI + DCMIPP ----
    let mut csi = Csi::new(p.CSI, Irqs, csi::Config::new(LaneCount::Two, CSI_RATE_MBPS));

    let dcmipp = Dcmipp::new(p.DCMIPP, Irqs);
    let (_pipe0, mut pipe1, _pipe2) = dcmipp.split();
    let mut p1cfg = Pipe1Config::new(InputSource::Csi, DcmippPixelFormat::Rgb565, FB_PITCH_BYTES);
    p1cfg.demosaic = Some(BayerPattern::Rggb);
    p1cfg.downsize = Some(DownsizeConfig {
        input: (SENSOR_W, SENSOR_H),
        output: (WIDTH as u16, HEIGHT as u16),
    });
    pipe1.configure(&p1cfg);
    pipe1.set_color_gains(STATIC_WB);
    pipe1.enable_rgb_stats(WIDTH as u16, HEIGHT as u16);

    // ---- LCD ----
    let mut panel = Rk050Hr18c::new(p.PE1, p.PQ3, p.PQ6);
    panel.power_on().await;
    let mut ltdc = Ltdc::<_, ltdc::Rgb888>::new_with_pins(
        p.LTDC, Irqs, p.PB13, p.PB14, p.PE11, p.PG13, p.PG15, p.PA7, p.PB2, p.PG6, p.PH3, p.PH6, p.PA8, p.PA2, p.PG12,
        p.PG1, p.PA1, p.PA0, p.PB15, p.PB12, p.PB11, p.PG8, p.PG0, p.PD9, p.PD15, p.PB4, p.PH4, p.PA15, p.PG11, p.PD8,
    );
    ltdc.init(&LTDC_CONFIG);
    let layer_config = LtdcLayerConfig {
        pixel_format: PixelFormat::RGB565,
        layer: LtdcLayer::Layer1,
        window_x0: 0,
        window_x1: WIDTH,
        window_y0: 0,
        window_y1: HEIGHT,
    };
    ltdc.init_layer(&layer_config, None);
    ltdc.init_buffer(LtdcLayer::Layer1, FB0_BASE as *const ());
    pac::LTDC.srcr().write(|w| w.set_imr(pac::ltdc::vals::Imr::Reload));

    // ---- SD card ----
    // Bump the per-block data-transfer timeout. The default of 5e6 cycles
    // at 24 MHz SD clock is ~200 ms; large SDXC cards can stall for ≥1 s
    // during internal garbage collection. 200 M cycles ≈ 8 s — generous
    // enough that we get an error only when the card is genuinely dead.
    let mut sd_cfg = embassy_stm32::sdmmc::Config::default();
    sd_cfg.data_transfer_timeout = 200_000_000;
    let mut sd = Sdmmc::new_4bit(p.SDMMC2, Irqs, p.PC2, p.PC3, p.PC4, p.PC5, p.PC0, p.PE4, sd_cfg);
    let mut cmd_block = CmdBlock::new();
    let mut sd_state = match StorageDevice::new_sd_card(&mut sd, &mut cmd_block, Hertz(24_000_000)).await {
        Ok(storage) => {
            info!("sd: card ready, {} blocks", storage.card().size());
            let block_dev = EmbassyBlockDevice {
                inner: RefCell::new(storage),
            };
            let mut volume_mgr: VolumeManager<_, _> = VolumeManager::new(block_dev, FixedTime);
            let next_idx = scan_next_index(&mut volume_mgr);
            info!("next image index = {}", next_idx);
            Some((volume_mgr, next_idx))
        }
        Err(e) => {
            info!("sd init failed, save disabled: {:?}", defmt::Debug2Format(&e));
            None
        }
    };

    // ---- Buttons ----
    let mut wb_button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Down, Irqs); // B2 = WB
    let mut save_button = ExtiInput::new(p.PE0, p.EXTI0, Pull::Down, Irqs); // B4 = Tamper = save

    // ---- Start streaming ----
    unsafe {
        pipe1.start_continuous(FB0_BASE as *mut u8, FB1_BASE as *mut u8);
    }
    csi.start();
    unwrap!(cam.start_streaming().await);
    info!("streaming; B4 = save BMP, B2 = cycle mode (Off/Wb/Ae/Both)");

    let mut mode = AutoMode::Off;
    let mut wb = STATIC_WB;
    let mut exposure = Exposure {
        shutter: 2000,
        gain_db_x10: 150,
    };
    let mut last_idx: u8 = 0;
    let mut frames: u32 = 0;

    loop {
        let event = select3(
            pipe1.wait_frame(),
            wb_button.wait_for_rising_edge(),
            save_button.wait_for_rising_edge(),
        )
        .await;

        match event {
            Either3::First(Ok(idx)) => {
                handle_frame(
                    idx,
                    frames,
                    mode,
                    &mut ltdc,
                    &mut pipe1,
                    &mut cam,
                    &mut wb,
                    &mut exposure,
                )
                .await;
                last_idx = idx;
                frames = frames.wrapping_add(1);
            }
            Either3::First(Err(_)) => {}
            Either3::Second(()) => {
                mode = mode.next();
                if !mode.wb() {
                    wb = STATIC_WB;
                    pipe1.set_color_gains(wb);
                }
                info!("mode = {}", mode);
            }
            Either3::Third(()) => {
                handle_save(&mut sd_state, &mut pipe1, last_idx).await;
                Timer::after_millis(150).await; // debounce
            }
        }
    }
}

async fn handle_frame(
    idx: u8,
    frames: u32,
    mode: AutoMode,
    ltdc: &mut Ltdc<'_, peripherals::LTDC, ltdc::Rgb888>,
    pipe1: &mut Pipe1<'_, DCMIPP>,
    cam: &mut Imx335<'_>,
    wb: &mut ChannelGains,
    exposure: &mut Exposure,
) {
    let ptr = if idx == 0 { FB0_BASE } else { FB1_BASE } as *const ();
    unwrap!(ltdc.set_buffer(LtdcLayer::Layer1, ptr).await);

    let (mr, mg, mb) = if mode.wb() || mode.ae() {
        pipe1.read_rgb_means()
    } else {
        (0, 0, 0)
    };
    let luma = (mr + mg + mb) / 3;

    if mode.wb() {
        auto_white_balance(pipe1, wb, mr, mg, mb);
    }
    if mode.ae() && (frames + 1).is_multiple_of(AE_PERIOD_FRAMES) {
        auto_exposure(cam, exposure, luma);
    }
    if (frames + 1).is_multiple_of(30) {
        info!("frame {}", frames + 1);
        if mode.ae() {
            info!(
                "ae: shutter={} gain_db_x10={} luma={}",
                exposure.shutter, exposure.gain_db_x10, luma
            );
        }
    }
}

/// Gray-world auto white balance. Drives R/G/B channel gains so the means
/// converge to a common target via an EMA, with a defensive clamp.
fn auto_white_balance(pipe1: &mut Pipe1<'_, DCMIPP>, wb: &mut ChannelGains, mr: u32, mg: u32, mb: u32) {
    if mr.min(mg).min(mb) == 0 {
        return;
    }
    let target = (mr + mg + mb) / 3;
    let f = |x: u32| (target as f32 / x.max(1) as f32).clamp(0.25, 3.5);
    wb.r = wb.r * 0.85 + f(mr) * 0.15;
    wb.g = wb.g * 0.85 + f(mg) * 0.15;
    wb.b = wb.b * 0.85 + f(mb) * 0.15;
    pipe1.set_color_gains(*wb);
}

/// Auto exposure: drives shutter (primary) and gain (handoff at the rails)
/// toward `TARGET_LUMA`. Skipped within ±10% deadband so a stable scene
/// produces zero I2C traffic.
fn auto_exposure(cam: &mut Imx335<'_>, exp: &mut Exposure, luma: u32) {
    if luma == 0 {
        return;
    }
    let in_deadband = luma >= TARGET_LUMA * 9 / 10 && luma <= TARGET_LUMA * 11 / 10;
    if in_deadband {
        return;
    }

    // Brightness ∝ integration = (VMAX - shutter). Work in exposure-line
    // space, EMA-track toward desired, convert back to a shutter value.
    let cur_int = VMAX_LINES.saturating_sub(exp.shutter);
    let ratio = (TARGET_LUMA as f32 / luma as f32).clamp(0.33, 3.0);
    let desired_int = (cur_int as f32 * ratio) as u32;
    let desired_int = desired_int.clamp(VMAX_LINES - MAX_SHUTTER, VMAX_LINES - MIN_SHUTTER);
    let new_int = (cur_int + desired_int) / 2;
    let new_shutter = VMAX_LINES - new_int;

    // Gain handoff: only step gain when the shutter loop has saturated
    // (or has plenty of headroom) AND the luma is on the right side of
    // target. Wide deadband prevents the two loops from fighting.
    let at_bright_rail = new_shutter <= MIN_SHUTTER + 20;
    let with_headroom = new_shutter > MIN_SHUTTER + 200;
    let new_gain = if at_bright_rail && luma < TARGET_LUMA * 9 / 10 {
        (exp.gain_db_x10 + 10).min(MAX_GAIN_DB_X10)
    } else if with_headroom && luma > TARGET_LUMA * 11 / 10 && exp.gain_db_x10 > MIN_GAIN_DB_X10 {
        exp.gain_db_x10.saturating_sub(10)
    } else {
        exp.gain_db_x10
    };

    // Tolerate the occasional IMX335 I2C NACK rather than panicking.
    if new_shutter != exp.shutter && cam.set_shutter_lines(new_shutter).is_ok() {
        exp.shutter = new_shutter;
    }
    if new_gain != exp.gain_db_x10 && cam.set_gain_db_x10(new_gain).is_ok() {
        exp.gain_db_x10 = new_gain;
    }
}

async fn handle_save<'a, 'b>(
    sd_state: &mut Option<(VolumeManager<EmbassyBlockDevice<'a, 'b>, FixedTime>, u32)>,
    pipe1: &mut Pipe1<'_, DCMIPP>,
    last_idx: u8,
) {
    let Some((volume_mgr, next_idx)) = sd_state.as_mut() else {
        info!("save: no SD card, ignored");
        return;
    };
    info!("save: capturing IMG{:04}.BMP", *next_idx);
    save_current_frame(volume_mgr, pipe1, last_idx, *next_idx).await;
    *next_idx = next_idx.wrapping_add(1);
}

/// Stop the live ping-pong, write the most-recently-filled framebuffer to
/// SD as a 16-bit RGB565 BMP, then resume streaming.
async fn save_current_frame<'a, 'b>(
    volume_mgr: &mut VolumeManager<EmbassyBlockDevice<'a, 'b>, FixedTime>,
    pipe1: &mut Pipe1<'_, DCMIPP>,
    last_idx: u8,
    file_idx: u32,
) {
    pipe1.stop();
    Timer::after_millis(20).await;
    cortex_m::asm::dsb();

    let snap_addr = if last_idx == 0 { FB0_BASE } else { FB1_BASE };
    info!("save: using FB at 0x{:08x}", snap_addr);
    let snap_slice: &[u16] =
        unsafe { core::slice::from_raw_parts(snap_addr as *const u16, WIDTH as usize * HEIGHT as usize) };

    match write_bmp_to_sd(volume_mgr, file_idx, snap_slice) {
        Ok(()) => info!("saved IMG{:04}.BMP", file_idx),
        Err(e) => error!("bmp write failed: {:?}", defmt::Debug2Format(&e)),
    }

    // Resume preview.
    unsafe {
        pipe1.start_continuous(FB0_BASE as *mut u8, FB1_BASE as *mut u8);
    }
}

fn write_bmp_to_sd<'a, 'b>(
    volume_mgr: &mut VolumeManager<EmbassyBlockDevice<'a, 'b>, FixedTime>,
    file_idx: u32,
    pixels: &[u16],
) -> Result<(), embedded_sdmmc::Error<embassy_stm32::sdmmc::Error>> {
    let volume = volume_mgr.open_volume(VolumeIdx(0))?;
    let root = volume.open_root_dir()?;
    let mut name = heapless::String::<13>::new();
    use core::fmt::Write as _;
    let _ = write!(name, "IMG{:04}.BMP", file_idx);
    let file = root.open_file_in_dir(name.as_str(), Mode::ReadWriteCreateOrTruncate)?;

    // 16-bit RGB565 BMP via BI_BITFIELDS (compression=3). The framebuffer
    // bytes go straight to disk — no per-pixel conversion, no extra buffer.
    // Layout: 14 B file header + 40 B DIB header + 12 B channel masks.
    let w = WIDTH as i32;
    let h = HEIGHT as i32;
    let row_bytes = (w as usize) * 2;
    let pixels_size = row_bytes * (h as usize);
    let pixel_offset: u32 = 14 + 40 + 12;
    let file_size: u32 = pixel_offset + pixels_size as u32;

    let mut hdr = [0u8; 14 + 40 + 12];
    hdr[0..2].copy_from_slice(b"BM");
    hdr[2..6].copy_from_slice(&file_size.to_le_bytes());
    hdr[10..14].copy_from_slice(&pixel_offset.to_le_bytes());
    hdr[14..18].copy_from_slice(&40u32.to_le_bytes()); // DIB size
    hdr[18..22].copy_from_slice(&w.to_le_bytes());
    hdr[22..26].copy_from_slice(&(-h).to_le_bytes()); // negative = top-down
    hdr[26..28].copy_from_slice(&1u16.to_le_bytes()); // planes
    hdr[28..30].copy_from_slice(&16u16.to_le_bytes()); // bpp
    hdr[30..34].copy_from_slice(&3u32.to_le_bytes()); // BI_BITFIELDS
    hdr[34..38].copy_from_slice(&(pixels_size as u32).to_le_bytes());
    hdr[54..58].copy_from_slice(&0xF800u32.to_le_bytes()); // R mask
    hdr[58..62].copy_from_slice(&0x07E0u32.to_le_bytes()); // G mask
    hdr[62..66].copy_from_slice(&0x001Fu32.to_le_bytes()); // B mask

    if let Err(e) = file.write(&hdr) {
        error!("hdr write failed: {:?}", defmt::Debug2Format(&e));
        let _ = file.close();
        let _ = root.close();
        let _ = volume.close();
        return Err(e);
    }

    // Stream the framebuffer in 16-row chunks just to emit a progress log
    // per chunk; the underlying SDMMC traffic is the same single-block
    // CMD24 stream regardless of write() size.
    let body: &[u8] = unsafe { core::slice::from_raw_parts(pixels.as_ptr() as *const u8, pixels_size) };
    const ROWS_PER_CHUNK: usize = 16;
    let mut y = 0usize;
    while y < h as usize {
        let rows_now = core::cmp::min(ROWS_PER_CHUNK, h as usize - y);
        let off = y * row_bytes;
        let len = rows_now * row_bytes;
        if let Err(e) = file.write(&body[off..off + len]) {
            error!("body write failed at row {}: {:?}", y, defmt::Debug2Format(&e));
            let _ = file.close();
            let _ = root.close();
            let _ = volume.close();
            return Err(e);
        }
        y += rows_now;
        info!("  ... {} / {} rows", y, h);
    }

    if let Err(e) = file.close() {
        error!("file close failed: {:?}", defmt::Debug2Format(&e));
        let _ = root.close();
        let _ = volume.close();
        return Err(e);
    }
    let _ = root.close();
    let _ = volume.close();
    Ok(())
}

fn scan_next_index<'a, 'b>(volume_mgr: &mut VolumeManager<EmbassyBlockDevice<'a, 'b>, FixedTime>) -> u32 {
    let mut max: i64 = -1;
    let res: Result<(), embedded_sdmmc::Error<embassy_stm32::sdmmc::Error>> = (|| {
        let volume = volume_mgr.open_volume(VolumeIdx(0))?;
        let root = volume.open_root_dir()?;
        root.iterate_dir(|entry| {
            let name = entry.name.base_name();
            let ext = entry.name.extension();
            if ext.eq_ignore_ascii_case(b"BMP") && name.len() == 7 && name.starts_with(b"IMG") {
                if let Ok(s) = core::str::from_utf8(&name[3..]) {
                    if let Ok(n) = s.parse::<u32>() {
                        if n as i64 > max {
                            max = n as i64;
                        }
                    }
                }
            }
        })?;
        Ok(())
    })();
    if let Err(e) = res {
        error!("scan_next_index: {:?}", defmt::Debug2Format(&e));
    }
    (max + 1) as u32
}

// ---- BlockDevice glue ----

struct EmbassyBlockDevice<'a, 'b> {
    inner: RefCell<StorageDevice<'a, 'b, Card>>,
}

impl<'a, 'b> BlockDevice for EmbassyBlockDevice<'a, 'b> {
    type Error = embassy_stm32::sdmmc::Error;

    fn read(&self, blocks: &mut [Block], start_block_idx: BlockIdx) -> Result<(), Self::Error> {
        let mut inner = self.inner.borrow_mut();
        for (i, block) in blocks.iter_mut().enumerate() {
            let mut data = DataBlock([0u32; 128]);
            block_on(inner.read_block(start_block_idx.0 + i as u32, &mut data))?;
            // DataBlock is repr-transparent over [u32; 128] = 512 bytes.
            // SAFETY: same size, properly aligned.
            unsafe {
                core::ptr::copy_nonoverlapping(data.0.as_ptr() as *const u8, block.contents.as_mut_ptr(), 512);
            }
        }
        Ok(())
    }

    fn write(&self, blocks: &[Block], start_block_idx: BlockIdx) -> Result<(), Self::Error> {
        let mut inner = self.inner.borrow_mut();
        for (i, block) in blocks.iter().enumerate() {
            let mut data = DataBlock([0u32; 128]);
            unsafe {
                core::ptr::copy_nonoverlapping(block.contents.as_ptr(), data.0.as_mut_ptr() as *mut u8, 512);
            }
            block_on(inner.write_block(start_block_idx.0 + i as u32, &data))?;
        }
        Ok(())
    }

    fn num_blocks(&self) -> Result<BlockCount, Self::Error> {
        let inner = self.inner.borrow();
        let bytes = inner.card().size();
        Ok(BlockCount((bytes / 512) as u32))
    }
}

struct FixedTime;
impl TimeSource for FixedTime {
    fn get_timestamp(&self) -> Timestamp {
        // 2026-01-01 00:00:00 — fine for FAT timestamps in a demo.
        Timestamp {
            year_since_1970: 56,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

// ---- RCC + RIFSC + SRAM helpers (same as bin/camera.rs) ----

fn rcc_config() -> Config {
    let mut config = Config::default();
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

    config.rcc.pll4 = Some(Pll::Bypass { source: Pllsel::Hsi });
    config.rcc.ic16 = Some(IcConfig {
        source: Icsel::Pll4,
        divider: Icint::Div2,
    });
    config.rcc.mux.ltdcsel = Ltdcsel::Ic16;

    config.rcc.ic17 = Some(IcConfig {
        source: Icsel::Pll1,
        divider: Icint::Div3,
    });
    config.rcc.mux.dcmippsel = Dcmippsel::Ic17;

    config.rcc.ic18 = Some(IcConfig {
        source: Icsel::Pll1,
        divider: Icint::Div60,
    });
    config
}

fn enable_all_sram() {
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
}

fn promote_axi_masters_to_secure() {
    pac::RIFSC.risc_seccfgr(2).modify(|w| {
        w.set_cfg(29, true);
    });
    pac::RIFSC.risc_privcfgr(2).modify(|w| {
        w.set_cfg(29, true);
    });
    pac::RIFSC.risc_seccfgr(3).modify(|w| {
        w.set_cfg(5, true);
        w.set_cfg(7, true);
        w.set_cfg(8, true);
    });
    pac::RIFSC.risc_privcfgr(3).modify(|w| {
        w.set_cfg(5, true);
        w.set_cfg(7, true);
        w.set_cfg(8, true);
    });
    for master in [8usize, 9, 10, 11] {
        pac::RIFSC.rimc_attr(master).modify(|w| {
            w.set_mcid(1);
            w.set_msec(true);
            w.set_mpriv(true);
        });
    }
}
