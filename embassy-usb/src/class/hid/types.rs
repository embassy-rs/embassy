//! HID protocol types shared by the device and host classes.

/// Get/Set Protocol mapping
/// See (7.2.5 and 7.2.6): <https://www.usb.org/sites/default/files/hid1_11.pdf>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum HidProtocolMode {
    /// Hid Boot Protocol Mode
    Boot = 0,
    /// Hid Report Protocol Mode
    Report = 1,
}

impl From<u8> for HidProtocolMode {
    fn from(mode: u8) -> HidProtocolMode {
        if mode == HidProtocolMode::Boot as u8 {
            HidProtocolMode::Boot
        } else {
            HidProtocolMode::Report
        }
    }
}

/// USB HID interface subclass values.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum HidSubclass {
    /// No subclass, standard HID device.
    No = 0,
    /// Boot interface subclass, supports BIOS boot protocol.
    Boot = 1,
}

/// USB HID protocol values.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum HidBootProtocol {
    /// No boot protocol.
    None = 0,
    /// Keyboard boot protocol.
    Keyboard = 1,
    /// Mouse boot protocol.
    Mouse = 2,
}

/// Report ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReportId {
    /// IN report
    In(u8),
    /// OUT report
    Out(u8),
    /// Feature report
    Feature(u8),
}

impl ReportId {
    /// Encodes the `wValue` of a GET_REPORT/SET_REPORT request: report type high, report ID low.
    pub(crate) const fn to_value(self) -> u16 {
        let (report_type, id) = match self {
            ReportId::In(id) => (1, id),
            ReportId::Out(id) => (2, id),
            ReportId::Feature(id) => (3, id),
        };
        (report_type << 8) | id as u16
    }

    pub(crate) const fn try_from(value: u16) -> Result<Self, ()> {
        match value >> 8 {
            1 => Ok(ReportId::In(value as u8)),
            2 => Ok(ReportId::Out(value as u8)),
            3 => Ok(ReportId::Feature(value as u8)),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_id_round_trips_through_w_value() {
        for (report, value) in [
            (ReportId::In(5), 0x0105),
            (ReportId::Out(0), 0x0200),
            (ReportId::Feature(0xff), 0x03ff),
        ] {
            assert_eq!(report.to_value(), value);
            assert_eq!(ReportId::try_from(value), Ok(report));
        }
        assert!(ReportId::try_from(0x0401).is_err());
    }
}
