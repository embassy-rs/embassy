#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::sai::{self, Sai};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, dma, peripherals};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use embedded_sdmmc::filesystem::ShortFileName;
use embedded_sdmmc::{BlockDevice, RawFile, SdCard, TimeSource, VolumeIdx, VolumeManager};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// Simple SD card audio streaming example for SAI.
// - Supports raw unsigned 16-bit PCM (.pcm)
// - Supports 16-bit mono WAV at 48 kHz (.wav)

const VOLUME_NUM: u32 = 3;
const VOLUME_DEN: u32 = 4;

fn scale_u16(sample: u16) -> u16 {
    ((sample as u32) * VOLUME_NUM / VOLUME_DEN) as u16
}

type SdSpiDev = ExclusiveDevice<
    Spi<'static, embassy_stm32::mode::Async, embassy_stm32::spi::mode::Master>,
    Output<'static>,
    NoDelay,
>;

type SdCardDev = SdCard<SdSpiDev, embassy_time::Delay>;

struct DummyTimesource;
impl embedded_sdmmc::TimeSource for DummyTimesource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

struct WavInfo {
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
    data_offset: usize,
}

fn parse_wav_header(buf: &[u8]) -> Option<WavInfo> {
    if buf.len() < 44 {
        return None;
    }
    if &buf[0..4] != b"RIFF" || &buf[8..12] != b"WAVE" {
        return None;
    }

    let mut fmt: Option<(u16, u16, u32, u16)> = None;
    let mut data_offset: Option<usize> = None;

    let mut off = 12usize;
    while off + 8 <= buf.len() {
        let chunk_id = &buf[off..off + 4];
        let chunk_size = u32::from_le_bytes([buf[off + 4], buf[off + 5], buf[off + 6], buf[off + 7]]) as usize;
        let chunk_data = off + 8;

        if chunk_id == b"fmt " && chunk_data + 16 <= buf.len() {
            let audio_format = u16::from_le_bytes([buf[chunk_data], buf[chunk_data + 1]]);
            let channels = u16::from_le_bytes([buf[chunk_data + 2], buf[chunk_data + 3]]);
            let sample_rate = u32::from_le_bytes([
                buf[chunk_data + 4],
                buf[chunk_data + 5],
                buf[chunk_data + 6],
                buf[chunk_data + 7],
            ]);
            let bits_per_sample = u16::from_le_bytes([buf[chunk_data + 14], buf[chunk_data + 15]]);
            fmt = Some((audio_format, channels, sample_rate, bits_per_sample));
        }

        if chunk_id == b"data" {
            data_offset = Some(chunk_data);
            break;
        }

        let mut next = chunk_data.saturating_add(chunk_size);
        if next % 2 != 0 {
            next += 1;
        }
        if next <= off {
            break;
        }
        off = next;
    }

    let (audio_format, channels, sample_rate, bits_per_sample) = fmt?;
    let data_offset = data_offset?;

    if audio_format != 1 {
        return None;
    }

    Some(WavInfo {
        sample_rate,
        channels,
        bits_per_sample,
        data_offset,
    })
}

async fn write_samples(sai_tx: &mut Sai<'static, peripherals::SAI1, u16>, samples: &[u16]) {
    if samples.is_empty() {
        return;
    }
    if let Err(e) = sai_tx.write(samples).await {
        warn!("SAI write error: {:?}", defmt::Debug2Format(&e));
    }
}

async fn play_pcm<D, T, const MAX_DIRS: usize, const MAX_FILES: usize, const MAX_VOLUMES: usize>(
    sai_tx: &mut Sai<'static, peripherals::SAI1, u16>,
    volume_mgr: &mut VolumeManager<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    file: RawFile,
) where
    D: BlockDevice,
    T: TimeSource,
{
    info!("Playing PCM file");

    let mut buf = [0u8; 512];
    let mut out = [0u16; 256];

    loop {
        let n = match volume_mgr.read(file, &mut buf) {
            Ok(n) => n,
            Err(e) => {
                warn!("SD read error: {:?}", defmt::Debug2Format(&e));
                break;
            }
        };
        if n == 0 {
            break;
        }

        let mut count = 0usize;
        let mut i = 0usize;
        while i + 1 < n && count < out.len() {
            let sample = u16::from_le_bytes([buf[i], buf[i + 1]]);
            out[count] = scale_u16(sample);
            count += 1;
            i += 2;
        }

        write_samples(sai_tx, &out[..count]).await;
    }
}

