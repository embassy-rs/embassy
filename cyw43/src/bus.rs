use embassy_futures::yield_now;
use embassy_time::Timer;
use embedded_hal_1::digital::OutputPin;
use futures::FutureExt;

use crate::consts::*;
use crate::slice8_mut;

/// Custom Spi Trait that _only_ supports the bus operation of the cyw43
/// Implementors are expected to hold the CS pin low during an operation.
pub trait SpiBusCyw43 {
    /// Issues a write command on the bus
    /// First 32 bits of `word` are expected to be a cmd word
    async fn cmd_write(&mut self, write: &[u32]) -> u32;

    /// Issues a read command on the bus
    /// `write` is expected to be a 32 bit cmd word
    /// `read` will contain the response of the device
    /// Backplane reads have a response delay that produces one extra unspecified word at the beginning of `read`.
    /// Callers that want to read `n` word from the backplane, have to provide a slice that is `n+1` words long.
    async fn cmd_read(&mut self, write: u32, read: &mut [u32]) -> u32;

    /// Wait for events from the Device. A typical implementation would wait for the IRQ pin to be high.
    /// The default implementation always reports ready, resulting in active polling of the device.
    async fn wait_for_event(&mut self) {
        yield_now().await;
    }
}

pub(crate) struct Bus<PWR, SPI> {
    backplane_window: u32,
    pwr: PWR,
    spi: SPI,
    status: u32,
}

