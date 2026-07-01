use embassy_time_driver::TICK_HZ;
use embedded_hal_timer::{Alarm, OverflowError, Timer};

use crate::{Duration, Instant};

/// Embedded-hal implementation on top of embassy-time
#[derive(Debug)]
pub struct HalTimer(Option<Instant>);

impl HalTimer {
    /// Construct a new timer that's not running.
    /// Make sure to call [Self::start].
    pub fn new() -> Self {
        Self(None)
    }
}

impl Timer for HalTimer {
    fn start(&mut self) {
        self.0 = Some(Instant::now());
    }

    fn tickrate(&self) -> u64 {
        TICK_HZ
    }

    fn elapsed_ticks(&self) -> Result<u64, OverflowError> {
        Ok(self.0.map(|i| i.elapsed().as_ticks()).unwrap_or(u64::MAX))
    }

    fn elapsed_nanos(&self) -> Result<u64, OverflowError> {
        Ok(self.0.map(|i| i.elapsed().as_nanos()).unwrap_or(u64::MAX))
    }

    fn elapsed_micros(&self) -> Result<u64, OverflowError> {
        Ok(self.0.map(|i| i.elapsed().as_micros()).unwrap_or(u64::MAX))
    }

    fn elapsed_millis(&self) -> Result<u64, OverflowError> {
        Ok(self.0.map(|i| i.elapsed().as_millis()).unwrap_or(u64::MAX))
    }

    fn elapsed_secs(&self) -> Result<u64, OverflowError> {
        Ok(self.0.map(|i| i.elapsed().as_secs()).unwrap_or(u64::MAX))
    }

    fn max_ticks(&self) -> u64 {
        u64::MAX
    }

    fn max_nanos(&self) -> u64 {
        u64::MAX
    }

    fn max_micros(&self) -> u64 {
        u64::MAX
    }

    fn max_millis(&self) -> u64 {
        u64::MAX
    }

    fn max_secs(&self) -> u64 {
        u64::MAX
    }
}

impl Alarm for HalTimer {
    async fn wait_until_ticks(&mut self, value: u64) -> Result<(), OverflowError> {
        let end_time = self
            .0
            .ok_or(OverflowError)?
            .checked_add(Duration::from_ticks(value))
            .ok_or(OverflowError)?;
        crate::Timer::at(end_time).await;
        Ok(())
    }

    async fn wait_until_nanos(&mut self, value: u64) -> Result<(), OverflowError> {
        let end_time = self
            .0
            .ok_or(OverflowError)?
            .checked_add(Duration::from_nanos(value))
            .ok_or(OverflowError)?;
        crate::Timer::at(end_time).await;
        Ok(())
    }

    async fn wait_until_micros(&mut self, value: u64) -> Result<(), OverflowError> {
        let end_time = self
            .0
            .ok_or(OverflowError)?
            .checked_add(Duration::from_micros(value))
            .ok_or(OverflowError)?;
        crate::Timer::at(end_time).await;
        Ok(())
    }

    async fn wait_until_millis(&mut self, value: u64) -> Result<(), OverflowError> {
        let end_time = self
            .0
            .ok_or(OverflowError)?
            .checked_add(Duration::from_millis(value))
            .ok_or(OverflowError)?;
        crate::Timer::at(end_time).await;
        Ok(())
    }

    async fn wait_until_secs(&mut self, value: u64) -> Result<(), OverflowError> {
        let end_time = self
            .0
            .ok_or(OverflowError)?
            .checked_add(Duration::from_secs(value))
            .ok_or(OverflowError)?;
        crate::Timer::at(end_time).await;
        Ok(())
    }
}
