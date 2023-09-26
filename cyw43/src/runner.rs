use embassy_futures::select::{select3, Either3};
use embassy_net_driver_channel as ch;
use embassy_sync::pubsub::PubSubBehavior;
use embassy_time::{block_for, Duration, Timer};
use embedded_hal_1::digital::OutputPin;

use crate::bluetooth::{CybtFwCb, HexFileData};
use crate::bus::Bus;
pub use crate::bus::SpiBusCyw43;
use crate::consts::*;
use crate::events::{Event, Events, Status};
use crate::fmt::Bytes;
use crate::ioctl::{IoctlState, IoctlType, PendingIoctl};
use crate::nvram::NVRAM;
use crate::structs::*;
use crate::{bluetooth, events, utilities, Core, CHIP, MTU};

#[cfg(feature = "firmware-logs")]
struct LogState {
    addr: u32,
    last_idx: usize,
    buf: [u8; 256],
    buf_count: usize,
}

#[cfg(feature = "firmware-logs")]
impl Default for LogState {
    fn default() -> Self {
        Self {
            addr: Default::default(),
            last_idx: Default::default(),
            buf: [0; 256],
            buf_count: Default::default(),
        }
    }
}

pub struct Runner<'a, PWR, SPI> {
    ch: ch::Runner<'a, MTU>,
    bus: Bus<PWR, SPI>,

    ioctl_state: &'a IoctlState,
    ioctl_id: u16,
    sdpcm_seq: u8,
    sdpcm_seq_max: u8,

    events: &'a Events,

    #[cfg(feature = "firmware-logs")]
    log: LogState,

    // Bluetooth circular buffers
    h2b_buf_addr: u32,
    h2b_buf_addr_pointer: u32,
    b2h_buf_addr: u32,
    b2h_buf_addr_pointer: u32,
}

