//! I2C slave wakeup-from-stop with the low-power executor.

#![no_std]
#![no_main]

use defmt::{info, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{self, I2c, SendStatus, SlaveAddrConfig, SlaveCommandKind};
use embassy_stm32::time::Hertz;
use embassy_stm32::{self, bind_interrupts, dma, peripherals, rcc};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
    DMA1_CHANNEL2_3 => dma::InterruptHandler<peripherals::DMA1_CH2>, dma::InterruptHandler<peripherals::DMA1_CH3>;
});

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init({
        let mut cfg = embassy_stm32::Config::default();

        // Use HSI
        cfg.rcc = rcc::Config {
            hsi: true,
            msi: None,
            sys: rcc::Sysclk::Hsi,
            ..Default::default()
        };

        // I2C wakeup from Stop requires HSI16 as the I2C1 kernel clock.
        cfg.rcc.mux.i2c1sel = rcc::mux::I2csel::Hsi;
        cfg
    });

    // Wake from Stop defaults the system clock to MSI, change to HSI.
    embassy_stm32::pac::RCC
        .cfgr()
        .modify(|w| w.set_stopwuck(embassy_stm32::pac::rcc::vals::Stopwuck::Hsi));

    const I2C_ADDR: u8 = 0x20;

    let mut i2c = {
        let mut cfg = i2c::Config::default();
        cfg.frequency = Hertz::khz(400);
        I2c::new(p.I2C1, p.PB6, p.PB7, p.DMA1_CH2, p.DMA1_CH3, Irqs, cfg)
            .into_slave_multimaster(SlaveAddrConfig::basic(I2C_ADDR))
    };

    let mut buffer = [0u8; 32];
    let mut buffer_len = 0usize;

    info!("i2c slave listening at address 0x{:02X}", I2C_ADDR);

    loop {
        // Change this to i2c.listen() to use SLEEP mode instead of STOP mode
        match i2c.listen_low_power().await {
            Ok(cmd) => match cmd.kind {
                SlaveCommandKind::Write => match i2c.respond_to_write(&mut buffer).await {
                    Ok(len) => {
                        buffer_len = len;
                        info!("i2c write: {} bytes: {:02X}", len, &buffer[..buffer_len]);
                    }
                    Err(e) => warn!("i2c write error: {:?}", e),
                },
                SlaveCommandKind::Read => match i2c.respond_to_read(&buffer[..buffer_len]).await {
                    Ok(SendStatus::Done) => info!("i2c read: {} bytes", buffer_len),
                    Ok(SendStatus::LeftoverBytes(n)) => {
                        info!("i2c read: {} of {} bytes", buffer_len - n, buffer_len)
                    }
                    Err(e) => warn!("i2c read error: {:?}", e),
                },
            },
            Err(e) => warn!("listen error: {:?}", e),
        }
    }
}
