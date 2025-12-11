use core::slice;

use aligned::Aligned;
use embassy_futures::yield_now;
use embassy_time::{Duration, Timer};
use embedded_hal_1::digital::OutputPin;

use crate::consts::*;
use crate::runner::SealedBus;

macro_rules! ALIGN_UINT {
    ($val:expr, $align:expr) => {
        ((($val) + ($align) - 1) & !(($align) - 1))
    };
}

macro_rules! WRITE_BYTES_PAD {
    ($len:expr) => {
        ALIGN_UINT!($len, 64)
    };
}

/// Custom Spi Trait that _only_ supports the bus operation of the cyw43
/// Implementors are expected to hold the CS pin low during an operation.
pub trait SdioBusCyw43<const SIZE: usize> {
    /// The error type for the BlockDevice implementation.
    type Error: core::fmt::Debug;

    /// The alignment requirements of the block buffers.
    type Align: aligned::Alignment;

    /// Doc
    fn set_bus_to_high_speed(&mut self, frequency: u32) -> Result<(), Self::Error>;

    /// Doc
    async fn cmd52(&mut self, arg: u32) -> Result<u32, Self::Error>;

    /// Doc
    async fn cmd53_block_read(
        &mut self,
        arg: u32,
        blocks: &mut [Aligned<Self::Align, [u8; SIZE]>],
    ) -> Result<(), Self::Error>;

    /// Doc
    async fn cmd53_byte_read(&mut self, arg: u32, buffer: &mut [u32]) -> Result<(), Self::Error>;

    /// Doc
    async fn cmd53_block_write(
        &mut self,
        arg: u32,
        blocks: &[Aligned<Self::Align, [u8; SIZE]>],
    ) -> Result<(), Self::Error>;

    /// Doc
    async fn cmd53_byte_write(&mut self, arg: u32, buffer: &[u32]) -> Result<(), Self::Error>;

    /// Wait for events from the Device. A typical implementation would wait for the IRQ pin to be high.
    /// The default implementation always reports ready, resulting in active polling of the device.
    async fn wait_for_event(&mut self) {
        yield_now().await;
    }
}

/// Doc
pub struct SdioBus<PWR, SDIO> {
    backplane_window: u32,
    _pwr: PWR,
    sdio: SDIO,
    status: u32,
}

