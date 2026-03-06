use core::mem;

use aligned::{A4, Aligned};
use embassy_time::{Delay, Duration, Timer};

use crate::WithContext;
use crate::consts::*;
use crate::runner::{BusType, SealedBus};
use crate::util::try_until;

#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "log", derive(derive_more::Display))]
enum Word {
    U8,
    U16,
    U32,
}

const BLOCK_SIZE: usize = BACKPLANE_MAX_TRANSFER_SIZE;

fn to_aligned<'a>(data: &'a [u8], buf: &'a mut Aligned<A4, [u8]>) -> &'a Aligned<A4, [u8]> {
    if (data.as_ptr() as usize).is_multiple_of(mem::align_of::<A4>()) {
        unsafe { &*(data as *const [u8] as *const Aligned<A4, [u8]>) }
    } else {
        buf[..data.len()].copy_from_slice(data);

        &buf[..data.len()]
    }
}

async fn with_aligned<'a, R>(
    data: &'a mut [u8],
    buf: &'a mut Aligned<A4, [u8]>,
    mut f: impl AsyncFnMut(&mut Aligned<A4, [u8]>) -> R,
) -> R {
    let ptr = data.as_mut_ptr();
    let is_aligned = (ptr as usize).is_multiple_of(align_of::<A4>());

    if is_aligned {
        // SAFETY: data is aligned to A4, and Aligned<A4, [u8]> is repr(transparent)
        f(unsafe { &mut *(data as *mut [u8] as *mut Aligned<A4, [u8]>) }).await
    } else {
        let ret = f(&mut buf[..data.len()]).await;

        data.copy_from_slice(&buf[..data.len()]);
        ret
    }
}

pub struct Config {
    pub max_f: u32,
}

/// Doc
pub struct SdioBus<SDIO>
where
    SDIO: ::sdio::MmcBus,
{
    backplane_window: u32,
    sdio: ::sdio::sdio::SdioCard<SDIO, Delay>,
}

impl<SDIO> SdioBus<SDIO>
where
    SDIO: ::sdio::MmcBus,
{
    pub(crate) fn new(sdio: SDIO) -> Self {
        Self {
            backplane_window: 0xAAAA_AAAA,
            sdio: ::sdio::sdio::SdioCard::new_uninit(sdio, Delay),
        }
    }

    async fn backplane_readn(&mut self, addr: u32, word: Word) -> u32 {
        trace!("backplane_readn addr = {:08x} len = {}", addr, word);

        self.backplane_set_window(addr).await;

        let mut bus_addr = addr & BACKPLANE_ADDRESS_MASK;
        if word == Word::U32 {
            bus_addr |= BACKPLANE_ADDRESS_32BIT_FLAG;
        }

        let val = match word {
            Word::U8 => self.read8(FUNC_BACKPLANE, bus_addr).await as u32,
            Word::U16 => self.read16(FUNC_BACKPLANE, bus_addr).await as u32,
            Word::U32 => self.read32(FUNC_BACKPLANE, bus_addr).await,
        };

        trace!("backplane_readn addr = {:08x} len = {} val = {:08x}", addr, word, val);

        self.backplane_set_window(CHIPCOMMON_BASE_ADDRESS).await;

        val
    }

    async fn backplane_writen(&mut self, addr: u32, val: u32, word: Word) {
        trace!("backplane_writen addr = {:08x} len = {} val = {:08x}", addr, word, val);

        self.backplane_set_window(addr).await;

        let mut bus_addr = addr & BACKPLANE_ADDRESS_MASK;
        if word == Word::U32 {
            bus_addr |= BACKPLANE_ADDRESS_32BIT_FLAG;
        }

        match word {
            Word::U8 => self.write8(FUNC_BACKPLANE, bus_addr, val.try_into().unwrap()).await,
            Word::U16 => self.write16(FUNC_BACKPLANE, bus_addr, val.try_into().unwrap()).await,
            Word::U32 => self.write32(FUNC_BACKPLANE, bus_addr, val).await,
        };

        self.backplane_set_window(CHIPCOMMON_BASE_ADDRESS).await;
    }

    async fn backplane_set_window(&mut self, addr: u32) {
        let new_window = addr & !BACKPLANE_ADDRESS_MASK;

        if (new_window >> 24) as u8 != (self.backplane_window >> 24) as u8 {
            self.write8(
                FUNC_BACKPLANE,
                REG_BACKPLANE_BACKPLANE_ADDRESS_HIGH,
                (new_window >> 24) as u8,
            )
            .await;
        }
        if (new_window >> 16) as u8 != (self.backplane_window >> 16) as u8 {
            self.write8(
                FUNC_BACKPLANE,
                REG_BACKPLANE_BACKPLANE_ADDRESS_MID,
                (new_window >> 16) as u8,
            )
            .await;
        }
        if (new_window >> 8) as u8 != (self.backplane_window >> 8) as u8 {
            self.write8(
                FUNC_BACKPLANE,
                REG_BACKPLANE_BACKPLANE_ADDRESS_LOW,
                (new_window >> 8) as u8,
            )
            .await;
        }
        self.backplane_window = new_window;
    }

    async fn cmd53_write(&mut self, func: u8, addr: u32, buf: &Aligned<A4, [u8]>) -> crate::Result<()> {
        self.sdio
            .cmd53_write::<BLOCK_SIZE>(func, addr, buf)
            .await
            .map_err(|_| crate::Error)
            .ctx("cmd53 write failed")
    }

    async fn cmd53_read(&mut self, func: u8, addr: u32, buf: &mut Aligned<A4, [u8]>) -> crate::Result<()> {
        self.sdio
            .cmd53_read::<BLOCK_SIZE>(func, addr, buf)
            .await
            .map_err(|_| crate::Error)
            .ctx("cmd53 read failed")
    }
}

