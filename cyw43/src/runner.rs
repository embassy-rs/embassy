use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;

use embassy_futures::select::{Either4, select4};
use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::LinkState;
use embassy_time::{Duration, Timer, block_for};

use crate::consts::*;
use crate::events::{Event, Events, Status};
use crate::fmt::Bytes;
use crate::ioctl::{IoctlState, IoctlType, PendingIoctl};
use crate::nvram::NVRAM;
pub use crate::spi::SpiBusCyw43;
use crate::structs::*;
use crate::util::{slice8_mut, slice16_mut};
use crate::{CHIP, Core, MTU, events, try_until};

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

pub(crate) enum BusType {
    Spi,
    Sdio,
}

pub(crate) trait SealedBus {
    const TYPE: BusType;

    async fn init(&mut self, bluetooth_enabled: bool);
    async fn wlan_read(&mut self, buf: &mut [u32], len_in_u8: u32);
    async fn wlan_write(&mut self, buf: &[u32]);
    #[allow(unused)]
    async fn bp_read(&mut self, addr: u32, data: &mut [u8]);
    async fn bp_write(&mut self, addr: u32, data: &[u8]);
    async fn bp_read8(&mut self, addr: u32) -> u8;
    async fn bp_write8(&mut self, addr: u32, val: u8);
    async fn bp_read16(&mut self, addr: u32) -> u16;
    #[allow(unused)]
    async fn bp_write16(&mut self, addr: u32, val: u16);
    #[allow(unused)]
    async fn bp_read32(&mut self, addr: u32) -> u32;
    async fn bp_write32(&mut self, addr: u32, val: u32);
    async fn read8(&mut self, func: u32, addr: u32) -> u8;
    async fn write8(&mut self, func: u32, addr: u32, val: u8);
    async fn read16(&mut self, func: u32, addr: u32) -> u16;
    async fn write16(&mut self, func: u32, addr: u32, val: u16);
    async fn read32(&mut self, func: u32, addr: u32) -> u32;
    #[allow(unused)]
    async fn write32(&mut self, func: u32, addr: u32, val: u32);
    async fn wait_for_event(&mut self);
}

#[allow(private_bounds)]
pub trait Bus: SealedBus {}
impl<T: SealedBus> Bus for T {}

/// Driver communicating with the WiFi chip.
pub struct Runner<'a, BUS> {
    ch: ch::Runner<'a, MTU>,
    pub(crate) bus: BUS,

    ioctl_state: &'a IoctlState,
    ioctl_id: u16,
    sdpcm_seq: u8,
    sdpcm_seq_max: u8,

    events: &'a Events,

    secure_network: &'a AtomicBool,
    join_ok: bool,
    key_exchange_ok: bool,
    auth_ok: bool,

    #[cfg(feature = "firmware-logs")]
    log: LogState,

    #[cfg(feature = "bluetooth")]
    pub(crate) bt: Option<crate::bluetooth::BtRunner<'a>>,
}

