//! Destructive ROM FlexSPI regression test for the FRDM-MCXA577's 8 MiB NOR.
//!
//! Erases/programs external flash `0x003e0000..0x003e2000`. Run only when that
//! region is expendable. Uses 30 MHz SDR options, not destructive DDR probing.
//! Select the MCXA577 probe explicitly if another board is attached:
//! `cargo build --release --bin flexspi-rom`, then
//! `probe-rs run --chip MCXA577 --probe <VID:PID:serial> --verify <ELF>`.

#![no_std]
#![no_main]

use defmt::{assert, assert_eq, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa::Peripherals;
use embassy_mcxa::config::Config;
use embassy_mcxa::rom::*;
use embassy_time::Timer;
use panic_probe as _;

const BASE: u32 = 0x003e_0000;
const CANARY: u32 = 0xa5a5_a5a5;

// ARM32 ABI sizes/offsets from the MCXA577 SDK 2026.09.00 ROM headers.
// Keep the owned Rust handle outside all of these ROM-facing structures.
const _: () = {
    use core::assert;
    use core::mem::{offset_of, size_of};
    assert!(size_of::<SerialNorConfigOption>() == 8);
    assert!(size_of::<FlexspiLutSeq>() == 4);
    assert!(size_of::<FlexspiDllTime>() == 2);
    assert!(size_of::<FlexspiMemConfig>() == 0x1c0);
    assert!(offset_of!(FlexspiMemConfig, lookup_table) == 0x80);
    assert!(offset_of!(FlexspiMemConfig, lut_custom_seq) == 0x180);
    assert!(offset_of!(FlexspiMemConfig, dll0_cr_val) == 0x1b0);
    assert!(size_of::<FlexspiNorConfig>() == 0x200);
    assert!(offset_of!(FlexspiNorConfig, page_size) == 0x1c0);
    assert!(offset_of!(FlexspiNorConfig, block_size) == 0x1d0);
    assert!(offset_of!(FlexspiNorConfig, flash_state_ctx) == 0x1d4);
    assert!(size_of::<FlexspiXfer>() == 36);
    assert!(offset_of!(FlexspiXfer, tx_buffer) == 20);
    assert!(offset_of!(FlexspiXfer, rx_buffer) == 28);
    assert!(size_of::<FlexspiNorVtable>() == 56);
    assert!(size_of::<RomApi>() == 32);
    assert!(size_of::<StandardVersion>() == 4);
    assert!(size_of::<FlashFfrConfig>() == 24);
    assert!(size_of::<FlashModeConfig>() == 20);
    assert!(size_of::<FlashConfig>() == 72);
    assert!(offset_of!(FlashConfig, mode_config) == 44);
    assert!(offset_of!(FlashConfig, nboot_ctx) == 64);
    assert!(offset_of!(FlashConfig, use_ahb_read) == 68);
};

fn ok(result: Result<(), FlexspiError>) {
    if let Err(e) = result {
        defmt::panic!("ROM error: {:?}", defmt::Debug2Format(&e));
    }
}

fn bytes(words: &[u32]) -> &[u8] {
    // SAFETY: bounded byte view of initialized word storage.
    unsafe { core::slice::from_raw_parts(words.as_ptr().cast(), core::mem::size_of_val(words)) }
}

fn pattern(address: u32) -> u8 {
    let h = address.wrapping_mul(2_654_435_761);
    (h ^ (h >> 16)) as u8
}

macro_rules! pins {
    ($p:ident) => {
        FlexspiNorPins::new(
            $p.P3_0.reborrow(),
            $p.P3_7.reborrow(),
            $p.P3_6.reborrow(),
            $p.P3_8.reborrow(),
            $p.P3_9.reborrow(),
            $p.P3_10.reborrow(),
            $p.P3_11.reborrow(),
        )
    };
}

macro_rules! pins_b {
    ($p:ident) => {
        FlexspiNorPins::new(
            $p.P3_17.reborrow(),
            $p.P3_16.reborrow(),
            $p.P3_1.reborrow(),
            $p.P3_15.reborrow(),
            $p.P3_14.reborrow(),
            $p.P3_13.reborrow(),
            $p.P3_12.reborrow(),
        )
    };
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_mcxa::init(Config::default());
    let api = get();
    exercise(api, &mut p);
    loop {
        Timer::after_secs(1).await;
        info!("ROM FlexSPI heartbeat");
    }
}

fn exercise(api: &RomApi, p: &mut Peripherals) {
    // SAFETY: only the raw transfer/page/LUT operations and known integer-backed
    // zero initialization need unsafe. Buffers stay live/aligned and the probed
    // geometry/LUT is checked. Constructors enforce controller/pin ownership.
    unsafe {
        info!("ROM FlexSPI regression BEGIN");
        info!(
            "ROM version {}.{}.{}",
            api.version().major,
            api.version().minor,
            api.version().bugfix
        );
        let option = SerialNorConfigOption {
            option0: 0xc000_0001,
            option1: 0,
        };

        let mut wrong_port_called = false;
        let wrong_port =
            api.flex_spi_with_options(p.FLEXSPI0.reborrow(), pins_b!(p), option, |_| wrong_port_called = true);
        assert!(matches!(wrong_port, Err(FlexspiError::InvalidArgument)));
        assert!(!wrong_port_called);
        drop(wrong_port);
        let bad_fcb = api.flex_spi_with_options(p.FLEXSPI0.reborrow(), pins!(p), option, |cfg| {
            cfg.mem_config.sflash_b1_size = cfg.mem_config.sflash_a1_size;
            cfg.mem_config.sflash_a1_size = 0;
        });
        assert!(matches!(bad_fcb, Err(FlexspiError::InvalidArgument)));
        drop(bad_fcb);
        info!("PASS pin/option and customized FCB port mismatches rejected");

        // There is no numeric instance argument: the singleton fixes instance 0.
        assert!(matches!(
            api.flex_spi_with_options_and_clock(
                p.FLEXSPI0.reborrow(),
                pins!(p),
                option,
                FlexspiClockSource::NoClock,
                |_| defmt::panic!("no-clock closure")
            ),
            Err(FlexspiError::InvalidArgument)
        ));
        info!("PASS NoClock rejection with controller/pin borrows released");

        // Known MCXA577 ROM 3.2.0 error stimuli, not arbitrary memory pointers.
        let mut failed_probe_called = false;
        let failed_probe = api.flex_spi_with_options(
            p.FLEXSPI0.reborrow(),
            pins!(p),
            SerialNorConfigOption {
                option0: 0xc002_0001,
                option1: 0,
            },
            |_| failed_probe_called = true,
        );
        assert!(matches!(failed_probe, Err(FlexspiError::InvalidArgument)));
        assert!(!failed_probe_called);
        drop(failed_probe);
        let invalid_init =
            api.flex_spi_with_options(p.FLEXSPI0.reborrow(), pins!(p), option, |cfg| cfg.mem_config.tag = 0);
        assert!(matches!(invalid_init, Err(FlexspiError::InvalidArgument)));
        drop(invalid_init);
        info!("PASS probe/init error stimuli; closure skipped on probe failure");

        let mut called = false;
        let mut lut = [0u32; 64];
        let mut legacy_config = None;
        let result = api.flex_spi_with_options(p.FLEXSPI0.reborrow(), pins!(p), option, |cfg| {
            called = true;
            info!(
                "SFDP page={} sector={} block={} A1={} B1={}",
                cfg.page_size,
                cfg.sector_size,
                cfg.block_size,
                cfg.mem_config.sflash_a1_size,
                cfg.mem_config.sflash_b1_size
            );
            assert_eq!(cfg.page_size, 256);
            assert_eq!(cfg.sector_size, 4096);
            assert_eq!(cfg.mem_config.sflash_a1_size, 8 * 1024 * 1024);
            assert_eq!(cfg.mem_config.sflash_b1_size, 0);
            lut.copy_from_slice(&cfg.mem_config.lookup_table);
            cfg.mem_config.timeout_in_ms = 1000;
            cfg.mem_config.read_sample_clk_src = 0;
            // All FCB fields are inline integers/arrays/unions, with no owned
            // allocation or destructor; retain a complete configuration fixture.
            legacy_config = Some(core::ptr::read(cfg));
        });
        let mut flash = match result {
            Ok(flash) => flash,
            Err(e) => defmt::panic!("constructor: {:?}", defmt::Debug2Format(&e)),
        };
        assert!(called);
        let mcr0 = core::ptr::read_volatile(0x4002_0000 as *const u32);
        assert_eq!((mcr0 >> 4) & 3, 0);
        info!(
            "PASS retry after errors and FCB override; driver={:08x}",
            flash.version()
        );

        let mut guard = [CANARY; 4];
        assert!(matches!(
            flash.read(&mut guard[1..2], BASE, 5),
            Err(FlexspiError::BufferTooSmall)
        ));
        assert_eq!(guard, [CANARY; 4]);
        assert!(matches!(
            flash.partial_program(BASE, &[0x12345678], 5),
            Err(FlexspiError::BufferTooSmall)
        ));
        assert!(matches!(
            flash.read(&mut [], BASE, 1),
            Err(FlexspiError::BufferTooSmall)
        ));
        assert!(matches!(
            flash.partial_program(BASE, &[], 1),
            Err(FlexspiError::BufferTooSmall)
        ));
        info!("PASS over-capacity and empty-buffer guards");
        assert!(matches!(
            flash.read(&mut [], BASE, 0),
            Err(FlexspiError::InvalidArgument)
        ));
        ok(flash.partial_program(BASE, &[], 0));
        info!("PASS ROM 3.2.0 zero-length behavior");

        ok(flash.erase(BASE, 0x2000));
        let mut sector = [0u32; 1024];
        ok(flash.read(&mut sector, BASE, 4096));
        assert!(sector.iter().all(|&word| word == u32::MAX));
        info!("PASS erase and 4096-byte read");

        let mut page = [0u32; 64];
        for (i, word) in page.iter_mut().enumerate() {
            *word = u32::from_le_bytes(core::array::from_fn(|j| pattern(BASE + (i * 4 + j) as u32)));
        }
        ok(flash.partial_program(BASE, &page, 256));
        for len in [1usize, 2, 3, 4, 5, 7, 17, 31, 32, 33, 63, 64, 65, 127, 255, 256] {
            let mut buf = [CANARY; 66];
            let words = len.div_ceil(4);
            ok(flash.read(&mut buf[1..1 + words], BASE, len as u32));
            assert_eq!(buf[0], CANARY);
            assert_eq!(buf[1 + words], CANARY);
            for (i, &byte) in bytes(&buf[1..])[..len].iter().enumerate() {
                assert_eq!(byte, pattern(BASE + i as u32));
            }
            for &byte in &bytes(&buf[1..])[len..words * 4] {
                assert_eq!(byte, 0xa5, "read modified byte beyond requested length");
            }
            info!("PASS read len={} exact bytes and canaries", len);
        }
        for off in [1u32, 3, 17, 127, 255] {
            let mut buf = [CANARY; 6];
            ok(flash.read(&mut buf[1..5], BASE + off, 13));
            for (i, &byte) in bytes(&buf[1..])[..13].iter().enumerate() {
                let expected = if off + (i as u32) < 256 {
                    pattern(BASE + off + i as u32)
                } else {
                    0xff
                };
                assert_eq!(byte, expected);
            }
            assert_eq!(buf[0], CANARY);
            assert_eq!(buf[5], CANARY);
            info!("PASS unaligned read offset={}", off);
        }
        for (index, len) in [1usize, 2, 3, 4, 5, 7, 17, 31, 33, 63, 127, 255]
            .into_iter()
            .enumerate()
        {
            let addr = BASE + 256 * (index as u32 + 1);
            let src = [0x78563412u32; 64];
            ok(flash.partial_program(addr, &src, len as u32));
            let mut buf = [0u32; 64];
            ok(flash.read(&mut buf, addr, 256));
            assert_eq!(&bytes(&buf)[..len], &bytes(&src)[..len]);
            assert!(bytes(&buf)[len..].iter().all(|&byte| byte == 0xff));
            info!("PASS partial_program len={} exact flash tail", len);
        }

        // Exercise the now-unsafe escape hatches only with proven buffer extents.
        // The full LUT is readable; reinstall an existing sequence, not new commands.
        ok(flash.update_lut(0, lut.as_ptr(), 1));
        let mut rx = [CANARY; 6];
        let mut xfer = FlexspiXfer {
            operation: FlexspiOperationType::Read,
            base_address: BASE,
            seq_id: 0,
            seq_num: 1,
            is_parallel_mode_enable: false,
            tx_buffer: core::ptr::null(),
            tx_size: 0,
            rx_buffer: rx[1..5].as_mut_ptr(),
            rx_size: 13,
        };
        ok(flash.xfer(&mut xfer));
        for (i, &byte) in bytes(&rx[1..])[..13].iter().enumerate() {
            assert_eq!(byte, pattern(BASE + i as u32));
        }
        assert_eq!(rx[0], CANARY);
        assert_eq!(rx[5], CANARY);
        assert!(bytes(&rx[1..5])[13..].iter().all(|&b| b == 0xa5));
        ok(flash.page_program(BASE + 0x1000, page.as_ptr()));
        let mut full_page = [0u32; 64];
        ok(flash.read(&mut full_page, BASE + 0x1000, 256));
        assert_eq!(full_page, page);
        info!("PASS raw LUT update, bounded xfer, and full-page program");
        drop(flash);

        assert_eq!(
            pack_serial_nor_option1(SerialNorFlashConnection::SinglePortB, 10, 11, 12, 0xd6, 0xe7),
            0x2abc_d6e7
        );
        let packed_option = SerialNorConfigOption {
            option0: pack_serial_nor_option0(
                SerialNorOptionSize::Option0AndOption1,
                SerialNorDeviceType::ReadSfdpSdr,
                SerialNorOptionPadEncoding::One,
                SerialNorOptionPadEncoding::One,
                SerialNorQuadModeSetting::NotConfigured,
                SerialNorMiscMode::Disabled,
                FlexspiSerialClockFrequency::MHz30,
            ),
            option1: pack_serial_nor_option1(SerialNorFlashConnection::SinglePortA, 0, 0, 0, 0, 0),
        };
        assert_eq!(packed_option.option0, 0xc100_0001);
        let mut clock_flash = match api.flex_spi_with_options_and_clock(
            p.FLEXSPI0.reborrow(),
            pins!(p),
            packed_option,
            FlexspiClockSource::FroHf,
            |_| {},
        ) {
            Ok(f) => f,
            Err(e) => defmt::panic!("pre-probe clock constructor: {:?}", defmt::Debug2Format(&e)),
        };
        let mut word = [0u32; 1];
        ok(clock_flash.read(&mut word, BASE, 4));
        assert_eq!(
            bytes(&word),
            &[pattern(BASE), pattern(BASE + 1), pattern(BASE + 2), pattern(BASE + 3)]
        );
        drop(clock_flash);
        info!("PASS drop/reacquire, packed options, and pre-probe FRO_HF selection");

        // Regression for the formerly poisoned legacy constructor. ROM 3.2.0
        // rejects the zero tag; there are no pointer-bearing FCB fields here.
        let mut invalid_config = match legacy_config {
            Some(config) => config,
            None => defmt::panic!("missing legacy configuration fixture"),
        };
        invalid_config.mem_config.tag = 0;
        assert!(matches!(
            api.flex_spi(p.FLEXSPI0.reborrow(), pins!(p), invalid_config),
            Err(FlexspiError::InvalidArgument)
        ));
        let retry = api.flex_spi_with_options(p.FLEXSPI0.reborrow(), pins!(p), option, |_| {});
        assert!(retry.is_ok(), "legacy init error must not poison the new constructor");
        drop(retry);
        info!("PASS legacy init-error recovery");
        info!("ROM FlexSPI regression PASSED");
    }
}
