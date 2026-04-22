use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;

use aligned::{A4, Aligned};
use embassy_futures::select::{Either4, select4};
use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::LinkState;
use embassy_time::Duration;

use crate::chip::{
    check_device_core_is_up, chip_specific_socsram_init, disable_device_core, reset_core, reset_device_core,
};
use crate::consts::*;
use crate::events::{Event, Events, Status};
use crate::fmt::Bytes;
use crate::ioctl::{IoctlState, IoctlType, PendingIoctl};
pub use crate::spi::SpiBusCyw43;
use crate::structs::*;
use crate::util::{aligned_mut, aligned_ref, slice8_mut, slice16_mut, try_until};
use crate::{Chip, ChipId, Core, MTU, WithContext, events, sdio};

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

pub(crate) enum BusConfig<'a> {
    #[allow(dead_code)]
    Spi(&'a ()),
    Sdio(&'a sdio::Config),
}

pub(crate) trait SealedBus {
    const TYPE: BusType;
    type Config;

    async fn init<'a>(&mut self, bluetooth: bool, config: &'a Self::Config) -> crate::Result<BusConfig<'a>>;
    async fn wlan_read(&mut self, buf: &mut Aligned<A4, [u8]>) -> crate::Result<()>;
    async fn wlan_write(&mut self, buf: &Aligned<A4, [u8]>);
    #[allow(unused)]
    async fn bp_read(&mut self, addr: u32, data: &mut [u8]) -> crate::Result<()>;
    async fn bp_write(&mut self, addr: u32, data: &[u8]) -> crate::Result<()>;
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

    fn bus_type(&self) -> BusType {
        Self::TYPE
    }
}

#[allow(private_bounds)]
pub trait Bus: SealedBus {}
impl<T: SealedBus> Bus for T {}

async fn wake_bus(bus: &mut impl Bus) -> crate::Result<()> {
    if matches!(bus.bus_type(), BusType::Sdio) {
        bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, BACKPLANE_HT_AVAIL_REQ)
            .await;

        try_until(
            async || bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & BACKPLANE_HT_AVAIL_REQ << 3 != 0,
            Duration::from_millis(5),
        )
        .await
        .ctx("timeout while requesting HT clock before SDIO access")?;
    }

    Ok(())
}

async fn wlan_read(bus: &mut impl Bus, buf: &mut Aligned<A4, [u8]>) -> crate::Result<()> {
    wake_bus(bus).await?;
    bus.wlan_read(buf).await.map_err(|_| crate::Error)
}

async fn wlan_write(bus: &mut impl Bus, buf: &Aligned<A4, [u8]>) -> crate::Result<()> {
    wake_bus(bus).await?;
    bus.wlan_write(buf).await;

    Ok(())
}

/// Driver communicating with the WiFi chip.
pub struct Runner<'a, BUS: Bus, CHIP: Chip> {
    ch: ch::Runner<'a, MTU>,
    bus: BUS,
    chip: CHIP,

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

