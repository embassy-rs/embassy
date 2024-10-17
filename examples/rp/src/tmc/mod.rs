//! Trinamic Motor Control Module.
//!
//! Currently includes support for the:
//!
//! [X] TMC5130 IC
//! [X] TMCM6214 6-axis Drive Module
#[path = "tmc5130/mod.rs"]
pub mod tmc5130;

pub mod params;

#[derive(Debug, Clone, Copy)]
pub enum Speed {
    Rpm(f32),
    Pps(i32),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    Continuous = 0,
    Finite = 1,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    #[default]
    Stopped = 0,
    Clockwise = 1,
    CounterClockwise = 2,
}

impl Status {
    pub fn reverse(&self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
            Self::Stopped => Self::Stopped,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MotorIndex {
    One = 0,
    Two = 1,
    Three = 2,
    Four = 3,
    Five = 4,
    Six = 5,
}

#[derive(Debug)]
pub struct Motor {
    pub direction: Status,
    pub id: MotorIndex,
    pub last_position: (i32, u64),
    pub mode: Mode,
    pub position: i32,
    pub pps: i32, // pps == pulses per seconds == microsteps/second
    pub target_position: i32,
    pub dirty: bool,
}

impl Motor {
    pub fn new(id: MotorIndex) -> Self {
        Self {
            direction: Status::Stopped,
            id,
            last_position: (0, 0),
            mode: Mode::Continuous,
            position: 0,
            pps: 0,
            target_position: 0,
            dirty: false,
        }
    }
}

pub const TMC_MICROSTEPS_PER_STEP: u64 = 256;
pub const TMC_STEPS_PER_REV: f32 = 200.;

/// Calculate pps for a given rpm value.
pub fn rpm_to_pps<T: From<core::primitive::f64>>(rpm: f32) -> T {
    let pps: f64 = revolutions_to_usteps::<f64>(rpm as f64) / 60_f64;
    T::from(pps)
}

/// Calculate rpm for a given pps value.
pub fn pps_to_rpm<T: From<core::primitive::f64>>(pps: i32) -> T {
    let rpm = (pps as f64) / (TMC_MICROSTEPS_PER_STEP as f64) / (TMC_STEPS_PER_REV as f64) * 60_f64;
    T::from(rpm)
}

/// Calculate rpm for a given pps value.
pub fn revolutions_to_usteps<T: From<core::primitive::f64>>(revolutions: f64) -> T {
    let usteps: f64 = (TMC_MICROSTEPS_PER_STEP as f64) * (TMC_STEPS_PER_REV as f64) * revolutions;
    T::from(usteps)
}

#[inline(always)]
pub const fn vactual_to_signed_pps(vactual: u32) -> i32 {
    const D: i32 = 0b1000_0000_0000_0000_0000_0000;

    let out = vactual as i32 - D;
    if out < 0 {
        out + D
    } else {
        out - D
    }
}
