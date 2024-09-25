use crate::pac::lptim::vals;

/// Direction of a low-power timer channel
pub enum ChannelDirection {
    /// Use channel as a PWM output
    OutputPwm,
    /// Use channel as an input capture
    InputCapture,
}

impl From<ChannelDirection> for vals::Ccsel {
    fn from(direction: ChannelDirection) -> Self {
        match direction {
            ChannelDirection::OutputPwm => vals::Ccsel::OUTPUTCOMPARE,
            ChannelDirection::InputCapture => vals::Ccsel::INPUTCAPTURE,
        }
    }
}
