//! This example receives inputs on SPDIFRX and outputs on SAI4.
//!
//! Only very few controllers connect the SPDIFRX symbol clock to a SAI peripheral's clock input.
//! However, this is necessary for synchronizing the symbol rates and avoiding glitches.
#![no_std]
#![no_main]

use defmt::{info, trace};
use embassy_executor::Spawner;
use embassy_futures::select::{self, select, Either};
use embassy_stm32::spdifrx::{self, Spdifrx};
use embassy_stm32::{bind_interrupts, peripherals, sai};
use grounded::uninit::GroundedArrayCell;
use hal::sai::*;
use {defmt_rtt as _, embassy_stm32 as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    SPDIF_RX => spdifrx::GlobalInterruptHandler<peripherals::SPDIFRX1>;
});

const CHANNEL_COUNT: usize = 2;
const BLOCK_LENGTH: usize = 64;
const HALF_DMA_BUFFER_LENGTH: usize = BLOCK_LENGTH * CHANNEL_COUNT;
const DMA_BUFFER_LENGTH: usize = HALF_DMA_BUFFER_LENGTH * 2; //  2 half-blocks

// DMA buffers must be in special regions. Refer https://embassy.dev/book/#_stm32_bdma_only_working_out_of_some_ram_regions
#[link_section = ".sram1"]
static mut SPDIFRX_BUFFER: GroundedArrayCell<u32, DMA_BUFFER_LENGTH> = GroundedArrayCell::uninit();

#[link_section = ".sram4"]
static mut SAI_BUFFER: GroundedArrayCell<u32, DMA_BUFFER_LENGTH> = GroundedArrayCell::uninit();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut peripheral_config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        peripheral_config.rcc.hsi = Some(HSIPrescaler::DIV1);
        peripheral_config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV16,
            mul: PllMul::MUL200,
            divp: Some(PllDiv::DIV2), // 400 MHz
            divq: Some(PllDiv::DIV2),
            divr: Some(PllDiv::DIV2),
        });
        peripheral_config.rcc.sys = Sysclk::PLL1_P;
        peripheral_config.rcc.ahb_pre = AHBPrescaler::DIV2;
        peripheral_config.rcc.apb1_pre = APBPrescaler::DIV2;
        peripheral_config.rcc.apb2_pre = APBPrescaler::DIV2;
        peripheral_config.rcc.apb3_pre = APBPrescaler::DIV2;
        peripheral_config.rcc.apb4_pre = APBPrescaler::DIV2;

        peripheral_config.rcc.mux.spdifrxsel = mux::Spdifrxsel::PLL1_Q;
    }
    let mut p = embassy_stm32::init(peripheral_config);

    info!("SPDIFRX to SAI4 bridge");

    // Use SPDIFRX clock for SAI.
    // This ensures equal rates of sample production and consumption.
    let clk_source = embassy_stm32::pac::rcc::vals::Saiasel::_RESERVED_5;
    embassy_stm32::pac::RCC.d3ccipr().modify(|w| {
        w.set_sai4asel(clk_source);
    });

    let sai_buffer: &mut [u32] = unsafe {
        SAI_BUFFER.initialize_all_copied(0);
        let (ptr, len) = SAI_BUFFER.get_ptr_len();
        core::slice::from_raw_parts_mut(ptr, len)
    };

    let spdifrx_buffer: &mut [u32] = unsafe {
        SPDIFRX_BUFFER.initialize_all_copied(0);
        let (ptr, len) = SPDIFRX_BUFFER.get_ptr_len();
        core::slice::from_raw_parts_mut(ptr, len)
    };

    let mut sai_transmitter = new_sai_transmitter(
        p.SAI4.reborrow(),
        p.PD13.reborrow(),
        p.PC1.reborrow(),
        p.PD12.reborrow(),
        p.BDMA_CH0.reborrow(),
        sai_buffer,
    );
    let mut spdif_receiver = new_spdif_receiver(
        p.SPDIFRX1.reborrow(),
        p.PD7.reborrow(),
        p.DMA2_CH7.reborrow(),
        spdifrx_buffer,
    );
    spdif_receiver.start();

    let mut renew_sai = false;
    loop {
        let mut buf = [0u32; HALF_DMA_BUFFER_LENGTH];

        if renew_sai {
            renew_sai = false;
            trace!("Renew SAI.");
            drop(sai_transmitter);
            sai_transmitter = new_sai_transmitter(
                p.SAI4.reborrow(),
                p.PD13.reborrow(),
                p.PC1.reborrow(),
                p.PD12.reborrow(),
                p.BDMA_CH0.reborrow(),
                sai_buffer,
            );
        }

        match select(spdif_receiver.read(&mut buf), sai_transmitter.wait_write_error()).await {
            Either::First(spdif_read_result) => match spdif_read_result {
                Ok(_) => (),
                Err(spdifrx::Error::RingbufferError(_)) => {
                    trace!("SPDIFRX ringbuffer error. Renew.");
                    drop(spdif_receiver);
                    spdif_receiver = new_spdif_receiver(
                        p.SPDIFRX1.reborrow(),
                        p.PD7.reborrow(),
                        p.DMA2_CH7.reborrow(),
                        spdifrx_buffer,
                    );
                    spdif_receiver.start();
                    continue;
                }
                Err(spdifrx::Error::ChannelSyncError) => {
                    trace!("SPDIFRX channel sync (left/right assignment) error.");
                    continue;
                }
            },
            Either::Second(_) => {
                renew_sai = true;
                continue;
            }
        };

        renew_sai = sai_transmitter.write(&buf).await.is_err();
    }
}

/// Creates a new SPDIFRX instance for receiving sample data.
///
/// Used (again) after dropping the SPDIFRX instance, in case of errors (e.g. source disconnect).
fn new_spdif_receiver<'d>(
    spdifrx: &'d mut peripherals::SPDIFRX1,
    input_pin: &'d mut peripherals::PD7,
    dma: &'d mut peripherals::DMA2_CH7,
    buf: &'d mut [u32],
) -> Spdifrx<'d, peripherals::SPDIFRX1> {
    Spdifrx::new(spdifrx, Irqs, spdifrx::Config::default(), input_pin, dma, buf)
}

/// Creates a new SAI4 instance for transmitting sample data.
///
/// Used (again) after dropping the SAI4 instance, in case of errors (e.g. buffer overrun).
fn new_sai_transmitter<'d>(
    sai: &'d mut peripherals::SAI4,
    sck: &'d mut peripherals::PD13,
    sd: &'d mut peripherals::PC1,
    fs: &'d mut peripherals::PD12,
    dma: &'d mut peripherals::BDMA_CH0,
    buf: &'d mut [u32],
) -> Sai<'d, peripherals::SAI4, u32> {
    let mut sai_config = hal::sai::Config::default();
    sai_config.slot_count = hal::sai::word::U4(CHANNEL_COUNT as u8);
    sai_config.slot_enable = 0xFFFF; // All slots
    sai_config.data_size = sai::DataSize::Data32;
    sai_config.frame_length = (CHANNEL_COUNT * 32) as u8;
    sai_config.master_clock_divider = hal::sai::MasterClockDivider::MasterClockDisabled;

    let (sub_block_tx, _) = hal::sai::split_subblocks(sai);
    Sai::new_asynchronous(sub_block_tx, sck, sd, fs, dma, buf, sai_config)
}
