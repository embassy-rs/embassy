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
const DCDC_TRIM_ADDRESSES: [(usize, usize); 3] = [
    (NMPA_BASE + 0xe0, NMPA_BASE + 0xe4),
    (NMPA_BASE + 0xe8, NMPA_BASE + 0xec),
    (NMPA_BASE + 0xd8, NMPA_BASE + 0xdc),
];
#[cfg(feature = "lpc55-core0")]
const DCDC_PROFILE_MAX_HZ: [u32; 3] = [100_000_000, 130_000_000, 150_000_000];
#[cfg(feature = "lpc55s16")]
const DCDC_PROFILE_MAX_HZ: [u32; 3] = [72_000_000, 100_000_000, 150_000_000];
const PVT_MONITOR_0_RINGO: usize = NMPA_BASE + 0x130;
const PVT_MONITOR_1_RINGO: usize = NMPA_BASE + 0x140;

const PROCESS_NNN_AVG_HZ: u32 = 19_300_000;
const PROCESS_NNN_STD_HZ: u32 = 400_000;
const PROCESS_NNN_LIMITS: u32 = 6;
const PROCESS_NNN_MIN_HZ: u32 = PROCESS_NNN_AVG_HZ - PROCESS_NNN_LIMITS * PROCESS_NNN_STD_HZ;
const PROCESS_NNN_MAX_HZ: u32 = PROCESS_NNN_AVG_HZ + PROCESS_NNN_LIMITS * PROCESS_NNN_STD_HZ;

#[derive(Clone, Copy)]
#[repr(usize)]
enum DcdcPowerProfile {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy)]
#[repr(usize)]
enum ProcessCorner {
    Slow,
    Nominal,
    Fast,
}

const DCDC_PROFILES: [DcdcPowerProfile; 3] = [DcdcPowerProfile::Low, DcdcPowerProfile::Medium, DcdcPowerProfile::High];

#[cfg(feature = "lpc55-core0")]
const PROCESS_VOLTAGES_MV: [[u32; 3]; 3] = [[1075, 1150, 1200], [1000, 1100, 1150], [1000, 1025, 1050]];
#[cfg(feature = "lpc55s16")]
const PROCESS_VOLTAGES_MV: [[u32; 3]; 3] = [[1100, 1150, 1200], [1050, 1075, 1150], [1000, 1025, 1050]];

const SYSTEM_VOLTAGE_REGISTERS: [(u32, u8, u8, u8); 11] = [
    (950, 0, 10, 15),
    (975, 1, 12, 17),
    (1000, 2, 14, 19),
    (1025, 3, 17, 22),
    (1050, 4, 20, 25),
    (1075, 5, 22, 27),
    (1100, 6, 24, 29),
    (1125, 7, 27, 30),
    (1150, 8, 30, 31),
    (1175, 9, 30, 31),
    (u32::MAX, 10, 30, 31),
];

pub(crate) fn set_voltage_for_freq(system_freq_hz: u32) {
    critical_section::with(|_| {
        let profile = match DCDC_PROFILE_MAX_HZ.iter().position(|&max_hz| system_freq_hz <= max_hz) {
            Some(index) => DCDC_PROFILES[index],
            None => panic!(
                "LPC55 frequency {} Hz exceeds the 150 MHz voltage profile",
                system_freq_hz
            ),
        };

        set_dcdc_power_profile(profile);
        set_voltage_for_process(profile);
    });
}

fn set_dcdc_power_profile(profile: DcdcPowerProfile) {
    let (trim_0_address, trim_1_address) = DCDC_TRIM_ADDRESSES[profile as usize];
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

fn voltage_for_process(corner: ProcessCorner, profile: DcdcPowerProfile) -> u32 {
    PROCESS_VOLTAGES_MV[corner as usize][profile as usize]
}

fn set_system_voltage(system_voltage_mv: u32) {
    let (_, dcdc, ldo_ao, ldo_ao_boost) = SYSTEM_VOLTAGE_REGISTERS
        .iter()
        .copied()
        .find(|(max_mv, _, _, _)| system_voltage_mv <= *max_mv)
        .expect("system voltage table must cover every u32 value");

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
