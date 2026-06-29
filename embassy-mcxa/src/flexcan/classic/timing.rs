//! This module calculates ideal bit timing values for classic FlexCAN.
//! 
//! A lot of the code in this module was translated into Rust from the `fsl_flexcan.c` source file inside `hal_nxp` Zephyr module.
//! As of June 2026, that source file is hosted here: https://github.com/zephyrproject-rtos/hal_nxp/blob/master/mcux/mcux-sdk-ng/drivers/flexcan/fsl_flexcan.c
//! 
//! Here is the original copyright notice from that source file:
//! 
//! ```text
//! Copyright 2020-2021, 2026 NXP
//! 
//! SPDX-License-Identifier: BSD-3-Clause
//! ```
//! 
//! Note: This module targets the Enhanced CAN Bit Timing registers (rather than the older CBT[BTF] and CTRL1 timing registers).
//! This is because all modern MCXA chips with FlexCAN support the Enhanced CAN Bit Timing registers, and they're better than the older timing registers.

// These are a bunch of constants taken from the hal_nxp C code.
// They're not even that necessary in this project since we have a PAC (i.e., we're not manually doing a bunch of shifting and masking in registers),
// but having the `MAX` values is kind of nice as iteration bounds, and mirroring the layout keeps this file easy to compare against the upstream driver.

#![allow(dead_code)] // need this since some of the constants are considered unused by rust-analyzer even though they're used by other constants

use crate::flexcan::classic::Info;
use embassy_time::Duration;

/* According to CiA doc 301 v4.2.0 and previous version. */
const IDEAL_SP_LOW:                 u32 = 750;
const IDEAL_SP_MID:                 u32 = 800;
const IDEAL_SP_HIGH:                u32 = 875;
const IDEAL_SP_FACTOR:              u32 = 1000;

/* TSEG1 corresponds to the sum of PROPSEG and PSEG1, TSEG2 corresponds to the PSEG2 value. */
const MIN_TIME_SEGMENT2:            u32 = 2;

/* Define maximum classic CAN bit rate supported by FLEXCAN. */
const MAX_CAN_BITRATE:              u32 = 1000000;

const CAN_ENCBT_NTSEG1_MASK:        u32 = 0xFF;
const CAN_ENCBT_NTSEG1_SHIFT:       u32 = 0;
const CAN_ENCBT_NTSEG2_MASK:        u32 = 0x7F000;
const CAN_ENCBT_NTSEG2_SHIFT:       u32 = 12;
const CAN_ENCBT_NRJW_MASK:          u32 = 0x1FC00000;
const CAN_ENCBT_NRJW_SHIFT:         u32 = 22;
const CAN_EPRS_ENPRESDIV_MASK:      u32 = 0x3FF;
const CAN_EPRS_ENPRESDIV_SHIFT:     u32 = 0;
const MAX_NTSEG1:                   u32 = CAN_ENCBT_NTSEG1_MASK >> CAN_ENCBT_NTSEG1_SHIFT;
const MAX_NTSEG2:                   u32 = CAN_ENCBT_NTSEG2_MASK >> CAN_ENCBT_NTSEG2_SHIFT;
const MAX_NRJW:                     u32 = CAN_ENCBT_NRJW_MASK >> CAN_ENCBT_NRJW_SHIFT;
const MAX_ENPRESDIV:                u32 = CAN_EPRS_ENPRESDIV_MASK >> CAN_EPRS_ENPRESDIV_SHIFT;
const ENCBT_MAX_TIME_QUANTA:        u32 = 1 + MAX_NTSEG1 + 1 + MAX_NTSEG2 + 1;
const ENCBT_MIN_TIME_QUANTA:        u32 = 8;

/// Bit-timing values as written to the ENCBT/EPRS registers.
///
/// These fields here are stored 0-based (i.e., the register-encoding form, where the actual number of time
/// quanta is `field + 1`). This is what the C driver does.
#[derive(Default, Debug)]
struct FlexcanTimingConfig {
    /// Clock Pre-scaler Division Factor.
    pub(crate) pre_divider: u16,

