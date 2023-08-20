use embassy_time::{Timer, Duration};
use embedded_hal_1::digital::OutputPin;

use crate::{consts::*, CHIP, bus::Bus, SpiBusCyw43};

#[derive(Debug)]
pub(crate) struct CybtFwCb<'a> {
    pub p_fw_mem_start: &'a [u8],
    pub fw_len: u32,
    pub p_next_line_start: &'a [u8],
}

#[derive(Debug)]
pub(crate) struct HexFileData<'a> {
    pub addr_mode: i32,
    pub hi_addr: u16,
    pub dest_addr: u32,
    pub p_ds: &'a mut [u8],
}

pub(crate) fn is_aligned(a: u32, x: u32) -> bool {
    (a & (x - 1)) == 0
}

pub(crate) fn round_down(x: u32, a: u32) -> u32 {
    x & !(a - 1)
}

pub(crate) fn cybt_fw_get_data(p_btfw_cb: &mut CybtFwCb, hfd: &mut HexFileData) -> u32 {
    let mut abs_base_addr32 = 0;

    loop {
        let num_bytes = p_btfw_cb.p_next_line_start[0];
        p_btfw_cb.p_next_line_start = &p_btfw_cb.p_next_line_start[1..];

        let addr = (p_btfw_cb.p_next_line_start[0] as u16) << 8 | p_btfw_cb.p_next_line_start[1] as u16;
        p_btfw_cb.p_next_line_start = &p_btfw_cb.p_next_line_start[2..];

        let line_type = p_btfw_cb.p_next_line_start[0];
        p_btfw_cb.p_next_line_start = &p_btfw_cb.p_next_line_start[1..];

        if num_bytes == 0 {
            break;
        }

        hfd.p_ds[..num_bytes as usize].copy_from_slice(&p_btfw_cb.p_next_line_start[..num_bytes as usize]);
        p_btfw_cb.p_next_line_start = &p_btfw_cb.p_next_line_start[num_bytes as usize..];

        match line_type {
            BTFW_HEX_LINE_TYPE_EXTENDED_ADDRESS => {
                hfd.hi_addr = (hfd.p_ds[0] as u16) << 8 | hfd.p_ds[1] as u16;
                hfd.addr_mode = BTFW_ADDR_MODE_EXTENDED;
            }
            BTFW_HEX_LINE_TYPE_EXTENDED_SEGMENT_ADDRESS => {
                hfd.hi_addr = (hfd.p_ds[0] as u16) << 8 | hfd.p_ds[1] as u16;
                hfd.addr_mode = BTFW_ADDR_MODE_SEGMENT;
            }
            BTFW_HEX_LINE_TYPE_ABSOLUTE_32BIT_ADDRESS => {
                abs_base_addr32 = (hfd.p_ds[0] as u32) << 24 | (hfd.p_ds[1] as u32) << 16 |
                                  (hfd.p_ds[2] as u32) << 8 | hfd.p_ds[3] as u32;
                hfd.addr_mode = BTFW_ADDR_MODE_LINEAR32;
            }
            BTFW_HEX_LINE_TYPE_DATA => {
                hfd.dest_addr = addr as u32;
                match hfd.addr_mode {
                    BTFW_ADDR_MODE_EXTENDED => hfd.dest_addr += (hfd.hi_addr as u32) << 16,
                    BTFW_ADDR_MODE_SEGMENT => hfd.dest_addr += (hfd.hi_addr as u32) << 4,
                    BTFW_ADDR_MODE_LINEAR32 => hfd.dest_addr += abs_base_addr32,
                    _ => {}
                }
                return num_bytes as u32;
            }
            _ => {}
        }
    }
    0
}

