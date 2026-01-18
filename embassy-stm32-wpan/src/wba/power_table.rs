//! TX Power Tables for STM32WBA BLE
//!
//! This module defines the TX power tables required by the BLE link layer.

use crate::bindings::link_layer::{_power_table_id_t, power_table_entry};

/// VDD LDO value for maximum power mode
const VDD_LDO_VALUE_MAX_POWER: u8 = 0x70;
/// VDD LDO value for low power mode
const VDD_LDO_VALUE_LOW_POWER: u8 = 0x20;
/// VDD LDO value 2 ID 0
const VDD_LDO_VALUE_2_ID_0: u8 = 0x00;

/// TX power table for maximum power mode
#[unsafe(link_section = ".rodata")]
static LL_TX_POWER_TABLE_MAX_POWER: [power_table_entry; 31] = [
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x02,
        epa_bypass: 0x01,
        tx_pwr: -20,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x03,
        epa_bypass: 0x01,
        tx_pwr: -19,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x04,
        epa_bypass: 0x01,
        tx_pwr: -18,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x05,
        epa_bypass: 0x01,
        tx_pwr: -17,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x06,
        epa_bypass: 0x01,
        tx_pwr: -16,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x07,
        epa_bypass: 0x01,
        tx_pwr: -15,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x08,
        epa_bypass: 0x01,
        tx_pwr: -14,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x09,
        epa_bypass: 0x01,
        tx_pwr: -13,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x0A,
        epa_bypass: 0x01,
        tx_pwr: -12,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x0B,
        epa_bypass: 0x01,
        tx_pwr: -11,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x0C,
        epa_bypass: 0x01,
        tx_pwr: -10,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x0D,
        epa_bypass: 0x01,
        tx_pwr: -9,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x0E,
        epa_bypass: 0x01,
        tx_pwr: -8,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x0F,
        epa_bypass: 0x01,
        tx_pwr: -7,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x10,
        epa_bypass: 0x01,
        tx_pwr: -6,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x11,
        epa_bypass: 0x01,
        tx_pwr: -5,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x12,
        epa_bypass: 0x01,
        tx_pwr: -4,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x13,
        epa_bypass: 0x01,
        tx_pwr: -3,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x14,
        epa_bypass: 0x01,
        tx_pwr: -2,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x15,
        epa_bypass: 0x01,
        tx_pwr: -1,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x16,
        epa_bypass: 0x01,
        tx_pwr: 0,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x17,
        epa_bypass: 0x01,
        tx_pwr: 1,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x18,
        epa_bypass: 0x01,
        tx_pwr: 2,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x18,
        epa_bypass: 0x01,
        tx_pwr: 3,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x19,
        epa_bypass: 0x01,
        tx_pwr: 4,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x19,
        epa_bypass: 0x01,
        tx_pwr: 5,
    },
    power_table_entry {
        vddh_pa: 0x03,
        internal_pa_code: 0x19,
        epa_bypass: 0x01,
        tx_pwr: 6,
    },
    power_table_entry {
        vddh_pa: 0x05,
        internal_pa_code: 0x19,
        epa_bypass: 0x01,
        tx_pwr: 7,
    },
    power_table_entry {
        vddh_pa: 0x06,
        internal_pa_code: 0x19,
        epa_bypass: 0x01,
        tx_pwr: 8,
    },
    power_table_entry {
        vddh_pa: 0x08,
        internal_pa_code: 0x19,
        epa_bypass: 0x01,
        tx_pwr: 9,
    },
    power_table_entry {
        vddh_pa: 0x0D,
        internal_pa_code: 0x19,
        epa_bypass: 0x01,
        tx_pwr: 10,
    },
];

/// TX power table for low power mode
#[unsafe(link_section = ".rodata")]
static LL_TX_POWER_TABLE_LOW_POWER: [power_table_entry; 24] = [
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x02,
        epa_bypass: 0x01,
        tx_pwr: -20,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x03,
        epa_bypass: 0x01,
        tx_pwr: -19,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x05,
        epa_bypass: 0x01,
        tx_pwr: -18,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x06,
        epa_bypass: 0x01,
        tx_pwr: -17,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x07,
        epa_bypass: 0x01,
        tx_pwr: -16,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x08,
        epa_bypass: 0x01,
        tx_pwr: -15,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x09,
        epa_bypass: 0x01,
        tx_pwr: -14,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x0A,
        epa_bypass: 0x01,
        tx_pwr: -13,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x0B,
        epa_bypass: 0x01,
        tx_pwr: -12,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x0C,
        epa_bypass: 0x01,
        tx_pwr: -11,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x0D,
        epa_bypass: 0x01,
        tx_pwr: -10,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x0E,
        epa_bypass: 0x01,
        tx_pwr: -9,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x0F,
        epa_bypass: 0x01,
        tx_pwr: -8,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x10,
        epa_bypass: 0x01,
        tx_pwr: -7,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x11,
        epa_bypass: 0x01,
        tx_pwr: -6,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x12,
        epa_bypass: 0x01,
        tx_pwr: -5,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x13,
        epa_bypass: 0x01,
        tx_pwr: -4,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x14,
        epa_bypass: 0x01,
        tx_pwr: -3,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x15,
        epa_bypass: 0x01,
        tx_pwr: -2,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x17,
        epa_bypass: 0x01,
        tx_pwr: -1,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x17,
        epa_bypass: 0x01,
        tx_pwr: 0,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x18,
        epa_bypass: 0x01,
        tx_pwr: 1,
    },
    power_table_entry {
        vddh_pa: 0x00,
        internal_pa_code: 0x19,
        epa_bypass: 0x01,
        tx_pwr: 2,
    },
    power_table_entry {
        vddh_pa: 0x02,
        internal_pa_code: 0x19,
        epa_bypass: 0x01,
        tx_pwr: 3,
    },
];

/// Wrapper type for _power_table_id_t that implements Sync
/// SAFETY: The contained data is read-only and only accessed from C code
#[repr(transparent)]
pub struct SyncPowerTableId(_power_table_id_t);

// SAFETY: The power table is only read, never mutated, and is accessed
// from a single-threaded embedded context
unsafe impl Sync for SyncPowerTableId {}

/// Supported TX power tables
#[unsafe(no_mangle)]
#[unsafe(link_section = ".rodata")]
pub static ll_tx_power_tables: [SyncPowerTableId; 2] = [
    SyncPowerTableId(_power_table_id_t {
        ptr_tx_power_table: LL_TX_POWER_TABLE_MAX_POWER.as_ptr(),
        tx_power_levels_count: LL_TX_POWER_TABLE_MAX_POWER.len() as u8,
        g_vdd_ldo_value_1: VDD_LDO_VALUE_MAX_POWER,
        g_vdd_ldo_value_2: VDD_LDO_VALUE_2_ID_0,
        power_table_id: 0,
    }),
    SyncPowerTableId(_power_table_id_t {
        ptr_tx_power_table: LL_TX_POWER_TABLE_LOW_POWER.as_ptr(),
        tx_power_levels_count: LL_TX_POWER_TABLE_LOW_POWER.len() as u8,
        g_vdd_ldo_value_1: VDD_LDO_VALUE_LOW_POWER,
        g_vdd_ldo_value_2: VDD_LDO_VALUE_2_ID_0,
        power_table_id: 1,
    }),
];

/// Number of supported TX power tables
#[unsafe(no_mangle)]
#[unsafe(link_section = ".rodata")]
pub static num_of_supported_power_tables: u8 = 2;
