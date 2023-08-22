#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
teleprobe_meta::target!(b"rpi-pico");

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config, InterruptHandler, Sample};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output, Pull};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_rp::init(Default::default());
    let _power_reg_pwm_mode = Output::new(p.PIN_23, Level::High);
    let _wifi_off = Output::new(p.PIN_25, Level::High);
    let mut adc = Adc::new(p.ADC, Irqs, Config::default());

    {
        {
            let mut p = Channel::new_pin(&mut p.PIN_26, Pull::Down);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() < 0b01_0000_0000);
            defmt::assert!(adc.read(&mut p).await.unwrap() < 0b01_0000_0000);
        }
        {
            let mut p = Channel::new_pin(&mut p.PIN_26, Pull::Up);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() > 0b11_0000_0000);
            defmt::assert!(adc.read(&mut p).await.unwrap() > 0b11_0000_0000);
        }
    }
    // not bothering with async reads from now on
    {
        {
            let mut p = Channel::new_pin(&mut p.PIN_27, Pull::Down);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() < 0b01_0000_0000);
        }
        {
            let mut p = Channel::new_pin(&mut p.PIN_27, Pull::Up);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() > 0b11_0000_0000);
        }
    }
    {
        {
            let mut p = Channel::new_pin(&mut p.PIN_28, Pull::Down);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() < 0b01_0000_0000);
        }
        {
            let mut p = Channel::new_pin(&mut p.PIN_28, Pull::Up);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() > 0b11_0000_0000);
        }
    }
    {
        // gp29 is connected to vsys through a 200k/100k divider,
        // adding pulls should change the value
        let low = {
            let mut p = Channel::new_pin(&mut p.PIN_29, Pull::Down);
            adc.blocking_read(&mut p).unwrap()
        };
        let none = {
            let mut p = Channel::new_pin(&mut p.PIN_29, Pull::None);
            adc.blocking_read(&mut p).unwrap()
        };
        let up = {
            let mut p = Channel::new_pin(&mut p.PIN_29, Pull::Up);
            adc.blocking_read(&mut p).unwrap()
        };
        defmt::assert!(low < none);
        defmt::assert!(none < up);
    }
    {
        let temp = convert_to_celsius(
            adc.read(&mut Channel::new_temp_sensor(&mut p.ADC_TEMP_SENSOR))
                .await
                .unwrap(),
        );
        defmt::assert!(temp > 0.0);
        defmt::assert!(temp < 60.0);
    }

    // run a bunch of conversions. we'll only check gp29 and the temp
    // sensor here for brevity, if those two work the rest will too.
    {
        // gp29 is connected to vsys through a 200k/100k divider,
        // adding pulls should change the value
        let mut low = [0u16; 16];
        let mut none = [0u8; 16];
        let mut up = [Sample::default(); 16];
        adc.read_many(
            &mut Channel::new_pin(&mut p.PIN_29, Pull::Down),
            &mut low,
            &mut p.DMA_CH0,
        )
        .await
        .unwrap();
        adc.read_many(
            &mut Channel::new_pin(&mut p.PIN_29, Pull::None),
            &mut none,
            &mut p.DMA_CH0,
        )
        .await
        .unwrap();
        adc.read_many_raw(&mut Channel::new_pin(&mut p.PIN_29, Pull::Up), &mut up, &mut p.DMA_CH0)
            .await;
        defmt::assert!(low.iter().zip(none.iter()).all(|(l, n)| *l >> 4 < *n as u16));
        defmt::assert!(up.iter().all(|s| s.good()));
        defmt::assert!(none.iter().zip(up.iter()).all(|(n, u)| (*n as u16) < u.value()));
    }
    {
        let mut temp = [0u16; 16];
        adc.read_many(
            &mut Channel::new_temp_sensor(&mut p.ADC_TEMP_SENSOR),
            &mut temp,
            &mut p.DMA_CH0,
        )
        .await
        .unwrap();
        let temp = temp.map(convert_to_celsius);
        defmt::assert!(temp.iter().all(|t| *t > 0.0));
        defmt::assert!(temp.iter().all(|t| *t < 60.0));
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

fn convert_to_celsius(raw_temp: u16) -> f32 {
    // According to chapter 4.9.5. Temperature Sensor in RP2040 datasheet
    27.0 - (raw_temp as f32 * 3.3 / 4096.0 - 0.706) / 0.001721 as f32
}
