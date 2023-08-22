use embassy_time::{Duration, Timer};
use embedded_hal_1::digital::OutputPin;

use crate::bus::Bus;
use crate::consts::*;
use crate::{SpiBusCyw43, CHIP};

fn is_aligned(a: u32, x: u32) -> bool {
    (a & (x - 1)) == 0
}

fn round_down(x: u32, a: u32) -> u32 {
    x & !(a - 1)
}

pub(crate) async fn upload_bluetooth_firmware<PWR: OutputPin, SPI: SpiBusCyw43>(
    bus: &mut Bus<PWR, SPI>,
    firmware_offsets: &[(u32, usize)],
    firmware: &[u8],
) {
    // buffer
    let mut aligned_data_buffer: [u8; 0x100] = [0; 0x100];
    // structs
    let mut fw_bytes_pointer = 0;
    for (index, &(dest_addr, num_fw_bytes)) in firmware_offsets.iter().enumerate() {
        let fw_bytes = &firmware[(fw_bytes_pointer)..(fw_bytes_pointer + num_fw_bytes)];
        assert!(fw_bytes.len() == num_fw_bytes);
        let mut dest_start_addr = dest_addr;
        let mut aligned_data_buffer_index: usize = 0;
        // pad start
        if !is_aligned(dest_start_addr, 4) {
            let num_pad_bytes = dest_start_addr % 4;
            let padded_dest_start_addr = round_down(dest_start_addr, 4);
            let memory_value = bus.bp_read32(padded_dest_start_addr).await;
            let memory_value_bytes = memory_value.to_le_bytes(); // TODO: le or be
            // Copy the previous memory value's bytes to the start
            for i in 0..num_pad_bytes as usize {
                aligned_data_buffer[aligned_data_buffer_index] = memory_value_bytes[i];
                aligned_data_buffer_index += 1;
            }
            // Copy the firmware bytes after the padding bytes
            for i in 0..num_fw_bytes as usize {
                aligned_data_buffer[aligned_data_buffer_index] = fw_bytes[i];
                aligned_data_buffer_index += 1;
            }
            dest_start_addr = padded_dest_start_addr;
        } else {
            // Directly copy fw_bytes into aligned_data_buffer if no start padding is required
            for i in 0..num_fw_bytes as usize {
                aligned_data_buffer[aligned_data_buffer_index] = fw_bytes[i];
                aligned_data_buffer_index += 1;
            }
        }
        // pad end
        let mut dest_end_addr = dest_start_addr + aligned_data_buffer_index as u32;
        if !is_aligned(dest_end_addr, 4) {
            let offset = dest_end_addr % 4;
            let num_pad_bytes_end = 4 - offset;
            let padded_dest_end_addr = round_down(dest_end_addr, 4);
            let memory_value = bus.bp_read32(padded_dest_end_addr).await;
            let memory_value_bytes = memory_value.to_le_bytes(); // TODO: le or be
            // Append the necessary memory bytes to pad the end of aligned_data_buffer
            for i in offset..4 {
                aligned_data_buffer[aligned_data_buffer_index] = memory_value_bytes[i as usize];
                aligned_data_buffer_index += 1;
            }
            dest_end_addr += num_pad_bytes_end;
        } else {
            // pad end alignment not needed
        }
        let buffer_to_write = &aligned_data_buffer[0..aligned_data_buffer_index as usize];
        assert!(dest_start_addr % 4 == 0);
        assert!(dest_end_addr % 4 == 0);
        assert!(aligned_data_buffer_index % 4 == 0);
        // write in 0x40 chunks TODO: is this needed or can we write straight away
        let chunk_size = 0x40;
        for (i, chunk) in buffer_to_write.chunks(chunk_size).enumerate() {
            let offset = i * chunk_size;
            bus.bp_write(dest_start_addr + (offset as u32), chunk).await;
        }
        // increment fw_bytes_pointer
        fw_bytes_pointer += num_fw_bytes;
        // sleep TODO: is this needed
        Timer::after(Duration::from_millis(1)).await;
    }
}