impl<SDIO> SealedBus for SdioBus<SDIO>
where
    SDIO: ::sdio::MmcBus,
{
    const TYPE: BusType = BusType::Sdio;
    type Config = Config;

    async fn init<'a>(&mut self, _bluetooth_enabled: bool, config: &'a Config) -> crate::Result<()> {
        // acquire the bus
        self.sdio.reacquire(config.max_f).await.map_err(|_| crate::Error)?;

        // whd_bus_sdio_init

        // set up backplane
        try_until(
            async || {
                self.write8(FUNC_BUS, SDIOD_CCCR_IOEN, SDIO_FUNC_ENABLE_1 as u8).await;

                self.read8(FUNC_BUS, SDIOD_CCCR_IOEN).await as u32 == SDIO_FUNC_ENABLE_1
            },
            Duration::from_millis(500),
        )
        .await
        .ctx("timeout while setting up the backplane")?;

        debug!("backplane is up");

        // Set the block size
        try_until(
            async || {
                self.write8(FUNC_BUS, SDIOD_CCCR_BLKSIZE_0, SDIO_64B_BLOCK as u8).await;

                self.read8(FUNC_BUS, SDIOD_CCCR_BLKSIZE_0).await as u32 == SDIO_64B_BLOCK
            },
            Duration::from_millis(500),
        )
        .await
        .ctx("timeout while setting block size")?;

        self.write8(FUNC_BUS, SDIOD_CCCR_BLKSIZE_0, SDIO_64B_BLOCK as u8).await;
        self.write8(FUNC_BUS, SDIOD_CCCR_F1BLKSIZE_0, SDIO_64B_BLOCK as u8)
            .await;
        self.write8(FUNC_BUS, SDIOD_CCCR_F2BLKSIZE_0, SDIO_64B_BLOCK as u8)
            .await;
        self.write8(FUNC_BUS, SDIOD_CCCR_F2BLKSIZE_1, 0).await;

        // Enable/Disable Client interrupts
        self.write8(
            FUNC_BUS,
            SDIOD_CCCR_INTEN,
            (INTR_CTL_MASTER_EN | INTR_CTL_FUNC1_EN | INTR_CTL_FUNC2_EN) as u8,
        )
        .await;

        // Wait till the backplane is ready
        try_until(
            async || self.read8(FUNC_BUS, SDIOD_CCCR_IORDY).await as u32 & SDIO_FUNC_READY_1 != 0,
            Duration::from_millis(500),
        )
        .await
        .ctx("timeout while waiting for backplane to be ready")?;

        Ok(())
    }

    async fn wlan_read(&mut self, buf: &mut Aligned<A4, [u8]>) -> crate::Result<()> {
        if let Err(e) = self.cmd53_read(FUNC_WLAN, 0, buf).await {
            buf.fill(0);
            // A timed-out partial F2 read leaves the same packet pending forever.
            // Mirror WHD's abort path so the device can reset its F2 read state.
            self.write8(FUNC_BUS, SDIOD_CCCR_IOABORT, FUNC_WLAN).await;
            self.write8(FUNC_BACKPLANE, REG_BACKPLANE_FRAME_CONTROL, SFC_RF_TERM)
                .await;

            Err(e)
        } else {
            Ok(())
        }
    }

    async fn wlan_write(&mut self, buf: &mut Aligned<A4, [u8]>) -> crate::Result<()> {
        self.cmd53_write(FUNC_WLAN, 0, &buf[4..]).await
    }

    async fn bp_read(&mut self, mut addr: u32, mut data: &mut [u8], buf: &mut Aligned<A4, [u8]>) -> crate::Result<()> {
        trace!("bp_read addr = {:08x}, len = {}", addr, data.len());

        // It seems the HW force-aligns the addr
        // to 2 if data.len() >= 2
        // to 4 if data.len() >= 4
        // To simplify, enforce 4-align for now.
        assert!(addr.is_multiple_of(4));

        loop {
            // Ensure transfer doesn't cross a window boundary.
            let window_offs = addr & BACKPLANE_ADDRESS_MASK;
            let window_remaining = BACKPLANE_WINDOW_SIZE - window_offs as usize;
            let len = data.len().min(BLOCK_BUFFER_SIZE).min(window_remaining);

            self.backplane_set_window(addr).await;

            with_aligned(&mut data[..len], buf, async |buf| {
                self.cmd53_read(FUNC_BACKPLANE, addr & BACKPLANE_ADDRESS_MASK, buf)
                    .await
            })
            .await?;

            // Advance ptr.
            addr += len as u32;
            if data.len() == len {
                break;
            } else {
                data = &mut data[len..];
            }
        }

        self.backplane_set_window(CHIPCOMMON_BASE_ADDRESS).await;

        Ok(())
    }

    async fn bp_write(&mut self, mut addr: u32, mut data: &[u8], buf: &mut Aligned<A4, [u8]>) -> crate::Result<()> {
        trace!("bp_write addr = {:08x}, len = {}", addr, data.len());

        // It seems the HW force-aligns the addr
        // to 2 if data.len() >= 2
        // to 4 if data.len() >= 4
        // To simplify, enforce 4-align for now.
        assert!(addr.is_multiple_of(4));

        loop {
            // Ensure transfer doesn't cross a window boundary.
            let window_offs = addr & BACKPLANE_ADDRESS_MASK;
            let window_remaining = BACKPLANE_WINDOW_SIZE - window_offs as usize;
            let len = data.len().min(BLOCK_BUFFER_SIZE).min(window_remaining);

            self.backplane_set_window(addr).await;

            self.cmd53_write(
                FUNC_BACKPLANE,
                addr & BACKPLANE_ADDRESS_MASK,
                to_aligned(&data[..len], buf),
            )
            .await?;

            // Advance ptr.
            addr += len as u32;
            if data.len() == len {
                break;
            } else {
                data = &data[len..];
            }
        }

        self.backplane_set_window(CHIPCOMMON_BASE_ADDRESS).await;

        Ok(())
    }

    async fn bp_read8(&mut self, addr: u32) -> u8 {
        self.backplane_readn(addr, Word::U8).await as u8
    }

    async fn bp_write8(&mut self, addr: u32, val: u8) {
        self.backplane_writen(addr, val as u32, Word::U8).await
    }

    async fn bp_read16(&mut self, addr: u32) -> u16 {
        self.backplane_readn(addr, Word::U16).await as u16
    }

    #[allow(unused)]
    async fn bp_write16(&mut self, addr: u32, val: u16) {
        self.backplane_writen(addr, val as u32, Word::U16).await
    }

    #[allow(unused)]
    async fn bp_read32(&mut self, addr: u32) -> u32 {
        self.backplane_readn(addr, Word::U32).await
    }

    async fn bp_write32(&mut self, addr: u32, val: u32) {
        self.backplane_writen(addr, val, Word::U32).await
    }

    async fn read8(&mut self, func: u8, addr: u32) -> u8 {
        self.sdio.cmd52_read(func, addr).await.unwrap_or_default()
    }

    async fn write8(&mut self, func: u8, addr: u32, val: u8) {
        let _ = self.sdio.cmd52_write(func, addr, val).await;
    }

    async fn read16(&mut self, func: u8, addr: u32) -> u16 {
        let mut val: Aligned<A4, [u8; _]> = Aligned([0u8; 2]);
        let _ = self.cmd53_read(func, addr, &mut val).await;

        u16::from_le_bytes(*val)
    }

    async fn write16(&mut self, func: u8, addr: u32, val: u16) {
        let val: Aligned<A4, [u8; 2]> = Aligned(val.to_le_bytes());
        let _ = self.cmd53_write(func, addr, &val).await;
    }

    async fn read32(&mut self, func: u8, addr: u32) -> u32 {
        let mut val: Aligned<A4, [u8; _]> = Aligned([0u8; 4]);
        let _ = self.cmd53_read(func, addr, &mut val).await;

        u32::from_le_bytes(*val)
    }

    async fn write32(&mut self, func: u8, addr: u32, val: u32) {
        let val: Aligned<A4, [u8; 4]> = Aligned(val.to_le_bytes());
        let _ = self.cmd53_write(func, addr, &val).await;
    }

    async fn wait_for_event(&mut self) {
        Timer::after(Duration::from_millis(10)).await;
        // self.sdio.wait_for_event().await;
    }
}
