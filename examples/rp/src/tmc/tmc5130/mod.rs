use core::time::Duration;
use embedded_hal_1::digital::OutputPin;
use embedded_hal_async::spi::SpiBus;

mod constants;
pub use constants::*;

pub use super::{params, vactual_to_signed_pps, Mode, Speed, Status};

pub const NUMBER_MOTORS: usize = 1;
pub const DEFAULT_INTERVAL_MS: u64 = 100;
pub const DEFAULT_FREQUENCY_SCALING: f32 = 1.21_f32;

#[derive(Debug)]
pub enum Error {
    SpiTransfer,
    OutputPin,
    MessageError,
    InitParamsNotSet,
    HomingParamsNotSet,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum HomingPhase {
    #[default]
    Init,
    BackOffStarted,
    StallSeekStarted,
    Stalled,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum FrequencyScalingPhase {
    #[default]
    Init,
    Waiting,
    FirstPoint,
    SecondPoint,
}

#[derive(Debug, Default)]
#[repr(u8)]
pub enum MotorStatus {
    #[default]
    Operational,
    Homing(HomingPhase),
    FrequencyScaling(FrequencyScalingPhase),
}

#[derive(Debug, Default)]
pub struct FrequencyScalingDataPoint {
    ticks: MonotonicTicks,
    position: i32,
}

#[derive(Debug, Default)]
pub struct FrequencyScalingData {
    start: FrequencyScalingDataPoint,
    end: FrequencyScalingDataPoint,
}

impl FrequencyScalingData {
    fn calc_scaling(&self, vmax: i32) -> f32 {
        (vmax as f32 * (self.end.ticks.saturating_sub(self.start.ticks) as f32) / (TICKS_PER_SECOND as f32))
            / (self.end.position.abs_diff(self.start.position) as f32)
    }
}

#[derive(Debug)]
pub struct TMC5130 {
    pub status: MotorStatus,
    pub motor: Motor,
    pub init_params: Option<params::InitParams>,
    pub homing_params: Option<params::HomeParams>,
    homing_attempts: u32,
    pub frequency_scaling: f32,
    pub frequency_scaling_data: FrequencyScalingData,
}

impl TMC5130 {
    pub fn new() -> Self {
        Self {
            status: MotorStatus::Operational,
            motor: Motor::new(MotorIndex::One),
            init_params: None,
            homing_params: None,
            homing_attempts: 0,
            frequency_scaling: DEFAULT_FREQUENCY_SCALING,
            frequency_scaling_data: FrequencyScalingData::default(),
        }
    }

    pub fn transact<'a, O: OutputPin, F: FnOnce() -> R + 'a, R>(&self, cs: &'a mut O, act: F) -> R {
        while cs.set_low().is_err() {}
        let res = act();
        while cs.set_high().is_err() {}
        res
    }

    #[inline(always)]
    const fn to_read_data(&self, address: u8) -> [u8; 5] {
        self.to_data(address, 0x00)
    }

    #[inline(always)]
    const fn to_write_data(&self, address: u8, data: u32) -> [u8; 5] {
        const TMC5130_WRITE_MASK: u8 = 0x80;
        self.to_data(address | TMC5130_WRITE_MASK, data)
    }

    #[inline(always)]
    const fn to_data(&self, address: u8, data: u32) -> [u8; 5] {
        [
            address,
            (data >> 24) as u8,
            (data >> 16) as u8,
            (data >> 8) as u8,
            data as u8,
        ]
    }

    #[inline(always)]
    const fn field_get(&self, data: u32, mask: u32, shift: u8) -> u32 {
        (data & mask) >> shift
    }

    #[inline(always)]
    const fn field_set(&self, data: u32, mask: u32, shift: u8, value: u32) -> u32 {
        (data & !mask) | ((value << shift) & mask)
    }

    #[inline(always)]
    fn field_read<SPI: Transfer<u8>, CS: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        d: &mut D,
        address: u8,
        mask: u32,
        shift: u8,
    ) -> Result<u32, Error> {
        Ok(self.field_get(self.read_register(spi, cs, d, address)?, mask, shift))
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn field_write<SPI: Transfer<u8>, CS: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        d: &mut D,
        address: u8,
        mask: u32,
        shift: u8,
        value: u32,
    ) -> Result<(), Error> {
        self.write_register(spi, cs, d, address, (value << shift) & mask)
    }

    #[inline(always)]
    fn field_update<SPI: Transfer<u8>, CS: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        d: &mut D,
        address: u8,
        mask: u32,
        shift: u8,
        value: u32,
    ) -> Result<(), Error> {
        let v = self.field_set(self.read_register(spi, cs, d, address)?, mask, shift, value);
        self.write_register(spi, cs, d, address, v)
    }

