use super::{Mode, Speed, Status};

#[derive(Debug, Clone, Copy)]
pub struct InitParams {
    pub gconf: u32,
    pub slaveconf: u32,
    pub ihold_irun: u32,
    pub tpwmthrs: u32,
    pub tcoolthrs: u32,
    pub thigh: u32,
    pub a1: u32,
    pub v1: u32,
    pub amax: u32,
    pub dmax: u32,
    pub vmax: u32,
    pub d1: u32,
    pub vstop: u32,
    pub chopconf: u32,
    pub coolconf: u32,
}

impl Default for InitParams {
    fn default() -> Self {
        Self {
            gconf: 0x00000000,
            slaveconf: 0x00000000,
            ihold_irun: 0x00071001,
            tpwmthrs: 0x00000000,
            tcoolthrs: 0x00000090,
            thigh: 0x00000010,
            a1: 0x00000000,
            v1: 0x00000000,
            amax: 0x0008F530,
            dmax: 0x0008F530,
            vmax: 0x00000000,
            d1: 0x00000001,
            vstop: 0x00000009,
            chopconf: 0x00010395,
            coolconf: 0x1040000,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigParams {
    pub reset_position: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct StartParams {
    pub mode: Mode,
    pub direction: Status,
    pub speed: Speed,
    pub position: Option<u64>,
    pub reset: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct MoveToParams {
    pub speed: Speed,
    pub position: i32,
    pub reset: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ChangeSpeedParams {
    pub speed: Speed,
}

#[derive(Debug, Clone, Copy)]
pub struct HomeParams {
    pub direction: Status,
    pub timeout_ms: u32,
    pub backoff_steps: u32,
    pub speed: Speed,
    pub tcoolthrs: u32,
    pub ihold_irun: u32,
    pub coolconf: u32,
}
