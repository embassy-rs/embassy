//! Runtime control of the ROM-configured digital glitch detector.
//!
//! The platform must provide valid, retained ROM calibration and configuration,
//! preserved functional/reference clocks, and permitted security/privilege access
//! to DGDET0 and SECCON. Access violations can fault; they are not recoverable
//! driver errors. Only the documented access gates and reset release are checked.
//! Configuration lost across a power transition must be restored by the platform
//! before using this driver. Factory trim and response routing are not changed.
//!
//! Start/stop sequencing is a best-effort extension of the documented voltage
//! update procedure, not a vendor-documented restart recipe. `Running` describes
//! only the core gate and software isolation controls, not active protection,
//! detector readiness, characterized thresholds, or reset-response policy.
//! No wake vote is retained and dropping the driver does not change hardware.

use crate::clocks::{VddLevel, gdet_access_ready};
use crate::pac::gdet::VolSel;
use crate::pac::seccon::{Gdet0CtrlGdetErrClr, Gdet0CtrlGdetEvtcntClr, Gdet0CtrlGdetIsoSw};
use crate::{Peri, pac, peripherals};

struct Info {
    regs: pac::gdet::Gdet,
    control: pac::seccon::Seccon,
}

static INFO: Info = Info {
    regs: pac::DGDET0,
    control: pac::SECCON0,
};

/// Observed core clock gate and software isolation controls.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    /// Core clock ungated and software isolation released; not protection ready.
    Running,
    /// Core clock gated and software isolation forced.
    Stopped,
    /// Mixed controls, or access clocks/reset are not ready.
    Unknown,
}

/// Read-only adoption failure.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
    /// A documented access clock is gated or DGDET0 reset is asserted.
    ClockNotReady,
}

/// Runtime control failure.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Start was rejected without writes because the controls indicate running.
    AlreadyRunning,
    /// Start was rejected without writes because the controls are mixed.
    NotStopped,
    /// A documented access clock is gated or DGDET0 reset is asserted.
    ClockNotReady,
    /// A control write did not read back as expected.
    ///
    /// Isolation and gating were attempted as best-effort containment. A stopped
    /// state is not guaranteed; inspect `state()` before further operations.
    ReadbackMismatch,
}

/// Exclusive owner of DGDET0 and its narrow SECCON control register.
pub struct Gdet<'d> {
    _detector: Peri<'d, peripherals::DGDET0>,
    _control: Peri<'d, peripherals::GDET0_CTRL>,
    info: &'static Info,
}

impl<'d> Gdet<'d> {
    /// Adopts the existing detector without writing any hardware controls.
    ///
    /// The platform prerequisites in the module documentation apply even when
    /// the detector was disabled at boot. This does not establish valid trim,
    /// configure security access, enable clocks, or release reset.
    pub fn new(
        detector: Peri<'d, peripherals::DGDET0>,
        control: Peri<'d, peripherals::GDET0_CTRL>,
    ) -> Result<Self, SetupError> {
        if !gdet_access_ready() {
            return Err(SetupError::ClockNotReady);
        }
        Ok(Self {
            _detector: detector,
            _control: control,
            info: &INFO,
        })
    }

    /// Reads the actual controls, or returns `Unknown` if access is not ready.
    pub fn state(&self) -> State {
        critical_section::with(|_| {
            if !gdet_access_ready() {
                return State::Unknown;
            }
            self.read_state()
        })
    }

    /// Forces isolation before gating the core; normalizes mixed controls.
    ///
    /// An already fully stopped detector is left untouched. Status and event
    /// counters are not cleared. A readback failure triggers best-effort
    /// containment but does not guarantee a stopped state.
    pub fn stop(&mut self) -> Result<(), Error> {
        critical_section::with(|_| {
            if !gdet_access_ready() {
                return Err(Error::ClockNotReady);
            }

            if self.read_state() == State::Stopped {
                return Ok(());
            }

            self.write_isolation(Gdet0CtrlGdetIsoSw::Enable);
            cortex_m::asm::dsb();

            if !self.is_isolated() {
                return self.readback_failure();
            }

            self.info.regs.gdet_enable1().write(|w| w.set_en1(true));
            cortex_m::asm::dsb();

            if self.read_state() != State::Stopped {
                return self.readback_failure();
            }
            Ok(())
        })
    }