pub(crate) async fn upload_bluetooth_firmware<PWR: OutputPin, SPI: SpiBusCyw43>(bus: &mut Bus<PWR, SPI>, firmware: &[u8]) {
    // read version
    let version_length = firmware[0];
    let _version = &firmware[1..=version_length as usize];
    // skip version + 1 extra byte as per cybt_shared_bus_driver.c
    let firmware = &firmware[version_length as usize + 2..];
    // buffer
    let mut data_buffer: [u8; 0x100] = [0; 0x100]; 
    // structs
    let mut btfw_cb = CybtFwCb {
        p_fw_mem_start: firmware,
        fw_len: firmware.len() as u32,
        p_next_line_start: firmware,
    };
    let mut hfd = HexFileData {
        addr_mode: BTFW_ADDR_MODE_UNKNOWN,
        hi_addr: 0,
        dest_addr: 0,
        p_ds: &mut data_buffer,
    };
    loop {
        let mut aligned_data_buffer: [u8; 0x100] = [0; 0x100];
        let num_fw_bytes = cybt_fw_get_data(&mut btfw_cb, &mut hfd);
        if num_fw_bytes == 0 {
            break;
        }
        let fw_bytes = &hfd.p_ds[0..num_fw_bytes as usize];
        let mut dest_start_addr = hfd.dest_addr + CHIP.bluetooth_base_address;
        let mut aligned_data_buffer_index: usize = 0;
        // pad start
        if !is_aligned(dest_start_addr, 4) {
            let num_pad_bytes = dest_start_addr % 4;
            let padded_dest_start_addr = round_down(dest_start_addr, 4);
            let mut memory_value_bytes = [0; 4];
            bus.bp_read(padded_dest_start_addr, &mut memory_value_bytes).await;
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
            let mut memory_value_bytes = [0; 4];
            bus.bp_read(padded_dest_end_addr, &mut memory_value_bytes).await;
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
        debug!("upload_bluetooth_firmware: dest_start_addr = {:x} dest_end_addr = {:x} aligned_data_buffer_index = {} buffer_to_write = {:02x}", dest_start_addr, dest_end_addr, aligned_data_buffer_index, buffer_to_write);
        assert!(dest_start_addr % 4 == 0);
        assert!(dest_end_addr % 4 == 0);
        assert!(aligned_data_buffer_index % 4 == 0);
        bus.bp_write(dest_start_addr, buffer_to_write).await;
    }
}

pub(crate) async fn wait_bt_ready<PWR: OutputPin, SPI: SpiBusCyw43>(bus: &mut Bus<PWR, SPI>) {
    debug!("wait_bt_ready");
    let mut success = false;
    for _ in 0..300 {
        let val = bus.bp_read32(BT_CTRL_REG_ADDR).await;
        // TODO: do we need to swap endianness on this read?
        debug!("BT_CTRL_REG_ADDR = {:08x}", val);
        /*if val & BTSDIO_REG_FW_RDY_BITMASK != 0 {
            break;
        }*/
        // TODO: should be 00000000 until it is 0x01000100
        if val == 0x01000100 {
            success = true;
            break;
        }
        Timer::after(Duration::from_millis(1)).await;
    }
    assert!(success == true);
}

pub(crate) async fn wait_bt_awake<PWR: OutputPin, SPI: SpiBusCyw43>(bus: &mut Bus<PWR, SPI>) {
    debug!("wait_bt_awake");        
    loop {
        let val = bus.bp_read32(BT_CTRL_REG_ADDR).await;
        // TODO: do we need to swap endianness on this read?
        debug!("BT_CTRL_REG_ADDR = {:08x}", val);
        if val & BTSDIO_REG_BT_AWAKE_BITMASK != 0 {
            break;
        }
        Timer::after(Duration::from_millis(100)).await;
    }
}

pub(crate) async fn bt_set_host_ready<PWR: OutputPin, SPI: SpiBusCyw43>(bus: &mut Bus<PWR, SPI>) {
    debug!("bt_set_host_ready");        
    let old_val = bus.bp_read32(HOST_CTRL_REG_ADDR).await;
    // TODO: do we need to swap endianness on this read?
    let new_val = old_val | BTSDIO_REG_SW_RDY_BITMASK;
    bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
}

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

pub(crate) async fn bt_set_intr<PWR: OutputPin, SPI: SpiBusCyw43>(bus: &mut Bus<PWR, SPI>) {
    debug!("bt_set_intr");        
    let old_val = bus.bp_read32(HOST_CTRL_REG_ADDR).await;
    let new_val = old_val | BTSDIO_REG_DATA_VALID_BITMASK;
    bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
}

pub(crate) async fn init_bluetooth<PWR: OutputPin, SPI: SpiBusCyw43>(bus: &mut Bus<PWR, SPI>, firmware: &[u8]) {
    bus.bp_write32(CHIP.bluetooth_base_address + BT2WLAN_PWRUP_ADDR, BT2WLAN_PWRUP_WAKE).await;
    upload_bluetooth_firmware(bus, firmware).await;
    wait_bt_ready(bus).await;
    // TODO: cybt_init_buffer();
    wait_bt_awake(bus).await;
    bt_set_host_ready(bus).await;
    bt_toggle_intr(bus).await;
}
