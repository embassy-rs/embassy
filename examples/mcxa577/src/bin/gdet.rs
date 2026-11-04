#![no_std]
#![no_main]

//! GDET runtime control checks, not glitch-protection validation.
//!
//! Requires valid, retained ROM calibration and platform-configured security /
//! privilege access, functional clocks, access gates and reset release. These
//! checks neither verify calibration nor configure tamper-response routing.
//! There is no early-startup hook: this cannot prevent resets during voltage
//! changes in the preceding `hal::init`. No glitch injection or power changes
//! are performed after initialization. `Running` describes controls only, not
//! validated protection. Restart is a best-effort adaptation of the documented
//! voltage-update procedure, not a vendor-documented restart recipe.
//! The voltage argument is derived from the settled nominal SPC regulator
//! selection with normal drive strength; it is not an electrical measurement.

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use hal::clocks::VddLevel;
use hal::gdet::{Error, Gdet, SetupError, State};
use hal::pac;
use hal::pac::spc::{ActiveCfgCoreldoVddDs, ActiveCfgCoreldoVddLvl};
use panic_probe as _;

// Read documented controls only; never inspect opaque calibration registers.
fn controls() -> (bool, u8) {
    (
        pac::DGDET0.gdet_enable1().read().en1(),
        pac::seccon.gdet0_ctrl().read().gdet_iso_sw() as u8,
    )
}

fn assert_stable(gdet: &Gdet<'_>, expected: State) {
    assert_eq!(gdet.state(), expected);
    let (gated, isolation) = controls();

    match expected {
        State::Stopped => {
            assert!(gated);
            assert_eq!(isolation, 2);
        }
        State::Running => {
            assert!(!gated);
            assert_ne!(isolation, 2);
        }
        State::Unknown => panic!("expected a stable control state"),
    }

    let control = pac::seccon.gdet0_ctrl().read();
    assert_eq!(control.gdet_evtcnt_clr() as u8, 0);
    assert_eq!(control.gdet_err_clr() as u8, 0);
    // Counter/status preservation across the driver's soft reset is not assumed.
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = hal::config::Config::default();
    let p = hal::init(config);

    // The requested default level may leave the ROM's active selection intact.
    // Interpret only the settled, normal-drive regulator path; do not change it.
    assert!(!pac::SPC0.sc().read().busy());
    let active = pac::SPC0.active_cfg().read();
    assert_eq!(active.coreldo_vdd_ds(), ActiveCfgCoreldoVddDs::Normal);
    let applied_voltage = match active.coreldo_vdd_lvl() {
        ActiveCfgCoreldoVddLvl::Mid => VddLevel::MidDriveMode,
        ActiveCfgCoreldoVddLvl::Normal => VddLevel::NormalMode,
        ActiveCfgCoreldoVddLvl::Over => VddLevel::OverDriveMode,
        _ => panic!("unsupported active core voltage selection"),
    };

    // Read-only adoption: no reset, retrimming or clock workaround on failure.
    // Defer core/control PAC reads until the constructor checks access clocks.
    let mut gdet = match Gdet::new(p.DGDET0, p.GDET0_CTRL) {
        Ok(gdet) => gdet,
        Err(SetupError::ClockNotReady) => {
            defmt::error!("GDET PRECONDITION FAILURE: access clocks/reset are not ready");
            panic!("GDET access precondition failed");
        }
        Err(_) => panic!("unexpected GDET setup failure"),
    };

    let initial_state = gdet.state();
    defmt::info!("Initial GDET control state: {:?}", initial_state);
    if initial_state == State::Running || initial_state == State::Unknown {
        let before = controls();
        let expected = if initial_state == State::Running {
            Error::AlreadyRunning
        } else {
            Error::NotStopped
        };
        assert_eq!(gdet.start(applied_voltage), Err(expected));
        assert_eq!(controls(), before);
        assert_eq!(gdet.state(), initial_state);
    }

    gdet.stop().unwrap();
    assert_stable(&gdet, State::Stopped);
    let stopped = controls();
    gdet.stop().unwrap();
    assert_eq!(controls(), stopped);
    assert_stable(&gdet, State::Stopped);

    // Use only the nominal settled regulator selection, not other voltage modes.
    gdet.start(applied_voltage).unwrap();
    assert_stable(&gdet, State::Running);
    let running = controls();
    assert_eq!(gdet.start(applied_voltage), Err(Error::AlreadyRunning));
    assert_eq!(controls(), running);
    assert_stable(&gdet, State::Running);

    gdet.stop().unwrap();
    assert_stable(&gdet, State::Stopped);
    defmt::info!("GDET control checks OK (not glitch-protection validation)");
    cortex_m::asm::bkpt();
}
