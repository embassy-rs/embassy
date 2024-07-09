use super::{Function, Scale};

/// Error for [Cordic](super::Cordic)
#[derive(Debug)]
pub enum CordicError {
    /// Config error
    ConfigError(ConfigError),
    /// Argument length is incorrect
    ArgumentLengthIncorrect,
    /// Result buffer length error
    ResultLengthNotEnough,
    /// Input value is out of range for Q1.x format
    NumberOutOfRange(NumberOutOfRange),
    /// Argument error
    ArgError(ArgError),
}

impl From<ConfigError> for CordicError {
    fn from(value: ConfigError) -> Self {
        Self::ConfigError(value)
    }
}

impl From<NumberOutOfRange> for CordicError {
    fn from(value: NumberOutOfRange) -> Self {
        Self::NumberOutOfRange(value)
    }
}

impl From<ArgError> for CordicError {
    fn from(value: ArgError) -> Self {
        Self::ArgError(value)
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for CordicError {
    fn format(&self, fmt: defmt::Formatter) {
        use CordicError::*;

        match self {
            ConfigError(e) => defmt::write!(fmt, "{}", e),
            ResultLengthNotEnough => defmt::write!(fmt, "Output buffer length is not long enough"),
            ArgumentLengthIncorrect => defmt::write!(fmt, "Argument length incorrect"),
            NumberOutOfRange(e) => defmt::write!(fmt, "{}", e),
            ArgError(e) => defmt::write!(fmt, "{}", e),
        }
    }
}

/// Error during parsing [Cordic::Config](super::Config)
#[allow(dead_code)]
#[derive(Debug)]
pub struct ConfigError {
    pub(super) func: Function,
    pub(super) scale_range: [u8; 2],
}

#[cfg(feature = "defmt")]
impl defmt::Format for ConfigError {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "For FUNCTION: {},", self.func);

        if self.scale_range[0] == self.scale_range[1] {
            defmt::write!(fmt, " SCALE value should be {}", self.scale_range[0])
        } else {
            defmt::write!(
                fmt,
                " SCALE value should be {} <= SCALE <= {}",
                self.scale_range[0],
                self.scale_range[1]
            )
        }
    }
}

/// Input value is out of range for Q1.x format
#[allow(missing_docs)]
#[derive(Debug)]
pub enum NumberOutOfRange {
    BelowLowerBound,
    AboveUpperBound,
}

#[cfg(feature = "defmt")]
impl defmt::Format for NumberOutOfRange {
    fn format(&self, fmt: defmt::Formatter) {
        use NumberOutOfRange::*;

        match self {
            BelowLowerBound => defmt::write!(fmt, "input value should be equal or greater than -1"),
            AboveUpperBound => defmt::write!(fmt, "input value should be equal or less than 1"),
        }
    }
}

/// Error on checking input arguments
#[allow(dead_code)]
#[derive(Debug)]
pub struct ArgError {
    pub(super) func: Function,
    pub(super) scale: Option<Scale>,
    pub(super) arg_range: [f32; 2], // only for debug display, f32 is ok
    pub(super) inclusive_upper_bound: bool,
    pub(super) arg_type: ArgType,
}

#[cfg(feature = "defmt")]
impl defmt::Format for ArgError {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "For FUNCTION: {},", self.func);

        if let Some(scale) = self.scale {
            defmt::write!(fmt, " when SCALE is {},", scale);
        }

        defmt::write!(fmt, " {} should be", self.arg_type);

        if self.inclusive_upper_bound {
            defmt::write!(
                fmt,
                " {} <= {} <= {}",
                self.arg_range[0],
                self.arg_type,
                self.arg_range[1]
            )
        } else {
            defmt::write!(
                fmt,
                " {} <= {} < {}",
                self.arg_range[0],
                self.arg_type,
                self.arg_range[1]
            )
        };
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(super) enum ArgType {
    Arg1,
    Arg2,
}
