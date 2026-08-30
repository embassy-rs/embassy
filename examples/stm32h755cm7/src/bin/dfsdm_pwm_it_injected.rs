#![no_std]
#![no_main]
use core::mem::MaybeUninit;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dfsdm::config_types::{CkoutDivider, FilterOrder, FilterParameters, InternalSpiMode};
use embassy_stm32::dfsdm::{FilterConfig, Flt0, TransceiverConfig, TransceiverConfigOnline};
use embassy_stm32::gpio::{Level, Output, OutputType, Speed};
use embassy_stm32::peripherals::DFSDM1;
use embassy_stm32::rcc::{self};
use embassy_stm32::time::{Hertz, khz};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::{SharedData, bind_interrupts, dfsdm, pac};
use embassy_time::Instant;
use panic_probe as _;

#[unsafe(link_section = ".ram_d3.shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

bind_interrupts!(struct Irqs {
    DFSDM1_FLT0 => dfsdm::InterruptHandler<DFSDM1, Flt0>;
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

    //==================================================
    // Goal: Dim LED according to PDM mic input using DFSM, DMA and PWM;
    // Maybe enable breakinput to blink rapidly when detecting disconnection
    //==================================================

    // A0   PA3     MIC_SEL
    // A2   PC3_C   MIT_DAT
    // A4   PC2_C   MIC_CLK

    let p = embassy_stm32::init_primary(config, &SHARED_DATA);
    info!("Hello World!");

    // let mut ld1 = Output::new(p.PB0, Level::High, Speed::Low);
    // let mut ld2 = Output::new(p.PE1, Level::High, Speed::Low);
    // let mut ld3 = Output::new(p.PB14, Level::High, Speed::Low);

    let ld2_pwm_pin: PwmPin<'_, embassy_stm32::peripherals::TIM12, embassy_stm32::timer::Ch1> =
        PwmPin::new(p.PB14, OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        p.TIM12,
        Some(ld2_pwm_pin),
        None,
        None,
        None,
        khz(10),
        Default::default(),
    );
    let mut pwm_ld2 = pwm.ch1();
    pwm_ld2.enable();

    // Setup mic as left channel, data is valid at clock low, to rising clock edge samples signal
    let _mic_sel = Output::new(p.PA3, Level::Low, Speed::Low);

    let mic_clk_freq = Hertz::mhz(2);
    let prescaler = rcc::frequency::<DFSDM1>() / mic_clk_freq;

    println!("Trying prescaler={}", prescaler);
    // Start driver instantiation using DFSDM1 with a CKOUT pin on pin C2
    let dfsdm1 = dfsdm::Dfsdm::new_ckout(
        p.DFSDM1,
        p.PC2,
        dfsdm::config_types::CkoutSource::System,
        CkoutDivider::try_from(prescaler as u16).expect("Divider wrong?"),
    );
    println!("Running with prescaler={}", prescaler);

    let split = dfsdm1.configure_pins(|creator| {
        (
            creator.ch0.none(),
            creator.ch1.datin(p.PC3),
            creator.ch2.none(),
            creator.ch3.none(),
            creator.ch4.none(),
            creator.ch5.none(),
            creator.ch6.none(),
            creator.ch7.none(),
        )
    });

    let tcv_cfg = TransceiverConfig::default();
    let tcv_cfg_online = TransceiverConfigOnline::default();
    let channel_mic = split
        .ch1
        .new_spi_int(InternalSpiMode::SpiRising)
        .configure(&tcv_cfg, &tcv_cfg_online)
        .enable();

    //TODO the enable semantics should really also be linked to channel assignments in filters?

    // 2MHz/100/50 = 400Hz 3dB frequency

    let filter_params =
        FilterParameters::try_new(FilterOrder::Sinc3 { fosr: 100 }, 50).expect("This is inside the bounds");
    let gain = filter_params.total_gain();

    let flt_cfg = FilterConfig {
        // filter_cfg: FilterParameters::try_new(FilterOrder::Sinc3 { fosr: 5 }, 4).expect("This is inside the bounds"),
        filter_params,
    };
    let mut flt0 = split
        .flt0
        .configure(&flt_cfg)
        .assign_regular_transceiver(&channel_mic)
        .assign_injected_transceivers(&[&channel_mic])
        .enable();

    let mut dc_offset: i32 = 0;
    let mut bass_signal: i32 = 0;
    let mut envelope: u32 = 0;

    // Tuning parameters
    const LPF_SHIFT: u32 = 3; // Low-pass filter speed (lower = tighter bass, try 2 to 4)
    const DECAY_SHIFT: u32 = 5; // Envelope decay speed
    const DC_SHIFT: u32 = 8; // DC blocker speed
    let _ = gain; // theoretical filter gain, no longer used directly for scaling

    // ---- AGC (automatic gain control) ----
    // Feedback loop: a multiplicative gain applied to `envelope`, adjusted
    // over time so the *average* output level settles near a target
    // brightness. Fast attack (gain goes down quickly) protects against
    // clipping on loud content; slow release (gain goes up slowly) lets
    // quieter / less bass-heavy material gradually brighten up on its own,
    // regardless of what played before it.

    // Fixed-point gain, Q16 (65536 = 1.0x)
    let mut agc_gain_q16: u32 = 1 << 16;
    const AGC_GAIN_MIN_Q16: u32 = 1 << 10; // floor, ~0.016x, avoids gain collapsing to zero
    const AGC_GAIN_MAX_Q16: u32 = 1 << 24; // ceiling, ~256x, avoids amplifying noise floor forever on silence

    // Running average of the (gain-scaled) output level, used to drive the AGC loop
    let mut avg_scaled: u32 = 0;
    const AVG_SHIFT: u32 = 7; // averaging window, roughly a few hundred ms at this loop rate

    // Target: aim for the average scaled output to sit at this fraction of max_duty_cycle.
    // Not 100%, so there's still headroom for transients/peaks above the average.
    const TARGET_NUM: u64 = 11;
    const TARGET_DEN: u64 = 20; // ~55%

    // Gain adjustment speed: attack (turn gain down) is faster than release
    // (turn gain up), same idea as a compressor - react quickly to loud
    // content, recover slowly so it doesn't pump/flicker.
    const GAIN_ATTACK_SHIFT: u32 = 9; // faster
    const GAIN_RELEASE_SHIFT: u32 = 16; // slower

    // Below this envelope level we don't push the gain up further - avoids
    // the AGC amplifying mic self-noise/silence into a slowly brightening glow.
    const NOISE_GATE: u32 = 50;

    // EMA step with rounding and a guaranteed minimum step of 1, so the
    // filter can't stall (a plain ">>" would give step=0 whenever
    // 0 < |diff| < 2^shift, freezing the filter on small differences).
    #[inline]
    fn ema_step(diff: i32, shift: u32) -> i32 {
        if diff == 0 {
            0
        } else if diff > 0 {
            ((diff + (1 << (shift - 1))) >> shift).max(1)
        } else {
            ((diff - (1 << (shift - 1))) >> shift).min(-1)
        }
    }

    // ---- Metering ----
    // Tracks how much wall-clock time is spent actually computing (from the
    // instant a result becomes available to the instant processing for that
    // sample finishes) versus how much time is spent idling/polling while
    // waiting for the next result. Printed once per second.
    let mut wait_start = Instant::now(); // start of the current "waiting for a sample" span
    let mut busy_us_acc: u64 = 0; // accumulated compute time this reporting window
    let mut wait_us_acc: u64 = 0; // accumulated wait time this reporting window
    let mut sample_count: u32 = 0; // samples processed this reporting window
    let mut stats_window_start = Instant::now();
    const STATS_INTERVAL_US: u64 = 1_000_000; // report every 1s

    loop {
        let (data, _channel) = flt0.read_injected(Irqs).await;

        let result_ready_at = Instant::now();
        let wait_dur = result_ready_at - wait_start;

        // 1. DC Blocker (Track the slow-moving average to remove mic bias)
        // Shift by 8 makes it track very slowly, ignoring the fast bass beats
        dc_offset += ema_step(data - dc_offset, DC_SHIFT);
        let ac_signal = data - dc_offset;

        // 2. Rectifier (Absolute value of the AC signal)
        let abs = ac_signal.unsigned_abs() as i32;

        // 3. Software Low-Pass Filter (Kills mid/high bleed!)
        // Formula: bass_signal = bass_signal + (abs - bass_signal) / 2^LPF_SHIFT
        bass_signal += ema_step(abs - bass_signal, LPF_SHIFT);

        // Convert to u32 for the envelope math
        let bass_abs = bass_signal as u32;

        // 4. Envelope Follower (Diode Detector)
        if bass_abs > envelope {
            envelope = bass_abs; // Fast attack
        } else {
            let decay_step = envelope >> DECAY_SHIFT; // Slow decay
            if decay_step > 0 {
                envelope -= decay_step;
            } else if envelope > 0 {
                envelope -= 1;
            }
        }

        // 5. AGC: apply current gain, then adjust gain based on how the
        // running average compares to the target.
        let max_duty = pwm_ld2.max_duty_cycle();

        let scaled = ((envelope as u64) * (agc_gain_q16 as u64)) >> 16;
        let scaled = scaled.min(u32::MAX as u64) as u32;

        avg_scaled = (avg_scaled as i64 + ema_step(scaled as i32 - avg_scaled as i32, AVG_SHIFT) as i64) as u32;

        let target = ((max_duty as u64) * TARGET_NUM / TARGET_DEN) as u32;

        if avg_scaled > target {
            let reduction = (agc_gain_q16 >> GAIN_ATTACK_SHIFT).max(1);
            agc_gain_q16 = agc_gain_q16.saturating_sub(reduction).max(AGC_GAIN_MIN_Q16);
        } else if avg_scaled < target && envelope > NOISE_GATE {
            let increase = (agc_gain_q16 >> GAIN_RELEASE_SHIFT).max(1);
            agc_gain_q16 = agc_gain_q16.saturating_add(increase).min(AGC_GAIN_MAX_Q16);
        }

        // 6. PWM Output, clamped defensively against max_duty_cycle
        let duty = scaled.min(max_duty);
        pwm_ld2.set_duty_cycle(duty);

        flt0.start_regular_conversion();

        let processing_done_at = Instant::now();
        let busy_dur = processing_done_at - result_ready_at;

        busy_us_acc += busy_dur.as_micros();
        wait_us_acc += wait_dur.as_micros();
        sample_count += 1;

        // Next wait span starts now
        wait_start = Instant::now();

        // Print metering stats roughly once per second
        let now = Instant::now();
        let window_elapsed_us = (now - stats_window_start).as_micros();
        if window_elapsed_us >= STATS_INTERVAL_US {
            let total_us = busy_us_acc + wait_us_acc;
            let busy_pct = if total_us > 0 {
                busy_us_acc as f32 * 100.0 / total_us as f32
            } else {
                0.0
            };
            let avg_period_us = if sample_count > 0 {
                total_us / sample_count as u64
            } else {
                0
            };

            info!(
                "metering: samples={} busy_us={} wait_us={} busy_pct={}% avg_period_us={} window_us={}",
                sample_count,
                busy_us_acc,
                wait_us_acc,
                Display2Format(&format_args!("{:.3}", busy_pct)),
                avg_period_us,
                window_elapsed_us,
            );

            busy_us_acc = 0;
            wait_us_acc = 0;
            sample_count = 0;
            stats_window_start = now;
        }
    }
}