    #[inline(always)]
    fn field_update_and_verify<SPI: Transfer<u8>, CS: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        d: &mut D,
        address: u8,
        mask: u32,
        shift: u8,
        value: u32,
    ) -> Result<(), Error> {
        self.field_update(spi, cs, d, address, mask, shift, value)?;

        while self.field_read(spi, cs, d, address, mask, shift)? != value {
            self.field_update(spi, cs, d, address, mask, shift, value)?;
        }

        Ok(())
    }

    #[inline(always)]
    pub fn write_register<SPI: Transfer<u8>, CS: OutputPin, D: DelayUs<MonotonicTicks>>(
        &self,
        spi: &mut SPI,
        cs: &mut CS,
        d: &mut D,
        address: u8,
        data: u32,
    ) -> Result<(), Error> {
        self.transact(cs, d, || {
            spi.transfer(&mut self.to_write_data(address, data)).map(|_| {})
        })
        .map_err(|_| Error::SpiTransfer)?;
        Ok(())
    }

    #[inline(always)]
    pub fn read_register<SPI: Transfer<u8>, CS: OutputPin, D: DelayUs<MonotonicTicks>>(
        &self,
        spi: &mut SPI,
        cs: &mut CS,
        d: &mut D,
        address: u8,
    ) -> Result<u32, Error> {
        let mut data = 0x00;
        self.transact(cs, d, || spi.transfer(&mut self.to_read_data(address)).map(|_| {}))
            .map_err(|_| Error::SpiTransfer)?;
        d.delay_us(10);
        self.transact(cs, d, || {
            spi.transfer(&mut self.to_read_data(address)).map(|r| {
                data = ((r[1] as u32) << 24) | ((r[2] as u32) << 16) | ((r[3] as u32) << 8) | (r[4] as u32);
            })
        })
        .map_err(|_| Error::SpiTransfer)?;
        Ok(data)
    }

    pub fn init<SPI: Transfer<u8>, CS: OutputPin, EN: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        en: &mut EN,
        d: &mut D,
    ) -> Result<Option<Duration>, Error> {
        en.set_high().map_err(|_| Error::OutputPin)?;

        let p = match &self.init_params {
            Some(p) => *p,
            None => params::InitParams::default(),
        };

        self.read_register(spi, cs, d, REG::TMC5130_RAMPSTAT)?;

        self.write_register(spi, cs, d, REG::TMC5130_GCONF, p.gconf)?;
        self.write_register(spi, cs, d, REG::TMC5130_SLAVECONF, p.slaveconf)?;
        self.write_register(spi, cs, d, REG::TMC5130_IHOLD_IRUN, p.ihold_irun)?;
        self.write_register(spi, cs, d, REG::TMC5130_TPWMTHRS, p.tpwmthrs)?;
        self.write_register(spi, cs, d, REG::TMC5130_TCOOLTHRS, p.tcoolthrs)?;
        self.write_register(spi, cs, d, REG::TMC5130_THIGH, p.thigh)?;
        self.write_register(spi, cs, d, REG::TMC5130_A1, p.a1)?;
        self.write_register(spi, cs, d, REG::TMC5130_V1, p.v1)?;
        self.write_register(spi, cs, d, REG::TMC5130_AMAX, p.amax)?;
        self.write_register(spi, cs, d, REG::TMC5130_DMAX, p.dmax)?;
        self.write_register(spi, cs, d, REG::TMC5130_VMAX, p.vmax)?;
        self.write_register(spi, cs, d, REG::TMC5130_D1, p.d1)?;
        self.write_register(spi, cs, d, REG::TMC5130_VSTOP, p.vstop)?;
        self.write_register(spi, cs, d, REG::TMC5130_CHOPCONF, p.chopconf)?;
        self.write_register(spi, cs, d, REG::TMC5130_COOLCONF, p.coolconf)?;

        en.set_low().map_err(|_| Error::OutputPin)?;

        Ok(None)
    }

    #[inline(always)]
    pub fn get_xactual<SPI: Transfer<u8>, CS: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        d: &mut D,
    ) -> Result<i32, Error> {
        Ok(self.field_read(
            spi,
            cs,
            d,
            REG::TMC5130_XACTUAL,
            MASK::TMC5130_XACTUAL_MASK,
            SHIFT::TMC5130_XACTUAL_SHIFT,
        )? as i32)
    }

    pub fn set_xactual<SPI: Transfer<u8>, CS: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        d: &mut D,
        value: u32,
    ) -> Result<(), Error> {
        let v = vactual_to_signed_pps(value);
        self.field_update(
            spi,
            cs,
            d,
            REG::TMC5130_XACTUAL,
            MASK::TMC5130_XACTUAL_MASK,
            SHIFT::TMC5130_XACTUAL_SHIFT,
            v as u32,
        )
    }

    #[inline(always)]
    pub fn get_vactual<SPI: Transfer<u8>, CS: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        d: &mut D,
    ) -> Result<u32, Error> {
        self.field_read(
            spi,
            cs,
            d,
            REG::TMC5130_VACTUAL,
            MASK::TMC5130_VACTUAL_MASK,
            SHIFT::TMC5130_VACTUAL_SHIFT,
        )
    }

    #[inline(always)]
    pub fn get_signed_vactual<SPI: Transfer<u8>, CS: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        d: &mut D,
    ) -> Result<i32, Error> {
        let v = self.get_vactual(spi, cs, d)?;
        Ok(vactual_to_signed_pps(v))
    }

    pub fn speed_to_pps(&self, speed: Speed) -> i32 {
        match speed {
            Speed::Rpm(v) => self.rpm_to_pps(v),
            Speed::Pps(v) => v,
        }
    }

    pub fn start<SPI: Transfer<u8>, CS: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        d: &mut D,
        params: &params::StartParams,
    ) -> Result<Option<Duration>, Error> {
        // stop
        self.stop(spi, cs, d)?;

        // mode
        match params.mode {
            Mode::Continuous => {
                // direction
                let direction: u32 = match params.direction {
                    Status::Clockwise => 0x01,
                    Status::CounterClockwise => 0x02,
                    _ => 0x01,
                };

                self.field_update_and_verify(
                    spi,
                    cs,
                    d,
                    REG::TMC5130_RAMPMODE,
                    MASK::TMC5130_RAMPMODE_MASK,
                    SHIFT::TMC5130_RAMPMODE_SHIFT,
                    direction,
                )?;

                self.field_update(
                    spi,
                    cs,
                    d,
                    REG::TMC5130_VMAX,
                    MASK::TMC5130_VMAX_MASK,
                    SHIFT::TMC5130_VMAX_SHIFT,
                    self.speed_to_pps(params.speed) as u32,
                )
            }
            Mode::Finite => {
                self.field_update_and_verify(
                    spi,
                    cs,
                    d,
                    REG::TMC5130_RAMPMODE,
                    MASK::TMC5130_RAMPMODE_MASK,
                    SHIFT::TMC5130_RAMPMODE_SHIFT,
                    0x00,
                )?;

                match (params.position, params.reset) {
                    (None, _) => Err(Error::MessageError),
                    (Some(position), reset) => {
                        if reset {
                            self.field_update(
                                spi,
                                cs,
                                d,
                                REG::TMC5130_XACTUAL,
                                MASK::TMC5130_XACTUAL_MASK,
                                SHIFT::TMC5130_XACTUAL_SHIFT,
                                0,
                            )?;

                            let mut count: usize = 0;

                            while self.get_xactual(spi, cs, d)? != 0 && count < 5 {
                                self.stop(spi, cs, d)?;
                                self.field_update(
                                    spi,
                                    cs,
                                    d,
                                    REG::TMC5130_XACTUAL,
                                    MASK::TMC5130_XACTUAL_MASK,
                                    SHIFT::TMC5130_XACTUAL_SHIFT,
                                    0,
                                )?;
                                count += 1;
                            }
                        }

                        let position: i32 = match params.direction {
                            Status::Clockwise => (position as i32).saturating_add(1),
                            Status::CounterClockwise => -((position as i32).saturating_add(1)),
                            _ => position as i32,
                        };

                        self.motor.target_position = position;

                        self.field_update_and_verify(
                            spi,
                            cs,
                            d,
                            REG::TMC5130_XTARGET,
                            MASK::TMC5130_XTARGET_MASK,
                            SHIFT::TMC5130_XTARGET_SHIFT,
                            position as u32,
                        )
                    }
                }?;

                self.field_update(
                    spi,
                    cs,
                    d,
                    REG::TMC5130_VMAX,
                    MASK::TMC5130_VMAX_MASK,
                    SHIFT::TMC5130_VMAX_SHIFT,
                    self.speed_to_pps(params.speed) as u32,
                )
            }
        }?;

        self.motor.direction = params.direction;
        self.motor.pps = self.speed_to_pps(params.speed);
        self.motor.mode = params.mode;
        self.motor.dirty = true;

        Ok(None)
    }

    pub fn stop<SPI: Transfer<u8>, CS: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        d: &mut D,
    ) -> Result<Option<Duration>, Error> {
        self.field_update(
            spi,
            cs,
            d,
            REG::TMC5130_VMAX,
            MASK::TMC5130_VMAX_MASK,
            SHIFT::TMC5130_VMAX_SHIFT,
            0x00,
        )?;
        Ok(None)
    }

    pub fn change_speed<SPI: Transfer<u8>, CS: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        d: &mut D,
        params: &params::ChangeSpeedParams,
    ) -> Result<Option<Duration>, Error> {
        self.field_update(
            spi,
            cs,
            d,
            REG::TMC5130_VMAX,
            MASK::TMC5130_VMAX_MASK,
            SHIFT::TMC5130_VMAX_SHIFT,
            self.speed_to_pps(params.speed) as u32,
        )?;

        Ok(None)
    }

    pub fn move_to<SPI: Transfer<u8>, CS: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        d: &mut D,
        params: &params::MoveToParams,
    ) -> Result<Option<Duration>, Error> {
        // stop
        self.stop(spi, cs, d)?;

        if params.reset {
            self.field_update(
                spi,
                cs,
                d,
                REG::TMC5130_XACTUAL,
                MASK::TMC5130_XACTUAL_MASK,
                SHIFT::TMC5130_XACTUAL_SHIFT,
                0,
            )?;
        }

        let direction = if self.motor.position > params.position {
            Status::Clockwise
        } else {
            Status::CounterClockwise
        };

        self.motor.target_position = params.position;

        self.field_update_and_verify(
            spi,
            cs,
            d,
            REG::TMC5130_XTARGET,
            MASK::TMC5130_XTARGET_MASK,
            SHIFT::TMC5130_XTARGET_SHIFT,
            params.position as u32,
        )?;

        self.field_update(
            spi,
            cs,
            d,
            REG::TMC5130_VMAX,
            MASK::TMC5130_VMAX_MASK,
            SHIFT::TMC5130_VMAX_SHIFT,
            self.speed_to_pps(params.speed) as u32,
        )?;

        self.motor.direction = direction;
        self.motor.pps = self.speed_to_pps(params.speed);
        self.motor.mode = Mode::Finite;

        Ok(None)
    }

    pub fn config<SPI: Transfer<u8>, CS: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        d: &mut D,
        params: &params::ConfigParams,
    ) -> Result<Option<Duration>, Error> {
        if params.reset_position {
            self.field_update(
                spi,
                cs,
                d,
                REG::TMC5130_XACTUAL,
                MASK::TMC5130_XACTUAL_MASK,
                SHIFT::TMC5130_XACTUAL_SHIFT,
                0,
            )?;
            self.field_update_and_verify(
                spi,
                cs,
                d,
                REG::TMC5130_XTARGET,
                MASK::TMC5130_XTARGET_MASK,
                SHIFT::TMC5130_XTARGET_SHIFT,
                0,
            )?;
        }

        Ok(None)
    }

    #[inline(always)]
    pub fn rpm_to_pps(&self, rpm: f32) -> i32 {
        let us_step_per_s = 256. * 200. * rpm / 60.;
        (us_step_per_s * self.frequency_scaling) as i32
    }

    #[inline(always)]
    pub fn pps_to_rpm(&self, pps: i32) -> f32 {
        let us_step_per_s = (pps as f32) / self.frequency_scaling;
        us_step_per_s / 256. / 200. * 60.
    }

    pub fn home<SPI: Transfer<u8>, CS: OutputPin, EN: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        en: &mut EN,
        d: &mut D,
    ) -> Result<Option<Duration>, Error> {
        let p = match &self.homing_params {
            Some(p) => Ok(*p),
            None => Err(Error::HomingParamsNotSet),
        }?;

        let phase = match self.status {
            MotorStatus::Homing(s) => s,
            _ => HomingPhase::Init,
        };

        match phase {
            HomingPhase::Init => {
                // stop
                self.stop(spi, cs, d)?;

                // reset from a stall
                self.read_register(spi, cs, d, REG::TMC5130_RAMPSTAT)?;

                // backoff
                let backoff_params = params::StartParams {
                    mode: Mode::Finite,
                    direction: p.direction.reverse(),
                    speed: p.speed,
                    position: Some(p.backoff_steps as u64),
                    reset: true,
                };

                self.start(spi, cs, d, &backoff_params)?;

                // update status
                log::info!("BackOffStarted...");
                self.status = MotorStatus::Homing(HomingPhase::BackOffStarted);

                // wait 10 ms before checking again
                // to allow for acceleration
                Ok(Some(Duration::from_millis(10)))
            }
            HomingPhase::BackOffStarted => {
                // wait for stopped
                if self.get_vactual(spi, cs, d)? != 0 {
                    Ok(Some(Duration::from_millis(10)))
                } else {
                    // start motion toward hard stop
                    // by starting stall seek

                    // 1. disable softstop
                    self.field_update(
                        spi,
                        cs,
                        d,
                        REG::TMC5130_SWMODE,
                        MASK::TMC5130_EN_SOFTSTOP_MASK,
                        SHIFT::TMC5130_EN_SOFTSTOP_SHIFT,
                        0,
                    )?;

                    // 2. enable sg_stop
                    self.field_update(
                        spi,
                        cs,
                        d,
                        REG::TMC5130_SWMODE,
                        MASK::TMC5130_SG_STOP_MASK,
                        SHIFT::TMC5130_SG_STOP_SHIFT,
                        1,
                    )?;

                    // update ihold_irun
                    self.write_register(spi, cs, d, REG::TMC5130_IHOLD_IRUN, p.ihold_irun)?;

                    // set SGT: this signed value controls StallGuard2 level for stall
                    // output and sets the optimum measurement range for
                    // readout. A lower value gives a higher sensitivity. Zero is
                    // the starting value working with most motors
                    self.write_register(spi, cs, d, REG::TMC5130_COOLCONF, p.coolconf)?;

                    // wait 10 ms to allow changes to propagate
                    d.delay_us(10_000 as MonotonicTicks);

                    let stall_seek_params = params::StartParams {
                        mode: Mode::Continuous,
                        direction: p.direction,
                        speed: p.speed,
                        position: None,
                        reset: false,
                    };

                    self.start(spi, cs, d, &stall_seek_params)?;

                    // update status
                    log::info!("StallSeekStarted...");
                    self.status = MotorStatus::Homing(HomingPhase::StallSeekStarted);

                    // reset counter
                    self.homing_attempts = 0;

                    // wait 200 ms before checking again
                    Ok(Some(Duration::from_millis(200)))
                }
            }
            HomingPhase::StallSeekStarted => {
                let is_stalled = self.field_read(
                    spi,
                    cs,
                    d,
                    REG::TMC5130_RAMPSTAT,
                    MASK::TMC5130_STATUS_SG_MASK,
                    SHIFT::TMC5130_STATUS_SG_SHIFT,
                )? != 0;

                let sg_val = self.read_register(spi, cs, d, REG::TMC5130_DRVSTATUS)?;
                let tstep_val = self.read_register(spi, cs, d, REG::TMC5130_TSTEP)?;
                log::debug!("sg_val: {:08X}, tstep_val: {:08X}", &sg_val, &tstep_val);

                if is_stalled {
                    // update status
                    self.status = MotorStatus::Homing(HomingPhase::Stalled);

                    // wait 100 ms before checking again
                    log::info!("stall detected..");
                    Ok(Some(Duration::from_millis(100)))
                } else {
                    log::info!("waiting for stall...");

                    self.homing_attempts += 1;

                    if self.homing_attempts >= 3000 {
                        log::warn!("timed out waiting for stall...");

                        self.status = MotorStatus::Homing(HomingPhase::Stalled);

                        // wait 10 ms before checking again
                        Ok(Some(Duration::from_millis(100)))
                    } else {
                        // wait 1 ms before checking again
                        Ok(Some(Duration::from_millis(1)))
                    }
                }
            }
            HomingPhase::Stalled => {
                // reset from a stall
                self.read_register(spi, cs, d, REG::TMC5130_RAMPSTAT)?;

                // reapply initialization params
                self.init(spi, cs, en, d)?;

                // backoff
                let backoff_params = params::StartParams {
                    mode: Mode::Finite,
                    direction: p.direction.reverse(),
                    speed: p.speed,
                    position: Some(p.backoff_steps as u64),
                    reset: true,
                };

                self.start(spi, cs, d, &backoff_params)?;

                // update status
                self.status = MotorStatus::Operational;
                // wait 10 ms before checking again
                Ok(Some(Duration::from_millis(10)))
            }
        }
    }

    pub fn find_frequency_scaling<SPI: Transfer<u8>, CS: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        d: &mut D,
        ticks: Option<MonotonicTicks>,
    ) -> Result<(Option<Duration>, Option<f32>), Error> {
        let phase = match self.status {
            MotorStatus::FrequencyScaling(s) => s,
            _ => FrequencyScalingPhase::Init,
        };

        const VMAX: i32 = 10_000;

        match phase {
            FrequencyScalingPhase::Init => {
                // stop
                self.stop(spi, cs, d)?;

                // start at 10_000
                let params = params::StartParams {
                    mode: Mode::Continuous,
                    direction: Status::Clockwise,
                    speed: Speed::Pps(VMAX),
                    position: None,
                    reset: true,
                };

                self.start(spi, cs, d, &params)?;

                self.status = MotorStatus::FrequencyScaling(FrequencyScalingPhase::Waiting);

                Ok((Some(Duration::from_millis(10)), None))
            }

            FrequencyScalingPhase::Waiting => {
                self.status = MotorStatus::FrequencyScaling(FrequencyScalingPhase::FirstPoint);
                Ok((Some(Duration::from_millis(50)), None))
            }

            FrequencyScalingPhase::FirstPoint => {
                if let Some(ticks) = ticks {
                    let position = self.get_xactual(spi, cs, d)?;
                    self.frequency_scaling_data.start = FrequencyScalingDataPoint { ticks, position };
                    self.status = MotorStatus::FrequencyScaling(FrequencyScalingPhase::SecondPoint);

                    Ok((Some(Duration::from_millis(1000)), None))
                } else {
                    self.stop(spi, cs, d)?;
                    self.status = MotorStatus::Operational;
                    Ok((Some(Duration::from_millis(10)), None))
                }
            }

            FrequencyScalingPhase::SecondPoint => {
                if let Some(ticks) = ticks {
                    let position = self.get_xactual(spi, cs, d)?;
                    self.frequency_scaling_data.end = FrequencyScalingDataPoint { ticks, position };

                    self.stop(spi, cs, d)?;

                    self.frequency_scaling = self.frequency_scaling_data.calc_scaling(VMAX);

                    self.status = MotorStatus::Operational;

                    Ok((Some(Duration::from_millis(50)), Some(self.frequency_scaling)))
                } else {
                    self.stop(spi, cs, d)?;
                    self.status = MotorStatus::Operational;
                    Ok((Some(Duration::from_millis(10)), None))
                }
            }
        }
    }

    pub fn events<SPI: Transfer<u8>, CS: OutputPin, EN: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        en: &mut EN,
        d: &mut D,
        ticks: MonotonicTicks,
    ) -> Result<(Duration, Response<{ NUMBER_MOTORS }>), Error> {
        match self.status {
            MotorStatus::Operational => {
                let mut response = StatusResponse::new(ticks);

                self.motor.position = self.get_xactual(spi, cs, d)?;
                self.motor.pps = ((self.get_signed_vactual(spi, cs, d)? as f32) / self.frequency_scaling) as i32;

                if !self.motor.dirty {
                    if self.motor.pps == 0 && self.motor.position == self.motor.last_position.0 {
                        self.motor.direction = Status::Stopped;
                    }
                }

                self.motor.last_position = (self.motor.position, ticks);

                self.motor.dirty = false;
                response.responses[0] = Some((&self.motor).into());

                Ok((Duration::from_millis(DEFAULT_INTERVAL_MS), Response::Status(response)))
            }

            MotorStatus::FrequencyScaling(_p) => {
                let (duration, response) = match self.find_frequency_scaling(spi, cs, d, Some(ticks))? {
                    (Some(d), Some(f)) => (d, Response::FrequencyScaling(FrequencyScalingResponse::new([Some(f)]))),
                    (Some(d), None) => (d, Response::Status(StatusResponse::new(ticks))),
                    _ => (
                        Duration::from_millis(DEFAULT_INTERVAL_MS),
                        Response::Status(StatusResponse::new(ticks)),
                    ),
                };
                Ok((duration, response))
            }

            MotorStatus::Homing(_p) => {
                let duration = match self.home(spi, cs, en, d)? {
                    Some(d) => d,
                    None => Duration::from_millis(DEFAULT_INTERVAL_MS),
                };
                Ok((duration, Response::Status(StatusResponse::new(ticks))))
            }
        }
    }

    pub fn handle_message<SPI: Transfer<u8>, CS: OutputPin, EN: OutputPin, D: DelayUs<MonotonicTicks>>(
        &mut self,
        spi: &mut SPI,
        cs: &mut CS,
        en: &mut EN,
        d: &mut D,
        message: Message<NUMBER_MOTORS>,
    ) -> Result<Option<Duration>, Error> {
        const MOTOR_INDEX: usize = 0;

        match message {
            Message::None => Ok(None),
            Message::MoveTo(p) => {
                if let Some(Some(p)) = p.get(MOTOR_INDEX) {
                    self.move_to(spi, cs, d, p)
                } else {
                    Ok(None)
                }
            }
            Message::Config(p) => {
                if let Some(Some(p)) = p.get(MOTOR_INDEX) {
                    self.config(spi, cs, d, p)
                } else {
                    Ok(None)
                }
            }
            Message::Stop(p) => {
                if let Some(Some(true)) = p.get(MOTOR_INDEX) {
                    self.stop(spi, cs, d)
                } else {
                    Ok(None)
                }
            }
            Message::ChangeSpeed(p) => {
                if let Some(Some(p)) = p.get(MOTOR_INDEX) {
                    self.change_speed(spi, cs, d, p)
                } else {
                    Ok(None)
                }
            }
            Message::Start(p) => {
                if let Some(Some(p)) = p.get(MOTOR_INDEX) {
                    self.start(spi, cs, d, p)
                } else {
                    Ok(None)
                }
            }
            Message::Init(mut p) => {
                if let Some(Some(p)) = p.get_mut(MOTOR_INDEX) {
                    self.init_params = Some(*p);
                }
                self.init(spi, cs, en, d)
            }
            Message::Home(mut p) => {
                if let Some(Some(p)) = p.get_mut(MOTOR_INDEX) {
                    self.homing_params = Some(*p);
                }
                self.home(spi, cs, en, d)
            }
            Message::FrequencyScaling(p) => {
                if let Some(Some(true)) = p.get(MOTOR_INDEX) {
                    Ok(self.find_frequency_scaling(spi, cs, d, None)?.0)
                } else {
                    Ok(None)
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vactual_conversion() {
        let minus_five_rpm: u32 = 0x00ffebd6;
        let five_rpm: u32 = 0x0000142a;
        let zero_rpm: u32 = 0x00000000;

        assert_eq!(
            vactual_to_signed_pps(minus_five_rpm),
            -1 * vactual_to_signed_pps(five_rpm)
        );
        assert_eq!(vactual_to_signed_pps(zero_rpm), 0);
    }
}