impl<PWR, SDIO> SdioBus<PWR, SDIO>
where
    PWR: OutputPin,
    SDIO: SdioBusCyw43<64>,
{
    pub(crate) fn new(_pwr: PWR, sdio: SDIO) -> Self {
        Self {
            backplane_window: 0xAAAA_AAAA,
            _pwr,
            sdio,
            status: 0,
        }
    }

    async fn backplane_readn(&mut self, addr: u32, size: u32) -> u32 {
        trace!("backplane_readn addr = {:08x} len = {}", addr, size);

        self.backplane_set_window(addr).await;

        let mut bus_addr = addr & BACKPLANE_ADDRESS_MASK;
        if size == 4 {
            bus_addr |= BACKPLANE_ADDRESS_32BIT_FLAG;
        }

        let val = self.read_reg(FUNC_BACKPLANE, bus_addr, size as usize).await;

        trace!("backplane_readn addr = {:08x} len = {} val = {:08x}", addr, size, val);

        self.backplane_set_window(CHIPCOMMON_BASE_ADDRESS).await;

        return val;
    }

    async fn backplane_writen(&mut self, addr: u32, val: u32, size: u32) {
        trace!("backplane_writen addr = {:08x} len = {} val = {:08x}", addr, size, val);

        self.backplane_set_window(addr).await;

        let mut bus_addr = addr & BACKPLANE_ADDRESS_MASK;
        if size == 4 {
            bus_addr |= BACKPLANE_ADDRESS_32BIT_FLAG;
        }

        self.write_reg(FUNC_BACKPLANE, bus_addr, size as usize, val).await;

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

    async fn read_reg(&mut self, func: u32, reg: u32, size: usize) -> u32 {
        assert!(func == BACKPLANE_FUNCTION);

        if size == 1 {
            self.read_reg_u8(func, reg).await as u32
        } else {
            assert!(size == 4);

            self.read_reg_u32(func, reg).await
        }
    }

    async fn write_reg(&mut self, func: u32, reg: u32, size: usize, val: u32) {
        assert!(func == BACKPLANE_FUNCTION);

        if size == 1 {
            self.write_reg_u8(func, reg, val).await as u32;
        } else {
            assert!(size == 4);

            self.write_reg_u32(func, reg, val).await;
        }
    }

    async fn cmd52(&mut self, write: bool, func: u32, addr: u32, val: u32) -> isize {
        let arg: u32 = func << 28 | (addr & 0x1ffff) << 9 | (write as u32) << 31 | (val & 0xff);

        let ret = self.sdio.cmd52(arg).await.unwrap_or(u32::MAX) as isize;
        if ret != 0 {
            return ret;
        }

        return (ret & 0xff) as isize;
    }

    fn cmd53_arg(&mut self, write: bool, func: u32, addr: u32, mut len: u32) -> u32 {
        let block_size: u32;
        let block_mode: u32;
        if len <= 64 {
            // SDIO_BYTE_MODE (can go up to 512 bytes)
            // in this case the SDIO chuck of data must be a single block of the length of buf
            // block_size = len as u32;
            block_mode = 0;
        } else {
            // looks like block_size must be 64
            block_size = 64;
            block_mode = 1 << 27;
            len /= block_size;
        }

        func << 28 | block_mode | 1 << 26 | (addr & 0x1ffff) << 9 | (write as u32) << 31 | len as u32
    }

    async fn cmd53_write_half_word(&mut self, func: u32, addr: u32, buf: &[u32; 1]) -> isize {
        let arg = self.cmd53_arg(false, func, addr, 2);

        let _res = self.sdio.cmd53_byte_write(arg, buf).await;

        0
    }

    async fn cmd53_write(&mut self, func: u32, addr: u32, buf: &[u32]) -> isize {
        let arg = self.cmd53_arg(true, func, addr, size_of_val(buf) as u32);

        let _res = if size_of_val(buf) <= 64 {
            self.sdio.cmd53_byte_write(arg, buf).await
        } else {
            self.sdio
                .cmd53_block_write(arg, unsafe {
                    slice::from_raw_parts(buf.as_ptr() as *const _, buf.len() / (64 * size_of::<u32>()))
                })
                .await
        };

        0
    }

    async fn cmd53_read_half_word(&mut self, func: u32, addr: u32, buf: &mut [u32; 1]) -> isize {
        let arg = self.cmd53_arg(false, func, addr, 2);

        let _res = self.sdio.cmd53_byte_read(arg, buf).await;

        0
    }

    async fn cmd53_read(&mut self, func: u32, addr: u32, buf: &mut [u32]) -> isize {
        let arg = self.cmd53_arg(false, func, addr, size_of_val(buf) as u32);

        let _res = if size_of_val(buf) <= 64 {
            self.sdio.cmd53_byte_read(arg, buf).await
        } else {
            self.sdio
                .cmd53_block_read(arg, unsafe {
                    slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut _, buf.len() / (64 * size_of::<u32>()))
                })
                .await
        };

        0
    }

    async fn read_words(&mut self, func: u32, addr: u32, buf: &mut [u32]) -> isize {
        self.cmd53_read(func, addr, buf).await
    }

    async fn write_words(&mut self, func: u32, addr: u32, buf: &[u32]) -> isize {
        self.cmd53_write(func, addr, buf).await
    }

    async fn read_reg_u8(&mut self, func: u32, reg: u32) -> isize {
        self.cmd52(false, func, reg, 0).await
    }

    async fn read_reg_u32(&mut self, func: u32, reg: u32) -> u32 {
        let mut val = [0u32];
        self.cmd53_read(func, reg, &mut val).await;

        val[0]
    }

    async fn write_reg_u8(&mut self, func: u32, reg: u32, val: u32) -> isize {
        self.cmd52(true, func, reg, val).await
    }

    async fn write_reg_u32(&mut self, func: u32, reg: u32, val: u32) -> isize {
        self.cmd53_write(func, reg, &[val]).await
    }
}