    /// Re-sync Jump Width.
    pub(crate) r_jump_width: u8,

    /// Phase Segment 1.
    pub(crate) phase_seg_1: u8,

    /// Phase Segment 2.
    pub(crate) phase_seg_2: u8,

    /// Propagation Segment.
    pub(crate) prop_seg: u8,
}

/// Errors that may occur when configuring timing/bitrate.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimingError {
    /// Source clock is not an integer multiple of the requested bitrate. The source clock must be an integer multiple of the requested bitrate.
    BitrateIncompatibleWithClock,

    /// Requested bitrate is too high. Your requested bitrate must be <= 1 Mbps.
    BitrateTooHigh,

    /// You have attempted to configure a bitrate of zero, which is not allowed.
    ZeroBitrate,

    /// This error indicates that the hardware didn't response within a reasonable timeframe to a request the HAL made.
    Timeout,

    /// No combination of prescaler and segment values can produce the
    /// requested bitrate from the given source clock within hardware limits.
    NoValidTimingFound,
}

/// Rust version of `FLEXCAN_SetBaudrate()` from the NXP Zephyr HAL.
/// 
/// This HAL publicly uses the term `bitrate` in FlexCanConfig, since that seems to be a more accurate term here. But,
/// internally, this function (and everything else in the `timing` module) uses "baud rate" since that's what the functions
/// from NXP's C HAL use.
/// 
/// Note: calling this function will put the FlexCAN into Freeze Mode, since it eventually calls `set_timing_config()`.
pub(crate) fn set_baudrate(info: &Info, src_clk_hz: u32, baud_rate_bps: u32) -> Result<(), TimingError> {

    /* Calculate timing automatically with given Baud Rate. */
    let config = calculate_improved_timing_values(baud_rate_bps, src_clk_hz)?;

    Ok(set_timing_config(info, config)?)
}

/// Rust version of `FLEXCAN_GetSegments()` from the NXP Zephyr HAL.
fn get_segments(baud_rate_bps: u32, tq_num: u32, timing_config: &mut FlexcanTimingConfig) {
    let seg_1_max:   u32 = MAX_NTSEG2 + 1;
    let pro_seg_max: u32 = MAX_NTSEG1 - MAX_NTSEG2;
    let seg_1_temp:  u32;

    /* Try to find the ideal sample point, according to CiA 301 doc. */
    let ideal_sp: u32 = if baud_rate_bps == 1000000
    {
        IDEAL_SP_LOW
    }
    else if baud_rate_bps >= 800000
    {
        IDEAL_SP_MID
    }
    else
    {
        IDEAL_SP_HIGH
    };

    /* Calculates phase_seg_2. */
    timing_config.phase_seg_2 = (tq_num - (tq_num * ideal_sp) / IDEAL_SP_FACTOR) as u8;
    if timing_config.phase_seg_2 < (MIN_TIME_SEGMENT2 as u8)
    {
        timing_config.phase_seg_2 = MIN_TIME_SEGMENT2 as u8;
    }

    /* Calculates phase_seg_1 and prop_seg and try to make phase_seg_1 equal to phase_seg_2. */
    if (tq_num - (timing_config.phase_seg_2 as u32) - 1) > (seg_1_max + pro_seg_max)
    {
        seg_1_temp                = seg_1_max + pro_seg_max;
        timing_config.phase_seg_2 = (tq_num - 1 - seg_1_temp) as u8;
    }
    else
    {
        seg_1_temp = tq_num - (timing_config.phase_seg_2 as u32) - 1;
    }
    if seg_1_temp > ((timing_config.phase_seg_2 as u32) + pro_seg_max)
    {
        timing_config.prop_seg    = pro_seg_max as u8;
        timing_config.phase_seg_1 = (seg_1_temp - pro_seg_max) as u8;
    }
    else
    {
        timing_config.prop_seg    = (seg_1_temp - (timing_config.phase_seg_2 as u32)) as u8;
        timing_config.phase_seg_1 = timing_config.phase_seg_2;
    }

    /* r_jump_width (sjw) is the minimum value of phase_seg_1 and phase_seg_2. */
    timing_config.r_jump_width = if timing_config.phase_seg_1 > timing_config.phase_seg_2 { timing_config.phase_seg_2 } else { timing_config.phase_seg_1 };

    // using wrapping_sub() to replicate the behavior of `-= 1U` from the C driver
    timing_config.phase_seg_1  = timing_config.phase_seg_1.wrapping_sub(1);
    timing_config.phase_seg_2  = timing_config.phase_seg_2.wrapping_sub(1);
    timing_config.prop_seg     = timing_config.prop_seg.wrapping_sub(1);
    timing_config.r_jump_width = timing_config.r_jump_width.wrapping_sub(1);
}

