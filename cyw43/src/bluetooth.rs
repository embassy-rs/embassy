use core::cell::RefCell;
use core::future::Future;
use core::mem::MaybeUninit;

use bt_hci::transport::WithIndicator;
use bt_hci::{ControllerToHostPacket, FromHciBytes, HostToControllerPacket, PacketKind, WriteHci};
use embassy_futures::yield_now;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::zerocopy_channel;
use embassy_time::{Duration, Timer};
use embedded_hal_1::digital::OutputPin;

use crate::bus::Bus;
pub use crate::bus::SpiBusCyw43;
use crate::consts::*;
use crate::util::round_up;
use crate::{util, CHIP};

pub(crate) struct BtState {
    rx: [BtPacketBuf; 4],
    tx: [BtPacketBuf; 4],
    inner: MaybeUninit<BtStateInnre<'static>>,
}

impl BtState {
    pub const fn new() -> Self {
        Self {
            rx: [const { BtPacketBuf::new() }; 4],
            tx: [const { BtPacketBuf::new() }; 4],
            inner: MaybeUninit::uninit(),
        }
    }
}

struct BtStateInnre<'d> {
    rx: zerocopy_channel::Channel<'d, NoopRawMutex, BtPacketBuf>,
    tx: zerocopy_channel::Channel<'d, NoopRawMutex, BtPacketBuf>,
}

/// Bluetooth driver.
pub struct BtDriver<'d> {
    rx: RefCell<zerocopy_channel::Receiver<'d, NoopRawMutex, BtPacketBuf>>,
    tx: RefCell<zerocopy_channel::Sender<'d, NoopRawMutex, BtPacketBuf>>,
}

pub(crate) struct BtRunner<'d> {
    pub(crate) tx_chan: zerocopy_channel::Receiver<'d, NoopRawMutex, BtPacketBuf>,
    rx_chan: zerocopy_channel::Sender<'d, NoopRawMutex, BtPacketBuf>,

    // Bluetooth circular buffers
    addr: u32,
    h2b_write_pointer: u32,
    b2h_read_pointer: u32,
}

const BT_HCI_MTU: usize = 1024;

/// Represents a packet of size MTU.
pub(crate) struct BtPacketBuf {
    pub(crate) len: usize,
    pub(crate) buf: [u8; BT_HCI_MTU],
}

impl BtPacketBuf {
    /// Create a new packet buffer.
    pub const fn new() -> Self {
        Self {
            len: 0,
            buf: [0; BT_HCI_MTU],
        }
    }
}

pub(crate) fn new<'d>(state: &'d mut BtState) -> (BtRunner<'d>, BtDriver<'d>) {
    // safety: this is a self-referential struct, however:
    // - it can't move while the `'d` borrow is active.
    // - when the borrow ends, the dangling references inside the MaybeUninit will never be used again.
    let state_uninit: *mut MaybeUninit<BtStateInnre<'d>> =
        (&mut state.inner as *mut MaybeUninit<BtStateInnre<'static>>).cast();
    let state = unsafe { &mut *state_uninit }.write(BtStateInnre {
        rx: zerocopy_channel::Channel::new(&mut state.rx[..]),
        tx: zerocopy_channel::Channel::new(&mut state.tx[..]),
    });

    let (rx_sender, rx_receiver) = state.rx.split();
    let (tx_sender, tx_receiver) = state.tx.split();

    (
        BtRunner {
            tx_chan: tx_receiver,
            rx_chan: rx_sender,

            addr: 0,
            h2b_write_pointer: 0,
            b2h_read_pointer: 0,
        },
        BtDriver {
            rx: RefCell::new(rx_receiver),
            tx: RefCell::new(tx_sender),
        },
    )
}

pub(crate) struct CybtFwCb<'a> {
    pub p_next_line_start: &'a [u8],
}

pub(crate) struct HexFileData<'a> {
    pub addr_mode: i32,
    pub hi_addr: u16,
    pub dest_addr: u32,
    pub p_ds: &'a mut [u8],
}

