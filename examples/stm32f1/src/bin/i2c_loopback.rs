#![no_std]
#![no_main]

use core::future::pending;

use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::{
    Config, bind_interrupts,
    i2c::{self, I2c, SlaveAddrConfig, SlaveCommandKind},
    peripherals,
    time::khz,
};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let config = Config::default();
    let p = embassy_stm32::init(config);
    let i2c_address = 0x20;
    let i2c_frequency = khz({
        // 400
        100
    });

    join(
        async {
            let mut i2c_peripheral = I2c::new(p.I2C1, p.PB6, p.PB7, Irqs, p.DMA1_CH6, p.DMA1_CH7, {
                let mut config = i2c::Config::default();
                config.frequency = i2c_frequency;
                config
            })
            .into_slave_multimaster(SlaveAddrConfig::basic(i2c_address));
            loop {
                info!("[I2C1 (peripheral)] ready to receive I2C commands.");
                let command = match i2c_peripheral.listen().await {
                    Ok(command) => command,
                    Err(e) => {
                        warn!("[I2C1 (peripheral)] I2C error: {}", e);
                        continue;
                    }
                };
                match command.kind {
                    SlaveCommandKind::Read => {
                        info!("[I2C1 (peripheral)] read command started.");
                        i2c_peripheral
                            .respond_to_read(&[10, 11, 12, 13, 14, 15, 16, 17])
                            .await
                            .unwrap();
                    }
                    SlaveCommandKind::Write => {
                        info!("[I2C1 (peripheral)] write command started.");
                        let mut buffer = [Default::default(); 8];
                        let bytes_read = i2c_peripheral.respond_to_write(&mut buffer).await.unwrap();
                        info!("[I2C1 (peripheral)] received data: {}.", &buffer[..bytes_read]);
                    }
                }
            }
        },
        async {
            let mut i2c_controller = I2c::new(p.I2C2, p.PB10, p.PB11, Irqs, p.DMA1_CH4, p.DMA1_CH5, {
                let mut config = i2c::Config::default();
                config.frequency = i2c_frequency;
                config
            });
            info!("[I2C2 (controller)] writing data and then reading data.");
            let mut response = [Default::default(); 4];
            let write_buffer = {
                // [10]
                [10, 20]
            };
            let combined_write_read = true;
            if combined_write_read {
                i2c_controller
                    .write_read(i2c_address, &write_buffer, &mut response)
                    .await
                    .unwrap();
            } else {
                i2c_controller.write(i2c_address, &write_buffer).await.unwrap();
                i2c_controller.read(i2c_address, &mut response).await.unwrap();
            }
            info!(
                "[I2C2 (controller)] done writing and reading data. received data: {}.",
                response
            );
        },
    )
    .await;
    pending().await
}