impl<PWR, SDIO> SealedBus for SdioBus<PWR, SDIO>
where
    PWR: OutputPin,
    SDIO: SdioBusCyw43<64>,
{
    async fn init(&mut self, _bluetooth_enabled: bool) {
        // set up backplane
        let mut reg: u32 = 0;
        for i in 0..100 {
            self.write_reg_u8(BUS_FUNCTION, SDIOD_CCCR_IOEN, SDIO_FUNC_ENABLE_1)
                .await;
            if i != 0 {
                Timer::after_millis(1).await;
            }

            reg = self.read_reg_u8(BUS_FUNCTION, SDIOD_CCCR_IOEN).await as u32;
            if reg == SDIO_FUNC_ENABLE_1 {
                break;
            }
        }

        if reg != SDIO_FUNC_ENABLE_1 {
            trace!("no response from CYW43\n");
            return;
        }

        trace!("backplane is up");

        // set the bus to 4-bits
        // (we don't need to change our local SDIO config until we need cmd53)
        let reg = self.read_reg_u8(BUS_FUNCTION, SDIOD_CCCR_BICTRL).await as u32;

        self.write_reg_u8(BUS_FUNCTION, SDIOD_CCCR_BICTRL, (reg & !3) | 2).await;

        // set the block size
        self.write_reg_u8(BUS_FUNCTION, SDIOD_CCCR_BLKSIZE_0, SDIO_64B_BLOCK)
            .await;
        let reg = self.read_reg_u8(BUS_FUNCTION, SDIOD_CCCR_BLKSIZE_0).await as u32;
        if reg != SDIO_64B_BLOCK {
            trace!("can't set block size\n");

            return;
        }

        self.write_reg_u8(BUS_FUNCTION, SDIOD_CCCR_BLKSIZE_0, SDIO_64B_BLOCK)
            .await;
        self.write_reg_u8(BUS_FUNCTION, SDIOD_CCCR_F1BLKSIZE_0, SDIO_64B_BLOCK)
            .await;
        self.write_reg_u8(BUS_FUNCTION, SDIOD_CCCR_F2BLKSIZE_0, SDIO_64B_BLOCK)
            .await;
        self.write_reg_u8(BUS_FUNCTION, SDIOD_CCCR_F2BLKSIZE_1, 0).await;

        // Enable/Disable Client interrupts
        self.write_reg_u8(
            BUS_FUNCTION,
            SDIOD_CCCR_INTEN,
            INTR_CTL_MASTER_EN | INTR_CTL_FUNC1_EN | INTR_CTL_FUNC2_EN,
        )
        .await;

        let _ = self.sdio.set_bus_to_high_speed(25_000_000);

        // enable more than 25MHz bus
        let reg = self.read_reg_u8(BUS_FUNCTION, SDIOD_CCCR_SPEED_CONTROL).await as u32;
        if reg & 1 > 0 {
            self.write_reg_u8(BUS_FUNCTION, SDIOD_CCCR_SPEED_CONTROL, reg | 2).await;

            let _ = self.sdio.set_bus_to_high_speed(50_000_000);
        }

        // wait for backplane to be ready
        let mut reg: u32 = 0;
        for _ in 0..10 {
            reg = self.read_reg_u8(BUS_FUNCTION, SDIOD_CCCR_IORDY).await as u32;
            if (reg & SDIO_FUNC_READY_1) != 0 {
                break;
            }

            Timer::after(Duration::from_millis(1)).await;
        }

        if (reg & SDIO_FUNC_READY_1) == 0 {
            trace!("timeout waiting for backplane\n");

            return;
        }
    }

    async fn wlan_read(&mut self, buf: &mut [u32], len_in_u8: u32) {
        let buf = &mut buf[..(len_in_u8 as usize / size_of::<u32>())];

        assert!(buf.len() == WRITE_BYTES_PAD!(buf.len()));

        let buf_ptr = buf.as_mut_ptr();
        assert_eq!(buf_ptr.align_offset(4), 0);

        self.read_words(WLAN_FUNCTION, 0, unsafe {
            slice::from_raw_parts_mut(buf_ptr as *mut u32, buf.len() / size_of::<u32>())
        })
        .await;
    }

    async fn wlan_write(&mut self, buf: &[u32]) {
        self.write_words(WLAN_FUNCTION, 0, buf).await;
    }

    #[allow(unused)]
    async fn bp_read(&mut self, mut addr: u32, mut data: &mut [u8]) {
        unimplemented!()
    }

    /// A.K.A. cyw43_download_resource
    async fn bp_write(&mut self, addr: u32, data: &[u8]) {
        assert!(data.len() == WRITE_BYTES_PAD!(data.len()));

        trace!("bp_write addr = {:08x}", addr);

        // It seems the HW force-aligns the addr
        // to 2 if data.len() >= 2
        // to 4 if data.len() >= 4
        // To simplify, enforce 4-align for now.
        assert!(addr % 4 == 0);

        let mut offset = 0usize;
        for chunk in data.chunks(BACKPLANE_MAX_TRANSFER_SIZE) {
            let mut dest_addr = addr as usize + offset;

            assert!(
                ((dest_addr & BACKPLANE_ADDRESS_MASK as usize) + chunk.len()) <= (BACKPLANE_ADDRESS_MASK as usize + 1)
            );
            self.backplane_set_window(dest_addr as u32).await;

            dest_addr &= BACKPLANE_ADDRESS_MASK as usize;

            let chunk_ptr = chunk.as_ptr();
            assert_eq!(chunk_ptr.align_offset(4), 0);

            if self
                .write_words(BACKPLANE_FUNCTION, dest_addr as u32, unsafe {
                    slice::from_raw_parts(chunk_ptr as *const u32, chunk.len() / size_of::<u32>())
                })
                .await as u32
                != 0
            {
                trace!("bp write fail");

                return;
            }

            offset += chunk.len();
        }

        // TODO: implement verify download
    }

    async fn bp_read8(&mut self, addr: u32) -> u8 {
        self.backplane_readn(addr, 1).await as u8
    }

    async fn bp_write8(&mut self, addr: u32, val: u8) {
        self.backplane_writen(addr, val as u32, 1).await
    }

    async fn bp_read16(&mut self, addr: u32) -> u16 {
        self.backplane_readn(addr, 2).await as u16
    }

    #[allow(unused)]
    async fn bp_write16(&mut self, addr: u32, val: u16) {
        self.backplane_writen(addr, val as u32, 2).await
    }

    #[allow(unused)]
    async fn bp_read32(&mut self, addr: u32) -> u32 {
        self.backplane_readn(addr, 4).await
    }

    async fn bp_write32(&mut self, addr: u32, val: u32) {
        self.backplane_writen(addr, val, 4).await
    }

    async fn read8(&mut self, func: u32, addr: u32) -> u8 {
        self.read_reg_u8(func, addr).await as u8
    }

    async fn write8(&mut self, func: u32, addr: u32, val: u8) {
        self.write_reg_u8(func, addr, val as u32).await;
    }

    async fn read16(&mut self, func: u32, addr: u32) -> u16 {
        let mut val = [0u32];
        self.cmd53_read_half_word(func, addr, &mut val).await;

        u16::from_be_bytes(val[0].to_be_bytes()[..2].try_into().unwrap())
    }

    #[allow(unused)]
    async fn write16(&mut self, func: u32, addr: u32, val: u16) {
        self.cmd53_write_half_word(func, addr, &[val as u32]).await;
    }

    async fn read32(&mut self, func: u32, addr: u32) -> u32 {
        self.read_reg_u32(func, addr).await
    }

    #[allow(unused)]
    async fn write32(&mut self, func: u32, addr: u32, val: u32) {
        self.write_reg_u32(func, addr, val).await;
    }

    async fn wait_for_event(&mut self) {
        self.sdio.wait_for_event().await;
    }

    fn status(&self) -> u32 {
        self.status
    }
}