impl<'a, BUS> Runner<'a, BUS>
where
    BUS: Bus,
{
    pub(crate) fn new(
        ch: ch::Runner<'a, MTU>,
        bus: BUS,
        ioctl_state: &'a IoctlState,
        events: &'a Events,
        secure_network: &'a AtomicBool,
        #[cfg(feature = "bluetooth")] bt: Option<crate::bluetooth::BtRunner<'a>>,
    ) -> Self {
        Self {
            ch,
            bus,
            ioctl_state,
            ioctl_id: 0,
            sdpcm_seq: 0,
            sdpcm_seq_max: 1,
            events,
            secure_network,
            join_ok: false,
            key_exchange_ok: false,
            auth_ok: false,
            #[cfg(feature = "firmware-logs")]
            log: LogState::default(),
            #[cfg(feature = "bluetooth")]
            bt,
        }
    }

    pub(crate) async fn init(&mut self, wifi_fw: &[u8], bt_fw: Option<&[u8]>) -> Result<(), ()> {
        self.bus.init(bt_fw.is_some()).await;

        // Init ALP (Active Low Power) clock
        debug!("init alp");
        match BUS::TYPE {
            BusType::Spi => {
                self.bus
                    .write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, BACKPLANE_ALP_AVAIL_REQ)
                    .await;

                // Not present in whd driver
                debug!("set f2 watermark");
                self.bus
                    .write8(FUNC_BACKPLANE, REG_BACKPLANE_FUNCTION2_WATERMARK, 0x10)
                    .await;
                let watermark = self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_FUNCTION2_WATERMARK).await;
                debug!("watermark = {:02x}", watermark);
                assert!(watermark == 0x10);
            }
            BusType::Sdio => {
                self.bus
                    .write8(
                        FUNC_BACKPLANE,
                        REG_BACKPLANE_CHIP_CLOCK_CSR,
                        BACKPLANE_FORCE_HW_CLKREQ_OFF | BACKPLANE_ALP_AVAIL_REQ | BACKPLANE_FORCE_ALP,
                    )
                    .await;
            }
        }

        debug!("waiting for clock...");
        if !try_until(
            async || self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & BACKPLANE_ALP_AVAIL != 0,
            Duration::from_millis(100),
        )
        .await
        {
            debug!("timeout while waiting for alp clock!");
            return Err(());
        }
        debug!("clock ok");

        // clear request for ALP
        debug!("clear request for ALP");
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, 0).await;

        let chip_id = match BUS::TYPE {
            BusType::Spi => self.bus.bp_read16(CHIPCOMMON_BASE_ADDRESS).await,
            BusType::Sdio => {
                // Disable the extra sdio pull-ups
                // self.bus.write8(BACKPLANE_FUNCTION, SDIO_PULL_UP, 0).await;

                // Enable f1 and f2
                self.bus
                    .write8(
                        BUS_FUNCTION,
                        SDIOD_CCCR_IOEN,
                        (SDIO_FUNC_ENABLE_1 | SDIO_FUNC_ENABLE_2) as u8,
                    )
                    .await;

                // Enable out-of-band interrupt signal
                // whd_bus_sdio_init_oob_intr

                // Note: only GPIO0 using rising edge is currently supported
                self.bus
                    .write8(
                        BUS_FUNCTION,
                        SDIOD_SEP_INT_CTL,
                        (SEP_INTR_CTL_MASK | SEP_INTR_CTL_EN | SEP_INTR_CTL_POL) as u8,
                    )
                    .await;

                // Enable f2 interrupt only
                self.bus
                    .write8(
                        BUS_FUNCTION,
                        SDIOD_CCCR_INTEN,
                        (INTR_CTL_MASTER_EN | INTR_CTL_FUNC2_EN) as u8,
                    )
                    .await;

                self.bus.read8(BUS_FUNCTION, SDIOD_CCCR_IORDY).await;

                let reg = self.bus.read8(BUS_FUNCTION, SDIOD_CCCR_BRCM_CARDCAP).await;
                if reg & SDIOD_CCCR_BRCM_CARDCAP_SECURE_MODE as u8 != 0 {
                    debug!("chip supports bootloader handshake");

                    let devctrl = self.bus.read8(FUNC_BACKPLANE, SBSDIO_DEVICE_CTL).await;

                    self.bus
                        .write8(
                            FUNC_BACKPLANE,
                            SBSDIO_DEVICE_CTL,
                            devctrl | SBSDIO_DEVCTL_ADDR_RST as u8,
                        )
                        .await;

                    let addr_low = self.bus.read8(BACKPLANE_FUNCTION, SBSDIO_FUNC1_SBADDRLOW).await as u32;
                    let addr_mid = self.bus.read8(BACKPLANE_FUNCTION, SBSDIO_FUNC1_SBADDRMID).await as u32;
                    let addr_high = self.bus.read8(BACKPLANE_FUNCTION, SBSDIO_FUNC1_SBADDRHIGH).await as u32;

                    let reg_addr = ((addr_low << 8) | (addr_mid << 16) | (addr_high << 24)) + SDIO_CORE_CHIPID_REG;

                    self.bus.write8(FUNC_BACKPLANE, SBSDIO_DEVICE_CTL, devctrl).await;

                    self.bus.bp_read16(reg_addr).await
                } else {
                    self.bus.bp_read16(CHIPCOMMON_BASE_ADDRESS).await
                }
            }
        };

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
        self.bus.bp_write(ram_addr, wifi_fw).await;

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
        debug!("waiting for HT clock...");
        while self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & 0x80 == 0 {}

        // "Set up the interrupt mask and enable interrupts"
        debug!("setup interrupt mask");
        self.bus
            .bp_write32(CHIP.sdiod_core_base_address + SDIO_INT_HOST_MASK, I_HMB_SW_MASK)
            .await;

        match BUS::TYPE {
            BusType::Sdio => {
                self.bus
                    .bp_write8(CHIP.sdiod_core_base_address + SDIO_FUNCTION_INT_MASK, 2 | 1)
                    .await;

                // "Lower F2 Watermark to avoid DMA Hang in F2 when SD Clock is stopped."
                // Sounds scary...
                self.bus
                    .write8(FUNC_BACKPLANE, REG_BACKPLANE_FUNCTION2_WATERMARK, SDIO_F2_WATERMARK)
                    .await;
            }

            BusType::Spi => {
                // Set up the interrupt mask and enable interrupts
                if bt_fw.is_some() {
                    debug!("bluetooth setup interrupt mask");
                    self.bus
                        .bp_write32(CHIP.sdiod_core_base_address + SDIO_INT_HOST_MASK, I_HMB_FC_CHANGE)
                        .await;
                }

                self.bus
                    .write16(FUNC_BUS, REG_BUS_INTERRUPT_ENABLE, IRQ_F2_PACKET_AVAILABLE)
                    .await;

                // "Lower F2 Watermark to avoid DMA Hang in F2 when SD Clock is stopped."
                // Sounds scary...
                self.bus
                    .write8(FUNC_BACKPLANE, REG_BACKPLANE_FUNCTION2_WATERMARK, SPI_F2_WATERMARK)
                    .await;
            }
        }

        // wait for F2 to be ready
        debug!("waiting for F2 to be ready...");
        if !try_until(
            async || match BUS::TYPE {
                BusType::Sdio => self.bus.read8(FUNC_BUS, SDIOD_CCCR_IORDY).await as u32 & SDIO_FUNC_READY_2 != 0,
                BusType::Spi => self.bus.read32(FUNC_BUS, REG_BUS_STATUS).await & STATUS_F2_RX_READY != 0,
            },
            Duration::from_millis(1000),
        )
        .await
        {
            debug!("timeout while waiting for function 2 to be ready");
            return Err(());
        }

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

        #[cfg(feature = "bluetooth")]
        if let Some(bt_fw) = bt_fw {
            self.bt.as_mut().unwrap().init_bluetooth(&mut self.bus, bt_fw).await;
        }

        debug!("cyw43 runner init done");

        Ok(())
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

    /// Run the CYW43 event handling loop.
    pub async fn run(mut self) -> ! {
        let mut buf = [0; 512];
        loop {
            #[cfg(feature = "firmware-logs")]
            self.log_read().await;

            if self.has_credit() {
                let ioctl = self.ioctl_state.wait_pending();
                let wifi_tx = self.ch.tx_buf();
                #[cfg(feature = "bluetooth")]
                let bt_tx = async {
                    match &mut self.bt {
                        Some(bt) => bt.tx_chan.receive().await,
                        None => core::future::pending().await,
                    }
                };
                #[cfg(not(feature = "bluetooth"))]
                let bt_tx = core::future::pending::<()>();

                // interrupts aren't working yet for bluetooth. Do busy-polling instead.
                // Note for this to work `ev` has to go last in the `select()`. It prefers
                // first futures if they're ready, so other select branches don't get starved.`
                #[cfg(feature = "bluetooth")]
                let ev = core::future::ready(());
                #[cfg(not(feature = "bluetooth"))]
                let ev = self.bus.wait_for_event();

                match select4(ioctl, wifi_tx, bt_tx, ev).await {
                    Either4::First(PendingIoctl {
                        buf: iobuf,
                        kind,
                        cmd,
                        iface,
                    }) => {
                        self.send_ioctl(kind, cmd, iface, unsafe { &*iobuf }, &mut buf).await;
                        self.check_status(&mut buf).await;
                    }
                    Either4::Second(packet) => {
                        trace!("tx pkt {:02x}", Bytes(&packet[..packet.len().min(48)]));

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
                    }
                    Either4::Third(_) => {
                        #[cfg(feature = "bluetooth")]
                        self.bt.as_mut().unwrap().hci_write(&mut self.bus).await;
                    }
                    Either4::Fourth(()) => {
                        self.handle_irq(&mut buf).await;

                        // If we do busy-polling, make sure to yield.
                        // `handle_irq` will only do a 32bit read if there's no work to do, which is really fast.
                        // Depending on optimization level, it is possible that the 32-bit read finishes on
                        // first poll, so it never yields and we starve all other tasks.
                        #[cfg(feature = "bluetooth")]
                        embassy_futures::yield_now().await;
                    }
                }
            } else {
                warn!("TX stalled");
                match BUS::TYPE {
                    BusType::Sdio => {
                        // whd_bus_sdio_poke_wlan
                        self.bus
                            .bp_write32(CHIP.sdiod_core_base_address + SDIO_TO_SB_MAILBOX, SMB_DEV_INT)
                            .await;
                    }
                    BusType::Spi => {}
                }
                self.bus.wait_for_event().await;
                self.handle_irq(&mut buf).await;
            }
        }
    }

    /// Wait for IRQ on F2 packet available
    async fn handle_irq(&mut self, buf: &mut [u32; 512]) {
        match BUS::TYPE {
            BusType::Sdio => {
                self.check_status(buf).await;
            }
            BusType::Spi => {
                // Receive stuff
                let irq = self.bus.read16(FUNC_BUS, REG_BUS_INTERRUPT).await;
                if irq != 0 {
                    trace!("irq{}", FormatInterrupt(irq));
                }

                if irq & IRQ_F2_PACKET_AVAILABLE != 0 {
                    self.check_status(buf).await;
                }

                if irq & IRQ_DATA_UNAVAILABLE != 0 {
                    // this seems to be ignorable with no ill effects.
                    trace!("IRQ DATA_UNAVAILABLE, clearing...");
                    self.bus.write16(FUNC_BUS, REG_BUS_INTERRUPT, 1).await;
                }

                #[cfg(feature = "bluetooth")]
                if let Some(bt) = &mut self.bt {
                    bt.handle_irq(&mut self.bus).await;
                }
            }
        }
    }

    /// Handle F2 events while status register is set
    async fn check_status(&mut self, buf: &mut [u32; 512]) {
        loop {
            match BUS::TYPE {
                BusType::Spi => {
                    let status = self.bus.read32(BUS_FUNCTION, SPI_STATUS_REGISTER).await;
                    trace!("check status{}", FormatStatus(status));

                    if status & STATUS_F2_PKT_AVAILABLE != 0 {
                        let len = (status & STATUS_F2_PKT_LEN_MASK) >> STATUS_F2_PKT_LEN_SHIFT;
                        self.bus.wlan_read(buf, len).await;
                        trace!("rx {:02x}", Bytes(&slice8_mut(buf)[..(len as usize).min(48)]));
                        self.rx(&mut slice8_mut(buf)[..len as usize]);
                    } else {
                        break;
                    }
                }
                BusType::Sdio => {
                    // whd_bus_sdio_packet_available_to_read
                    let status = self.bus.bp_read32(CHIP.sdiod_core_base_address + SDIO_INT_STATUS).await;
                    if status & I_HMB_HOST_INT == 0 {
                        trace!("pkt not available to read");
                        break;
                    }

                    let hmb_data = self
                        .bus
                        .bp_read32(CHIP.sdiod_core_base_address + SDIO_TO_HOST_MAILBOX_DATA)
                        .await;
                    if hmb_data > 0 {
                        self.bus
                            .bp_write32(CHIP.sdiod_core_base_address + SDIO_TO_SB_MAILBOX, SMB_INT_ACK)
                            .await;

                        trace!("hmb ack");
                    }

                    if hmb_data & I_HMB_DATA_FWHALT != 0 {
                        trace!("hmb data fault");
                        break;
                    }

                    if status & HOSTINTMASK != 0 {
                        self.bus
                            .bp_write32(CHIP.sdiod_core_base_address + SDIO_INT_STATUS, status & HOSTINTMASK)
                            .await;
                    }

                    trace!("pkt ready...");
                    let mut hwtag = &mut buf[..4];
                    self.bus.wlan_read(&mut hwtag, INITIAL_READ).await;
                    {
                        let hwtag = slice16_mut(&mut hwtag);
                        if (hwtag[0] | hwtag[1]) == 0 || (hwtag[0] ^ hwtag[1]) != 0xFFFF {
                            trace!("hwtag mismatch (hwtag[0] = {}, hwtag[1] = {})", hwtag[0], hwtag[1]);
                            break;
                        }
                    }

                    if slice16_mut(&mut hwtag)[0] == 12 {
                        self.bus.wlan_read(&mut hwtag[1..], 8).await;
                        let Some(sdpcm_header) = SdpcmHeader::parse_header(slice8_mut(&mut hwtag)) else {
                            trace!("failed to parse sdpcm header");
                            break;
                        };

                        self.update_credit(&sdpcm_header);

                        break;
                    }

                    if slice16_mut(&mut hwtag)[0] <= INITIAL_READ as u16 {
                        trace!("no extra space required");
                        break;
                    }

                    let extra_space_required = slice16_mut(&mut hwtag)[0] - INITIAL_READ as u16;

                    self.bus.wlan_read(&mut buf[1..], extra_space_required as u32).await;
                    let len = INITIAL_READ as usize + extra_space_required as usize;

                    trace!("rx {:02x}", Bytes(&slice8_mut(buf)[..len.min(48)]));
                    self.rx(&mut slice8_mut(buf)[..len as usize]);
                }
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

                let event_type = Event::from(event_packet.msg.event_type as u8);
                let status = EStatus::from(event_packet.msg.status as u8);
                debug!(
                    "=== EVENT {:?}: {:?} {:02x}",
                    event_type,
                    event_packet.msg,
                    Bytes(evt_data)
                );

                let update_link_status = match (
                    event_type,
                    status,
                    event_packet.msg.flags,
                    event_packet.msg.reason,
                    event_packet.msg.auth_type,
                ) {
                    // Events indicating that the link is down
                    // Event LINK with flag 0 indicates link down. reason = 1: loss of signal (e.g. out of range), reason = 2: controlled network shutdown
                    // Event AUTH with status FAIL, reason 16, and auth_type 3 is specific for WPA3 networks
                    (Event::LINK, EStatus::SUCCESS, 0, ..)
                    | (Event::DEAUTH, EStatus::SUCCESS, ..)
                    | (Event::AUTH, EStatus::FAIL, _, 16, 3) => {
                        self.auth_ok = false;
                        self.join_ok = false;
                        self.key_exchange_ok = false;
                        true
                    }
                    // Update auth flag. Ignore unsolicited events.
                    // When changing passwords on a WPA3 AP which we are already connected to, or we roam to, PSK_SUP events indicating
                    // success are still sent. Only the AUTH events indicate failure and this flag helps cover that scenario
                    (Event::AUTH, status, ..) if status != EStatus::UNSOLICITED => {
                        self.auth_ok = status == EStatus::SUCCESS;
                        debug!("auth_ok flag: {}", self.auth_ok as u8);
                        false
                    }
                    // Successfully joined the network. Open or WPA3 networks are now fully connected - WPA1/2 networks additionally require a successful key exchange.
                    (Event::JOIN, EStatus::SUCCESS, ..) => {
                        self.join_ok = true;
                        true
                    }

                    // Key exchange events (PSK_SUP) for secure networks
                    // The status codes for PSK_SUP events seem to have different meanings from other event types

                    // Successful key exchange, indicated by a PSK_SUP event with status 6 "UNSOLICITED"
                    // Disregard if auth_ok is false, which can happen in WPA3 networks
                    (Event::PSK_SUP, EStatus::UNSOLICITED, 0, 0, _) => {
                        if self.auth_ok {
                            self.key_exchange_ok = true;
                            true
                        } else {
                            false
                        }
                    }
                    // Ignore PSK_SUP events with reason 14 as they are often sent when the device roams from one AP to another
                    (Event::PSK_SUP, _, _, 14, _) => false,
                    // Other PSK_SUP events indicate key exchange errors
                    (Event::PSK_SUP, ..) => {
                        self.key_exchange_ok = false;
                        true
                    }
                    _ => false,
                };

                if update_link_status {
                    let secure_network = self.secure_network.load(Relaxed);
                    let link_state = if self.join_ok && (!secure_network || self.key_exchange_ok) {
                        LinkState::Up
                    } else {
                        LinkState::Down
                    };

                    self.ch.set_link_state(link_state);

                    debug!(
                        "link_ok: {}, secure_network: {}, auth_ok: {}, password_ok: {}, link_state {}",
                        self.join_ok as u8,
                        secure_network as u8,
                        self.auth_ok as u8,
                        self.key_exchange_ok as u8,
                        link_state as u8
                    );
                }

                if self.events.mask.is_enabled(event_type) {
                    let status = event_packet.msg.status;
                    let event_payload = match event_type {
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
                    self.events
                        .queue
                        .immediate_publisher()
                        .publish_immediate(events::Message::new(Status { event_type, status }, event_payload));
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

    async fn send_ioctl(&mut self, kind: IoctlType, cmd: Ioctl, iface: u32, data: &[u8], buf: &mut [u32; 512]) {
        let buf8 = slice8_mut(buf);

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
            cmd: cmd as u32,
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

        Timer::after_millis(1).await;

        self.bus
            .bp_write8(base + AI_IOCTRL_OFFSET, AI_IOCTRL_BIT_CLOCK_EN)
            .await;
        let _ = self.bus.bp_read8(base + AI_IOCTRL_OFFSET).await;

        Timer::after_millis(1).await;
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
