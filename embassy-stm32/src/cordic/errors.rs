use super::{Function, Scale};

/// Error for [Cordic](super::Cordic)
#[derive(Debug)]
pub enum CordicError {
    /// Config error
    ConfigError(ConfigError),
    /// Argument error
    ArgError(ArgError),
    /// Output buffer length error
    OutputLengthNotEnough,
}

#[cfg(feature = "defmt")]
impl defmt::Format for CordicError {
    fn format(&self, fmt: defmt::Formatter) {
        use CordicError::*;

        match self {
            ConfigError(e) => defmt::write!(fmt, "{}", e),
            ArgError(e) => defmt::write!(fmt, "{}", e),
            OutputLengthNotEnough => defmt::write!(fmt, "Output buffer length is not long enough"),
        }
    }
}

/// Error dring parsing [Cordic::Config](super::Config)
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

/// Error on checking input arguments
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

        let arg_string = match self.arg_type {
            ArgType::Arg1 => "ARG1",
            ArgType::Arg2 => "ARG2",
        };

        defmt::write!(fmt, " {} should be", arg_string);

        let inclusive_string = if self.inclusive_upper_bound { "=" } else { "" };

        defmt::write!(
            fmt,
            " {} <= {} <{} {}",
            self.arg_range[0],
            arg_string,
            inclusive_string,
            self.arg_range[1]
        )
    }
}

#[derive(Debug)]
pub(super) enum ArgType {
    Arg1,
    Arg2,
}
