// required-features: sdmmc
#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use aligned::Aligned;
use block_device_driver::BlockDevice as _;
use common::*;
use defmt::assert_eq;
use embassy_executor::Spawner;
use embassy_stm32::sdmmc::Sdmmc;
use embassy_stm32::{bind_interrupts, peripherals, sdmmc};
use embassy_time::Delay;
use sdio::BlockDevice;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SDIO => sdmmc::InterruptHandler<peripherals::SDIO>;
    DMA2_STREAM3 => embassy_stm32::dma::InterruptHandler<peripherals::DMA2_CH3>;
});

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let p = init();

    let (mut sdmmc, mut dma, mut clk, mut cmd, mut d0, mut d1, mut d2, mut d3) =
        (p.SDIO, p.DMA2_CH3, p.PC12, p.PD2, p.PC8, p.PC9, p.PC10, p.PC11);

    // Arbitrary block index
    let block_idx = 16;

    let mut pattern1 = Aligned([0u8; 512]);
    let mut pattern2 = Aligned([0u8; 512]);
    for i in 0..512 {
        pattern1[i] = i as u8;
        pattern2[i] = !i as u8;
    }
    let patterns = [pattern1.clone(), pattern2.clone()];

    let mut block = [Aligned([0u8; 512])];
    let mut blocks = [Aligned([0u8; 512]), Aligned([0u8; 512])];
    {
        // ======== Try 4bit. ==============
        info!("initializing in 4-bit mode...");
        let mut s = Sdmmc::new_4bit(
            sdmmc.reborrow(),
            dma.reborrow(),
            Irqs,
            clk.reborrow(),
            cmd.reborrow(),
            d0.reborrow(),
            d1.reborrow(),
            d2.reborrow(),
            d3.reborrow(),
            Default::default(),
        );

        {
            let mut storage = loop {
                if let Ok(storage) = BlockDevice::new_sd_card(&mut s, 24_000_000, Delay).await {
                    break storage;
                }
            };

            let card = storage.card();

            info!("Card: {:#?}", Debug2Format(&card));

            // card_type: HighCapacity,
            // ocr: OCR: Operation Conditions Register {
            // Voltage Window (mV): (2700, 3600),
            // S18A (UHS-I only): true,
            // Over 2TB flag (SDUC only): false,
            // UHS-II Card: false,
            // Card Capacity Status (CSS): \"SDHC/SDXC/SDUC\",
            // Busy: false },
            // rca: 43690,
            // cid: CID: Card Identification { Manufacturer ID: 3,
            // OEM ID: \"SD\",
            // Product Name: \"SL08G\",
            // Product Revision: 128,
            // Product Serial Number: 701445767,
            // Manufacturing Date: (9,
            // 2015) },
            // csd: CSD: Card Specific Data { Transfer Rate: 50,
            // Block Count: 15523840,
            // Card Size (bytes): 7948206080,
            // Read I (@min VDD): 100 mA,
            // Write I (@min VDD): 10 mA,
            // Read I (@max VDD): 5 mA,
            // Write I (@max VDD): 45 mA,
            // Erase Size (Blocks): 1 },
            // scr: SCR: SD CARD Configuration Register { Version: Unknown,
            // 1-bit width: false,
            // 4-bit width: true },
            // status: SD Status { Bus Width: One,
            // Secured Mode: false,
            // SD Memory Card Type: 0,
            // Protected Area Size (B): 0,
            // Speed Class: 0,
            // Video Speed Class: 0,
            // Application Performance Class: 0,
            // Move Performance (MB/s): 0,
            // AU Size: 0,
            // Erase Size (units of AU): 0,
            // Erase Timeout (s): 0,
            // Discard Support: false } }

            defmt::assert!(card.scr.bus_width_four());

            info!("writing pattern1...");
            storage.write(block_idx, &[pattern1]).await.unwrap();

            info!("reading...");
            storage.read(block_idx, &mut block).await.unwrap();
            assert_eq!(*block[0], *pattern1);

            info!("writing pattern2...");
            storage.write(block_idx, &[pattern2]).await.unwrap();

            info!("reading...");
            storage.read(block_idx, &mut block).await.unwrap();
            assert_eq!(*block[0], *pattern2);

            info!("writing blocks [pattern1, pattern2]...");
            storage.write(block_idx, &patterns).await.unwrap();

            info!("reading blocks...");
            storage.read(block_idx, &mut blocks).await.unwrap();

            for (block, pattern) in blocks.iter().zip(patterns) {
                assert_eq!(**block, *pattern);
            }
        }
    }

    {
        // ======== Try 1bit. ==============
        info!("initializing in 1-bit mode...");
        let mut s = Sdmmc::new_1bit(
            sdmmc.reborrow(),
            dma.reborrow(),
            Irqs,
            clk.reborrow(),
            cmd.reborrow(),
            d0.reborrow(),
            Default::default(),
        );

        {
            let mut storage = loop {
                if let Ok(storage) = BlockDevice::new_sd_card(&mut s, 15_000_000, Delay).await {
                    break storage;
                }
            };

            let card = storage.card();

            info!("Card: {:#?}", Debug2Format(&card));

            // card_type: HighCapacity,
            // ocr: OCR: Operation Conditions Register {
            // Voltage Window (mV): (2700, 3600),
            // S18A (UHS-I only): true,
            // Over 2TB flag (SDUC only): false,
            // UHS-II Card: false,
            // Card Capacity Status (CSS): \"SDHC/SDXC/SDUC\",
            // Busy: false },
            // rca: 43690,
            // cid: CID: Card Identification { Manufacturer ID: 3,
            // OEM ID: \"SD\",
            // Product Name: \"SL08G\",
            // Product Revision: 128,
            // Product Serial Number: 701445767,
            // Manufacturing Date: (9,
            // 2015) },
            // csd: CSD: Card Specific Data { Transfer Rate: 50,
            // Block Count: 15523840,
            // Card Size (bytes): 7948206080,
            // Read I (@min VDD): 100 mA,
            // Write I (@min VDD): 10 mA,
            // Read I (@max VDD): 5 mA,
            // Write I (@max VDD): 45 mA,
            // Erase Size (Blocks): 1 },
            // scr: SCR: SD CARD Configuration Register { Version: Unknown,
            // 1-bit width: false,
            // 4-bit width: true },
            // status: SD Status { Bus Width: One,
            // Secured Mode: false,
            // SD Memory Card Type: 0,
            // Protected Area Size (B): 0,
            // Speed Class: 0,
            // Video Speed Class: 0,
            // Application Performance Class: 0,
            // Move Performance (MB/s): 0,
            // AU Size: 0,
            // Erase Size (units of AU): 0,
            // Erase Timeout (s): 0,
            // Discard Support: false } }

            defmt::assert!(card.scr.bus_width_four());

            info!("writing pattern1...");
            storage.write(block_idx, &[pattern1]).await.unwrap();

            info!("reading...");
            storage.read(block_idx, &mut block).await.unwrap();
            assert_eq!(*block[0], *pattern1);

            info!("writing pattern2...");
            storage.write(block_idx, &[pattern2]).await.unwrap();

            info!("reading...");
            storage.read(block_idx, &mut block).await.unwrap();
            assert_eq!(*block[0], *pattern2);

            info!("writing blocks [pattern1, pattern2]...");
            storage.write(block_idx, &patterns).await.unwrap();

            info!("reading blocks...");
            storage.read(block_idx, &mut blocks).await.unwrap();

            for (block, pattern) in blocks.iter().zip(patterns) {
                assert_eq!(**block, *pattern);
            }
        }
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
