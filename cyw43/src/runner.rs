use embassy_futures::select::{select3, Either3};
use embassy_net_driver_channel as ch;
use embassy_sync::pubsub::PubSubBehavior;
use embassy_time::{block_for, Duration, Timer};
use embedded_hal_1::digital::OutputPin;

use crate::bus::Bus;
use crate::bluetooth::{self, CybtFwCb, HexFileData};
pub use crate::bus::SpiBusCyw43;
use crate::consts::*;
use crate::events::{Event, Events, Status};
use crate::fmt::Bytes;
use crate::ioctl::{IoctlState, IoctlType, PendingIoctl};
use crate::nvram::NVRAM;
use crate::structs::*;
use crate::{events, slice8_mut, Core, CHIP, MTU};

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
        }
    }

    async fn upload_bluetooth_firmware(&mut self, firmware: &[u8]) {
        // read version
        let version_length = firmware[0];
        let _version = &firmware[1..=version_length as usize];
        // skip version + 1 extra byte as per cybt_shared_bus_driver.c
        let firmware = &firmware[version_length as usize + 2..];
        // buffer
        let mut data_buffer: [u8; 0x100] = [0; 0x100]; 
        let mut aligned_data_buffer: [u8; 0x100] = [0; 0x100]; 
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
            let num_fw_bytes = bluetooth::cybt_fw_get_data(&mut btfw_cb, &mut hfd);
            if num_fw_bytes == 0 {
                break;
            }
            let fw_bytes = &hfd.p_ds[0..num_fw_bytes as usize];
            let ori_dest_start_addr = hfd.dest_addr + CHIP.bluetooth_base_address;
            let mut dest_start_addr = hfd.dest_addr + CHIP.bluetooth_base_address;
            let mut aligned_data_buffer_index: usize = 0;
            debug!("upload_bluetooth_firmware: pre-alignment: dest_start_addr = {:08x} aligned_data_buffer_index = {} fw_bytes = {:02x}", dest_start_addr, aligned_data_buffer_index, fw_bytes);
            // pad start
            if !bluetooth::is_aligned(dest_start_addr, 4) {
                let num_pad_bytes = dest_start_addr % 4;
                let padded_dest_start_addr = bluetooth::round_down(dest_start_addr, 4);
                let mut memory_value_bytes = [0; 4];
                self.bus.bp_read(padded_dest_start_addr, &mut memory_value_bytes).await;
                debug!("upload_bluetooth_firmware: pad start ori_dest_start_addr = {:08x} bp_read({:08x}) memory_value_bytes = {:02x}", ori_dest_start_addr, padded_dest_start_addr, memory_value_bytes);
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
                debug!("upload_bluetooth_firmware: pad start alignment needed: dest_start_addr = {:08x} padded_dest_start_addr = {:08x} aligned_data_buffer_index = {} aligned_data_buffer = {:02x} num_pad_bytes = {}", dest_start_addr, padded_dest_start_addr, aligned_data_buffer_index, aligned_data_buffer, num_pad_bytes);
                dest_start_addr = padded_dest_start_addr;
            } else {
                // Directly copy fw_bytes into aligned_data_buffer if no start padding is required
                for i in 0..num_fw_bytes as usize {
                    aligned_data_buffer[aligned_data_buffer_index] = fw_bytes[i];
                    aligned_data_buffer_index += 1;
                }
                debug!("upload_bluetooth_firmware: pad start alignment not needed: dest_start_addr = {:08x} aligned_data_buffer_index = {} aligned_data_buffer = {:02x}", dest_start_addr, aligned_data_buffer_index, aligned_data_buffer);
            }
            // pad end
            let mut dest_end_addr = dest_start_addr + aligned_data_buffer_index as u32;
            if !bluetooth::is_aligned(dest_end_addr, 4) {
                let offset = dest_end_addr % 4;
                let num_pad_bytes_end = 4 - offset;
                let padded_dest_end_addr = bluetooth::round_down(dest_end_addr, 4);
                let mut memory_value_bytes = [0; 4];
                self.bus.bp_read(padded_dest_end_addr, &mut memory_value_bytes).await;
                debug!("upload_bluetooth_firmware: pad end ori_dest_start_addr = {:08x} bp_read({:08x}) memory_value_bytes = {:02x}", ori_dest_start_addr, padded_dest_end_addr, memory_value_bytes);
                // Append the necessary memory bytes to pad the end of aligned_data_buffer
                debug!("upload_bluetooth_firmware: pad end alignment needed: dest_end_addr = {:08x} padded_dest_end_addr = {:08x} offset = {} num_pad_bytes_end = {} aligned_data_buffer_index = {}", dest_end_addr, padded_dest_end_addr, offset, num_pad_bytes_end, aligned_data_buffer_index);
                for i in offset..4 {
                    aligned_data_buffer[aligned_data_buffer_index] = memory_value_bytes[i as usize];
                    aligned_data_buffer_index += 1;
                }
                debug!("upload_bluetooth_firmware: pad end alignment needed: dest_start_addr = {:08x} dest_end_addr = {:08x} padded_dest_end_addr = {:08x} aligned_data_buffer_index = {} num_pad_bytes_end = {} offset = {} aligned_data_buffer = {:02x}", dest_start_addr, dest_end_addr, padded_dest_end_addr, aligned_data_buffer_index, num_pad_bytes_end, offset, aligned_data_buffer);
                dest_end_addr += num_pad_bytes_end;
            } else {
                debug!("upload_bluetooth_firmware: pad end alignment not needed: dest_start_addr = {:08x} dest_end_addr = {:08x} aligned_data_buffer_index = {} aligned_data_buffer = {:02x}", dest_start_addr, dest_end_addr, aligned_data_buffer_index, aligned_data_buffer);
            }
            let buffer_to_write = &aligned_data_buffer[0..aligned_data_buffer_index as usize];
            debug!("upload_bluetooth_firmware: dest_start_addr = {:x} dest_end_addr = {:x} aligned_data_buffer_index = {} buffer_to_write = {:02x}", dest_start_addr, dest_end_addr, aligned_data_buffer_index, buffer_to_write);
            assert!(dest_start_addr % 4 == 0);
            assert!(dest_end_addr % 4 == 0);
            assert!(aligned_data_buffer_index % 4 == 0);
            self.bus.bp_write(dest_start_addr, buffer_to_write).await;
        }
    }

    async fn wait_bt_ready(&mut self) {
        debug!("wait_bt_ready");
        let mut success = false;
        for _ in 0..300 {
            let val = self.bus.bp_read32(BT_CTRL_REG_ADDR).await;
            // TODO: do we need to swap endianness on this read?
            if val != 0 {
                debug!("BT_CTRL_REG_ADDR = {:08x}", val);
            }
            /*if val & BTSDIO_REG_FW_RDY_BITMASK != 0 {
                break;
            }*/
            // TODO: should be 00000000 until it is 0x01000100
            if val == 0x01000100 {
                success = true;
                break;
            }
        }
        assert!(success == true);
    }

    async fn wait_bt_awake(&mut self) {
        debug!("wait_bt_awake");        
        loop {
            let val = self.bus.bp_read32(BT_CTRL_REG_ADDR).await;
            // TODO: do we need to swap endianness on this read?
            debug!("BT_CTRL_REG_ADDR = {:08x}", val);
            if val & BTSDIO_REG_BT_AWAKE_BITMASK != 0 {
                break;
            }
            Timer::after(Duration::from_millis(100)).await;
        }
    }

    async fn bt_set_host_ready(&mut self) {
        debug!("bt_set_host_ready");        
        let old_val = self.bus.bp_read32(HOST_CTRL_REG_ADDR).await;
        // TODO: do we need to swap endianness on this read?
        let new_val = old_val | BTSDIO_REG_SW_RDY_BITMASK;
        self.bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
    }

    async fn bt_set_awake(&mut self) {
        debug!("bt_set_awake");        
        let old_val = self.bus.bp_read32(HOST_CTRL_REG_ADDR).await;
        // TODO: do we need to swap endianness on this read?
        let new_val = old_val | BTSDIO_REG_WAKE_BT_BITMASK;
        self.bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
    }

    async fn bt_toggle_intr(&mut self) {
        debug!("bt_toggle_intr");        
        let old_val = self.bus.bp_read32(HOST_CTRL_REG_ADDR).await;
        // TODO: do we need to swap endianness on this read?
        let new_val = old_val ^ BTSDIO_REG_DATA_VALID_BITMASK;
        self.bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
    }

    async fn bt_set_intr(&mut self) {
        debug!("bt_set_intr");        
        let old_val = self.bus.bp_read32(HOST_CTRL_REG_ADDR).await;
        let new_val = old_val | BTSDIO_REG_DATA_VALID_BITMASK;
        self.bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
    }

    async fn validate_firmware(&mut self, ram_addr: u32, firmware: &[u8]) {
        let mut i = 0;
        let mut mem_bytes = [0; 0x100];
        while i < firmware.len() {
            debug!("i = {:08x}", i);
    
            self.bus.bp_read((ram_addr as usize + i) as u32, &mut mem_bytes).await;
    
            let slice_len = if 0x100 <= firmware.len() - i { 0x100 } else { firmware.len() - i };
            let firmware_slice = &firmware[i..i+slice_len];
            let mem_slice = &mem_bytes[..slice_len];

            assert_eq!(firmware_slice.len(), mem_slice.len());
            for j in 0..firmware_slice.len() {
                if firmware_slice[j] != mem_slice[j] {
                    // TODO: assert
                    debug!("{:08x} firmware_slice[{}] != mem_slice[{}] {:02x} != {:02x}", ram_addr as usize + i + j, j, j, firmware_slice[j], mem_slice[j]);
                }
            }
    
            i += slice_len;
        }
    }
    
    pub async fn init(&mut self, firmware: &[u8], bluetooth_firmware: &[u8]) {
        debug!("runner init");

        self.bus.init().await;

        // Init ALP (Active Low Power) clock
        debug!("init alp");
        self.bus
            .write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, BACKPLANE_ALP_AVAIL_REQ)
            .await;
        
        // check we can set the bluetooth watermark
        debug!("set bluetooth watermark");
        self.bus
            .write8(FUNC_BACKPLANE, REG_BACKPLANE_FUNCTION2_WATERMARK, 0x10)
            .await;
        let watermark = self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_FUNCTION2_WATERMARK).await;
        debug!("watermark = {:02x}", watermark);
        assert!(watermark == 0x10);
        
        debug!("waiting for clock...");
        while self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & BACKPLANE_ALP_AVAIL == 0 {}
        debug!("clock ok");

        // clear request for ALP
        debug!("clear request for ALP");
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, 0).await;

        debug!("read chip id");
        let chip_id = self.bus.bp_read16(0x1800_0000).await;
        debug!("chip ID: {}", chip_id);

        // Upload firmware.
        self.core_disable(Core::WLAN).await;
        self.core_disable(Core::SOCSRAM).await;
        self.core_reset(Core::SOCSRAM).await;

        // this is 4343x specific stuff: Disable remap for SRAM_3
        self.bus.bp_write32(CHIP.socsram_base_address + 0x10, 3).await;
        self.bus.bp_write32(CHIP.socsram_base_address + 0x44, 0).await;

        let ram_addr = CHIP.atcm_ram_base_address;

        debug!("loading fw");
        self.bus.bp_write(ram_addr, firmware).await;
        self.validate_firmware(ram_addr, firmware).await;

        // TODO: load bluetooth firmware here or not? C does it after CLM is loaded
        //self.init_bluetooth(bluetooth_firmware).await;

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
        self.bus.bp_write32(CHIP.sdiod_core_base_address + SDIO_INT_HOST_MASK, I_HMB_SW_MASK).await;

        // Set up the interrupt mask and enable interrupts
        debug!("bluetooth setup interrupt mask");
        self.bus.bp_write32(CHIP.sdiod_core_base_address + SDIO_INT_HOST_MASK, I_HMB_FC_CHANGE).await;

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
        /*let mut val = self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_WAKEUP_CTRL).await;
        val |= 0x02; // WAKE_TILL_HT_AVAIL
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_WAKEUP_CTRL, val).await;
        self.bus.write8(FUNC_BUS, 0xF0, 0x08).await; // SDIOD_CCCR_BRCM_CARDCAP.CMD_NODEC = 1
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, 0x02).await; // SBSDIO_FORCE_HT

        let mut val = self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_SLEEP_CSR).await;
        val |= 0x01; // SBSDIO_SLPCSR_KEEP_SDIO_ON
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_SLEEP_CSR, val).await;*/

        // clear pulls
        debug!("clear pad pulls");
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_PULL_UP, 0).await;
        let _ = self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_PULL_UP).await;

        // start HT clock
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, 0x10).await; // SBSDIO_HT_AVAIL_REQ
        debug!("waiting for HT clock...");
        while self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & 0x80 == 0 {}
        debug!("clock ok");

        // TODO: We always seem to start with a data unavailable error - so clear it now
        // uint16_t spi_int_status = cyw43_read_reg_u16(self, BUS_FUNCTION, SPI_INTERRUPT_REGISTER);
        // if (spi_int_status & DATA_UNAVAILABLE) {
        //    cyw43_write_reg_u16(self, BUS_FUNCTION, SPI_INTERRUPT_REGISTER, spi_int_status);
        //}

        // TODO: cyw43_ll_bus_sleep(self_in, false);

        #[cfg(feature = "firmware-logs")]
        self.log_init().await;

        debug!("wifi init done");
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

    pub async fn init_bluetooth(&mut self, bt_firmware: &[u8]) {
        self.bus.bp_write32(CHIP.bluetooth_base_address + BT2WLAN_PWRUP_ADDR, BT2WLAN_PWRUP_WAKE).await;
        self.upload_bluetooth_firmware(bt_firmware).await;
        self.wait_bt_ready().await;
        // TODO: cybt_init_buffer();
        self.wait_bt_awake().await;
        self.bt_set_host_ready().await;
        self.bt_toggle_intr().await;
    }

    pub async fn run(mut self, bluetooth_firmware: &[u8]) -> ! {
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
                        debug!("PendingIoctl cmd = {:08x} buf = {:02x}", cmd, buf);
                        if cmd == 0xFFFFFFFF { // fake command i came up with to do bluetooth init?
                            debug!("PendingIoctl init_bluetooth");
                            self.init_bluetooth(bluetooth_firmware).await;
                        } else {
                            self.send_ioctl(kind, cmd, iface, unsafe { &*iobuf }).await;
                            self.check_status(&mut buf).await;
                        }
                    }
                    Either3::Second(packet) => {
                        trace!("tx pkt {:02x}", Bytes(&packet[..packet.len().min(48)]));
                        // TODO?
                    }
                    /*Either3::Second(packet) => {
                        trace!("tx pkt {:02x}", Bytes(&packet[..packet.len().min(48)]));

                        let mut buf = [0; 512];
                        let buf8 = slice8_mut(&mut buf);

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
                    }*/
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
        trace!("irq{} {:04x}", FormatInterrupt(irq), irq);

        if irq & IRQ_F2_PACKET_AVAILABLE != 0 {
            debug!("IRQ F2_PACKET_AVAILABLE");
            self.check_status(buf).await;
        }

        if irq & IRQ_F1_INTR != 0 {
            debug!("IRQ IRQ_F1_INTR");
            // TODO
        }

        if irq & IRQ_DATA_UNAVAILABLE != 0 {
            // TODO what should we do here?
            warn!("IRQ DATA_UNAVAILABLE, clearing...");
            self.bus.write16(FUNC_BUS, REG_BUS_INTERRUPT, 1).await;
        }

        if irq & IRQ_COMMAND_ERROR != 0 {
            // TODO what should we do here?
            warn!("IRQ COMMAND_ERROR, clearing...");
            self.bus.write16(FUNC_BUS, REG_BUS_INTERRUPT, 1).await;
        }

        if irq & IRQ_DATA_ERROR != 0 {
            // TODO what should we do here?
            warn!("IRQ DATA_ERROR, clearing...");
            self.bus.write16(FUNC_BUS, REG_BUS_INTERRUPT, 1).await;
        }

        if irq & IRQ_F1_OVERFLOW != 0 {
            // TODO what should we do here?
            warn!("IRQ F1_OVERFLOW, clearing...");
            self.bus.write16(FUNC_BUS, REG_BUS_INTERRUPT, 1).await;
        }
    }

    /// Handle F2 events while status register is set
    async fn check_status(&mut self, buf: &mut [u32; 512]) {
        loop {
            let status = self.bus.status();
            trace!("check status{}", FormatStatus(status));

            let sdio_int_status = self.bus.bp_read32(CHIP.sdiod_core_base_address + SDIO_INT_STATUS).await;
            if sdio_int_status & I_HMB_FC_CHANGE != 0 {
                debug!("sdio_int_status & I_HMB_FC_CHANGE != 0");
            }

            if status & STATUS_F2_PKT_AVAILABLE != 0 {
                let len = (status & STATUS_F2_PKT_LEN_MASK) >> STATUS_F2_PKT_LEN_SHIFT;
                self.bus.wlan_read(buf, len).await;
                trace!("rx {:02x}", Bytes(&slice8_mut(buf)[..(len as usize).min(48)]));
                self.rx(&mut slice8_mut(buf)[..len as usize]);
            } else {
                break;
            }

            
        }
    }

    fn rx(&mut self, packet: &mut [u8]) {
        debug!("runner rx packet = {:02x}", packet);

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

                    debug!("runner rx CHANNEL_TYPE_CONTROL response = {:02x}", response);

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
        let buf8 = slice8_mut(&mut buf);

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
        debug!("core_disable");

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
        debug!("core_reset");

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
        debug!("core_is_up");

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
