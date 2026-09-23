#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dma::{self, Channel, TransferOptions};
use embassy_stm32::peripherals::{self};
use embassy_stm32::{SharedData, bind_interrupts};
use panic_probe as _;

#[unsafe(link_section = ".ram_d3.shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

bind_interrupts! (struct Irqs{
    MDMA => dma::InterruptHandler<peripherals::MDMA_CH0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::Div1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul50,
            divp: Some(PllDiv::Div2),
            divq: Some(PllDiv::Div8), // 100mhz
            divr: None,
        });
        config.rcc.sys = Sysclk::Pll1P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::Div2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
    }

    let p = embassy_stm32::init_primary(config, &SHARED_DATA);
    info!("Hello World!");

    // Generate a 32-element array with a distinct pattern for each index
    // This ensures we aren't accidentally transferring the same word 32 times
    // or skipping/offsetting any bytes.
    let source: [u32; 32] = core::array::from_fn(|i| (i as u32).wrapping_mul(0x01010101) ^ 0xDEADBEEF);

    let mut dest: [u32; 32] = [0; 32];

    let mut dma_ch = Channel::new(p.MDMA_CH0, Irqs);
    let tfer_opts = TransferOptions::default();

    println!("DMA starting...");
    println!("Source [0]: {:#010x}, Source [31]: {:#010x}", source[0], source[31]);
    println!("Dest   [0]: {:#010x}, Dest   [31]: {:#010x}", dest[0], dest[31]);

    // No request number needed as MEM2MEM transfers on MDMA are software-controlled. Defaulting to 0.
    let tfer = unsafe { dma_ch.transfer::<u32, u32>(0, source.as_slice(), dest.as_mut_ptr(), tfer_opts) };

    tfer.await;

    println!("DMA finished.");
    println!("Dest   [0]: {:#010x}, Dest   [31]: {:#010x}", dest[0], dest[31]);

    // Verify every single element
    let mut mismatch = false;
    for (i, (s, d)) in source.iter().zip(dest.iter()).enumerate() {
        if s != d {
            println!("Mismatch at index {}: src={:#010x}, dst={:#010x}", i, s, d);
            mismatch = true;
        }
    }

    if !mismatch {
        println!("Success: All 32 elements (128 bytes) transferred perfectly!");
    }

    loop {}
}
