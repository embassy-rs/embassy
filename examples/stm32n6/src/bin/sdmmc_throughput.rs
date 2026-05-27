#![no_std]
#![no_main]

//! SDMMC2 throughput demo on the STM32N6570-DK microSD slot.
//! Negotiates UHS-I SDR104 (DLYB-tuned), mounts FAT32 via
//! `embedded-fatfs`, writes a 16 MiB file, and reports MiB/s for both
//! the extend (cluster-alloc) and rewrite (pre-allocated) phases.
//!
//! Pinout per UM3300 §8.9: CK/CMD/D0/D1/D2/D3 = PC2/PC3/PC4/PC5/PC0/PE4,
//! SD_SEL (PO5) Low = 3.3 V on this board.

#[path = "../sd_save.rs"]
mod sd_save;

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::rcc::{CpuClk, IcConfig, Icint, Icsel, Pll, Plldivm, Pllpdiv, Pllsel, SysClk};
use embassy_stm32::sdmmc::Sdmmc;
use embassy_stm32::sdmmc::sd::{Addressable, CmdBlock, StorageDevice};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, peripherals};
use embassy_time::Instant;
use embedded_io_async::{Seek as _, Write as _};
use {defmt_rtt as _, panic_probe as _};

use crate::sd_save::mount as sd_mount;

bind_interrupts!(struct Irqs {
    SDMMC2 => embassy_stm32::sdmmc::InterruptHandler<peripherals::SDMMC2>;
});

const FILE_NAME: &str = "TPUT.BIN";
const FILE_BYTES: usize = 16 * 1024 * 1024;
const CHUNK_BYTES: usize = 32 * 1024;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(rcc_config());

    let vswitch = Output::new(p.PO5, Level::Low, Speed::Low);

    let mut sd_cfg = embassy_stm32::sdmmc::Config::default();
    sd_cfg.data_transfer_timeout = 200_000_000;
    sd_cfg.use_cmd23 = true;
    sd_cfg.use_acmd23 = true;

    let mut sd = Sdmmc::new_4bit_with_vswitch_dlyb(
        p.SDMMC2,
        Irqs,
        p.PC2,
        p.PC3,
        p.PC4,
        p.PC5,
        p.PC0,
        p.PE4,
        vswitch,
        p.DLYB_SDMMC2,
        sd_cfg,
    );

    let mut cmd_block = CmdBlock::new();
    // 110 MHz triggers SDR104 negotiation (>100 MHz). With the default
    // 100 MHz kernel + CLKDIV bypass the wire stays at 100 MHz.
    let storage = match StorageDevice::new_sd_card(&mut sd, &mut cmd_block, Hertz(110_000_000)).await {
        Ok(s) => s,
        Err(e) => {
            error!("sd init failed: {:?}", e);
            return;
        }
    };
    info!(
        "sd: card ready, {} bytes ({} MiB)",
        storage.card().size(),
        storage.card().size() / (1024 * 1024)
    );

    let fs = match sd_mount(storage).await {
        Ok(fs) => fs,
        Err(_) => {
            error!("fatfs mount failed");
            return;
        }
    };

    let mut pattern = [0u8; CHUNK_BYTES];
    for (i, b) in pattern.iter_mut().enumerate() {
        *b = i as u8;
    }
    let pattern = &pattern[..];

    let chunks = FILE_BYTES / CHUNK_BYTES;
    let bytes = (chunks * CHUNK_BYTES) as u64;
    let (extend_elapsed, rewrite_elapsed) = {
        let root = fs.root_dir();
        let mut file = match root.create_file(FILE_NAME).await {
            Ok(f) => f,
            Err(_) => {
                error!("create_file failed");
                return;
            }
        };
        if file.truncate().await.is_err() {
            error!("truncate failed");
            return;
        }

        // Phase 1: fresh write — data + FAT cluster-allocation churn.
        let extend_start = Instant::now();
        for _ in 0..chunks {
            if file.write_all(pattern).await.is_err() {
                error!("extend write failed");
                break;
            }
        }
        let _ = file.flush().await;
        let extend_elapsed = extend_start.elapsed();

        // Phase 2: rewrite over the now-allocated clusters — bus-bound.
        if file.seek(embedded_io_async::SeekFrom::Start(0)).await.is_err() {
            error!("seek failed");
            return;
        }
        let rewrite_start = Instant::now();
        for _ in 0..chunks {
            if file.write_all(pattern).await.is_err() {
                error!("rewrite failed");
                break;
            }
        }
        let _ = file.flush().await;
        let rewrite_elapsed = rewrite_start.elapsed();
        (extend_elapsed, rewrite_elapsed)
    };
    let _ = fs.unmount().await;

    let report = |phase: &str, elapsed: embassy_time::Duration| {
        let secs = elapsed.as_micros() as f32 / 1_000_000.0;
        let mibs = (bytes as f32 / 1_048_576.0) / secs;
        info!(
            "{}: wrote {} bytes in {} ms = {} MiB/s",
            phase,
            bytes,
            elapsed.as_millis(),
            mibs
        );
    };
    report("extend (cluster alloc + data)", extend_elapsed);
    report("rewrite (data only, pre-allocated)", rewrite_elapsed);

    info!("done");
    loop {
        embassy_time::Timer::after_secs(60).await;
    }
}

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

    config
}
