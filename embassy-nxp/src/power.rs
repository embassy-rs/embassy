/*
 * Copyright (c) 2016, Freescale Semiconductor, Inc.
 * Copyright 2016, NXP
 * All rights reserved.
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

use core::ptr::read_volatile;

use crate::pac;

#[cfg(feature = "lpc55-core0")]
const NMPA_BASE: usize = 0x0009_fc00;
#[cfg(feature = "lpc55s16")]
const NMPA_BASE: usize = 0x0003_fc00;
const DCDC_PROFILE_LOW_0: usize = NMPA_BASE + 0xe0;
const DCDC_PROFILE_LOW_1: usize = NMPA_BASE + 0xe4;
const DCDC_PROFILE_MEDIUM_0: usize = NMPA_BASE + 0xe8;
const DCDC_PROFILE_MEDIUM_1: usize = NMPA_BASE + 0xec;
const DCDC_PROFILE_HIGH_0: usize = NMPA_BASE + 0xd8;
const DCDC_PROFILE_HIGH_1: usize = NMPA_BASE + 0xdc;
const PVT_MONITOR_0_RINGO: usize = NMPA_BASE + 0x130;
const PVT_MONITOR_1_RINGO: usize = NMPA_BASE + 0x140;

#[cfg(feature = "lpc55-core0")]
const DCDC_PROFILE_LOW_MAX_HZ: u32 = 100_000_000;
#[cfg(feature = "lpc55-core0")]
const DCDC_PROFILE_MEDIUM_MAX_HZ: u32 = 130_000_000;
#[cfg(feature = "lpc55s16")]
const DCDC_PROFILE_LOW_MAX_HZ: u32 = 72_000_000;
#[cfg(feature = "lpc55s16")]
const DCDC_PROFILE_MEDIUM_MAX_HZ: u32 = 100_000_000;
const DCDC_PROFILE_HIGH_MAX_HZ: u32 = 150_000_000;

const PROCESS_NNN_AVG_HZ: u32 = 19_300_000;
const PROCESS_NNN_STD_HZ: u32 = 400_000;
const PROCESS_NNN_LIMITS: u32 = 6;
const PROCESS_NNN_MIN_HZ: u32 = PROCESS_NNN_AVG_HZ - PROCESS_NNN_LIMITS * PROCESS_NNN_STD_HZ;
const PROCESS_NNN_MAX_HZ: u32 = PROCESS_NNN_AVG_HZ + PROCESS_NNN_LIMITS * PROCESS_NNN_STD_HZ;

#[derive(Clone, Copy)]
enum DcdcPowerProfile {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy)]
enum ProcessCorner {
    Slow,
    Nominal,
    Fast,
}

pub(crate) fn set_voltage_for_freq(system_freq_hz: u32) {
    critical_section::with(|_| {
        let profile = if system_freq_hz <= DCDC_PROFILE_LOW_MAX_HZ {
            DcdcPowerProfile::Low
        } else if system_freq_hz <= DCDC_PROFILE_MEDIUM_MAX_HZ {
            DcdcPowerProfile::Medium
        } else if system_freq_hz <= DCDC_PROFILE_HIGH_MAX_HZ {
            DcdcPowerProfile::High
        } else {
            panic!(
                "LPC55 frequency {} Hz exceeds the 150 MHz voltage profile",
                system_freq_hz
            );
        };

        set_dcdc_power_profile(profile);
        set_voltage_for_process(profile);
    });
}

fn set_dcdc_power_profile(profile: DcdcPowerProfile) {
    let (trim_0_address, trim_1_address) = match profile {
        DcdcPowerProfile::Low => (DCDC_PROFILE_LOW_0, DCDC_PROFILE_LOW_1),
        DcdcPowerProfile::Medium => (DCDC_PROFILE_MEDIUM_0, DCDC_PROFILE_MEDIUM_1),
        DcdcPowerProfile::High => (DCDC_PROFILE_HIGH_0, DCDC_PROFILE_HIGH_1),
    };
    let trim_0 = read_nmpa_word(trim_0_address);
    let trim_1 = read_nmpa_word(trim_1_address);

    if trim_0 & 1 != 0 {
        pac::PMC.dcdc0().write_value(pac::pmc::regs::Dcdc0(trim_0 >> 1));
        pac::PMC.dcdc1().write_value(pac::pmc::regs::Dcdc1(trim_1));
    }
}

fn process_corner() -> ProcessCorner {
    let ringo_0 = valid_ringo_or_nominal(read_nmpa_word(PVT_MONITOR_0_RINGO));
    let ringo_1 = valid_ringo_or_nominal(read_nmpa_word(PVT_MONITOR_1_RINGO));
    let ringo_hz = ringo_0.min(ringo_1);

    if ringo_hz <= PROCESS_NNN_MIN_HZ {
        ProcessCorner::Slow
    } else if ringo_hz <= PROCESS_NNN_MAX_HZ {
        ProcessCorner::Nominal
    } else {
        ProcessCorner::Fast
    }
}

fn valid_ringo_or_nominal(trim: u32) -> u32 {
    if trim & 1 != 0 { trim >> 1 } else { PROCESS_NNN_AVG_HZ }
}

fn set_voltage_for_process(profile: DcdcPowerProfile) {
    set_system_voltage(voltage_for_process(process_corner(), profile));
}

#[cfg(feature = "lpc55-core0")]
fn voltage_for_process(corner: ProcessCorner, profile: DcdcPowerProfile) -> u32 {
    match (corner, profile) {
        (ProcessCorner::Slow, DcdcPowerProfile::Low) => 1075,
        (ProcessCorner::Slow, DcdcPowerProfile::Medium) => 1150,
        (ProcessCorner::Slow, DcdcPowerProfile::High) => 1200,
        (ProcessCorner::Nominal, DcdcPowerProfile::Low) => 1000,
        (ProcessCorner::Nominal, DcdcPowerProfile::Medium) => 1100,
        (ProcessCorner::Nominal, DcdcPowerProfile::High) => 1150,
        (ProcessCorner::Fast, DcdcPowerProfile::Low) => 1000,
        (ProcessCorner::Fast, DcdcPowerProfile::Medium) => 1025,
        (ProcessCorner::Fast, DcdcPowerProfile::High) => 1050,
    }
}

#[cfg(feature = "lpc55s16")]
fn voltage_for_process(corner: ProcessCorner, profile: DcdcPowerProfile) -> u32 {
    match (corner, profile) {
        (ProcessCorner::Slow, DcdcPowerProfile::Low) => 1100,
        (ProcessCorner::Slow, DcdcPowerProfile::Medium) => 1150,
        (ProcessCorner::Slow, DcdcPowerProfile::High) => 1200,
        (ProcessCorner::Nominal, DcdcPowerProfile::Low) => 1050,
        (ProcessCorner::Nominal, DcdcPowerProfile::Medium) => 1075,
        (ProcessCorner::Nominal, DcdcPowerProfile::High) => 1150,
        (ProcessCorner::Fast, DcdcPowerProfile::Low) => 1000,
        (ProcessCorner::Fast, DcdcPowerProfile::Medium) => 1025,
        (ProcessCorner::Fast, DcdcPowerProfile::High) => 1050,
    }
}

fn set_system_voltage(system_voltage_mv: u32) {
    let (dcdc, ldo_ao, ldo_ao_boost) = if system_voltage_mv <= 950 {
        (0, 10, 15)
    } else if system_voltage_mv <= 975 {
        (1, 12, 17)
    } else if system_voltage_mv <= 1000 {
        (2, 14, 19)
    } else if system_voltage_mv <= 1025 {
        (3, 17, 22)
    } else if system_voltage_mv <= 1050 {
        (4, 20, 25)
    } else if system_voltage_mv <= 1075 {
        (5, 22, 27)
    } else if system_voltage_mv <= 1100 {
        (6, 24, 29)
    } else if system_voltage_mv <= 1125 {
        (7, 27, 30)
    } else if system_voltage_mv <= 1150 {
        (8, 30, 31)
    } else if system_voltage_mv <= 1175 {
        (9, 30, 31)
    } else {
        (10, 30, 31)
    };

    pac::PMC.ldopmu().modify(|w| {
        w.set_vadj(pac::pmc::vals::Vadj::from_bits(ldo_ao));
        w.set_vadj_boost(ldo_ao_boost);
    });
    pac::PMC
        .dcdc0()
        .modify(|w| w.set_vout(pac::pmc::vals::Vout::from_bits(dcdc)));
}

fn read_nmpa_word(address: usize) -> u32 {
    // SAFETY: These fixed, word-aligned addresses are documented NMPA words for the selected LPC55 device.
    unsafe { read_volatile(address as *const u32) }
}
