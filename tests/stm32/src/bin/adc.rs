#![no_std]
#![no_main]

// required-features: adc

#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel, BorrowedAdcChannel, Config, SampleTime};
use embassy_stm32::gpio::{Flex, Pull, Speed};
use panic_probe as _;

/// How a known level is put on the test pin.
#[derive(Copy, Clone, PartialEq, Eq, defmt::Format)]
enum PinMethod {
    /// Digital input with the internal pull resistor. Works on ADCs that sample a pin in any GPIO
    /// mode (the small G0/C0/U0 ADCs).
    Pull,
    /// Analog mode with the internal pull resistor still enabled, for GPIOs that keep the pull
    /// resistors active in analog mode.
    AnalogPull,
    /// Drive the pin as an output, then switch it to analog mode and sample the charge held by
    /// the pin capacitance. Needed on ADCs whose input switch only closes in analog mode; the
    /// sampling capacitor shares the charge, so the levels only reach part of the range.
    Drive,
    /// The on-chip DAC drives the pin (boards with `ADC_DAC`).
    #[cfg(feature = "adc-dac")]
    Dac,
}

struct Driver<'a> {
    flex: Flex<'a>,
    #[cfg(feature = "adc-dac")]
    dac: embassy_stm32::dac::DacChannel<'a, embassy_stm32::mode::Blocking>,
}

