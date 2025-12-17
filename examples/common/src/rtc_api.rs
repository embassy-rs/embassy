use core::result::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub week_day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RtcError {
    HardwareError,
    InvalidInput,
}

pub trait RtcInstance {
    fn set_date_time(&mut self, new_date_time: DateTime) -> Result<(), RtcError>;
    fn get_date_time(&mut self) -> Result<DateTime, RtcError>;
}

pub struct Rtc<T: RtcInstance> {
    peripheral: T,
}

impl<T: RtcInstance> Rtc<T> {
    pub fn new(peripheral: T) -> Self {
        Self { peripheral }
    }

    pub fn set_date_time(&mut self, new_date_time: DateTime) -> Result<(), RtcError> {
        self.peripheral.set_date_time(new_date_time)
    }

    pub fn get_date_time(&mut self) -> Result<DateTime, RtcError> {
        self.peripheral.get_date_time()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRtc {
        date_time: DateTime,
    }

    impl MockRtc {
        fn new() -> Self {
            Self {
                date_time: DateTime {
                    year: 2000,
                    month: 1,
                    day: 1,
                    week_day: 1,
                    hour: 0,
                    minute: 0,
                    second: 0,
                },
            }
        }
    }

    impl RtcInstance for MockRtc {
        fn set_date_time(&mut self, new_date_time: DateTime) -> Result<(), RtcError> {
            self.date_time = new_date_time;
            Ok(())
        }

        fn get_date_time(&mut self) -> Result<DateTime, RtcError> {
            Ok(self.date_time)
        }
    }

    #[test]
    fn test_rtc_instantiates_correctly() {
        let mock = MockRtc::new();
        let _rtc = Rtc::new(mock);
        // Just verify it compiles and can be created
        assert!(true);
    }

    #[test]
    fn test_rtc_defaults_to_correct_date() {
        let mock = MockRtc::new();
        let mut rtc = Rtc::new(mock);

        let expected_value = DateTime {
            year: 2000,
            month: 1,
            day: 1,
            week_day: 1,
            hour: 0,
            minute: 0,
            second: 0,
        };

        assert_eq!(rtc.get_date_time().unwrap(), expected_value);
    }

    #[test]
    fn test_rtc_updates_date_time_correctly() {
        let mock = MockRtc::new();
        let mut rtc = Rtc::new(mock);

        let expected_value = DateTime {
            year: 2022,
            month: 12,
            day: 18,
            week_day: 7,
            hour: 0,
            minute: 0,
            second: 0,
        };
        rtc.set_date_time(expected_value).unwrap();
        assert_eq!(rtc.get_date_time().unwrap(), expected_value);
    }
}