pub(crate) fn read_firmware_patch_line(p_btfw_cb: &mut CybtFwCb, hfd: &mut HexFileData) -> u32 {
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
                abs_base_addr32 = (hfd.p_ds[0] as u32) << 24
                    | (hfd.p_ds[1] as u32) << 16
                    | (hfd.p_ds[2] as u32) << 8
                    | hfd.p_ds[3] as u32;
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

impl<'a> BtRunner<'a> {
    pub(crate) async fn init_bluetooth(&mut self, bus: &mut Bus<impl OutputPin, impl SpiBusCyw43>, firmware: &[u8]) {
        trace!("init_bluetooth");
        bus.bp_write32(CHIP.bluetooth_base_address + BT2WLAN_PWRUP_ADDR, BT2WLAN_PWRUP_WAKE)
            .await;
        Timer::after(Duration::from_millis(2)).await;
        self.upload_bluetooth_firmware(bus, firmware).await;
        self.wait_bt_ready(bus).await;
        self.init_bt_buffers(bus).await;
        self.wait_bt_awake(bus).await;
        self.bt_set_host_ready(bus).await;
        self.bt_toggle_intr(bus).await;
    }

    pub(crate) async fn upload_bluetooth_firmware(
        &mut self,
        bus: &mut Bus<impl OutputPin, impl SpiBusCyw43>,
        firmware: &[u8],
    ) {
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
            let num_fw_bytes = read_firmware_patch_line(&mut btfw_cb, &mut hfd);
            if num_fw_bytes == 0 {
                break;
            }
            let fw_bytes = &hfd.p_ds[0..num_fw_bytes as usize];
            let mut dest_start_addr = hfd.dest_addr + CHIP.bluetooth_base_address;
            let mut aligned_data_buffer_index: usize = 0;
            // pad start
            if !util::is_aligned(dest_start_addr, 4) {
                let num_pad_bytes = dest_start_addr % 4;
                let padded_dest_start_addr = util::round_down(dest_start_addr, 4);
                let memory_value = bus.bp_read32(padded_dest_start_addr).await;
                let memory_value_bytes = memory_value.to_le_bytes();
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
            if !util::is_aligned(dest_end_addr, 4) {
                let offset = dest_end_addr % 4;
                let num_pad_bytes_end = 4 - offset;
                let padded_dest_end_addr = util::round_down(dest_end_addr, 4);
                let memory_value = bus.bp_read32(padded_dest_end_addr).await;
                let memory_value_bytes = memory_value.to_le_bytes();
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
            bus.bp_write(dest_start_addr, buffer_to_write).await;
        }
    }

    pub(crate) async fn wait_bt_ready(&mut self, bus: &mut Bus<impl OutputPin, impl SpiBusCyw43>) {
        trace!("wait_bt_ready");
        let mut success = false;
        for _ in 0..300 {
            let val = bus.bp_read32(BT_CTRL_REG_ADDR).await;
            trace!("BT_CTRL_REG_ADDR = {:08x}", val);
            if val & BTSDIO_REG_FW_RDY_BITMASK != 0 {
                success = true;
                break;
            }
            Timer::after(Duration::from_millis(1)).await;
        }
        assert!(success == true);
    }

    pub(crate) async fn wait_bt_awake(&mut self, bus: &mut Bus<impl OutputPin, impl SpiBusCyw43>) {
        trace!("wait_bt_awake");
        let mut success = false;
        for _ in 0..300 {
            let val = bus.bp_read32(BT_CTRL_REG_ADDR).await;
            trace!("BT_CTRL_REG_ADDR = {:08x}", val);
            if val & BTSDIO_REG_BT_AWAKE_BITMASK != 0 {
                success = true;
                break;
            }
            Timer::after(Duration::from_millis(1)).await;
        }
        assert!(success == true);
    }

    pub(crate) async fn bt_set_host_ready(&mut self, bus: &mut Bus<impl OutputPin, impl SpiBusCyw43>) {
        trace!("bt_set_host_ready");
        let old_val = bus.bp_read32(HOST_CTRL_REG_ADDR).await;
        // TODO: do we need to swap endianness on this read?
        let new_val = old_val | BTSDIO_REG_SW_RDY_BITMASK;
        bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
    }

    // TODO: use this
    #[allow(dead_code)]
    pub(crate) async fn bt_set_awake(&mut self, bus: &mut Bus<impl OutputPin, impl SpiBusCyw43>, awake: bool) {
        trace!("bt_set_awake");
        let old_val = bus.bp_read32(HOST_CTRL_REG_ADDR).await;
        // TODO: do we need to swap endianness on this read?
        let new_val = if awake {
            old_val | BTSDIO_REG_WAKE_BT_BITMASK
        } else {
            old_val & !BTSDIO_REG_WAKE_BT_BITMASK
        };
        bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
    }

    pub(crate) async fn bt_toggle_intr(&mut self, bus: &mut Bus<impl OutputPin, impl SpiBusCyw43>) {
        trace!("bt_toggle_intr");
        let old_val = bus.bp_read32(HOST_CTRL_REG_ADDR).await;
        // TODO: do we need to swap endianness on this read?
        let new_val = old_val ^ BTSDIO_REG_DATA_VALID_BITMASK;
        bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
    }

    // TODO: use this
    #[allow(dead_code)]
    pub(crate) async fn bt_set_intr(&mut self, bus: &mut Bus<impl OutputPin, impl SpiBusCyw43>) {
        trace!("bt_set_intr");
        let old_val = bus.bp_read32(HOST_CTRL_REG_ADDR).await;
        let new_val = old_val | BTSDIO_REG_DATA_VALID_BITMASK;
        bus.bp_write32(HOST_CTRL_REG_ADDR, new_val).await;
    }

    pub(crate) async fn init_bt_buffers(&mut self, bus: &mut Bus<impl OutputPin, impl SpiBusCyw43>) {
        trace!("init_bt_buffers");
        self.addr = bus.bp_read32(WLAN_RAM_BASE_REG_ADDR).await;
        assert!(self.addr != 0);
        trace!("wlan_ram_base_addr = {:08x}", self.addr);
        bus.bp_write32(self.addr + BTSDIO_OFFSET_HOST2BT_IN, 0).await;
        bus.bp_write32(self.addr + BTSDIO_OFFSET_HOST2BT_OUT, 0).await;
        bus.bp_write32(self.addr + BTSDIO_OFFSET_BT2HOST_IN, 0).await;
        bus.bp_write32(self.addr + BTSDIO_OFFSET_BT2HOST_OUT, 0).await;
    }

    async fn bt_bus_request(&mut self, bus: &mut Bus<impl OutputPin, impl SpiBusCyw43>) {
        // TODO: CYW43_THREAD_ENTER mutex?
        self.bt_set_awake(bus, true).await;
        self.wait_bt_awake(bus).await;
    }

    pub(crate) async fn hci_write(&mut self, bus: &mut Bus<impl OutputPin, impl SpiBusCyw43>) {
        self.bt_bus_request(bus).await;

        // NOTE(unwrap): we only call this when we do have a packet in the queue.
        let buf = self.tx_chan.try_receive().unwrap();
        debug!("HCI tx: {:02x}", crate::fmt::Bytes(&buf.buf[..buf.len]));

        let len = buf.len as u32 - 1; // len doesn't include hci type byte
        let rounded_len = round_up(len, 4);
        let total_len = 4 + rounded_len;

        let read_pointer = bus.bp_read32(self.addr + BTSDIO_OFFSET_HOST2BT_OUT).await;
        let available = read_pointer.wrapping_sub(self.h2b_write_pointer + 4) % BTSDIO_FWBUF_SIZE;
        if available < total_len {
            warn!(
                "bluetooth tx queue full, retrying. len {} available {}",
                total_len, available
            );
            yield_now().await;
            return;
        }

        // Build header
        let mut header = [0u8; 4];
        header[0] = len as u8;
        header[1] = (len >> 8) as u8;
        header[2] = (len >> 16) as u8;
        header[3] = buf.buf[0]; // HCI type byte

        // Write header
        let addr = self.addr + BTSDIO_OFFSET_HOST_WRITE_BUF + self.h2b_write_pointer;
        bus.bp_write(addr, &header).await;
        self.h2b_write_pointer = (self.h2b_write_pointer + 4) % BTSDIO_FWBUF_SIZE;

        // Write payload.
        let payload = &buf.buf[1..][..rounded_len as usize];
        if self.h2b_write_pointer as usize + payload.len() > BTSDIO_FWBUF_SIZE as usize {
            // wraparound
            let n = BTSDIO_FWBUF_SIZE - self.h2b_write_pointer;
            let addr = self.addr + BTSDIO_OFFSET_HOST_WRITE_BUF + self.h2b_write_pointer;
            bus.bp_write(addr, &payload[..n as usize]).await;
            let addr = self.addr + BTSDIO_OFFSET_HOST_WRITE_BUF;
            bus.bp_write(addr, &payload[n as usize..]).await;
        } else {
            // no wraparound
            let addr = self.addr + BTSDIO_OFFSET_HOST_WRITE_BUF + self.h2b_write_pointer;
            bus.bp_write(addr, payload).await;
        }
        self.h2b_write_pointer = (self.h2b_write_pointer + payload.len() as u32) % BTSDIO_FWBUF_SIZE;

        // Update pointer.
        bus.bp_write32(self.addr + BTSDIO_OFFSET_HOST2BT_IN, self.h2b_write_pointer)
            .await;

        self.bt_toggle_intr(bus).await;

        self.tx_chan.receive_done();
    }

    async fn bt_has_work(&mut self, bus: &mut Bus<impl OutputPin, impl SpiBusCyw43>) -> bool {
        let int_status = bus.bp_read32(CHIP.sdiod_core_base_address + SDIO_INT_STATUS).await;
        if int_status & I_HMB_FC_CHANGE != 0 {
            bus.bp_write32(
                CHIP.sdiod_core_base_address + SDIO_INT_STATUS,
                int_status & I_HMB_FC_CHANGE,
            )
            .await;
            return true;
        }
        return false;
    }

    pub(crate) async fn handle_irq(&mut self, bus: &mut Bus<impl OutputPin, impl SpiBusCyw43>) {
        if self.bt_has_work(bus).await {
            loop {
                // Check if we have data.
                let write_pointer = bus.bp_read32(self.addr + BTSDIO_OFFSET_BT2HOST_IN).await;
                let available = write_pointer.wrapping_sub(self.b2h_read_pointer) % BTSDIO_FWBUF_SIZE;
                if available == 0 {
                    break;
                }

                // read header
                let mut header = [0u8; 4];
                let addr = self.addr + BTSDIO_OFFSET_HOST_READ_BUF + self.b2h_read_pointer;
                bus.bp_read(addr, &mut header).await;

                // calc length
                let len = header[0] as u32 | ((header[1]) as u32) << 8 | ((header[2]) as u32) << 16;
                let rounded_len = round_up(len, 4);
                if available < 4 + rounded_len {
                    warn!("ringbuf data not enough for a full packet?");
                    break;
                }
                self.b2h_read_pointer = (self.b2h_read_pointer + 4) % BTSDIO_FWBUF_SIZE;

                // Obtain a buf from the channel.
                let buf = self.rx_chan.send().await;

                buf.buf[0] = header[3]; // hci packet type
                let payload = &mut buf.buf[1..][..rounded_len as usize];
                if self.b2h_read_pointer as usize + payload.len() > BTSDIO_FWBUF_SIZE as usize {
                    // wraparound
                    let n = BTSDIO_FWBUF_SIZE - self.b2h_read_pointer;
                    let addr = self.addr + BTSDIO_OFFSET_HOST_READ_BUF + self.b2h_read_pointer;
                    bus.bp_read(addr, &mut payload[..n as usize]).await;
                    let addr = self.addr + BTSDIO_OFFSET_HOST_READ_BUF;
                    bus.bp_read(addr, &mut payload[n as usize..]).await;
                } else {
                    // no wraparound
                    let addr = self.addr + BTSDIO_OFFSET_HOST_READ_BUF + self.b2h_read_pointer;
                    bus.bp_read(addr, payload).await;
                }
                self.b2h_read_pointer = (self.b2h_read_pointer + payload.len() as u32) % BTSDIO_FWBUF_SIZE;
                bus.bp_write32(self.addr + BTSDIO_OFFSET_BT2HOST_OUT, self.b2h_read_pointer)
                    .await;

                buf.len = 1 + len as usize;
                debug!("HCI rx: {:02x}", crate::fmt::Bytes(&buf.buf[..buf.len]));

                self.rx_chan.send_done();

                self.bt_toggle_intr(bus).await;
            }
        }
    }
}

impl<'d> embedded_io_async::ErrorType for BtDriver<'d> {
    type Error = core::convert::Infallible;
}

impl<'d> bt_hci::transport::Transport for BtDriver<'d> {
    fn read<'a>(&self, rx: &'a mut [u8]) -> impl Future<Output = Result<ControllerToHostPacket<'a>, Self::Error>> {
        async {
            let ch = &mut *self.rx.borrow_mut();
            let buf = ch.receive().await;
            let n = buf.len;
            assert!(n < rx.len());
            rx[..n].copy_from_slice(&buf.buf[..n]);
            ch.receive_done();

            let kind = PacketKind::from_hci_bytes_complete(&rx[..1]).unwrap();
            let (res, _) = ControllerToHostPacket::from_hci_bytes_with_kind(kind, &rx[1..n]).unwrap();
            Ok(res)
        }
    }

    /// Write a complete HCI packet from the tx buffer
    fn write<T: HostToControllerPacket>(&self, val: &T) -> impl Future<Output = Result<(), Self::Error>> {
        async {
            let ch = &mut *self.tx.borrow_mut();
            let buf = ch.send().await;
            let buf_len = buf.buf.len();
            let mut slice = &mut buf.buf[..];
            WithIndicator::new(val).write_hci(&mut slice).unwrap();
            buf.len = buf_len - slice.len();
            ch.send_done();
            Ok(())
        }
    }
}
