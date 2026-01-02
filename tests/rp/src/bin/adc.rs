#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

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

    #[cfg(any(feature = "rp2040", feature = "rp235xa"))]
    let (mut a, mut b, mut c, mut d) = (p.PIN_26, p.PIN_27, p.PIN_28, p.PIN_29);
    #[cfg(feature = "rp235xb")]
    let (mut a, mut b, mut c, mut d) = (p.PIN_44, p.PIN_45, p.PIN_46, p.PIN_47);

    {
        {
            let mut p = Channel::new_pin(a.reborrow(), Pull::Down);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() < 0b01_0000_0000);
            defmt::assert!(adc.read(&mut p).await.unwrap() < 0b01_0000_0000);
        }
        {
            let mut p = Channel::new_pin(a.reborrow(), Pull::Up);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() > 0b11_0000_0000);
            defmt::assert!(adc.read(&mut p).await.unwrap() > 0b11_0000_0000);
        }
    }
    // not bothering with async reads from now on
    {
        {
            let mut p = Channel::new_pin(b.reborrow(), Pull::Down);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() < 0b01_0000_0000);
        }
        {
            let mut p = Channel::new_pin(b.reborrow(), Pull::Up);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() > 0b11_0000_0000);
        }
    }
    {
        {
            let mut p = Channel::new_pin(c.reborrow(), Pull::Down);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() < 0b01_0000_0000);
        }
        {
            let mut p = Channel::new_pin(c.reborrow(), Pull::Up);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() > 0b11_0000_0000);
        }
    }
    {
        // gp29 is connected to vsys through a 200k/100k divider,
        // adding pulls should change the value
        let low = {
            let mut p = Channel::new_pin(d.reborrow(), Pull::Down);
            adc.blocking_read(&mut p).unwrap()
        };
        let none = {
            let mut p = Channel::new_pin(d.reborrow(), Pull::None);
            adc.blocking_read(&mut p).unwrap()
        };
        let up = {
            let mut p = Channel::new_pin(d.reborrow(), Pull::Up);
            adc.blocking_read(&mut p).unwrap()
        };
        defmt::assert!(low < none);
        defmt::assert!(none < up);
    }
    {
        let temp = convert_to_celsius(
            adc.read(&mut Channel::new_temp_sensor(p.ADC_TEMP_SENSOR.reborrow()))
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
            &mut Channel::new_pin(d.reborrow(), Pull::Down),
            &mut low,
            1,
            p.DMA_CH0.reborrow(),
        )
        .await
        .unwrap();
        adc.read_many(
            &mut Channel::new_pin(d.reborrow(), Pull::None),
            &mut none,
            1,
            p.DMA_CH0.reborrow(),
        )
        .await
        .unwrap();
        adc.read_many_raw(
            &mut Channel::new_pin(d.reborrow(), Pull::Up),
            &mut up,
            1,
            p.DMA_CH0.reborrow(),
        )
        .await;
        defmt::assert!(low.iter().zip(none.iter()).all(|(l, n)| *l >> 4 < *n as u16));
        defmt::assert!(up.iter().all(|s| s.good()));
        defmt::assert!(none.iter().zip(up.iter()).all(|(n, u)| (*n as u16) < u.value()));
    }
    {
        let mut temp = [0u16; 16];
        adc.read_many(
            &mut Channel::new_temp_sensor(p.ADC_TEMP_SENSOR.reborrow()),
            &mut temp,
            1,
            p.DMA_CH0.reborrow(),
        )
        .await
        .unwrap();
        let temp = temp.map(convert_to_celsius);
        defmt::assert!(temp.iter().all(|t| *t > 0.0));
        defmt::assert!(temp.iter().all(|t| *t < 60.0));
    }
    {
        let mut multi = [0u16; 2];
        let mut channels = [
            Channel::new_pin(a.reborrow(), Pull::Up),
            Channel::new_temp_sensor(p.ADC_TEMP_SENSOR.reborrow()),
        ];
        adc.read_many_multichannel(&mut channels, &mut multi, 1, p.DMA_CH0.reborrow())
            .await
            .unwrap();
        defmt::assert!(multi[0] > 3_000);
        let temp = convert_to_celsius(multi[1]);
        defmt::assert!(temp > 0.0 && temp < 60.0);
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

fn convert_to_celsius(raw_temp: u16) -> f32 {
    // According to chapter 4.9.5. Temperature Sensor in RP2040 datasheet
    27.0 - (raw_temp as f32 * 3.3 / 4096.0 - 0.706) / 0.001721 as f32
}