async fn play_wav<D, T, const MAX_DIRS: usize, const MAX_FILES: usize, const MAX_VOLUMES: usize>(
    sai_tx: &mut Sai<'static, peripherals::SAI1, u16>,
    volume_mgr: &mut VolumeManager<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    file: RawFile,
) where
    D: BlockDevice,
    T: TimeSource,
{
    let mut header = [0u8; 512];
    let header_len = match volume_mgr.read(file, &mut header) {
        Ok(n) => n,
        Err(e) => {
            warn!("SD read error: {:?}", defmt::Debug2Format(&e));
            return;
        }
    };

    let info = match parse_wav_header(&header[..header_len]) {
        Some(info) => info,
        None => {
            warn!("Invalid WAV header");
            return;
        }
    };

    if info.sample_rate != 48_000 || info.channels != 1 || info.bits_per_sample != 16 {
        warn!(
            "Unsupported WAV format: {} Hz, {} ch, {} bits",
            info.sample_rate, info.channels, info.bits_per_sample
        );
        return;
    }

    info!("Playing WAV file: 48 kHz mono 16-bit");

    let mut out = [0u16; 256];
    let mut buf = [0u8; 512];

    if info.data_offset > header_len {
        let mut remaining = info.data_offset - header_len;
        while remaining > 0 {
            let to_read = remaining.min(buf.len());
            let n = match volume_mgr.read(file, &mut buf[..to_read]) {
                Ok(n) => n,
                Err(e) => {
                    warn!("SD read error: {:?}", defmt::Debug2Format(&e));
                    return;
                }
            };
            if n == 0 {
                warn!("Unexpected end of file while seeking data");
                return;
            }
            remaining -= n;
        }
    } else if info.data_offset < header_len {
        let mut i = info.data_offset;
        let mut count = 0usize;
        while i + 1 < header_len && count < out.len() {
            let signed = i16::from_le_bytes([header[i], header[i + 1]]);
            let unsigned = (signed as i32 + 0x8000) as u16;
            out[count] = scale_u16(unsigned);
            count += 1;
            i += 2;
        }
        write_samples(sai_tx, &out[..count]).await;
    }

    loop {
        let n = match volume_mgr.read(file, &mut buf) {
            Ok(n) => n,
            Err(e) => {
                warn!("SD read error: {:?}", defmt::Debug2Format(&e));
                break;
            }
        };
        if n == 0 {
            break;
        }

        let mut count = 0usize;
        let mut i = 0usize;
        while i + 1 < n && count < out.len() {
            let signed = i16::from_le_bytes([buf[i], buf[i + 1]]);
            let unsigned = (signed as i32 + 0x8000) as u16;
            out[count] = scale_u16(unsigned);
            count += 1;
            i += 2;
        }

        write_samples(sai_tx, &out[..count]).await;
    }
}