impl<PWR, SPI> Bus<PWR, SPI>
where
    PWR: OutputPin,
    SPI: SpiBusCyw43,
{
    pub(crate) fn new(pwr: PWR, spi: SPI) -> Self {
        Self {
            backplane_window: 0xAAAA_AAAA,
            pwr,
            spi,
            status: 0,
        }
    }

    pub async fn init(&mut self) {
        // Reset
        self.pwr.set_low().unwrap();
        Timer::after_millis(20).await;
        self.pwr.set_high().unwrap();
        Timer::after_millis(250).await;

        while self
            .read32_swapped(REG_BUS_TEST_RO)
            .inspect(|v| trace!("{:#x}", v))
            .await
            != FEEDBEAD
        {}

        self.write32_swapped(REG_BUS_TEST_RW, TEST_PATTERN).await;
        let val = self.read32_swapped(REG_BUS_TEST_RW).await;
        trace!("{:#x}", val);
        assert_eq!(val, TEST_PATTERN);

        let val = self.read32_swapped(REG_BUS_CTRL).await;
        trace!("{:#010b}", (val & 0xff));

        // 32-bit word length, little endian (which is the default endianess).
        self.write32_swapped(
            REG_BUS_CTRL,
            WORD_LENGTH_32 | HIGH_SPEED | INTERRUPT_HIGH | WAKE_UP | STATUS_ENABLE | INTERRUPT_WITH_STATUS,
        )
        .await;

        let val = self.read8(FUNC_BUS, REG_BUS_CTRL).await;
        trace!("{:#b}", val);

        let val = self.read32(FUNC_BUS, REG_BUS_TEST_RO).await;
        trace!("{:#x}", val);
        assert_eq!(val, FEEDBEAD);
        let val = self.read32(FUNC_BUS, REG_BUS_TEST_RW).await;
        trace!("{:#x}", val);
        assert_eq!(val, TEST_PATTERN);
    }

    pub async fn wlan_read(&mut self, buf: &mut [u32], len_in_u8: u32) {
        let cmd = cmd_word(READ, INC_ADDR, FUNC_WLAN, 0, len_in_u8);
        let len_in_u32 = (len_in_u8 as usize + 3) / 4;

        self.status = self.spi.cmd_read(cmd, &mut buf[..len_in_u32]).await;
    }

    pub async fn wlan_write(&mut self, buf: &[u32]) {
        let cmd = cmd_word(WRITE, INC_ADDR, FUNC_WLAN, 0, buf.len() as u32 * 4);
        //TODO try to remove copy?
        let mut cmd_buf = [0_u32; 513];
        cmd_buf[0] = cmd;
        cmd_buf[1..][..buf.len()].copy_from_slice(buf);

        self.status = self.spi.cmd_write(&cmd_buf[..buf.len() + 1]).await;
    }

    #[allow(unused)]
    pub async fn bp_read(&mut self, mut addr: u32, mut data: &mut [u8]) {
        // It seems the HW force-aligns the addr
        // to 2 if data.len() >= 2
        // to 4 if data.len() >= 4
        // To simplify, enforce 4-align for now.
        assert!(addr % 4 == 0);

        // Backplane read buffer has one extra word for the response delay.
        let mut buf = [0u32; BACKPLANE_MAX_TRANSFER_SIZE / 4 + 1];

        while !data.is_empty() {
            // Ensure transfer doesn't cross a window boundary.
            let window_offs = addr & BACKPLANE_ADDRESS_MASK;
            let window_remaining = BACKPLANE_WINDOW_SIZE - window_offs as usize;

            let len = data.len().min(BACKPLANE_MAX_TRANSFER_SIZE).min(window_remaining);

            self.backplane_set_window(addr).await;

            let cmd = cmd_word(READ, INC_ADDR, FUNC_BACKPLANE, window_offs, len as u32);

            // round `buf` to word boundary, add one extra word for the response delay
            self.status = self.spi.cmd_read(cmd, &mut buf[..(len + 3) / 4 + 1]).await;

            // when writing out the data, we skip the response-delay byte
            data[..len].copy_from_slice(&slice8_mut(&mut buf[1..])[..len]);

            // Advance ptr.
            addr += len as u32;
            data = &mut data[len..];
        }
    }

    pub async fn bp_write(&mut self, mut addr: u32, mut data: &[u8]) {
        // It seems the HW force-aligns the addr
        // to 2 if data.len() >= 2
        // to 4 if data.len() >= 4
        // To simplify, enforce 4-align for now.
        assert!(addr % 4 == 0);

        let mut buf = [0u32; BACKPLANE_MAX_TRANSFER_SIZE / 4 + 1];

        while !data.is_empty() {
            // Ensure transfer doesn't cross a window boundary.
            let window_offs = addr & BACKPLANE_ADDRESS_MASK;
            let window_remaining = BACKPLANE_WINDOW_SIZE - window_offs as usize;

            let len = data.len().min(BACKPLANE_MAX_TRANSFER_SIZE).min(window_remaining);
            slice8_mut(&mut buf[1..])[..len].copy_from_slice(&data[..len]);

            self.backplane_set_window(addr).await;

            let cmd = cmd_word(WRITE, INC_ADDR, FUNC_BACKPLANE, window_offs, len as u32);
            buf[0] = cmd;

            self.status = self.spi.cmd_write(&buf[..(len + 3) / 4 + 1]).await;

            // Advance ptr.
            addr += len as u32;
            data = &data[len..];
        }
    }

    pub async fn bp_read8(&mut self, addr: u32) -> u8 {
        self.backplane_readn(addr, 1).await as u8
    }

    pub async fn bp_write8(&mut self, addr: u32, val: u8) {
        self.backplane_writen(addr, val as u32, 1).await
    }

    pub async fn bp_read16(&mut self, addr: u32) -> u16 {
        self.backplane_readn(addr, 2).await as u16
    }

    #[allow(unused)]
    pub async fn bp_write16(&mut self, addr: u32, val: u16) {
        self.backplane_writen(addr, val as u32, 2).await
    }

    #[allow(unused)]
    pub async fn bp_read32(&mut self, addr: u32) -> u32 {
        self.backplane_readn(addr, 4).await
    }

    pub async fn bp_write32(&mut self, addr: u32, val: u32) {
        self.backplane_writen(addr, val, 4).await
    }

    async fn backplane_readn(&mut self, addr: u32, len: u32) -> u32 {
        self.backplane_set_window(addr).await;

        let mut bus_addr = addr & BACKPLANE_ADDRESS_MASK;
        if len == 4 {
            bus_addr |= BACKPLANE_ADDRESS_32BIT_FLAG
        }
        self.readn(FUNC_BACKPLANE, bus_addr, len).await
    }

    async fn backplane_writen(&mut self, addr: u32, val: u32, len: u32) {
        self.backplane_set_window(addr).await;

        let mut bus_addr = addr & BACKPLANE_ADDRESS_MASK;
        if len == 4 {
            bus_addr |= BACKPLANE_ADDRESS_32BIT_FLAG
        }
        self.writen(FUNC_BACKPLANE, bus_addr, val, len).await
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

    pub async fn read8(&mut self, func: u32, addr: u32) -> u8 {
        self.readn(func, addr, 1).await as u8
    }

    pub async fn write8(&mut self, func: u32, addr: u32, val: u8) {
        self.writen(func, addr, val as u32, 1).await
    }

    pub async fn read16(&mut self, func: u32, addr: u32) -> u16 {
        self.readn(func, addr, 2).await as u16
    }

    #[allow(unused)]
    pub async fn write16(&mut self, func: u32, addr: u32, val: u16) {
        self.writen(func, addr, val as u32, 2).await
    }

    pub async fn read32(&mut self, func: u32, addr: u32) -> u32 {
        self.readn(func, addr, 4).await
    }

    #[allow(unused)]
    pub async fn write32(&mut self, func: u32, addr: u32, val: u32) {
        self.writen(func, addr, val, 4).await
    }

    async fn readn(&mut self, func: u32, addr: u32, len: u32) -> u32 {
        let cmd = cmd_word(READ, INC_ADDR, func, addr, len);
        let mut buf = [0; 2];
        // if we are reading from the backplane, we need an extra word for the response delay
        let len = if func == FUNC_BACKPLANE { 2 } else { 1 };

        self.status = self.spi.cmd_read(cmd, &mut buf[..len]).await;

        // if we read from the backplane, the result is in the second word, after the response delay
        if func == FUNC_BACKPLANE {
            buf[1]
        } else {
            buf[0]
        }
    }

    async fn writen(&mut self, func: u32, addr: u32, val: u32, len: u32) {
        let cmd = cmd_word(WRITE, INC_ADDR, func, addr, len);

        self.status = self.spi.cmd_write(&[cmd, val]).await;
    }

    async fn read32_swapped(&mut self, addr: u32) -> u32 {
        let cmd = cmd_word(READ, INC_ADDR, FUNC_BUS, addr, 4);
        let cmd = swap16(cmd);
        let mut buf = [0; 1];

        self.status = self.spi.cmd_read(cmd, &mut buf).await;

        swap16(buf[0])
    }

    async fn write32_swapped(&mut self, addr: u32, val: u32) {
        let cmd = cmd_word(WRITE, INC_ADDR, FUNC_BUS, addr, 4);
        let buf = [swap16(cmd), swap16(val)];

        self.status = self.spi.cmd_write(&buf).await;
    }

    pub async fn wait_for_event(&mut self) {
        self.spi.wait_for_event().await;
    }

    pub fn status(&self) -> u32 {
        self.status
    }
}

fn swap16(x: u32) -> u32 {
    x.rotate_left(16)
}

fn cmd_word(write: bool, incr: bool, func: u32, addr: u32, len: u32) -> u32 {
    (write as u32) << 31 | (incr as u32) << 30 | (func & 0b11) << 28 | (addr & 0x1FFFF) << 11 | (len & 0x7FF)
}
