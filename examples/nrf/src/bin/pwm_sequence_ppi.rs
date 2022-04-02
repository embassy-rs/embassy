#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(array_from_fn)]

use core::future::pending;
use defmt::*;
use embassy::executor::Spawner;
use embassy_nrf::gpio::{Input, Pull};
use embassy_nrf::gpiote::{InputChannel, InputChannelPolarity};
use embassy_nrf::ppi::Ppi;
use embassy_nrf::pwm::{
    Config, Prescaler, SequenceConfig, SequencePwm, SingleSequenceMode, SingleSequencer,
};
use embassy_nrf::Peripherals;

use defmt_rtt as _; // global logger
use panic_probe as _;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let seq_words: [u16; 5] = [1000, 250, 100, 50, 0];

    let mut config = Config::default();
    config.prescaler = Prescaler::Div128;
    // 1 period is 1000 * (128/16mhz = 0.000008s = 0.008ms) = 8us
    // but say we want to hold the value for 250ms 250ms/8 = 31.25 periods
    // so round to 31 - 1 (we get the one period for free remember)
    // thus our sequence takes 5 * 250ms or 1.25 seconds
    let mut seq_config = SequenceConfig::default();
    seq_config.refresh = 30;

    let mut pwm = unwrap!(SequencePwm::new_1ch(p.PWM0, p.P0_13, config));

    // pwm.stop() deconfigures pins, and then the task_start_seq0 task cant work
    // so its going to have to start running in order load the configuration

    let button1 = InputChannel::new(
        p.GPIOTE_CH0,
        Input::new(p.P0_11, Pull::Up),
        InputChannelPolarity::HiToLo,
    );

    let button2 = InputChannel::new(
        p.GPIOTE_CH1,
        Input::new(p.P0_12, Pull::Up),
        InputChannelPolarity::HiToLo,
    );

    // messing with the pwm tasks is ill advised
    // Times::Ininite and Times even are seq0, Times odd is seq1
    let start = unsafe { pwm.task_start_seq0() };
    let stop = unsafe { pwm.task_stop() };

    let sequencer = SingleSequencer::new(&mut pwm, &seq_words, seq_config);
    unwrap!(sequencer.start(SingleSequenceMode::Infinite));

    let mut ppi = Ppi::new_one_to_one(p.PPI_CH1, button1.event_in(), start);
    ppi.enable();

    let mut ppi2 = Ppi::new_one_to_one(p.PPI_CH0, button2.event_in(), stop);
    ppi2.enable();

    info!("PPI setup!");
    info!("Press button 1 to start LED 1");
    info!("Press button 2 to stop LED 1");
    info!("Note! task_stop stops the sequence, but not the pin output");

    // Block forever so the above drivers don't get dropped
    pending::<()>().await;
}