bind_interrupts!(struct Irqs {
    GPDMA1_CHANNEL0 => dma::InterruptHandler<peripherals::GPDMA1_CH0>;
    GPDMA1_CHANNEL1 => dma::InterruptHandler<peripherals::GPDMA1_CH1>;
    GPDMA1_CHANNEL2 => dma::InterruptHandler<peripherals::GPDMA1_CH2>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL12,
            divq: Some(PllDiv::DIV4),
            divr: Some(PllDiv::DIV5),
            divp: Some(PllDiv::DIV30),
            frac: Some(2363),
        });

        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV1;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
        config.rcc.apb7_pre = APBPrescaler::DIV1;
        config.rcc.ahb5_pre = AHB5Prescaler::DIV2;
        config.rcc.voltage_scale = VoltageScale::RANGE1;

        config.rcc.mux.sai1sel = mux::Sai1sel::PLL1_Q;
    }

    let p = embassy_stm32::init(config);

    info!("SDMMC SAI example");

    let (sai_a, _sai_b) = sai::split_subblocks(p.SAI1);

    static SAI_DMA_BUF: StaticCell<[u16; 4096]> = StaticCell::new();
    let sai_dma_buf = SAI_DMA_BUF.init([0u16; 4096]);

    let mut sai_cfg = sai::Config::default();
    sai_cfg.mode = sai::Mode::Master;
    sai_cfg.tx_rx = sai::TxRx::Transmitter;
    sai_cfg.stereo_mono = sai::StereoMono::Mono;
    sai_cfg.data_size = sai::DataSize::Data16;
    sai_cfg.bit_order = sai::BitOrder::MsbFirst;
    sai_cfg.slot_size = sai::SlotSize::Channel32;
    sai_cfg.slot_count = sai::word::U4(2);
    sai_cfg.slot_enable = 0b11;
    sai_cfg.first_bit_offset = sai::word::U5(0);
    sai_cfg.frame_sync_polarity = sai::FrameSyncPolarity::ActiveLow;
    sai_cfg.frame_sync_offset = sai::FrameSyncOffset::BeforeFirstBit;
    sai_cfg.frame_length = 32;
    sai_cfg.frame_sync_active_level_length = sai::word::U7(16);
    sai_cfg.fifo_threshold = sai::FifoThreshold::Quarter;
    sai_cfg.master_clock_divider = sai::MasterClockDivider::DIV4;

    let mut sai_tx = Sai::new_asynchronous(sai_a, p.PA7, p.PB14, p.PA8, p.GPDMA1_CH2, sai_dma_buf, Irqs, sai_cfg);

    let _max98357a_sd = Output::new(p.PA1, Level::High, Speed::Low);

    let mut spi_cfg = spi::Config::default();
    spi_cfg.frequency = Hertz(400_000);
    let spi = Spi::new(p.SPI1, p.PB4, p.PA15, p.PB3, p.GPDMA1_CH0, p.GPDMA1_CH1, Irqs, spi_cfg);
    let cs = Output::new(p.PA6, Level::High, Speed::VeryHigh);
    let spi_dev: SdSpiDev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let sd: SdCardDev = SdCard::new(spi_dev, embassy_time::Delay);

    embassy_time::Timer::after_millis(50).await;
    sd.spi(|dev| {
        let dummy = [0xFFu8; 10];
        let _ = dev.bus_mut().write(&dummy);
    });

    info!("SD size {} bytes", sd.num_bytes().unwrap_or(0));

    let mut spi_cfg2 = spi::Config::default();
    spi_cfg2.frequency = Hertz(8_000_000);
    let _ = sd.spi(|dev| dev.bus_mut().set_config(&spi_cfg2));

    static VOLUME_MANAGER: StaticCell<VolumeManager<SdCardDev, DummyTimesource>> = StaticCell::new();
    let vol_mgr: &'static mut VolumeManager<SdCardDev, DummyTimesource> =
        VOLUME_MANAGER.init(VolumeManager::new(sd, DummyTimesource));

    let raw_vol = match vol_mgr.open_raw_volume(VolumeIdx(0)) {
        Ok(v) => v,
        Err(e) => {
            warn!("SD open_raw_volume error: {:?}", defmt::Debug2Format(&e));
            return;
        }
    };
    let raw_root = match vol_mgr.open_root_dir(raw_vol) {
        Ok(r) => r,
        Err(e) => {
            warn!("SD open_root_dir error: {:?}", defmt::Debug2Format(&e));
            return;
        }
    };

    let mut names: heapless::Vec<ShortFileName, 16> = heapless::Vec::new();
    let _ = vol_mgr.iterate_dir(raw_root, |de| {
        if !de.attributes.is_directory() {
            let ext = de.name.extension();
            if ext == b"PCM" || ext == b"WAV" {
                let _ = names.push(de.name.clone());
            }
        }
    });

    if names.is_empty() {
        warn!("No .pcm or .wav files in SD root");
        return;
    }

    let name = &names[0];
    info!("Playing {}", defmt::Debug2Format(name));

    let file = match vol_mgr.open_file_in_dir(raw_root, name, embedded_sdmmc::Mode::ReadOnly) {
        Ok(f) => f,
        Err(e) => {
            warn!("SD open_file error: {:?}", defmt::Debug2Format(&e));
            return;
        }
    };

    let raw_file = file;
    if name.extension() == b"PCM" {
        play_pcm(&mut sai_tx, vol_mgr, raw_file).await;
    } else {
        play_wav(&mut sai_tx, vol_mgr, raw_file).await;
    }

    let _ = vol_mgr.close_file(file);

    let _ = spawner;
}
