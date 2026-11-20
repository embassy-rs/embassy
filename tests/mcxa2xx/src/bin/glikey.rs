#![no_std]
#![no_main]

//! GLIKEY controller checks without protected target writes.
//!
//! Platform security/privilege access, clocks and reset release must already be
//! configured. Construction resets the FSM as in the NXP SDK example; it does
//! not change clocks, persistent locks or security policy. No target is written.

teleprobe_meta::target!(b"frdm-mcx-a266");

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use hal::glikey::{Error, Glikey, State, Target};
use panic_probe as _;

fn snapshot(glikey: &Glikey<'_>) -> (State, bool, bool, bool, bool) {
    let status = glikey.status();
    (
        status.state,
        status.errors.fsm,
        status.errors.write_index,
        status.errors.read_index,
        status.interrupt_pending,
    )
}

fn assert_clean(glikey: &Glikey<'_>, expected: State) {
    let status = glikey.status();
    assert_eq!(status.state, expected);
    assert!(!status.errors.any());
}

fn assert_initialized(glikey: &Glikey<'_>) {
    let status = glikey.status();
    assert!(matches!(status.state, State::Init | State::Locked));
    assert!(!status.errors.any());
    let control = hal::pac::GLIKEY0.ctrl_0().read();
    assert_eq!(control.write_index(), 0);
    assert_eq!(control.wr_en_0(), 2);
    assert_eq!(hal::pac::GLIKEY0.ctrl_1().read().wr_en_1(), 0);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = hal::init(hal::config::Config::default());

    // The SDK resets the FSM before enabling an index; existing locks survive.
    {
        let mut glikey = Glikey::new(p.GLIKEY0.reborrow()).expect("STOP: GLIKEY setup precondition failed");
        assert_initialized(&glikey);
        // This query changes only GLIKEY's READ_INDEX, not the protected target.
        assert!(!glikey.is_index_locked(Target::SramExecute).unwrap());
        assert_initialized(&glikey);

        for _ in 0..2 {
            glikey.enable_write(Target::SramExecute).unwrap();
            assert_clean(&glikey, State::WriteEnabled);

            let enabled = snapshot(&glikey);
            assert_eq!(glikey.enable_write(Target::SramExecute), Err(Error::NotIdle));
            assert_eq!(snapshot(&glikey), enabled);

            // No protected register is written while the grant is active.
            glikey.disable_write().unwrap();
            assert_clean(&glikey, State::Init);
            let idle = snapshot(&glikey);
            glikey.disable_write().unwrap();
            assert_eq!(snapshot(&glikey), idle);
            assert_clean(&glikey, State::Init);
        }

        assert!(!glikey.is_index_locked(Target::SramExecute).unwrap());
        assert_clean(&glikey, State::Init);
    }
    // Reconstruction starts a new FSM sequence rather than adopting live state.
    let mut glikey = Glikey::new(p.GLIKEY0.reborrow()).expect("STOP: GLIKEY reconstruction failed");
    assert_initialized(&glikey);
    assert!(!glikey.is_index_locked(Target::SramExecute).unwrap());
    assert_initialized(&glikey);

    defmt::info!("GLIKEY controller checks OK (no protected target writes)");
    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
