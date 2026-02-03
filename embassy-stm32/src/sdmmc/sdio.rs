use core::future::poll_fn;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::Poll;

use aligned::{A4, Aligned};
use sdio_host::common_cmd::{R1, Resp, cmd};
use sdio_host::sd::{BusWidth, OCR, RCA, SD};
use sdio_host::{Cmd, sd_cmd};

use crate::sdmmc::{
    CommandResponse, DatapathMode, Error, Sdmmc, TypedResp, aligned_mut, aligned_ref, block_size, slice8_mut,
    slice8_ref,
};
use crate::time::Hertz;

/// R4: OCR register
pub struct R4;

impl Resp for R4 {}

impl TypedResp for R4 {
    type Word = u32;
}

impl From<CommandResponse<R4>> for OCR<SD> {
    fn from(value: CommandResponse<R4>) -> Self {
        OCR::<SD>::from(value.0)
    }
}

/// R5: IO_RW_DIRECT Response
pub struct R5;

impl Resp for R5 {}

impl TypedResp for R5 {
    type Word = u32;
}

/// ACMD5: IO Op Command
///
/// * `switch_to_1_8v_request` - Switch to 1.8V signaling
/// * `voltage_window` - 9-bit bitfield that represents the voltage window
/// supported by the host. Use 0x1FF to indicate support for the full range of
/// voltages
pub fn io_send_op_cond(switch_to_1_8v_request: bool, voltage_window: u16) -> Cmd<R4> {
    let arg: u32 = u32::from(switch_to_1_8v_request) << 24 | u32::from(voltage_window & 0x1FF) << 15;
    cmd(5, arg)
}

/// Aligned data block for SDMMC transfers.
///
/// This is a 64-byte array, aligned to 4 bytes to satisfy DMA requirements.
#[repr(align(4))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DataBlock(pub [u32; 16]);

impl DataBlock {
    /// Create a new DataBlock
    pub const fn new() -> Self {
        DataBlock([0u32; 16])
    }
}

impl Deref for DataBlock {
    type Target = [u8; 64];

    fn deref(&self) -> &Self::Target {
        unwrap!(slice8_ref(&self.0[..]).try_into())
    }
}

impl DerefMut for DataBlock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unwrap!(slice8_mut(&mut self.0[..]).try_into())
    }
}

/// Storage Device
pub struct SerialDataInterface<'a, 'b> {
    /// Inner member
    sdmmc: &'a mut Sdmmc<'b>,
}

/// Card Storage Device
impl<'a, 'b> SerialDataInterface<'a, 'b> {
    /// Create a new SD card
    pub async fn new(sdmmc: &'a mut Sdmmc<'b>, freq: Hertz) -> Result<Self, Error> {
        let mut s = Self { sdmmc };

        s.acquire(freq).await?;

        Ok(s)
    }

    /// Initializes the card into a known state (or at least tries to).
    async fn acquire(&mut self, _freq: Hertz) -> Result<(), Error> {
        let _scoped_wake_guard = self.sdmmc.info.rcc.wake_guard();

        let _bus_width = match self.sdmmc.bus_width() {
            BusWidth::Eight => return Err(Error::BusWidth),
            bus_width => bus_width,
        };

        // While the SD/SDIO card or eMMC is in identification mode,
        // the SDMMC_CK frequency must be no more than 400 kHz.
        self.sdmmc.init_idle()?;

        // Get IO OCR
        let _ocr: OCR<SD> = self.sdmmc.cmd(io_send_op_cond(false, 0x0), false, false)?.into();

        // UDB-based SDIO does not support io volt switch sequence

        // Get RCA
        let rca: RCA<SD> = self.sdmmc.cmd(sd_cmd::send_relative_address(), true, false)?.into();
        trace!("sdio: got rca {}", rca.address());

        // Select the card with RCA
        self.sdmmc.select_card(Some(rca.address()))?;
        trace!("sdio: selected card {}", rca.address());

        Ok(())
    }

    /// Set the bus to the 4-bit high-speed frequency
    pub fn set_bus_to_high_speed(&mut self, frequency: Hertz) -> Result<(), Error> {
        self.sdmmc.clkcr_set_clkdiv(frequency, BusWidth::Four)?;

        Ok(())
    }

    /// Run cmd52
    pub async fn cmd52(&mut self, arg: u32) -> Result<u16, Error> {
        self.sdmmc
            .cmd(cmd::<R5>(52, arg), true, false)
            .map(|r| r.0.try_into().unwrap())
    }

    /// Read in block mode using cmd53
    pub async fn cmd53_block_read(&mut self, arg: u32, blocks: &mut [DataBlock]) -> Result<(), Error> {
        let _scoped_wake_guard = self.sdmmc.info.rcc.wake_guard();

        // NOTE(unsafe) reinterpret buffer as &mut [u32]
        let buffer = unsafe {
            core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut u32, size_of_val(blocks) / size_of::<u32>())
        };

        let transfer = self.sdmmc.prepare_datapath_read(
            aligned_mut(buffer),
            DatapathMode::Block(block_size(size_of::<DataBlock>())),
        );
        self.sdmmc.cmd(cmd::<R1>(53, arg), true, true)?;

        self.sdmmc.complete_datapath_transfer(transfer, false).await?;
        self.sdmmc.clear_interrupt_flags();

