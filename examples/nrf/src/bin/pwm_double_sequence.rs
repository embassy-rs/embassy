#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::pwm::{
    Config, Prescaler, Sequence, SequenceConfig, SequenceMode, SequencePwm, Sequencer,
    StartSequence,
};
use embassy_nrf::Peripherals;

use defmt_rtt as _; // global logger
use panic_probe as _;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let seq_words_0: [u16; 5] = [1000, 250, 100, 50, 0];
    let seq_words_1: [u16; 4] = [50, 100, 250, 1000];

    let mut config = Config::default();
    config.prescaler = Prescaler::Div128;
    // 1 period is 1000 * (128/16mhz = 0.000008s = 0.008ms) = 8us
    // but say we want to hold the value for 5000ms
    // so we want to repeat our value as many times as necessary until 5000ms passes
    // want 5000/8 = 625 periods total to occur, so 624 (we get the one period for free remember)
    let mut seq_config = SequenceConfig::default();
    seq_config.refresh = 624;
    // thus our sequence takes 5 * 5000ms or 25 seconds

    let mut pwm = unwrap!(SequencePwm::new_1ch(p.PWM0, p.P0_13, config));

    let sequence_0 = Sequence::new(&seq_words_0, seq_config.clone());
    let sequence_1 = Sequence::new(&seq_words_1, seq_config);
    let sequencer = Sequencer::new(&mut pwm, sequence_0, Some(sequence_1));
    unwrap!(sequencer.start(StartSequence::Zero, SequenceMode::Loop(1)));

    // we can abort a sequence if we need to before its complete with pwm.stop()
    // or stop is also implicitly called when the pwm peripheral is dropped
    // when it goes out of scope
    Timer::after(Duration::from_millis(40000)).await;
    info!("pwm stopped early!");
}
