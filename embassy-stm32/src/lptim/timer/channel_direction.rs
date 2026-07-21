use crate::lptim::vals::Ccsel;

/// Direction of a low-power timer channel
pub enum ChannelDirection {
    /// Use channel as a PWM output
    OutputPwm,
    /// Use channel as an input capture
    InputCapture,
}

impl ChannelDirection {
    pub(crate) fn ccsel(&self) -> Ccsel {
        match self {
            ChannelDirection::OutputPwm => Ccsel::OutputCompare,
            ChannelDirection::InputCapture => Ccsel::InputCapture,
        }
    }

    #[cfg(lptim_n6)]
    pub(crate) fn ccsel_bool(&self) -> bool {
        match self {
            ChannelDirection::OutputPwm => false,
            ChannelDirection::InputCapture => true,
        }
    }
}