        Ok(())
    }

    /// Read in multibyte mode using cmd53
    pub async fn cmd53_byte_read(&mut self, arg: u32, buffer: &mut Aligned<A4, [u8]>) -> Result<(), Error> {
        let _scoped_wake_guard = self.sdmmc.info.rcc.wake_guard();

        // trace!("byte read start (len): {:#x} ({})", arg, buffer.len());

        let transfer = self.sdmmc.prepare_datapath_read(buffer, DatapathMode::Byte);
        self.sdmmc.cmd(cmd::<R1>(53, arg), true, true)?;

        // trace!("byte read before complete");

        self.sdmmc.complete_datapath_transfer(transfer, false).await?;
        self.sdmmc.clear_interrupt_flags();

        // trace!("byte read stop");

        Ok(())
    }

    /// Write in block mode using cmd53
    pub async fn cmd53_block_write(&mut self, arg: u32, blocks: &[DataBlock]) -> Result<(), Error> {
        let _scoped_wake_guard = self.sdmmc.info.rcc.wake_guard();

        // NOTE(unsafe) reinterpret buffer as &mut [u32]
        let buffer = unsafe {
            core::slice::from_raw_parts_mut(blocks.as_ptr() as *mut u32, size_of_val(blocks) / size_of::<u32>())
        };

        #[cfg(sdmmc_v1)]
        self.sdmmc.cmd(cmd::<R1>(53, arg), true, true)?;

        let transfer = self.sdmmc.prepare_datapath_write(
            aligned_ref(buffer),
            DatapathMode::Block(block_size(size_of::<DataBlock>())),
        );

        #[cfg(sdmmc_v2)]
        self.sdmmc.cmd(cmd::<R1>(53, arg), true, true)?;

        self.sdmmc.complete_datapath_transfer(transfer, false).await?;
        self.sdmmc.clear_interrupt_flags();

        Ok(())
    }

    /// Write in multibyte mode using cmd53
    pub async fn cmd53_byte_write(&mut self, arg: u32, buffer: &Aligned<A4, [u8]>) -> Result<(), Error> {
        let _scoped_wake_guard = self.sdmmc.info.rcc.wake_guard();

        #[cfg(sdmmc_v1)]
        self.sdmmc.cmd(cmd::<R1>(53, arg), true, true)?;

        let transfer = self.sdmmc.prepare_datapath_write(buffer, DatapathMode::Byte);

        #[cfg(sdmmc_v2)]
        self.sdmmc.cmd(cmd::<R1>(53, arg), true, true)?;

        self.sdmmc.complete_datapath_transfer(transfer, false).await?;
        self.sdmmc.clear_interrupt_flags();

        Ok(())
    }

    /// Wait for an interrupt event
    pub async fn wait_for_event(&mut self) {
        poll_fn(|cx| {
            self.sdmmc.state.it_waker.register(cx.waker());

            compiler_fence(Ordering::Release);

            let status = self.sdmmc.info.regs.star().read();
            let icr = self.sdmmc.info.regs.icr();
            let maskr = self.sdmmc.info.regs.maskr();

            if status.sdioit() {
                icr.write(|w| w.set_sdioitc(true));

                Poll::Ready(())
            } else {
                // Note maskr could be modified from irq
                critical_section::with(|_| maskr.modify(|w| w.set_sdioitie(true)));

                Poll::Pending
            }
        })
        .await;
    }
}

impl<'a, 'b> Drop for SerialDataInterface<'a, 'b> {
    fn drop(&mut self) {
        self.sdmmc.on_drop();
    }
}

#[cfg(feature = "cyw43")]
impl<'a, 'b> cyw43::SdioBusCyw43<64> for SerialDataInterface<'a, 'b> {
    /// The error type for the BlockDevice implementation.
    type Error = Error;

    /// Doc
    fn set_bus_to_high_speed(&mut self, frequency: u32) -> Result<(), Self::Error> {
        self.set_bus_to_high_speed(Hertz(frequency))
    }

    /// Doc
    async fn cmd52(&mut self, arg: u32) -> Result<u16, Self::Error> {
        self.cmd52(arg).await
    }

    /// Doc
    async fn cmd53_block_read(&mut self, arg: u32, blocks: &mut [Aligned<A4, [u8; 64]>]) -> Result<(), Self::Error> {
        let blocks: &mut [DataBlock] =
            unsafe { core::slice::from_raw_parts_mut(blocks.as_mut_ptr() as *mut DataBlock, blocks.len()) };

        self.cmd53_block_read(arg, blocks).await
    }

    /// Doc
    async fn cmd53_byte_read(&mut self, arg: u32, buffer: &mut Aligned<A4, [u8]>) -> Result<(), Self::Error> {
        self.cmd53_byte_read(arg, buffer).await
    }

    /// Doc
    async fn cmd53_block_write(&mut self, arg: u32, blocks: &[Aligned<A4, [u8; 64]>]) -> Result<(), Self::Error> {
        let blocks: &[DataBlock] =
            unsafe { core::slice::from_raw_parts(blocks.as_ptr() as *const DataBlock, blocks.len()) };

        self.cmd53_block_write(arg, blocks).await
    }

    /// Doc
    async fn cmd53_byte_write(&mut self, arg: u32, buffer: &Aligned<A4, [u8]>) -> Result<(), Self::Error> {
        self.cmd53_byte_write(arg, buffer).await
    }

    async fn wait_for_event(&mut self) {
        self.wait_for_event().await
    }
}