    /// Starts only from `Stopped`, updating for the already applied voltage.
    ///
    /// `applied_voltage` must describe the physically applied and settled core
    /// voltage. This method does not change supply voltage, clocks, or memory
    /// timings, and does not protect earlier voltage transitions. Retained ROM
    /// calibration and the other module prerequisites must still hold.
    ///
    /// The core is ungated under forced isolation, then voltage controls are
    /// updated and soft reset is written high once. Isolation is released last.
    /// No reset pulse, self-clearing behavior, delay, or readiness is assumed.
    /// `AlreadyRunning` and `NotStopped` leave hardware untouched.
    pub fn start(&mut self, applied_voltage: VddLevel) -> Result<(), Error> {
        critical_section::with(|_| {
            if !gdet_access_ready() {
                return Err(Error::ClockNotReady);
            }

            match self.read_state() {
                State::Running => return Err(Error::AlreadyRunning),
                State::Unknown => return Err(Error::NotStopped),
                State::Stopped => {}
            }

            self.info.regs.gdet_enable1().write(|w| w.set_en1(false));
            cortex_m::asm::dsb();

            if self.info.regs.gdet_enable1().read().en1() || !self.is_isolated() {
                return self.readback_failure();
            }

            let voltage = match applied_voltage {
                VddLevel::MidDriveMode => VolSel::Voltage1V0,
                #[cfg(feature = "_mcxa5xx")]
                VddLevel::NormalMode => VolSel::Voltage1V1,
                VddLevel::OverDriveMode => VolSel::Voltage1V2,
            };

            self.info.regs.gdet_dly_ctrl().write(|w| {
                w.set_vol_sel(voltage);
                w.set_sw_vol_ctrl(true);
            });

            cortex_m::asm::dsb();
            let delay = self.info.regs.gdet_dly_ctrl().read();

            if delay.vol_sel() != voltage || !delay.sw_vol_ctrl() {
                return self.readback_failure();
            }

            self.info.regs.gdet_reset().write(|w| w.set_sft_rst(true));
            cortex_m::asm::dsb();
            self.write_isolation(Gdet0CtrlGdetIsoSw::Disable0);
            cortex_m::asm::dsb();

            if self.read_state() != State::Running {
                return self.readback_failure();
            }

            Ok(())
        })
    }

    // Call only with access ready and inside a critical section.
    fn read_state(&self) -> State {
        match (self.info.regs.gdet_enable1().read().en1(), self.is_isolated()) {
            (false, false) => State::Running,
            (true, true) => State::Stopped,
            _ => State::Unknown,
        }
    }

    fn is_isolated(&self) -> bool {
        self.info.control.gdet0_ctrl().read().gdet_iso_sw() == Gdet0CtrlGdetIsoSw::Enable
    }

    // Construct a fresh typed value: RMW could replay clear commands or status.
    // Call only inside a critical section; the token owns this register alone.
    fn write_isolation(&self, isolation: Gdet0CtrlGdetIsoSw) {
        let mut control = pac::seccon::Gdet0Ctrl::default();
        control.set_gdet_evtcnt_clr(Gdet0CtrlGdetEvtcntClr::Disable);
        control.set_gdet_err_clr(Gdet0CtrlGdetErrClr::Disable);
        control.set_gdet_iso_sw(isolation);
        self.info.control.gdet0_ctrl().write_value(control);
    }

    fn readback_failure(&self) -> Result<(), Error> {
        if gdet_access_ready() {
            self.write_isolation(Gdet0CtrlGdetIsoSw::Enable);
            cortex_m::asm::dsb();
            let _ = self.is_isolated();
            self.info.regs.gdet_enable1().write(|w| w.set_en1(true));
            cortex_m::asm::dsb();
            let _ = self.read_state();
        }
        Err(Error::ReadbackMismatch)
    }
}