impl<'a, BUS: Bus, CHIP: Chip> Runner<'a, BUS, CHIP> {
    pub(crate) fn new(
        ch: ch::Runner<'a, MTU>,
        bus: BUS,
        chip: CHIP,
        ioctl_state: &'a IoctlState,
        events: &'a Events,
        secure_network: &'a AtomicBool,
        #[cfg(feature = "bluetooth")] bt: Option<crate::bluetooth::BtRunner<'a>>,
    ) -> Self {
        Self {
            ch,
            bus,
            chip,
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

    async fn verify_download(&mut self, label: &str, addr: u32, data: &[u8]) -> crate::Result<()> {
        async fn bp_read_bytes<BUS: Bus, const N: usize>(
            bus: &mut BUS,
            addr: u32,
        ) -> crate::Result<Aligned<A4, [u8; N]>> {
            let mut buf = Aligned([0; N]);
            bus.bp_read(addr, &mut buf[..]).await?;
            Ok(buf)
        }

        fn sample_checksum(data: &[u8]) -> u32 {
            data.iter()
                .fold(0u32, |acc, &b| acc.wrapping_mul(33).wrapping_add(u32::from(b)))
        }

        const SAMPLE_LEN: usize = 16;
        const CHECKSUM_LEN: usize = 64;

        if data.len() < SAMPLE_LEN {
            debug!("{} verify skipped: image too small ({} bytes)", label, data.len());
            return Ok(());
        }

        let mut offsets = [0usize; 4];
        let mut offset_count = 0usize;
        for candidate in [
            0usize,
            (data.len() / 3) & !3usize,
            ((data.len() * 2) / 3) & !3usize,
            (data.len() - SAMPLE_LEN) & !3usize,
        ] {
            if !offsets[..offset_count].contains(&candidate) {
                offsets[offset_count] = candidate;
                offset_count += 1;
            }
        }

        for &offset in &offsets[..offset_count] {
            let actual = bp_read_bytes::<BUS, SAMPLE_LEN>(&mut self.bus, addr + offset as u32).await?;
            let expected = &data[offset..offset + SAMPLE_LEN];
            let ok = &actual[..] == expected;
            debug!(
                "{} sample @{:08x} checksum exp={:08x} got={:08x} match={}",
                label,
                addr + offset as u32,
                sample_checksum(expected),
                sample_checksum(&actual[..]),
                ok,
            );
            if !ok {
                return err!(
                    "{} expected: {:02x} acutal: {:02x}",
                    label,
                    Bytes(expected),
                    Bytes(&actual[..])
                );
            }
        }

        let checksum_span = data.len().min(CHECKSUM_LEN);
        for &offset in &[
            0usize,
            ((data.len() / 2).saturating_sub(checksum_span / 2)) & !3usize,
            (data.len().saturating_sub(checksum_span)) & !3usize,
        ] {
            let mut actual: Aligned<A4, [u8; CHECKSUM_LEN]> = Aligned([0; CHECKSUM_LEN]);
            self.bus
                .bp_read(addr + offset as u32, &mut actual[..checksum_span])
                .await?;
            let expected = &data[offset..offset + checksum_span];
            let actual = &actual[..checksum_span];
            let exp_sum = sample_checksum(expected);
            let got_sum = sample_checksum(actual);
            debug!(
                "{} chunk checksum @{:08x} len={} exp={:08x} got={:08x} match={}",
                label,
                addr + offset as u32,
                checksum_span,
                exp_sum,
                got_sum,
                exp_sum == got_sum,
            );

            if exp_sum != got_sum || actual != expected {
                return err!("{} chunk verify failed @{:08x}", label, addr + offset as u32);
            }
        }

        Ok(())
    }

    async fn write_reset_instruction(&mut self, wifi_fw: &[u8]) -> crate::Result<()> {
        if wifi_fw.len() < 4 {
            return err!("FW image too small to extract reset instruction");
        }

        // CR4-based chips boot firmware from ATCM, but still expect the first
        // reset instruction to be mirrored at backplane address 0. WHD's SDIO
        // path writes this word explicitly after the firmware download.
        let reset_instr = u32::from_le_bytes([wifi_fw[0], wifi_fw[1], wifi_fw[2], wifi_fw[3]]);
        self.bus.bp_write32(0, reset_instr).await;

        let reset_instr_rb = self.bus.bp_read32(0).await;
        debug!(
            "reset instruction @00000000 = {:08x} readback={:08x} match={}",
            reset_instr,
            reset_instr_rb,
            reset_instr_rb == reset_instr,
        );
        if reset_instr_rb != reset_instr {
            return err!("reset instruction write FAILED");
        }

        Ok(())
    }

    async fn sdio_init_oob_intr(&mut self, config: &sdio::Config) {
        if config.out_of_band_irq {
            self.bus
                .write8(FUNC_BUS, SDIOD_SEP_INT_CTL, SEP_INTR_CTL_MASK | SEP_INTR_CTL_EN)
                .await;
        }
    }

    async fn read_chip_id_sdio(&mut self, config: &sdio::Config) -> u16 {
        // Disable the extra sdio pull-ups
        self.bus.write8(FUNC_BACKPLANE, SDIO_PULL_UP, 0).await;

        self.bus
            .write8(FUNC_BUS, SDIOD_CCCR_IOEN, SDIO_FUNC_ENABLE_1 as u8)
            .await;

        // Enable f1 and f2
        self.bus
            .write8(
                FUNC_BUS,
                SDIOD_CCCR_INTEN,
                (INTR_CTL_MASTER_EN | INTR_CTL_FUNC1_EN | INTR_CTL_FUNC2_EN) as u8,
            )
            .await;

        self.sdio_init_oob_intr(config).await;

        // TODO: remove this after investigation
        self.bus
            .write8(
                FUNC_BUS,
                SDIOD_CCCR_IOEN,
                (SDIO_FUNC_ENABLE_1 | SDIO_FUNC_ENABLE_2) as u8,
            )
            .await;

        // Enable f2 interrupt only
        self.bus
            .write8(
                FUNC_BUS,
                SDIOD_CCCR_INTEN,
                (INTR_CTL_MASTER_EN | INTR_CTL_FUNC2_EN) as u8,
            )
            .await;

        let _ = self.bus.read8(FUNC_BUS, SDIOD_CCCR_IORDY).await;

        let reg = self.bus.read8(FUNC_BUS, SDIOD_CCCR_BRCM_CARDCAP).await;
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

            let addr_low = self.bus.read8(FUNC_BACKPLANE, SBSDIO_FUNC1_SBADDRLOW).await as u32;
            let addr_mid = self.bus.read8(FUNC_BACKPLANE, SBSDIO_FUNC1_SBADDRMID).await as u32;
            let addr_high = self.bus.read8(FUNC_BACKPLANE, SBSDIO_FUNC1_SBADDRHIGH).await as u32;

            let reg_addr = ((addr_low << 8) | (addr_mid << 16) | (addr_high << 24)) + SDIO_CORE_CHIPID_REG;

            self.bus.write8(FUNC_BACKPLANE, SBSDIO_DEVICE_CTL, devctrl).await;

            self.bus.bp_read16(reg_addr).await
        } else {
            self.bus.bp_read16(CHIPCOMMON_BASE_ADDRESS).await
        }
    }

    async fn init_cyw43439(&mut self) -> crate::Result<()> {
        if matches!(self.bus.bus_type(), BusType::Sdio) {
            self.bus
                .write8(FUNC_BACKPLANE, SDIO_SLEEP_CSR, SBSDIO_SLPCSR_KEEP_WL_KS as u8)
                .await;

            self.bus
                .write8(FUNC_BACKPLANE, SDIO_SLEEP_CSR, SBSDIO_SLPCSR_KEEP_WL_KS as u8)
                .await;

            assert!(self.bus.read8(FUNC_BACKPLANE, SDIO_SLEEP_CSR).await & SBSDIO_SLPCSR_KEEP_WL_KS as u8 != 0);
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

        try_until(
            async || self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & 0x80 != 0,
            Duration::from_millis(500),
        )
        .await
        .ctx("timeout waiting for HT clock")?;

        debug!("clock ok");

        Ok(())
    }

    async fn init_cyw4373(&mut self) -> crate::Result<()> {
        let wakeup_ctrl = self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_WAKEUP_CTRL).await;
        self.bus
            .write8(
                FUNC_BACKPLANE,
                REG_BACKPLANE_WAKEUP_CTRL,
                wakeup_ctrl | SBSDIO_WCTRL_WL_WAKE_TILL_ALP_AVAIL,
            )
            .await;

        self.bus
            .write8(
                FUNC_BUS,
                SDIOD_CCCR_BRCM_CARDCAP,
                SDIOD_CCCR_BRCM_CARDCAP_CMD_NODEC as u8,
            )
            .await;

        let sleep_csr = self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_SLEEP_CSR).await;
        if sleep_csr & SBSDIO_SLPCSR_KEEP_WL_KS as u8 == 0 {
            self.bus
                .write8(
                    FUNC_BACKPLANE,
                    REG_BACKPLANE_SLEEP_CSR,
                    sleep_csr | SBSDIO_SLPCSR_KEEP_WL_KS as u8,
                )
                .await;
        }

        Ok(())
    }

