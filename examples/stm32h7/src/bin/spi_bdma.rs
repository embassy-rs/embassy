#![no_std]
#![no_main]

use core::fmt::Write;
use core::str::from_utf8;

use cortex_m_rt::entry;
use defmt::*;
use embassy_executor::Executor;
use embassy_stm32::mode::Async;
use embassy_stm32::time::mhz;
use embassy_stm32::{Config, bind_interrupts, dma, peripherals, spi};
use grounded::uninit::GroundedArrayCell;
use heapless::String;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// Defined in memory.x
#[unsafe(link_section = ".ram_d3")]
static mut RAM_D3: GroundedArrayCell<u8, 256> = GroundedArrayCell::uninit();

bind_interrupts!(struct Irqs {
    BDMA_CHANNEL0 => dma::InterruptHandler<peripherals::BDMA_CH0>;
    BDMA_CHANNEL1 => dma::InterruptHandler<peripherals::BDMA_CH1>;
});

#[embassy_executor::task]
async fn main_task(mut spi: spi::Spi<'static, Async, spi::mode::Master>) {
    let (read_buffer, write_buffer) = unsafe {
        let ram = &mut *core::ptr::addr_of_mut!(RAM_D3);
        ram.initialize_all_copied(0);
        (
            ram.get_subslice_mut_unchecked(0, 128),
            ram.get_subslice_mut_unchecked(128, 128),
        )
    };

    for n in 0u32.. {
        let mut write: String<128> = String::new();
        core::write!(&mut write, "Hello DMA World {}!\r\n", n).unwrap();
        let read_buffer = &mut read_buffer[..write.len()];
        let write_buffer = &mut write_buffer[..write.len()];
        // copy data to write_buffer which is located in D3 domain, accessable by BDMA
        write_buffer.clone_from_slice(write.as_bytes());

        spi.transfer(read_buffer, write_buffer).await.ok();
        info!("read via spi+dma: {}", from_utf8(read_buffer).unwrap());
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            fracn: None,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8), // used by SPI3. 100Mhz.
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
    }
    let p = embassy_stm32::init(config);

    let mut spi_config = spi::Config::default();
    spi_config.frequency = mhz(1);

    let spi = spi::Spi::new(p.SPI6, p.PA5, p.PA7, p.PA6, p.BDMA_CH1, p.BDMA_CH0, Irqs, spi_config);

    let executor = EXECUTOR.init(Executor::new());

    executor.run(|spawner| {
        spawner.spawn(unwrap!(main_task(spi)));
    })
}
