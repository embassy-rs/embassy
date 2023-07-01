#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Config, ContinuousAdc, InterruptHandler, SamplingSpeed};
use embassy_rp::{adc, bind_interrupts, Peripheral};
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut adc = Adc::new(p.ADC, Irqs, Config::default());

    let mut p26 = p.PIN_26;
    let mut p27 = p.PIN_27;
    let mut p28 = p.PIN_28;

    let mut dma0 = p.DMA_CH0.into_ref();
    let mut dma1 = p.DMA_CH1.into_ref();

    for _ in 0..3 {
        for _ in 0..100 {
            let bx26 = adc.blocking_read(&mut p26);
            let bx27 = adc.blocking_read(&mut p27);
            let bx28 = adc.blocking_read(&mut p28);
            let bxt = adc.blocking_read_temperature();
            let x26 = adc.read(&mut p26).await;
            let x27 = adc.read(&mut p27).await;
            let x28 = adc.read(&mut p28).await;
            let xt = adc.read_temperature().await;

            info!("{} {} {} {} {} {} {} {}", x26, x27, x28, xt, bx26, bx27, bx28, bxt);
        }

        let mut buffer = [0u16; 24];
        adc.dma_read(
            &mut adc::input_from_pin(&mut p26, true).add_temperature().add(&mut p28),
            dma0.reborrow(),
            &mut buffer,
            SamplingSpeed::Fastest,
        )
        .await;
        info!("buffer {}", buffer);
    }

    info!("measuring with one dma channel at 10 microseconds per sample");
    for _ in 0..10 {
        // ordering of the input channels does not matter, only which one is the first one
        // this particular input leads to adc measuring p27 p28 temp p26 p28 temp p26 p28 ....
        let mut input = adc::input_from_pin(&mut p27, false)
            .add(&mut p26)
            .add_temperature()
            .add(&mut p28);

        let mut buffer = [0u16; 1000];

        let speed = SamplingSpeed::Interval {
            n: 48 * 10 - 1,
            frac: 0,
        };
        let start_time = Instant::now();
        adc.dma_read(&mut input, dma0.reborrow(), &mut buffer, speed).await;
        let t = Instant::now().duration_since(start_time);

        let mut sums = [0u32; 3]; // p26, p28, temp
        let mut j = 1; // starting pin in [p26, p28, temp] is p28
        for x in buffer[1..].into_iter() {
            // ignoring first pin (p27), because it is not part of round robin
            sums[j] += *x as u32;
            j = (j + 1) % 3;
        }

        info!(
            "t: {:?}, p26: {:?}, p28: {:?}, temp: {:?} (~{:?}°C)",
            t.as_micros(),
            mean(sums[0], 333),
            mean(sums[1], 333),
            mean(sums[2], 333),
            convert_to_celsius(mean(sums[2], 333)),
        );
    }

    info!("measuring continuously with two dma channels at 2 microseconds per sample");
    loop {
        let mut buffer1 = [0u16; 1000];
        let mut buffer2 = [0u16; 1000];

        for l in 1..50 {
            let data1 = &mut buffer1[..l];
            let data2 = &mut buffer2[..l]; // same size is not required

            // this particular input leads to adc measuring temp, p26, p27, p28, temp, p26, p27, p28, ...
            let input = adc::input_temperature(true).add(&mut p28).add(&mut p27).add(&mut p26);
            let mut control_input = [[0u32; 4]; 2];
            let speed = SamplingSpeed::Fastest;

            let mut sums = [0u32; 4]; // p26, p27, p28, temp
            let mut counts = [0u32; 4];
            let mut j = 3usize; // starting pin is temp
            let mut corrupted = false;
            #[allow(unused_assignments)]
            let mut in_time = true;

            let start = Instant::now();

            let mut cadc = ContinuousAdc::start_new(
                adc,
                dma0.reborrow(),
                dma1.reborrow(),
                input,
                speed,
                &mut control_input,
                data1,
            );

            for _ in 0..2499 {
                (cadc, in_time) = cadc.next(data2).await;
                evaluate(data1, &mut sums, &mut counts, &mut corrupted, &mut j, in_time);

                (cadc, in_time) = cadc.next(data1).await;
                evaluate(data2, &mut sums, &mut counts, &mut corrupted, &mut j, in_time);
            }

            // to have exactly 5000 runs
            (cadc, in_time) = cadc.next(data2).await;
            evaluate(data1, &mut sums, &mut counts, &mut corrupted, &mut j, in_time);

            // stop measurement, get adc back
            adc = cadc.stop().await;
            let t = Instant::now().duration_since(start).as_micros();
            evaluate(data2, &mut sums, &mut counts, &mut corrupted, &mut j, true);

            let mut c = heapless::String::<11>::new();
            if corrupted {
                c.push_str(", corrupted").unwrap();
            }
            info!(
                "data length: {:?}, t: {:?} micros, p26: {:?}, p27: {:?}, p28: {:?}, temp: {:?}{}",
                l,
                t,
                mean(sums[0], counts[0]),
                mean(sums[1], counts[1]),
                mean(sums[2], counts[2]),
                mean(sums[3], counts[3]),
                c
            );
        }
        Timer::after(Duration::from_millis(1000)).await;
    }
}

fn convert_to_celsius(raw_temp: u16) -> f32 {
    // According to chapter 4.9.5. Temperature Sensor in RP2040 datasheet
    27.0 - (raw_temp as f32 * 3.3 / 4096.0 - 0.706) / 0.001721 as f32
}

fn mean(sum: u32, len: u32) -> u16 {
    (0.5 + sum as f32 / len as f32) as u16
}

fn evaluate(
    data: &mut [u16],
    sums: &mut [u32; 4],
    counts: &mut [u32; 4],
    corrupted: &mut bool,
    j: &mut usize,
    in_time: bool,
) {
    *corrupted = *corrupted | !in_time;
    for i in 0..data.len() {
        sums[*j] += data[i] as u32;
        data[i] = 0;
        counts[*j] += 1;
        *j = (*j + 1) % 4;
    }
}