/// Rust version of `FLEXCAN_SetTimingConfig()` from the NXP Zephyr HAL.
fn set_timing_config(info: &Info, timing_config: FlexcanTimingConfig) -> Result<(), TimingError> {
    // Make sure we're in freeze mode
    const ENABLE_TIMEOUT: u64 = 10; // ms
    info.control.freeze(Some(Duration::from_millis(ENABLE_TIMEOUT))).map_err(|_| TimingError::Timeout)?;

    /* Enable extended Bit Timing register ENCBT. */
    info.control.regs().ctrl2().modify(|m| m.set_bte(true));

    /* Updating Timing Setting according to configuration structure. */
    info.control.regs().eprs().modify(|m| {
        m.set_enpresdiv(timing_config.pre_divider);
    });
    info.control.regs().encbt().modify(|m| {
        m.set_nrjw(timing_config.r_jump_width);
        m.set_ntseg1(((timing_config.phase_seg_1 as u32) + (timing_config.prop_seg as u32) + 1) as u8);
        m.set_ntseg2(timing_config.phase_seg_2);
    });

    Ok(())
}

/// Rust version of `FLEXCAN_CalculateImprovedTimingValues()` from the NXP Zephyr HAL.
fn calculate_improved_timing_values(baud_rate_bps: u32, src_clk_hz: u32) -> Result<FlexcanTimingConfig, TimingError> {

    /* Observe bit rate maximums and divisibility. */
    if baud_rate_bps == 0 {
        return Err(TimingError::ZeroBitrate);
    }
    if baud_rate_bps > MAX_CAN_BITRATE {
        return Err(TimingError::BitrateTooHigh);
    }
    if (src_clk_hz % baud_rate_bps) != 0 {
        return Err(TimingError::BitrateIncompatibleWithClock);
    }

    let mut best: Option<FlexcanTimingConfig> = None;
    let mut sp_temp: u32 = 1000;

    /*  Auto Improved Protocol timing for ENCBT. */
    for tq_num in (ENCBT_MIN_TIME_QUANTA..=ENCBT_MAX_TIME_QUANTA).rev() {
        let clk = baud_rate_bps * tq_num;

        if clk > src_clk_hz
        {
            continue; // tq_num too large: baud_rate_bps * tq_num would exceed src_clk_hz, meaning the prescaler would have to be < 1.
        }

        if (src_clk_hz % clk) != 0
        {
            continue; // src_clk_hz is not an exact integer multiple of (baud_rate_bps * tq_num), so this tq_num cannot produce the requested baud rate with zero error.
        }

        let mut config_temp = FlexcanTimingConfig::default();
        config_temp.pre_divider = ((src_clk_hz / clk) - 1) as u16;
        if (config_temp.pre_divider as u32) > MAX_ENPRESDIV
        {
            break; // pre_divider exceeds the hardware field maximum. Since pre_divider grows monotonically as tq_num shrinks, no smaller tq_num can succeed either. So, abort the search entirely.
        }

        /* Calculates the best timing configuration under current tqNum. */
        get_segments(baud_rate_bps, tq_num, &mut config_temp);
        /* Determine whether the calculated timing configuration can get the optimal sampling point. */
        let sp_metric = ((config_temp.phase_seg_2 as u32) + 1) * 1000 / tq_num;
        if sp_metric < sp_temp {
            sp_temp = sp_metric;
            best = Some(config_temp);
        }
    }

    best.ok_or(TimingError::NoValidTimingFound)
}
