#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dfsdm::config_types::{CkoutDivider, FilterOrder, FilterParameters, InternalSpiMode};
use embassy_stm32::dfsdm::{FilterConfig, TransceiverConfig, TransceiverConfigOnline};
use embassy_stm32::gpio::{Level, Output, OutputType, Speed};
use embassy_stm32::peripherals::DFSDM1;
use embassy_stm32::rcc::{self};
use embassy_stm32::time::{Hertz, khz};
use embassy_stm32::timer::simple_pwm::{PwmPin, PwmPinConfig, SimplePwm, SimplePwmChannel, SimplePwmChannels};
use embassy_stm32::{SharedData, dfsdm};
use embedded_hal::Pwm;
use panic_probe as _;

#[unsafe(link_section = ".ram_d3.shared_data")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

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

    let ford: u8 = 3;
    let fosr: u16 = 100;
    let iosr: u16 = 50;
    let gain = (fosr as u64).pow(ford as u32) * iosr as u64;
    // 2MHz/100/50 = 400Hz 3dB frequency

    let flt_cfg = FilterConfig {
        // filter_cfg: FilterParameters::try_new(FilterOrder::Sinc3 { fosr: 5 }, 4).expect("This is inside the bounds"),
        filter_params: FilterParameters::try_new(FilterOrder::Sinc3 { fosr: fosr }, iosr)
            .expect("This is inside the bounds"),
    };
    let mut flt0 = split
        .flt0
        .configure(&flt_cfg)
        .assign_regular_transceiver(&channel_mic)
        .enable();

    flt0.start_regular_conversion();

    let mut dc_offset: i32 = 0;
    let mut bass_signal: i32 = 0;
    let mut envelope: u32 = 0;

    // Tuning parameters
    const LPF_SHIFT: i32 = 3; // Low-pass filter speed (lower = tighter bass, try 2 to 4)
    const DECAY_SHIFT: u32 = 5; // Envelope decay speed

    loop {
        // ch_test.write_sample_standard(10);
        if let Some((data, _channel, _rpend)) = flt0.try_read_regular_result() {
            // 1. DC Blocker (Track the slow-moving average to remove mic bias)
            // Shift by 8 makes it track very slowly, ignoring the fast bass beats
            dc_offset += (data - dc_offset) >> 8;
            let ac_signal = data - dc_offset;

            // 2. Rectifier (Absolute value of the AC signal)
            let abs = ac_signal.abs() as i32;

            // 3. Software Low-Pass Filter (Kills mid/high bleed!)
            // Formula: bass_signal = bass_signal + (abs - bass_signal) / 2^LPF_SHIFT
            bass_signal += (abs - bass_signal) >> LPF_SHIFT;

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

            // 5. PWM Output
            // Note: Recalculate your 'gain' based on your new Sinc3 parameters!

            let duty = ((envelope as u64) * (pwm_ld2.max_duty_cycle() as u64) / (gain as u64)) as u32;
            pwm_ld2.set_duty_cycle(duty);

            flt0.start_regular_conversion();
        }

        // info!("led on!");
        // led.set_high();
        // Timer::after_millis(500).await;

        // info!("led off!");
        // led.set_low();
        // Timer::after_millis(500).await;
    }
}