impl<'a, PWR, SPI> Runner<'a, PWR, SPI>
where
    PWR: OutputPin,
    SPI: SpiBusCyw43,
{
    pub(crate) fn new(
        ch: ch::Runner<'a, MTU>,
        bus: Bus<PWR, SPI>,
        ioctl_state: &'a IoctlState,
        events: &'a Events,
    ) -> Self {
        Self {
            ch,
            bus,

            ioctl_state,
            ioctl_id: 0,
            sdpcm_seq: 0,
            sdpcm_seq_max: 1,

            events,

            #[cfg(feature = "firmware-logs")]
            log: LogState::default(),

            h2b_buf_addr: 0,
            h2b_buf_addr_pointer: 0,
            b2h_buf_addr: 0,
            b2h_buf_addr_pointer: 0,
        }
    }

    pub(crate) async fn init(&mut self, firmware: &[u8], bluetooth_firmware: Option<&[u8]>) {
        self.bus.init(bluetooth_firmware.is_some()).await;

        // Init ALP (Active Low Power) clock
        debug!("init alp");
        self.bus
            .write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, BACKPLANE_ALP_AVAIL_REQ)
            .await;

        // check we can set the bluetooth watermark
        if bluetooth_firmware.is_some() {
            debug!("set bluetooth watermark");
            self.bus
                .write8(FUNC_BACKPLANE, REG_BACKPLANE_FUNCTION2_WATERMARK, 0x10)
                .await;
            let watermark = self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_FUNCTION2_WATERMARK).await;
            debug!("watermark = {:02x}", watermark);
            assert!(watermark == 0x10);
        }

        debug!("waiting for clock...");
        while self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & BACKPLANE_ALP_AVAIL == 0 {}
        debug!("clock ok");

        // clear request for ALP
        debug!("clear request for ALP");
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, 0).await;

        let chip_id = self.bus.bp_read16(0x1800_0000).await;
        debug!("chip ID: {}", chip_id);

        // Upload firmware.
        self.core_disable(Core::WLAN).await;
        self.core_disable(Core::SOCSRAM).await; // TODO: is this needed if we reset right after?
        self.core_reset(Core::SOCSRAM).await;

        // this is 4343x specific stuff: Disable remap for SRAM_3
        self.bus.bp_write32(CHIP.socsram_base_address + 0x10, 3).await;
        self.bus.bp_write32(CHIP.socsram_base_address + 0x44, 0).await;

        let ram_addr = CHIP.atcm_ram_base_address;

        debug!("loading fw");
        self.bus.bp_write(ram_addr, firmware).await;

        debug!("loading nvram");
        // Round up to 4 bytes.
        let nvram_len = (NVRAM.len() + 3) / 4 * 4;
        self.bus
            .bp_write(ram_addr + CHIP.chip_ram_size - 4 - nvram_len as u32, NVRAM)
            .await;

        let nvram_len_words = nvram_len as u32 / 4;
        let nvram_len_magic = (!nvram_len_words << 16) | nvram_len_words;
        self.bus
            .bp_write32(ram_addr + CHIP.chip_ram_size - 4, nvram_len_magic)
            .await;

        // Start core!
        debug!("starting up core...");
        self.core_reset(Core::WLAN).await;
        assert!(self.core_is_up(Core::WLAN).await);

        // wait until HT clock is available; takes about 29ms
        debug!("wait for HT clock");
        while self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & 0x80 == 0 {}

        // "Set up the interrupt mask and enable interrupts"
        debug!("setup interrupt mask");
        self.bus
            .bp_write32(CHIP.sdiod_core_base_address + SDIO_INT_HOST_MASK, I_HMB_SW_MASK)
            .await;

        // Set up the interrupt mask and enable interrupts
        if bluetooth_firmware.is_some() {
            debug!("bluetooth setup interrupt mask");
            self.bus
                .bp_write32(CHIP.sdiod_core_base_address + SDIO_INT_HOST_MASK, I_HMB_FC_CHANGE)
                .await;
        }

        // TODO: turn interrupts on here or in bus.init()?
        /*self.bus
        .write16(FUNC_BUS, REG_BUS_INTERRUPT_ENABLE, IRQ_F2_PACKET_AVAILABLE)
        .await;*/

        // "Lower F2 Watermark to avoid DMA Hang in F2 when SD Clock is stopped."
        // Sounds scary...
        self.bus
            .write8(FUNC_BACKPLANE, REG_BACKPLANE_FUNCTION2_WATERMARK, SPI_F2_WATERMARK)
            .await;

        // wait for F2 to be ready
        debug!("waiting for F2 to be ready...");
        while self.bus.read32(FUNC_BUS, REG_BUS_STATUS).await & STATUS_F2_RX_READY == 0 {}

        // Some random configs related to sleep.
        // These aren't needed if we don't want to sleep the bus.
        // TODO do we need to sleep the bus to read the irq line, due to
        // being on the same pin as MOSI/MISO?

        /*
        let mut val = self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_WAKEUP_CTRL).await;
        val |= 0x02; // WAKE_TILL_HT_AVAIL
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_WAKEUP_CTRL, val).await;
        self.bus.write8(FUNC_BUS, 0xF0, 0x08).await; // SDIOD_CCCR_BRCM_CARDCAP.CMD_NODEC = 1
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, 0x02).await; // SBSDIO_FORCE_HT

        let mut val = self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_SLEEP_CSR).await;
        val |= 0x01; // SBSDIO_SLPCSR_KEEP_SDIO_ON
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_SLEEP_CSR, val).await;
         */

        // clear pulls
        debug!("clear pad pulls");
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_PULL_UP, 0).await;
        let _ = self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_PULL_UP).await;

        // start HT clock
        self.bus
            .write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, 0x10)
            .await; // SBSDIO_HT_AVAIL_REQ
        debug!("waiting for HT clock...");
        while self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & 0x80 == 0 {}
        debug!("clock ok");

        #[cfg(feature = "firmware-logs")]
        self.log_init().await;

        debug!("cyw43 runner init done");
    }

    #[cfg(feature = "firmware-logs")]
    async fn log_init(&mut self) {
        // Initialize shared memory for logging.

        let addr = CHIP.atcm_ram_base_address + CHIP.chip_ram_size - 4 - CHIP.socram_srmem_size;
        let shared_addr = self.bus.bp_read32(addr).await;
        debug!("shared_addr {:08x}", shared_addr);

        let mut shared = [0; SharedMemData::SIZE];
        self.bus.bp_read(shared_addr, &mut shared).await;
        let shared = SharedMemData::from_bytes(&shared);

        self.log.addr = shared.console_addr + 8;
    }

    #[cfg(feature = "firmware-logs")]
    async fn log_read(&mut self) {
        // Read log struct
        let mut log = [0; SharedMemLog::SIZE];
        self.bus.bp_read(self.log.addr, &mut log).await;
        let log = SharedMemLog::from_bytes(&log);

        let idx = log.idx as usize;

        // If pointer hasn't moved, no need to do anything.
        if idx == self.log.last_idx {
            return;
        }

        // Read entire buf for now. We could read only what we need, but then we
        // run into annoying alignment issues in `bp_read`.
        let mut buf = [0; 0x400];
        self.bus.bp_read(log.buf, &mut buf).await;

        while self.log.last_idx != idx as usize {
            let b = buf[self.log.last_idx];
            if b == b'\r' || b == b'\n' {
                if self.log.buf_count != 0 {
                    let s = unsafe { core::str::from_utf8_unchecked(&self.log.buf[..self.log.buf_count]) };
                    debug!("LOGS: {}", s);
                    self.log.buf_count = 0;
                }
            } else if self.log.buf_count < self.log.buf.len() {
                self.log.buf[self.log.buf_count] = b;
                self.log.buf_count += 1;
            }

            self.log.last_idx += 1;
            if self.log.last_idx == 0x400 {
                self.log.last_idx = 0;
            }
        }
    }

    pub(crate) async fn init_bluetooth(&mut self, firmware: &[u8]) {
        debug!("init_bluetooth");
        self.bus
            .bp_write32(CHIP.bluetooth_base_address + BT2WLAN_PWRUP_ADDR, BT2WLAN_PWRUP_WAKE)
            .await;
        Timer::after(Duration::from_millis(2)).await;
        self.upload_bluetooth_firmware(firmware).await;
        self.wait_bt_ready().await;
        self.init_bt_buffers().await;
        self.wait_bt_awake().await;
        self.bt_set_host_ready().await;
        self.bt_toggle_intr().await;
    }

    pub(crate) async fn upload_bluetooth_firmware(&mut self, firmware: &[u8]) {
        // read version
        let version_length = firmware[0];
        let _version = &firmware[1..=version_length as usize];
        // skip version + 1 extra byte as per cybt_shared_bus_driver.c
        let firmware = &firmware[version_length as usize + 2..];
        // buffers
        let mut data_buffer: [u8; 0x100] = [0; 0x100];
        let mut aligned_data_buffer: [u8; 0x100] = [0; 0x100];
        // structs
        let mut btfw_cb = CybtFwCb {
            p_next_line_start: firmware,
        };
        let mut hfd = HexFileData {
            addr_mode: BTFW_ADDR_MODE_EXTENDED,
            hi_addr: 0,
            dest_addr: 0,
            p_ds: &mut data_buffer,
        };
        loop {
            let num_fw_bytes = bluetooth::read_firmware_patch_line(&mut btfw_cb, &mut hfd);
            if num_fw_bytes == 0 {
                break;
            }
            let fw_bytes = &hfd.p_ds[0..num_fw_bytes as usize];
            let mut dest_start_addr = hfd.dest_addr + CHIP.bluetooth_base_address;
            let mut aligned_data_buffer_index: usize = 0;
            // pad start
            if utilities::is_aligned(dest_start_addr, 4) {
                let num_pad_bytes = dest_start_addr % 4;
                let padded_dest_start_addr = utilities::round_down(dest_start_addr, 4);
                let memory_value = self.bus.bp_read32(padded_dest_start_addr).await;
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
            if !utilities::is_aligned(dest_end_addr, 4) {
                let offset = dest_end_addr % 4;
                let num_pad_bytes_end = 4 - offset;
                let padded_dest_end_addr = utilities::round_down(dest_end_addr, 4);
                let memory_value = self.bus.bp_read32(padded_dest_end_addr).await;
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
                self.bus.bp_write(dest_start_addr + (offset as u32), chunk).await;
            }
            // sleep TODO: is this needed
            Timer::after(Duration::from_millis(1)).await;
        }
    }

    pub(crate) async fn wait_bt_ready(&mut self) {
        debug!("wait_bt_ready");
        let mut success = false;
        for _ in 0..300 {
            let val = self.bus.bp_read32(BT_CTRL_REG_ADDR).await;
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

    pub(crate) async fn wait_bt_awake(&mut self) {
        debug!("wait_bt_awake");
        let mut success = false;
        for _ in 0..300 {
            let val = self.bus.bp_read32(BT_CTRL_REG_ADDR).await;
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

    pub(crate) async fn bt_set_host_ready(&mut self) {
        debug!("bt_set_host_ready");
        let old_val = self.bus.bp_read32(HOST_CTRL_REG_ADDR).await;
        // TODO: do we need to swap endianness on this read?
        let new_val = old_val | BTSDIO_REG_SW_RDY_BITMASK;
        self.bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
    }

    // TODO: use this
    #[allow(dead_code)]
    pub(crate) async fn bt_set_awake(&mut self, awake: bool) {
        debug!("bt_set_awake");
        let old_val = self.bus.bp_read32(HOST_CTRL_REG_ADDR).await;
        // TODO: do we need to swap endianness on this read?
        let new_val = if awake {
            old_val | BTSDIO_REG_WAKE_BT_BITMASK
        } else {
            old_val & !BTSDIO_REG_WAKE_BT_BITMASK
        };
        self.bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
    }

    pub(crate) async fn bt_toggle_intr(&mut self) {
        debug!("bt_toggle_intr");
        let old_val = self.bus.bp_read32(HOST_CTRL_REG_ADDR).await;
        // TODO: do we need to swap endianness on this read?
        let new_val = old_val ^ BTSDIO_REG_DATA_VALID_BITMASK;
        self.bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
    }

    // TODO: use this
    #[allow(dead_code)]
    pub(crate) async fn bt_set_intr(&mut self) {
        debug!("bt_set_intr");
        let old_val = self.bus.bp_read32(HOST_CTRL_REG_ADDR).await;
        let new_val = old_val | BTSDIO_REG_DATA_VALID_BITMASK;
        self.bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
    }

    pub(crate) async fn init_bt_buffers(&mut self) {
        debug!("init_bt_buffers");
        let wlan_ram_base_addr = self.bus.bp_read32(WLAN_RAM_BASE_REG_ADDR).await;
        assert!(wlan_ram_base_addr != 0);
        debug!("wlan_ram_base_addr = {:08x}", wlan_ram_base_addr);
        self.h2b_buf_addr = wlan_ram_base_addr + BTSDIO_OFFSET_HOST_WRITE_BUF;
        self.b2h_buf_addr = wlan_ram_base_addr + BTSDIO_OFFSET_HOST_READ_BUF;
        let h2b_buf_in_addr = wlan_ram_base_addr + BTSDIO_OFFSET_HOST2BT_IN;
        let h2b_buf_out_addr = wlan_ram_base_addr + BTSDIO_OFFSET_HOST2BT_OUT;
        let b2h_buf_in_addr = wlan_ram_base_addr + BTSDIO_OFFSET_BT2HOST_IN;
        let b2h_buf_out_addr = wlan_ram_base_addr + BTSDIO_OFFSET_BT2HOST_OUT;
        self.bus.bp_write32(h2b_buf_in_addr, 0).await;
        self.bus.bp_write32(h2b_buf_out_addr, 0).await;
        self.bus.bp_write32(b2h_buf_in_addr, 0).await;
        self.bus.bp_write32(b2h_buf_out_addr, 0).await;
    }

    async fn bt_bus_request(&mut self) {
        // TODO: CYW43_THREAD_ENTER mutex?
        self.bt_set_awake(true).await;
        self.wait_bt_awake().await;
    }

    async fn bt_bus_release(&mut self) {
        // TODO: CYW43_THREAD_EXIT mutex?
    }

    #[allow(dead_code)]
    pub(crate) async fn hci_read(&mut self, buf: &mut [u8]) -> u32 {
        debug!("hci_read buf = {:02x}", buf);
        self.bt_bus_request().await;
        let mut header = [0 as u8; 4];
        self.bus
            .bp_read(self.b2h_buf_addr + self.b2h_buf_addr_pointer, &mut header)
            .await;
        self.b2h_buf_addr_pointer += header.len() as u32;
        debug!("hci_read heaer = {:02x}", header);
        // cybt_get_bt_buf_index(&fw_membuf_info);
        // fw_b2h_buf_count = CIRC_BUF_CNT(fw_membuf_info.bt2host_in_val, fw_membuf_info.bt2host_out_val);
        // cybt_mem_read_idx(B2H_BUF_ADDR_IDX, fw_membuf_info.bt2host_out_val, p_data, read_len);
        // cybt_mem_read_idx(B2H_BUF_ADDR_IDX, 0, p_data + first_read_len, second_read_len);
        // cybt_reg_write_idx(B2H_BUF_OUT_ADDR_IDX, new_b2h_out_val);
        self.bt_toggle_intr().await;
        let bytes_read = 0;
        self.bt_bus_release().await;
        return bytes_read;
    }

    #[allow(dead_code)]
    pub(crate) async fn hci_write(&mut self, buf: &[u8]) {
        let buf_len = buf.len();
        let algined_buf_len = utilities::round_up((buf_len + 3) as u32, 4);
        assert!(buf_len <= algined_buf_len as usize);
        let cmd_len = buf_len + 3 - 4; // add 3 bytes for SDIO header thingie?
        let mut buf_with_cmd = [0 as u8; 0x100];
        buf_with_cmd[0] = (cmd_len & 0xFF) as u8;
        buf_with_cmd[1] = ((cmd_len & 0xFF00) >> 8) as u8;
        buf_with_cmd[2] = 0x00;
        for i in 0..buf_len {
            buf_with_cmd[3 + i] = buf[i];
        }
        let padded_buf_with_cmd = &buf_with_cmd[0..algined_buf_len as usize];
        debug!("hci_write padded_buf_with_cmd = {:02x}", padded_buf_with_cmd);
        self.bt_bus_request().await;
        self.bus
            .bp_write(self.h2b_buf_addr + self.h2b_buf_addr_pointer, &padded_buf_with_cmd)
            .await;
        self.h2b_buf_addr_pointer += padded_buf_with_cmd.len() as u32;
        // TODO: handle wrapping based on BTSDIO_FWBUF_SIZE
        self.bt_toggle_intr().await;
        self.bt_bus_release().await;
    }

    pub(crate) async fn bt_has_work(&mut self) -> bool {
        let int_status = self.bus.bp_read32(CHIP.sdiod_core_base_address + SDIO_INT_STATUS).await;
        if int_status & I_HMB_FC_CHANGE != 0 {
            self.bus
                .bp_write32(
                    CHIP.sdiod_core_base_address + SDIO_INT_STATUS,
                    int_status & I_HMB_FC_CHANGE,
                )
                .await;
            return true;
        }
        return false;
    }

    pub async fn run(mut self) -> ! {
        let mut buf = [0; 512];
        loop {
            #[cfg(feature = "firmware-logs")]
            self.log_read().await;

            if self.has_credit() {
                let ioctl = self.ioctl_state.wait_pending();
                let tx = self.ch.tx_buf();
                let ev = self.bus.wait_for_event();

                match select3(ioctl, tx, ev).await {
                    Either3::First(PendingIoctl {
                        buf: iobuf,
                        kind,
                        cmd,
                        iface,
                    }) => {
                        self.send_ioctl(kind, cmd, iface, unsafe { &*iobuf }).await;
                        self.check_status(&mut buf).await;
                    }
                    Either3::Second(packet) => {
                        trace!("tx pkt {:02x}", Bytes(&packet[..packet.len().min(48)]));

                        let mut buf = [0; 512];
                        let buf8 = utilities::slice8_mut(&mut buf);

                        // There MUST be 2 bytes of padding between the SDPCM and BDC headers.
                        // And ONLY for data packets!
                        // No idea why, but the firmware will append two zero bytes to the tx'd packets
                        // otherwise. If the packet is exactly 1514 bytes (the max MTU), this makes it
                        // be oversized and get dropped.
                        // WHD adds it here https://github.com/Infineon/wifi-host-driver/blob/c04fcbb6b0d049304f376cf483fd7b1b570c8cd5/WiFi_Host_Driver/src/include/whd_sdpcm.h#L90
                        // and adds it to the header size her https://github.com/Infineon/wifi-host-driver/blob/c04fcbb6b0d049304f376cf483fd7b1b570c8cd5/WiFi_Host_Driver/src/whd_sdpcm.c#L597
                        // ¯\_(ツ)_/¯
                        const PADDING_SIZE: usize = 2;
                        let total_len = SdpcmHeader::SIZE + PADDING_SIZE + BdcHeader::SIZE + packet.len();

                        let seq = self.sdpcm_seq;
                        self.sdpcm_seq = self.sdpcm_seq.wrapping_add(1);

                        let sdpcm_header = SdpcmHeader {
                            len: total_len as u16, // TODO does this len need to be rounded up to u32?
                            len_inv: !total_len as u16,
                            sequence: seq,
                            channel_and_flags: CHANNEL_TYPE_DATA,
                            next_length: 0,
                            header_length: (SdpcmHeader::SIZE + PADDING_SIZE) as _,
                            wireless_flow_control: 0,
                            bus_data_credit: 0,
                            reserved: [0, 0],
                        };

                        let bdc_header = BdcHeader {
                            flags: BDC_VERSION << BDC_VERSION_SHIFT,
                            priority: 0,
                            flags2: 0,
                            data_offset: 0,
                        };
                        trace!("tx {:?}", sdpcm_header);
                        trace!("    {:?}", bdc_header);

                        buf8[0..SdpcmHeader::SIZE].copy_from_slice(&sdpcm_header.to_bytes());
                        buf8[SdpcmHeader::SIZE + PADDING_SIZE..][..BdcHeader::SIZE]
                            .copy_from_slice(&bdc_header.to_bytes());
                        buf8[SdpcmHeader::SIZE + PADDING_SIZE + BdcHeader::SIZE..][..packet.len()]
                            .copy_from_slice(packet);

                        let total_len = (total_len + 3) & !3; // round up to 4byte

                        trace!("    {:02x}", Bytes(&buf8[..total_len.min(48)]));

                        self.bus.wlan_write(&buf[..(total_len / 4)]).await;
                        self.ch.tx_done();
                        self.check_status(&mut buf).await;
                    }
                    Either3::Third(()) => {
                        self.handle_irq(&mut buf).await;
                    }
                }
            } else {
                warn!("TX stalled");
                self.bus.wait_for_event().await;
                self.handle_irq(&mut buf).await;
            }
        }
    }

    /// Wait for IRQ on F2 packet available
    async fn handle_irq(&mut self, buf: &mut [u32; 512]) {
        // Receive stuff
        let irq = self.bus.read16(FUNC_BUS, REG_BUS_INTERRUPT).await;
        trace!("irq{}", FormatInterrupt(irq));

        if irq & IRQ_F2_PACKET_AVAILABLE != 0 {
            self.check_status(buf).await;
        }

        if irq & IRQ_DATA_UNAVAILABLE != 0 {
            // TODO what should we do here?
            warn!("IRQ DATA_UNAVAILABLE, clearing...");
            self.bus.write16(FUNC_BUS, REG_BUS_INTERRUPT, 1).await;
        }
    }

    /// Handle F2 events while status register is set
    async fn check_status(&mut self, buf: &mut [u32; 512]) {
        loop {
            let status = self.bus.status();
            trace!("check status{}", FormatStatus(status));

            if status & STATUS_F2_PKT_AVAILABLE != 0 {
                let len = (status & STATUS_F2_PKT_LEN_MASK) >> STATUS_F2_PKT_LEN_SHIFT;
                self.bus.wlan_read(buf, len).await;
                trace!(
                    "rx {:02x}",
                    Bytes(&utilities::slice8_mut(buf)[..(len as usize).min(48)])
                );
                self.rx(&mut utilities::slice8_mut(buf)[..len as usize]);
            } else {
                break;
            }
        }
    }

    fn rx(&mut self, packet: &mut [u8]) {
        let Some((sdpcm_header, payload)) = SdpcmHeader::parse(packet) else {
            return;
        };

        self.update_credit(&sdpcm_header);

        let channel = sdpcm_header.channel_and_flags & 0x0f;

        match channel {
            CHANNEL_TYPE_CONTROL => {
                let Some((cdc_header, response)) = CdcHeader::parse(payload) else {
                    return;
                };
                trace!("    {:?}", cdc_header);

                if cdc_header.id == self.ioctl_id {
                    if cdc_header.status != 0 {
                        // TODO: propagate error instead
                        panic!("IOCTL error {}", cdc_header.status as i32);
                    }

                    self.ioctl_state.ioctl_done(response);
                }
            }
            CHANNEL_TYPE_EVENT => {
                let Some((_, bdc_packet)) = BdcHeader::parse(payload) else {
                    warn!("BDC event, incomplete header");
                    return;
                };

                let Some((event_packet, evt_data)) = EventPacket::parse(bdc_packet) else {
                    warn!("BDC event, incomplete data");
                    return;
                };

                const ETH_P_LINK_CTL: u16 = 0x886c; // HPNA, wlan link local tunnel, according to linux if_ether.h
                if event_packet.eth.ether_type != ETH_P_LINK_CTL {
                    warn!(
                        "unexpected ethernet type 0x{:04x}, expected Broadcom ether type 0x{:04x}",
                        event_packet.eth.ether_type, ETH_P_LINK_CTL
                    );
                    return;
                }
                const BROADCOM_OUI: &[u8] = &[0x00, 0x10, 0x18];
                if event_packet.hdr.oui != BROADCOM_OUI {
                    warn!(
                        "unexpected ethernet OUI {:02x}, expected Broadcom OUI {:02x}",
                        Bytes(&event_packet.hdr.oui),
                        Bytes(BROADCOM_OUI)
                    );
                    return;
                }
                const BCMILCP_SUBTYPE_VENDOR_LONG: u16 = 32769;
                if event_packet.hdr.subtype != BCMILCP_SUBTYPE_VENDOR_LONG {
                    warn!("unexpected subtype {}", event_packet.hdr.subtype);
                    return;
                }

                const BCMILCP_BCM_SUBTYPE_EVENT: u16 = 1;
                if event_packet.hdr.user_subtype != BCMILCP_BCM_SUBTYPE_EVENT {
                    warn!("unexpected user_subtype {}", event_packet.hdr.subtype);
                    return;
                }

                let evt_type = events::Event::from(event_packet.msg.event_type as u8);
                debug!(
                    "=== EVENT {:?}: {:?} {:02x}",
                    evt_type,
                    event_packet.msg,
                    Bytes(evt_data)
                );

                if self.events.mask.is_enabled(evt_type) {
                    let status = event_packet.msg.status;
                    let event_payload = match evt_type {
                        Event::ESCAN_RESULT if status == EStatus::PARTIAL => {
                            let Some((_, bss_info)) = ScanResults::parse(evt_data) else {
                                return;
                            };
                            let Some(bss_info) = BssInfo::parse(bss_info) else {
                                return;
                            };
                            events::Payload::BssInfo(*bss_info)
                        }
                        Event::ESCAN_RESULT => events::Payload::None,
                        _ => events::Payload::None,
                    };

                    // this intentionally uses the non-blocking publish immediate
                    // publish() is a deadlock risk in the current design as awaiting here prevents ioctls
                    // The `Runner` always yields when accessing the device, so consumers always have a chance to receive the event
                    // (if they are actively awaiting the queue)
                    self.events.queue.publish_immediate(events::Message::new(
                        Status {
                            event_type: evt_type,
                            status,
                        },
                        event_payload,
                    ));
                }
            }
            CHANNEL_TYPE_DATA => {
                let Some((_, packet)) = BdcHeader::parse(payload) else {
                    return;
                };
                trace!("rx pkt {:02x}", Bytes(&packet[..packet.len().min(48)]));

                match self.ch.try_rx_buf() {
                    Some(buf) => {
                        buf[..packet.len()].copy_from_slice(packet);
                        self.ch.rx_done(packet.len())
                    }
                    None => warn!("failed to push rxd packet to the channel."),
                }
            }
            _ => {}
        }
    }

    fn update_credit(&mut self, sdpcm_header: &SdpcmHeader) {
        if sdpcm_header.channel_and_flags & 0xf < 3 {
            let mut sdpcm_seq_max = sdpcm_header.bus_data_credit;
            if sdpcm_seq_max.wrapping_sub(self.sdpcm_seq) > 0x40 {
                sdpcm_seq_max = self.sdpcm_seq + 2;
            }
            self.sdpcm_seq_max = sdpcm_seq_max;
        }
    }

    fn has_credit(&self) -> bool {
        self.sdpcm_seq != self.sdpcm_seq_max && self.sdpcm_seq_max.wrapping_sub(self.sdpcm_seq) & 0x80 == 0
    }

    async fn send_ioctl(&mut self, kind: IoctlType, cmd: u32, iface: u32, data: &[u8]) {
        let mut buf = [0; 512];
        let buf8 = utilities::slice8_mut(&mut buf);

        let total_len = SdpcmHeader::SIZE + CdcHeader::SIZE + data.len();

        let sdpcm_seq = self.sdpcm_seq;
        self.sdpcm_seq = self.sdpcm_seq.wrapping_add(1);
        self.ioctl_id = self.ioctl_id.wrapping_add(1);

        let sdpcm_header = SdpcmHeader {
            len: total_len as u16, // TODO does this len need to be rounded up to u32?
            len_inv: !total_len as u16,
            sequence: sdpcm_seq,
            channel_and_flags: CHANNEL_TYPE_CONTROL,
            next_length: 0,
            header_length: SdpcmHeader::SIZE as _,
            wireless_flow_control: 0,
            bus_data_credit: 0,
            reserved: [0, 0],
        };

        let cdc_header = CdcHeader {
            cmd: cmd,
            len: data.len() as _,
            flags: kind as u16 | (iface as u16) << 12,
            id: self.ioctl_id,
            status: 0,
        };
        trace!("tx {:?}", sdpcm_header);
        trace!("    {:?}", cdc_header);

        buf8[0..SdpcmHeader::SIZE].copy_from_slice(&sdpcm_header.to_bytes());
        buf8[SdpcmHeader::SIZE..][..CdcHeader::SIZE].copy_from_slice(&cdc_header.to_bytes());
        buf8[SdpcmHeader::SIZE + CdcHeader::SIZE..][..data.len()].copy_from_slice(data);

        let total_len = (total_len + 3) & !3; // round up to 4byte

        trace!("    {:02x}", Bytes(&buf8[..total_len.min(48)]));

        self.bus.wlan_write(&buf[..total_len / 4]).await;
    }

    async fn core_disable(&mut self, core: Core) {
        let base = core.base_addr();

        // Dummy read?
        let _ = self.bus.bp_read8(base + AI_RESETCTRL_OFFSET).await;

        // Check it isn't already reset
        let r = self.bus.bp_read8(base + AI_RESETCTRL_OFFSET).await;
        if r & AI_RESETCTRL_BIT_RESET != 0 {
            return;
        }

        self.bus.bp_write8(base + AI_IOCTRL_OFFSET, 0).await;
        let _ = self.bus.bp_read8(base + AI_IOCTRL_OFFSET).await;

        block_for(Duration::from_millis(1));

        self.bus
            .bp_write8(base + AI_RESETCTRL_OFFSET, AI_RESETCTRL_BIT_RESET)
            .await;
        let _ = self.bus.bp_read8(base + AI_RESETCTRL_OFFSET).await;
    }

    async fn core_reset(&mut self, core: Core) {
        self.core_disable(core).await;

        let base = core.base_addr();
        self.bus
            .bp_write8(base + AI_IOCTRL_OFFSET, AI_IOCTRL_BIT_FGC | AI_IOCTRL_BIT_CLOCK_EN)
            .await;
        let _ = self.bus.bp_read8(base + AI_IOCTRL_OFFSET).await;

        self.bus.bp_write8(base + AI_RESETCTRL_OFFSET, 0).await;

        Timer::after(Duration::from_millis(1)).await;

        self.bus
            .bp_write8(base + AI_IOCTRL_OFFSET, AI_IOCTRL_BIT_CLOCK_EN)
            .await;
        let _ = self.bus.bp_read8(base + AI_IOCTRL_OFFSET).await;

        Timer::after(Duration::from_millis(1)).await;
    }

    async fn core_is_up(&mut self, core: Core) -> bool {
        let base = core.base_addr();

        let io = self.bus.bp_read8(base + AI_IOCTRL_OFFSET).await;
        if io & (AI_IOCTRL_BIT_FGC | AI_IOCTRL_BIT_CLOCK_EN) != AI_IOCTRL_BIT_CLOCK_EN {
            debug!("core_is_up: returning false due to bad ioctrl {:02x}", io);
            return false;
        }

        let r = self.bus.bp_read8(base + AI_RESETCTRL_OFFSET).await;
        if r & (AI_RESETCTRL_BIT_RESET) != 0 {
            debug!("core_is_up: returning false due to bad resetctrl {:02x}", r);
            return false;
        }

        true
    }
}