    pub(crate) async fn init(
        &mut self,
        wifi_fw: &Aligned<A4, [u8]>,
        nvram: &Aligned<A4, [u8]>,
        bt_fw: Option<&[u8]>,
        config: &BUS::Config,
    ) -> crate::Result<()> {
        match self.chip.id() {
            ChipId::C43439 => debug!("using cyw43439"),
            ChipId::C4373 => debug!("using cyw43437"),
        }

        let bus_config = self.bus.init(bt_fw.is_some(), config).await?;

        // Validate type consistency
        assert!(
            (matches!(self.bus.bus_type(), BusType::Sdio) && matches!(bus_config, BusConfig::Sdio(_)))
                || (matches!(self.bus.bus_type(), BusType::Spi) && matches!(bus_config, BusConfig::Spi(_)))
        );

        // Init ALP (Active Low Power) clock
        debug!("init alp");
        match self.bus.bus_type() {
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
        try_until(
            async || self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & BACKPLANE_ALP_AVAIL != 0,
            Duration::from_millis(100),
        )
        .await
        .ctx("timeout while waiting for alp clock!")?;

        debug!("clock ok");

        // clear request for ALP
        debug!("clear request for ALP");
        self.bus.write8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR, 0).await;

        let chip_id = match bus_config {
            BusConfig::Spi(_) => self.bus.bp_read16(CHIPCOMMON_BASE_ADDRESS).await,
            BusConfig::Sdio(config) => self.read_chip_id_sdio(config).await,
        };

        debug!("chip ID: {}", chip_id);

        if self.chip.id() != chip_id {
            warn!("chip ID does not match!");
        }

        let ram_addr = self.chip.atcm_ram_base_address();
        if ram_addr != 0 && matches!(self.bus.bus_type(), BusType::Sdio) {
            reset_core(&mut self.bus, self.chip, Core::WLAN, true, true).await?;
        } else {
            disable_device_core(&mut self.bus, self.chip, Core::WLAN, false).await?;
            disable_device_core(&mut self.bus, self.chip, Core::SOCSRAM, false).await?; // TODO: is this needed if we reset right after?
            reset_device_core(&mut self.bus, self.chip, Core::SOCSRAM, false).await?;

            chip_specific_socsram_init(&mut self.bus, self.chip).await?;
        }

        debug!("loading fw");
        self.bus.bp_write(ram_addr, wifi_fw).await.ctx("failed to write fw")?;
        self.verify_download("FW", ram_addr, wifi_fw).await?;
        if ram_addr != 0 {
            self.write_reset_instruction(wifi_fw).await?;
        }

        debug!("loading nvram");
        let nvram_len = (nvram.len() + 3) / 4 * 4;
        let nvram_addr = ram_addr + self.chip.chip_ram_size() - 4 - nvram_len as u32;
        self.bus
            .bp_write(nvram_addr, nvram)
            .await
            .ctx("failed to write nvram")?;
        self.verify_download("NVRAM", nvram_addr, nvram).await?;

        let nvram_len_words = nvram_len as u32 / 4;
        let nvram_len_magic = (!nvram_len_words << 16) | nvram_len_words;
        let magic_addr = ram_addr + self.chip.chip_ram_size() - 4;
        self.bus.bp_write32(magic_addr, nvram_len_magic).await;

        // Verify the magic word was written correctly.  A failed backplane write
        // (e.g. wrong window or alignment bug) would leave stale data and the
        // firmware would not find the NVRAM, causing F2 IORDY to never be set.
        let magic_rb = self.bus.bp_read32(magic_addr).await;
        debug!(
            "bp_write addr = {:08x}, len = {}  magic_addr={:08x} magic={:08x} readback={:08x} match={}",
            nvram_addr,
            nvram.len(),
            magic_addr,
            nvram_len_magic,
            magic_rb,
            magic_rb == nvram_len_magic,
        );
        if magic_rb != nvram_len_magic {
            return err!("NVRAM magic write FAILED — firmware cannot find NVRAM");
        }

        // Also read back the first 4 bytes of the NVRAM area to confirm the
        // NVRAM data write succeeded (first bytes of NVRAM should be 'N','V','R','A').
        let nvram_first_word = self.bus.bp_read32(nvram_addr).await;
        debug!(
            "NVRAM first word readback = {:08x} (expect 0x4152564e for 'NVRA' in LE)",
            nvram_first_word
        );

        debug!("starting up core...");
        if ram_addr != 0 && matches!(self.bus.bus_type(), BusType::Sdio) {
            reset_core(&mut self.bus, self.chip, Core::WLAN, false, false).await?;
        } else {
            reset_device_core(&mut self.bus, self.chip, Core::WLAN, false).await?;
            check_device_core_is_up(&mut self.bus, self.chip, Core::WLAN).await?;
        }

        debug!("waiting for HT clock...");
        try_until(
            async || self.bus.read8(FUNC_BACKPLANE, REG_BACKPLANE_CHIP_CLOCK_CSR).await & 0x80 != 0,
            Duration::from_millis(500),
        )
        .await
        .ctx("Timeout waiting for HT clock")?;

        // "Set up the interrupt mask and enable interrupts"
        debug!("setup interrupt mask");
        self.bus
            .bp_write32(self.chip.sdiod_core_base_address() + SDIO_INT_HOST_MASK, I_HMB_SW_MASK)
            .await;

        match self.bus.bus_type() {
            BusType::Sdio => {
                self.bus
                    .bp_write8(self.chip.sdiod_core_base_address() + SDIO_FUNCTION_INT_MASK, 2 | 1)
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
                        .bp_write32(
                            self.chip.sdiod_core_base_address() + SDIO_INT_HOST_MASK,
                            I_HMB_FC_CHANGE,
                        )
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
        try_until(
            async || match self.bus.bus_type() {
                BusType::Sdio => self.bus.read8(FUNC_BUS, SDIOD_CCCR_IORDY).await as u32 & SDIO_FUNC_READY_2 != 0,
                BusType::Spi => self.bus.read32(FUNC_BUS, REG_BUS_STATUS).await & STATUS_F2_RX_READY != 0,
            },
            Duration::from_millis(1000),
        )
        .await
        .ctx("timeout while waiting for function 2 to be ready")?;

        match self.chip.id() {
            ChipId::C4373 => self.init_cyw4373().await?,
            ChipId::C43439 => self.init_cyw43439().await?,
        }

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

        let addr = self.chip.atcm_ram_base_address() + self.chip.chip_ram_size() - 4 - self.chip.socram_srmem_size();
        let shared_addr = self.bus.bp_read32(addr).await;
        debug!("shared_addr {:08x}", shared_addr);

        let mut shared: Aligned<A4, [u8; _]> = Aligned([0; SharedMemData::SIZE]);
        let _ = self.bus.bp_read(shared_addr, &mut shared[..]).await;
        let shared = SharedMemData::from_bytes(&shared);

        self.log.addr = shared.console_addr + 8;
    }

    #[cfg(feature = "firmware-logs")]
    async fn log_read(&mut self) {
        // Read log struct
        let mut log: Aligned<A4, [u8; _]> = Aligned([0; SharedMemLog::SIZE]);
        let _ = self.bus.bp_read(self.log.addr, &mut log[..]).await;
        let log = SharedMemLog::from_bytes(&log);

        let idx = log.idx as usize;

        // If pointer hasn't moved, no need to do anything.
        if idx == self.log.last_idx {
            return;
        }

        // Read entire buf for now. We could read only what we need, but then we
        // run into annoying alignment issues in `bp_read`.
        let mut buf: Aligned<A4, [u8; _]> = Aligned([0; 0x400]);
        let _ = self.bus.bp_read(log.buf, &mut buf[..]).await;

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
                        Some(bt) => {
                            let _ = bt.tx_chan.receive().await;
                        }
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
                            .copy_from_slice(&*packet);

                        let total_len = (total_len + 3) & !3; // round up to 4byte

                        trace!("    {:02x}", Bytes(&buf8[..total_len.min(48)]));

                        let _ = wlan_write(&mut self.bus, &aligned_ref(&buf)[..total_len]).await;
                        packet.tx_done();
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
                if matches!(self.bus.bus_type(), BusType::Sdio) {
                    // whd_bus_sdio_poke_wlan
                    self.bus
                        .bp_write32(self.chip.sdiod_core_base_address() + SDIO_TO_SB_MAILBOX, SMB_DEV_INT)
                        .await;
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
                let irq = self
                    .bus
                    .bp_read32(self.chip.sdiod_core_base_address() + SDIO_INT_STATUS)
                    .await;

                let mut irq = irq;
                if irq & I_HMB_HOST_INT != 0 {
                    let hmb_data = self
                        .bus
                        .bp_read32(self.chip.sdiod_core_base_address() + SDIO_TO_HOST_MAILBOX_DATA)
                        .await;
                    if hmb_data != 0 {
                        self.bus
                            .bp_write32(self.chip.sdiod_core_base_address() + SDIO_TO_SB_MAILBOX, SMB_INT_ACK)
                            .await;
                        trace!("hmb ack");
                    }

                    if hmb_data & I_HMB_DATA_FWHALT != 0 {
                        debug!("hmb data fault");
                    }

                    self.bus
                        .bp_write32(self.chip.sdiod_core_base_address() + SDIO_INT_STATUS, I_HMB_HOST_INT)
                        .await;
                    irq &= !I_HMB_HOST_INT;
                }

                if irq & FRAME_AVAILABLE_MASK != 0 {
                    self.check_status(buf).await;
                }

                // Clear all observed interrupt bits after consuming available frames.
                // Previous code only cleared `irq & HOSTINTMASK` (0xF0), which left
                // CYW4373-specific bits (e.g. bit 17 = 0x20000) permanently set,
                // causing an infinite poll loop with no forward progress.
                // Writing the full `irq` (with I_HMB_HOST_INT already masked out by
                // its handler above) is safe: INT_STATUS is write-1-to-clear, so we
                // only clear the bits we actually read at the top of this function.
                if irq != 0 {
                    trace!("clear irq {:08x}", irq);
                    self.bus
                        .bp_write32(self.chip.sdiod_core_base_address() + SDIO_INT_STATUS, irq)
                        .await;
                }
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
            match self.bus.bus_type() {
                BusType::Spi => {
                    let status = self.bus.read32(FUNC_BUS, SPI_STATUS_REGISTER).await;
                    trace!("check status{}", FormatStatus(status));

                    if status & STATUS_F2_PKT_AVAILABLE != 0 {
                        let len = (status & STATUS_F2_PKT_LEN_MASK) >> STATUS_F2_PKT_LEN_SHIFT;
                        if wlan_read(&mut self.bus, &mut aligned_mut(buf)[..len as usize])
                            .await
                            .is_err()
                        {
                            debug!("spi wlan_read failed");
                            break;
                        }
                        trace!("rx {:02x}", Bytes(&slice8_mut(buf)[..(len as usize).min(48)]));
                        self.rx(&mut slice8_mut(buf)[..len as usize]);
                    } else {
                        break;
                    }
                }
                BusType::Sdio => {
                    if wlan_read(&mut self.bus, &mut aligned_mut(&mut buf[..1])).await.is_err() {
                        debug!("failed to read sdio hwtag");
                        break;
                    }
                    let (len, len_inv) = {
                        let hwtag = slice16_mut(&mut buf[..1]);

                        (hwtag[0], hwtag[1])
                    };

                    if (len | len_inv) == 0 || (len ^ len_inv) != 0xFFFF {
                        trace!("hwtag mismatch (hwtag[0] = {}, hwtag[1] = {})", len, len_inv);
                        break;
                    }

                    trace!("pkt ready...");
                    let len = len as usize;
                    if len > INITIAL_READ as usize {
                        if self
                            .bus
                            .wlan_read(&mut aligned_mut(&mut buf[1..])[..len - INITIAL_READ as usize])
                            .await
                            .is_err()
                        {
                            debug!("failed to read sdio payload, len={}", len);
                            break;
                        }
                    } else {
                        // TODO: investigate this condition
                        trace!("no extra space required");
                        continue;
                    }

                    if len == SdpcmHeader::SIZE {
                        let Some((sdpcm_header, _)) = SdpcmHeader::parse(slice8_mut(&mut buf[..3])) else {
                            debug!("failed to parse sdpcm header");
                            break;
                        };

                        self.update_credit(&sdpcm_header);
                    } else if len > SdpcmHeader::SIZE {
                        trace!("rx {:02x}", Bytes(&slice8_mut(buf)[..len.min(48)]));
                        self.rx(&mut slice8_mut(buf)[..len]);
                    }
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
                        warn!("IOCTL error {}", cdc_header.status as i32);
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
                    Some(mut buf) => {
                        buf[..packet.len()].copy_from_slice(packet);
                        buf.rx_done(packet.len())
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

        let total_len = (total_len + 3) & !3; // round up to 4byte,
        trace!("    {:02x}", Bytes(&buf8[..total_len.min(48)]));

        let _ = wlan_write(&mut self.bus, &aligned_ref(buf)[..total_len]).await;
    }
}
