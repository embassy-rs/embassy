#![no_std]
#![no_main]

// required-features: adc

#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::Peri;
use embassy_stm32::adc::{Adc, AdcChannel, BasicInstance, BorrowedAdcChannel, Config, Instance, RxDma, SampleTimes};
use embassy_stm32::gpio::{Flex, Pull, Speed};
use embassy_stm32::interrupt::typelevel::Binding;
use embassy_stm32::mode::Async;
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

    let mut pin = peri!(p, ADC_PIN);

    // A second handle on the pin, so we can drive it while the ADC owns it as a channel.
    let mut driver = Driver {
        flex: Flex::new(unsafe { pin.clone_unchecked() }),
        #[cfg(feature = "adc-dac")]
        dac: embassy_stm32::dac::DacChannel::new_blocking::<_, embassy_stm32::dac::Ch1>(peri!(p, ADC_DAC), unsafe {
            pin.clone_unchecked()
        }),
    };

    {
        let mut adc = Adc::new(peri!(p, ADC), irqs!(ADC_IRQ), Config::default());
        let vref = adc.enable_vrefint().degrade_adc();
        let temp = adc.enable_temperature().degrade_adc();
        let ch = pin.reborrow().degrade_adc();
        test_adc(
            "ADC",
            &mut adc,
            &mut driver,
            ch,
            Some(vref),
            Some(temp),
            peri!(p, ADC_DMA),
            irqs!(ADC),
        )
        .await;
    }

    // A second ADC of a different kind on the same chip (the 16-bit ADC1 next to the 12-bit
    // ADC3 of the H72x/H73x), converting the same pin. It has no internal channels.
    #[cfg(feature = "adc-b")]
    {
        let mut adc = Adc::new(peri!(p, ADC_B), irqs!(ADC_B_IRQ), Config::default());
        let ch = pin.reborrow().degrade_adc();
        test_adc(
            "ADC_B",
            &mut adc,
            &mut driver,
            ch,
            None,
            None,
            peri!(p, ADC_B_DMA),
            irqs!(ADC_B),
        )
        .await;
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

/// Run the whole test on one ADC. `vref` and `temp` are the VREFINT and temperature sensor channels,
/// for ADCs that have them.
async fn test_adc<'d, T: Instance, D: RxDma<T>>(
    name: &str,
    adc: &mut Adc<'d, T, Async>,
    flex: &mut Driver<'_>,
    mut ch: BorrowedAdcChannel<'_, T>,
    mut vref: Option<BorrowedAdcChannel<'_, T>>,
    temp: Option<BorrowedAdcChannel<'_, T>>,
    mut dma: Peri<'_, D>,
    dma_irqs: impl Binding<D::Interrupt, embassy_stm32::dma::InterruptHandler<D>> + Copy + 'd,
) where
    <T as BasicInstance>::Regs: SampleTimes,
{
    info!(
        "{}: adc clock: {} Hz, resolution {:?}",
        name,
        adc.clock().0,
        adc.resolution()
    );
    let max = adc.resolution().max_count();
    // The longest sample time.
    let st = adc.sample_time_for_us(1000);

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
            set_level(flex, m, true);
            hi = adc.blocking_read(&mut ch, st) as u32;
        }
        for _ in 0..3 {
            set_level(flex, m, false);
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
            set_level(flex, method, high);
            v = adc.blocking_read(&mut ch, st);
        }
        if high {
            check_high("blocking high", v);
        } else {
            check_low("blocking low", v);
        }
    }
    if let Some(vref) = &mut vref {
        check_vrefint("blocking vrefint", adc.blocking_read(vref, st));
    }

    // ---- interrupt-driven reads ----
    for high in [true, false] {
        let mut v = 0;
        for _ in 0..3 {
            set_level(flex, method, high);
            v = adc.read(&mut ch, st).await;
        }
        if high {
            check_high("async high", v);
        } else {
            check_low("async low", v);
        }
    }
    if let Some(vref) = &mut vref {
        check_vrefint("async vrefint", adc.read(vref, st).await);
    }

    // ---- DMA sequence reads ----
    // The pin goes first so that it is sampled right after it was driven; every ADC can scan
    // two channels in either order.
    info!(
        "pin channel {}, vrefint channel {:?}",
        ch.get_hw_channel(),
        vref.as_ref().map(|v| v.get_hw_channel())
    );

    for high in [true, false, true, false] {
        // Settle the sampling capacitor near the target level first.
        for _ in 0..2 {
            set_level(flex, method, high);
            adc.blocking_read(&mut ch, st);
        }
        let mut buf = [0u16; 4];
        set_level(flex, method, high);
        match &mut vref {
            Some(vref) => {
                let seq = [(ch.reborrow_adc(), st), (vref.reborrow_adc(), st)];
                adc.read_sequence(dma.reborrow(), dma_irqs, seq.into_iter(), None, &mut buf)
                    .await;
            }
            None => {
                let seq = [(ch.reborrow_adc(), st)];
                adc.read_sequence(dma.reborrow(), dma_irqs, seq.into_iter(), None, &mut buf[..1])
                    .await;
            }
        }
        info!("dma buf: {}", buf);
        // With the drive method the second sequence samples a partly discharged pin, so only
        // the first pin sample is checked.
        if high {
            check_high("dma high", buf[0]);
        } else {
            check_low("dma low", buf[0]);
        }
        if vref.is_some() {
            check_vrefint("dma vrefint", buf[1]);
            check_vrefint("dma vrefint 2", buf[3]);
        }
        if vref.is_some() && method != PinMethod::Drive {
            // The vrefint conversion in between discharges the sampling capacitor towards
            // VREFINT, so the second pin sample only recovers partway towards the pin level.
            // Require it to be clearly on the expected side of both the midpoint and the
            // vrefint level measured in this same sequence.
            let vref_avg = (buf[1] as u32 + buf[3] as u32) / 2;
            let margin = max / 64;
            info!("dma {} 2: {} (max {})", if high { "high" } else { "low" }, buf[2], max);
            if high {
                let threshold = mid.min(vref_avg) + margin;
                assert!(
                    buf[2] as u32 > threshold,
                    "dma high 2 expected high (> {}), got {}",
                    threshold,
                    buf[2]
                );
            } else {
                let threshold = mid.max(vref_avg) - margin;
                assert!(
                    (buf[2] as u32) < threshold,
                    "dma low 2 expected low (< {}), got {}",
                    threshold,
                    buf[2]
                );
            }
        }
    }

    // ---- configured sequence (one DMA transfer per call) ----
    {
        set_level(flex, method, true);
        let mut buf = [0u16; 2];
        let len = if vref.is_some() { 2 } else { 1 };
        let mut configured = match &mut vref {
            Some(vref) => {
                let seq = [(ch.reborrow_adc(), st), (vref.reborrow_adc(), st)];
                adc.configure_sequence(dma.reborrow(), seq.into_iter(), dma_irqs)
            }
            None => {
                let seq = [(ch.reborrow_adc(), st)];
                adc.configure_sequence(dma.reborrow(), seq.into_iter(), dma_irqs)
            }
        };
        for high in [true, false, true] {
            for _ in 0..3 {
                set_level(flex, method, high);
                configured.read(&mut buf[..len]).await;
            }
            info!("configured buf: {}", buf);
            if high {
                check_high("configured high", buf[0]);
            } else {
                check_low("configured low", buf[0]);
            }
            if len == 2 {
                check_vrefint("configured vrefint", buf[1]);
            }
        }
    }

    // ---- temperature sensor: just check it converts to something plausible ----
    if let Some(mut temp) = temp {
        let v = adc.blocking_read(&mut temp, st);
        info!("temperature sensor raw: {} (max {})", v, max);
        assert!(
            v > 0 && (v as u32) < max,
            "temperature sensor reading {} is not plausible",
            v
        );
    }

    info!("{}: OK", name);
}