fn set_level(driver: &mut Driver<'_>, method: PinMethod, high: bool) {
    let flex = &mut driver.flex;
    match method {
        #[cfg(feature = "adc-dac")]
        PinMethod::Dac => {
            driver.dac.set(embassy_stm32::dac::u12r(if high { 4000 } else { 50 }));
            cortex_m::asm::delay(10_000);
        }
        PinMethod::Pull => {
            flex.set_as_input(if high { Pull::Up } else { Pull::Down });
            cortex_m::asm::delay(10_000);
        }
        PinMethod::AnalogPull => {
            flex.set_as_input(if high { Pull::Up } else { Pull::Down });
            flex.set_as_analog();
            cortex_m::asm::delay(10_000);
        }
        PinMethod::Drive => {
            flex.set_as_input(Pull::None);
            if high {
                flex.set_high();
            } else {
                flex.set_low();
            }
            flex.set_as_output(Speed::Low);
            cortex_m::asm::delay(10_000);
            flex.set_as_analog();
        }
    }
}

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = init();

    let adc = peri!(p, ADC);
    let pin = peri!(p, ADC_PIN);
    let mut dma = peri!(p, ADC_DMA);
    let dma_irqs = irqs!(ADC);

    // A second handle on the pin, so we can drive it while the ADC owns it as a channel.
    let mut flex = Driver {
        flex: Flex::new(unsafe { pin.clone_unchecked() }),
        #[cfg(feature = "adc-dac")]
        dac: embassy_stm32::dac::DacChannel::new_blocking::<_, embassy_stm32::dac::Ch1>(peri!(p, ADC_DAC), unsafe {
            pin.clone_unchecked()
        }),
    };

    let mut adc = Adc::new(adc, irqs!(ADC_IRQ), Config::default());
    info!("adc clock: {} Hz, resolution {:?}", adc.clock().0, adc.resolution());
    let max = adc.resolution().max_count();
    let st = SampleTime::from_bits(7);

    // Type-erase the pin once (this also puts it in analog mode).
    let mut ch = pin.degrade_adc();
    let mut vref: BorrowedAdcChannel<'_, peris::ADC> = adc.enable_vrefint().degrade_adc();

    let check_vrefint = |what: &str, v: u16| {
        // VREFINT is ~1.2 V, so the reading gives VDDA, which is 1.8 V to 3.3 V on the farm.
        let vdda_mv = 1212 * max / (v as u32).max(1);
        info!("{}: {} (max {}), VDDA {} mV", what, v, max, vdda_mv);
        assert!((1700..=3600).contains(&vdda_mv), "{} expected ~1.2 V, got {}", what, v);
    };

    // ---- find out how this ADC wants its pin driven, and what levels that gives ----
    let mut found = None;
    let methods = [
        #[cfg(feature = "adc-dac")]
        PinMethod::Dac,
        PinMethod::Pull,
        PinMethod::AnalogPull,
        PinMethod::Drive,
    ];
    for m in methods {
        let mut hi = 0;
        let mut lo = 0;
        for _ in 0..3 {
            set_level(&mut flex, m, true);
            hi = adc.blocking_read(&mut ch, st) as u32;
        }
        for _ in 0..3 {
            set_level(&mut flex, m, false);
            lo = adc.blocking_read(&mut ch, st) as u32;
        }
        info!("{:?}: high {} low {} (max {})", m, hi, lo, max);
        if hi >= max * 50 / 100 && lo <= max * 35 / 100 && hi - lo >= max * 25 / 100 {
            found = Some((m, hi, lo));
            break;
        }
    }
    let (method, hi, lo) = unwrap!(found, "could not put a known level on the pin");
    let mid = (hi + lo) / 2;
    let check_high = |what: &str, v: u16| {
        info!("{}: {} (max {})", what, v, max);
        assert!(v as u32 > mid, "{} expected high (> {}), got {}", what, mid, v);
    };
    let check_low = |what: &str, v: u16| {
        info!("{}: {} (max {})", what, v, max);
        assert!((v as u32) < mid, "{} expected low (< {}), got {}", what, mid, v);
    };

    // ---- blocking reads ----
    for high in [true, false, true, false] {
        let mut v = 0;
        for _ in 0..3 {
            set_level(&mut flex, method, high);
            v = adc.blocking_read(&mut ch, st);
        }
        if high {
            check_high("blocking high", v);
        } else {
            check_low("blocking low", v);
        }
    }
    check_vrefint("blocking vrefint", adc.blocking_read(&mut vref, st));

    // ---- interrupt-driven reads ----
    for high in [true, false] {
        let mut v = 0;
        for _ in 0..3 {
            set_level(&mut flex, method, high);
            v = adc.read(&mut ch, st).await;
        }
        if high {
            check_high("async high", v);
        } else {
            check_low("async low", v);
        }
    }
    check_vrefint("async vrefint", adc.read(&mut vref, st).await);

    // ---- DMA sequence reads ----
    // The pin goes first so that it is sampled right after it was driven; every ADC can scan
    // two channels in either order.
    info!(
        "pin channel {}, vrefint channel {}",
        ch.get_hw_channel(),
        vref.get_hw_channel()
    );

    let mut buf = [0u16; 4];
    for high in [true, false, true, false] {
        // Settle the sampling capacitor near the target level first.
        for _ in 0..2 {
            set_level(&mut flex, method, high);
            adc.blocking_read(&mut ch, st);
        }
        let seq = [(ch.reborrow_adc(), st), (vref.reborrow_adc(), st)];
        set_level(&mut flex, method, high);
        adc.read_sequence(dma.reborrow(), dma_irqs, seq.into_iter(), None, &mut buf)
            .await;
        info!("dma buf: {}", buf);
        // With the drive method the second sequence samples a partly discharged pin, so only
        // the first pin sample is checked.
        if high {
            check_high("dma high", buf[0]);
        } else {
            check_low("dma low", buf[0]);
        }
        check_vrefint("dma vrefint", buf[1]);
        check_vrefint("dma vrefint 2", buf[3]);
        if method != PinMethod::Drive {
            if high {
                check_high("dma high 2", buf[2]);
            } else {
                check_low("dma low 2", buf[2]);
            }
        }
    }

    // ---- configured sequence (one DMA transfer per call) ----
    {
        set_level(&mut flex, method, true);
        let seq = [(ch.reborrow_adc(), st), (vref.reborrow_adc(), st)];
        let mut configured = adc.configure_sequence(dma.reborrow(), seq.into_iter(), dma_irqs);
        let mut buf = [0u16; 2];
        for high in [true, false, true] {
            for _ in 0..3 {
                set_level(&mut flex, method, high);
                configured.read(&mut buf).await;
            }
            info!("configured buf: {}", buf);
            if high {
                check_high("configured high", buf[0]);
            } else {
                check_low("configured low", buf[0]);
            }
            check_vrefint("configured vrefint", buf[1]);
        }
    }

    // ---- temperature sensor: just check it converts to something plausible ----
    {
        let mut temp: BorrowedAdcChannel<'_, peris::ADC> = adc.enable_temperature().degrade_adc();
        let v = adc.blocking_read(&mut temp, st);
        info!("temperature sensor raw: {} (max {})", v, max);
        assert!(
            v > 0 && (v as u32) < max,
            "temperature sensor reading {} is not plausible",
            v
        );
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