pub(crate) async fn wait_bt_ready<PWR: OutputPin, SPI: SpiBusCyw43>(bus: &mut Bus<PWR, SPI>) {
    debug!("wait_bt_ready");
    let mut success = false;
    for _ in 0..300 {
        let val = bus.bp_read32(BT_CTRL_REG_ADDR).await;
        // TODO: do we need to swap endianness on this read?
        debug!("BT_CTRL_REG_ADDR = {:08x}", val);
        if val & BTSDIO_REG_FW_RDY_BITMASK != 0 {
            success = true;
            break;
        }
        Timer::after(Duration::from_millis(1)).await;
    }
    assert!(success == true);
}

pub(crate) async fn wait_bt_awake<PWR: OutputPin, SPI: SpiBusCyw43>(bus: &mut Bus<PWR, SPI>) {
    debug!("wait_bt_awake");
    let mut success = false;
    for _ in 0..300 {
        let val = bus.bp_read32(BT_CTRL_REG_ADDR).await;
        // TODO: do we need to swap endianness on this read?
        debug!("BT_CTRL_REG_ADDR = {:08x}", val);
        if val & BTSDIO_REG_BT_AWAKE_BITMASK != 0 {
            success = true;
            break;
        }
        Timer::after(Duration::from_millis(1)).await;
    }
    assert!(success == true);
}

pub(crate) async fn bt_set_host_ready<PWR: OutputPin, SPI: SpiBusCyw43>(bus: &mut Bus<PWR, SPI>) {
    debug!("bt_set_host_ready");
    let old_val = bus.bp_read32(HOST_CTRL_REG_ADDR).await;
    // TODO: do we need to swap endianness on this read?
    let new_val = old_val | BTSDIO_REG_SW_RDY_BITMASK;
    bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
}

// TODO: use this
#[allow(dead_code)]
pub(crate) async fn bt_set_awake<PWR: OutputPin, SPI: SpiBusCyw43>(bus: &mut Bus<PWR, SPI>) {
    debug!("bt_set_awake");
    let old_val = bus.bp_read32(HOST_CTRL_REG_ADDR).await;
    // TODO: do we need to swap endianness on this read?
    let new_val = old_val | BTSDIO_REG_WAKE_BT_BITMASK;
    bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
}

pub(crate) async fn bt_toggle_intr<PWR: OutputPin, SPI: SpiBusCyw43>(bus: &mut Bus<PWR, SPI>) {
    debug!("bt_toggle_intr");
    let old_val = bus.bp_read32(HOST_CTRL_REG_ADDR).await;
    // TODO: do we need to swap endianness on this read?
    let new_val = old_val ^ BTSDIO_REG_DATA_VALID_BITMASK;
    bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
}

// TODO: use this
#[allow(dead_code)]
pub(crate) async fn bt_set_intr<PWR: OutputPin, SPI: SpiBusCyw43>(bus: &mut Bus<PWR, SPI>) {
    debug!("bt_set_intr");
    let old_val = bus.bp_read32(HOST_CTRL_REG_ADDR).await;
    let new_val = old_val | BTSDIO_REG_DATA_VALID_BITMASK;
    bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
}

pub(crate) async fn init_bluetooth<PWR: OutputPin, SPI: SpiBusCyw43>(
    bus: &mut Bus<PWR, SPI>,
    firmware_offsets: &[(u32, usize)],
    firmware: &[u8],
) {
    Timer::after(Duration::from_millis(100)).await;
    debug!("init_bluetooth");
    Timer::after(Duration::from_millis(100)).await;
    bus.bp_write32(CHIP.bluetooth_base_address + BT2WLAN_PWRUP_ADDR, BT2WLAN_PWRUP_WAKE)
        .await;
    Timer::after(Duration::from_millis(2)).await;
    upload_bluetooth_firmware(bus, firmware_offsets, firmware).await;
    wait_bt_ready(bus).await;
    // TODO: cybt_init_buffer();
    wait_bt_awake(bus).await;
    bt_set_host_ready(bus).await;
    bt_toggle_intr(bus).await;
}
