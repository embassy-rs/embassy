#![macro_use]

use core::marker::PhantomData;
use core::ptr;
use core::task::Poll;
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::util::Unborrow;
use embassy_hal_common::drop::DropBomb;
use embassy_hal_common::unborrow;
use futures::future::poll_fn;

use crate::gpio::sealed::Pin as _;
use crate::gpio::{self, Pin as GpioPin};
use crate::pac;

pub use crate::pac::qspi::ifconfig0::ADDRMODE_A as AddressMode;
pub use crate::pac::qspi::ifconfig0::PPSIZE_A as WritePageSize;
pub use crate::pac::qspi::ifconfig0::READOC_A as ReadOpcode;
pub use crate::pac::qspi::ifconfig0::WRITEOC_A as WriteOpcode;

// TODO
// - config:
//   - 32bit address mode
//   - SPI freq
//   - SPI sck delay
//   - Deep power down mode (DPM)
//   - SPI mode 3
// - activate/deactivate
// - set gpio in high drive

pub struct DeepPowerDownConfig {
    /// Time required for entering DPM, in units of 16us
    pub enter_time: u16,
    /// Time required for exiting DPM, in units of 16us
    pub exit_time: u16,
}

#[non_exhaustive]
pub struct Config {
    pub xip_offset: u32,
    pub read_opcode: ReadOpcode,
    pub write_opcode: WriteOpcode,
    pub write_page_size: WritePageSize,
    pub deep_power_down: Option<DeepPowerDownConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            read_opcode: ReadOpcode::READ4IO,
            write_opcode: WriteOpcode::PP4IO,
            xip_offset: 0,
            write_page_size: WritePageSize::_256BYTES,
            deep_power_down: None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    OutOfBounds,
    // TODO add "not in data memory" error and check for it
}

pub struct Qspi<'d, T: Instance, const FLASH_SIZE: usize> {
    irq: T::Interrupt,
    dpm_enabled: bool,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance, const FLASH_SIZE: usize> Qspi<'d, T, FLASH_SIZE> {
    pub fn new(
        _qspi: impl Unborrow<Target = T> + 'd,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
        sck: impl Unborrow<Target = impl GpioPin> + 'd,
        csn: impl Unborrow<Target = impl GpioPin> + 'd,
        io0: impl Unborrow<Target = impl GpioPin> + 'd,
        io1: impl Unborrow<Target = impl GpioPin> + 'd,
        io2: impl Unborrow<Target = impl GpioPin> + 'd,
        io3: impl Unborrow<Target = impl GpioPin> + 'd,
        config: Config,
    ) -> Qspi<'d, T, FLASH_SIZE> {
        unborrow!(irq, sck, csn, io0, io1, io2, io3);

        let r = T::regs();

        let sck = sck.degrade();
        let csn = csn.degrade();
        let io0 = io0.degrade();
        let io1 = io1.degrade();
        let io2 = io2.degrade();
        let io3 = io3.degrade();

        for pin in [&sck, &csn, &io0, &io1, &io2, &io3] {
            pin.set_high();
            pin.conf().write(|w| w.dir().output().drive().h0h1());
        }

        r.psel.sck.write(|w| unsafe { w.bits(sck.psel_bits()) });
        r.psel.csn.write(|w| unsafe { w.bits(csn.psel_bits()) });
        r.psel.io0.write(|w| unsafe { w.bits(io0.psel_bits()) });
        r.psel.io1.write(|w| unsafe { w.bits(io1.psel_bits()) });
        r.psel.io2.write(|w| unsafe { w.bits(io2.psel_bits()) });
        r.psel.io3.write(|w| unsafe { w.bits(io3.psel_bits()) });

        r.ifconfig0.write(|w| {
            w.addrmode().variant(AddressMode::_24BIT);
            w.dpmenable().bit(config.deep_power_down.is_some());
            w.ppsize().variant(config.write_page_size);
            w.readoc().variant(config.read_opcode);
            w.writeoc().variant(config.write_opcode);
            w
        });

        if let Some(dpd) = &config.deep_power_down {
            r.dpmdur.write(|w| unsafe {
                w.enter().bits(dpd.enter_time);
                w.exit().bits(dpd.exit_time);
                w
            })
        }

        r.ifconfig1.write(|w| unsafe {
            w.sckdelay().bits(80);
            w.dpmen().exit();
            w.spimode().mode0();
            w.sckfreq().bits(3);
            w
        });

        r.xipoffset.write(|w| unsafe {
            w.xipoffset().bits(config.xip_offset);
            w
        });

        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        // Enable it
        r.enable.write(|w| w.enable().enabled());

        let mut res = Self {
            dpm_enabled: config.deep_power_down.is_some(),
            irq,
            phantom: PhantomData,
        };

        r.events_ready.reset();
        r.intenset.write(|w| w.ready().set());

        r.tasks_activate.write(|w| w.tasks_activate().bit(true));

        res.blocking_wait_ready();

        res
    }

    fn on_interrupt(_: *mut ()) {
        let r = T::regs();
        let s = T::state();

        if r.events_ready.read().bits() != 0 {
            s.ready_waker.wake();
            r.intenclr.write(|w| w.ready().clear());
        }
    }

    pub async fn custom_instruction(
        &mut self,
        opcode: u8,
        req: &[u8],
        resp: &mut [u8],
    ) -> Result<(), Error> {
        let bomb = DropBomb::new();

        let len = core::cmp::max(req.len(), resp.len()) as u8;
        self.custom_instruction_start(opcode, req, len)?;

        self.wait_ready().await;

        self.custom_instruction_finish(resp)?;

        bomb.defuse();

        Ok(())
    }

    pub fn blocking_custom_instruction(
        &mut self,
        opcode: u8,
        req: &[u8],
        resp: &mut [u8],
    ) -> Result<(), Error> {
        let len = core::cmp::max(req.len(), resp.len()) as u8;
        self.custom_instruction_start(opcode, req, len)?;

        self.blocking_wait_ready();

        self.custom_instruction_finish(resp)?;

        Ok(())
    }

    fn custom_instruction_start(&mut self, opcode: u8, req: &[u8], len: u8) -> Result<(), Error> {
        assert!(req.len() <= 8);

        let mut dat0: u32 = 0;
        let mut dat1: u32 = 0;

        for i in 0..4 {
            if i < req.len() {
                dat0 |= (req[i] as u32) << (i * 8);
            }
        }
        for i in 0..4 {
            if i + 4 < req.len() {
                dat1 |= (req[i + 4] as u32) << (i * 8);
            }
        }

        let r = T::regs();
        r.cinstrdat0.write(|w| unsafe { w.bits(dat0) });
        r.cinstrdat1.write(|w| unsafe { w.bits(dat1) });

        r.events_ready.reset();
        r.intenset.write(|w| w.ready().set());

        r.cinstrconf.write(|w| {
            let w = unsafe { w.opcode().bits(opcode) };
            let w = unsafe { w.length().bits(len + 1) };
            let w = w.lio2().bit(true);
            let w = w.lio3().bit(true);
            let w = w.wipwait().bit(true);
            let w = w.wren().bit(true);
            let w = w.lfen().bit(false);
            let w = w.lfstop().bit(false);
            w
        });
        Ok(())
    }

    fn custom_instruction_finish(&mut self, resp: &mut [u8]) -> Result<(), Error> {
        let r = T::regs();

        let dat0 = r.cinstrdat0.read().bits();
        let dat1 = r.cinstrdat1.read().bits();
        for i in 0..4 {
            if i < resp.len() {
                resp[i] = (dat0 >> (i * 8)) as u8;
            }
        }
        for i in 0..4 {
            if i + 4 < resp.len() {
                resp[i] = (dat1 >> (i * 8)) as u8;
            }
        }
        Ok(())
    }

    async fn wait_ready(&mut self) {
        poll_fn(move |cx| {
            let r = T::regs();
            let s = T::state();
            s.ready_waker.register(cx.waker());
            if r.events_ready.read().bits() != 0 {
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await
    }

    fn blocking_wait_ready(&mut self) {
        loop {
            let r = T::regs();
            if r.events_ready.read().bits() != 0 {
                break;
            }
        }
    }

    fn start_read(&mut self, address: usize, data: &mut [u8]) -> Result<(), Error> {
        assert_eq!(data.as_ptr() as u32 % 4, 0);
        assert_eq!(data.len() as u32 % 4, 0);
        assert_eq!(address as u32 % 4, 0);
        if address > FLASH_SIZE {
            return Err(Error::OutOfBounds);
        }

        let r = T::regs();

        r.read
            .src
            .write(|w| unsafe { w.src().bits(address as u32) });
        r.read
            .dst
            .write(|w| unsafe { w.dst().bits(data.as_ptr() as u32) });
        r.read
            .cnt
            .write(|w| unsafe { w.cnt().bits(data.len() as u32) });

        r.events_ready.reset();
        r.intenset.write(|w| w.ready().set());
        r.tasks_readstart.write(|w| w.tasks_readstart().bit(true));

        Ok(())
    }

    fn start_write(&mut self, address: usize, data: &[u8]) -> Result<(), Error> {
        //info!("start_write ptr {}", data.as_ptr() as u32);
        assert_eq!(data.as_ptr() as u32 % 4, 0);
        //info!("start_write OK ptr");
        assert_eq!(data.len() as u32 % 4, 0);
        //info!("start_write OK len");
        assert_eq!(address as u32 % 4, 0);
        //info!("start_write OK addr");
        if address > FLASH_SIZE {
            return Err(Error::OutOfBounds);
        }

        let r = T::regs();
        r.write
            .src
            .write(|w| unsafe { w.src().bits(data.as_ptr() as u32) });
        r.write
            .dst
            .write(|w| unsafe { w.dst().bits(address as u32) });
        r.write
            .cnt
            .write(|w| unsafe { w.cnt().bits(data.len() as u32) });

        r.events_ready.reset();
        r.intenset.write(|w| w.ready().set());
        r.tasks_writestart.write(|w| w.tasks_writestart().bit(true));

        Ok(())
    }

    fn start_erase(&mut self, address: usize) -> Result<(), Error> {
        assert_eq!(address as u32 % 4096, 0);
        if address > FLASH_SIZE {
            return Err(Error::OutOfBounds);
        }

        let r = T::regs();
        r.erase
            .ptr
            .write(|w| unsafe { w.ptr().bits(address as u32) });
        r.erase.len.write(|w| w.len()._4kb());

        r.events_ready.reset();
        r.intenset.write(|w| w.ready().set());
        r.tasks_erasestart.write(|w| w.tasks_erasestart().bit(true));

        Ok(())
    }

    pub async fn read(&mut self, address: usize, data: &mut [u8]) -> Result<(), Error> {
        let bomb = DropBomb::new();

        self.start_read(address, data)?;
        self.wait_ready().await;

        bomb.defuse();

        Ok(())
    }

    pub async fn write(&mut self, address: usize, data: &[u8]) -> Result<(), Error> {
        let bomb = DropBomb::new();

        //info!("WRITE {} bytes at {}", data.len(), address);
        self.start_write(address, data)?;
        //info!("STARTED");
        self.wait_ready().await;
        //info!("WRITE DONE");

        bomb.defuse();

        Ok(())
    }

    pub async fn erase(&mut self, address: usize) -> Result<(), Error> {
        let bomb = DropBomb::new();

        self.start_erase(address)?;
        self.wait_ready().await;

        bomb.defuse();

        Ok(())
    }

    pub fn blocking_read(&mut self, address: usize, data: &mut [u8]) -> Result<(), Error> {
        self.start_read(address, data)?;
        self.blocking_wait_ready();
        Ok(())
    }

    pub fn blocking_write(&mut self, address: usize, data: &[u8]) -> Result<(), Error> {
        self.start_write(address, data)?;
        self.blocking_wait_ready();
        Ok(())
    }

    pub fn blocking_erase(&mut self, address: usize) -> Result<(), Error> {
        self.start_erase(address)?;
        self.blocking_wait_ready();
        Ok(())
    }
}

impl<'d, T: Instance, const FLASH_SIZE: usize> Drop for Qspi<'d, T, FLASH_SIZE> {
    fn drop(&mut self) {
        let r = T::regs();

        if self.dpm_enabled {
            trace!("qspi: doing deep powerdown...");

            r.ifconfig1.modify(|_, w| w.dpmen().enter());

            // Wait for DPM enter.
            // Unfortunately we must spin. There's no way to do this interrupt-driven.
            // The READY event does NOT fire on DPM enter (but it does fire on DPM exit :shrug:)
            while r.status.read().dpm().is_disabled() {}

            // Wait MORE for DPM enter.
            // I have absolutely no idea why, but the wait above is not enough :'(
            // Tested with mx25r64 in nrf52840-dk, and with mx25r16 in custom board
            cortex_m::asm::delay(4096);
        }

        // it seems events_ready is not generated in response to deactivate. nrfx doesn't wait for it.
        r.tasks_deactivate.write(|w| w.tasks_deactivate().set_bit());

        // Workaround https://infocenter.nordicsemi.com/topic/errata_nRF52840_Rev1/ERR/nRF52840/Rev1/latest/anomaly_840_122.html?cp=4_0_1_2_1_7
        // Note that the doc has 2 register writes, but the first one is really the write to tasks_deactivate,
        // so we only do the second one here.
        unsafe { ptr::write_volatile(0x40029054 as *mut u32, 1) }

        r.enable.write(|w| w.enable().disabled());

        self.irq.disable();

        // Note: we do NOT deconfigure CSN here. If DPM is in use and we disconnect CSN,
        // leaving it floating, the flash chip might read it as zero which would cause it to
        // spuriously exit DPM.
        gpio::deconfigure_pin(r.psel.sck.read().bits());
        gpio::deconfigure_pin(r.psel.io0.read().bits());
        gpio::deconfigure_pin(r.psel.io1.read().bits());
        gpio::deconfigure_pin(r.psel.io2.read().bits());
        gpio::deconfigure_pin(r.psel.io3.read().bits());

        trace!("qspi: dropped");
    }
}

use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};

impl<'d, T: Instance, const FLASH_SIZE: usize> ErrorType for Qspi<'d, T, FLASH_SIZE> {
    type Error = Error;
}

impl NorFlashError for Error {
    fn kind(&self) -> NorFlashErrorKind {
        NorFlashErrorKind::Other
    }
}

impl<'d, T: Instance, const FLASH_SIZE: usize> ReadNorFlash for Qspi<'d, T, FLASH_SIZE> {
    const READ_SIZE: usize = 4;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(offset as usize, bytes)?;
        Ok(())
    }

    fn capacity(&self) -> usize {
        FLASH_SIZE
    }
}

impl<'d, T: Instance, const FLASH_SIZE: usize> NorFlash for Qspi<'d, T, FLASH_SIZE> {
    const WRITE_SIZE: usize = 4;
    const ERASE_SIZE: usize = 4096;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        for address in (from as usize..to as usize).step_by(<Self as NorFlash>::ERASE_SIZE) {
            self.blocking_erase(address)?;
        }
        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(offset as usize, bytes)?;
        Ok(())
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "nightly")]
    {
        use embedded_storage_async::nor_flash::{AsyncNorFlash, AsyncReadNorFlash};
        use core::future::Future;

        impl<'d, T: Instance, const FLASH_SIZE: usize> AsyncNorFlash for Qspi<'d, T, FLASH_SIZE> {
            const WRITE_SIZE: usize = <Self as NorFlash>::WRITE_SIZE;
            const ERASE_SIZE: usize = <Self as NorFlash>::ERASE_SIZE;

            type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;
            fn write<'a>(&'a mut self, offset: u32, data: &'a [u8]) -> Self::WriteFuture<'a> {
                async move { self.write(offset as usize, data).await }
            }

            type EraseFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;
            fn erase<'a>(&'a mut self, from: u32, to: u32) -> Self::EraseFuture<'a> {
                async move {
                    for address in (from as usize..to as usize).step_by(<Self as AsyncNorFlash>::ERASE_SIZE) {
                        self.erase(address).await?
                    }
                    Ok(())
                }
            }
        }

        impl<'d, T: Instance, const FLASH_SIZE: usize> AsyncReadNorFlash for Qspi<'d, T, FLASH_SIZE> {
            const READ_SIZE: usize = 4;
            type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;
            fn read<'a>(&'a mut self, address: u32, data: &'a mut [u8]) -> Self::ReadFuture<'a> {
                async move { self.read(address as usize, data).await }
            }

            fn capacity(&self) -> usize {
                FLASH_SIZE
            }
        }
    }
}

pub(crate) mod sealed {
    use embassy::waitqueue::AtomicWaker;

    use super::*;

    pub struct State {
        pub ready_waker: AtomicWaker,
    }
    impl State {
        pub const fn new() -> Self {
            Self {
                ready_waker: AtomicWaker::new(),
            }
        }
    }

    pub trait Instance {
        fn regs() -> &'static pac::qspi::RegisterBlock;
        fn state() -> &'static State;
    }
}

pub trait Instance: Unborrow<Target = Self> + sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

macro_rules! impl_qspi {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::qspi::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::qspi::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
            fn state() -> &'static crate::qspi::sealed::State {
                static STATE: crate::qspi::sealed::State = crate::qspi::sealed::State::new();
                &STATE
            }
        }
        impl crate::qspi::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::$irq;
        }
    };
}
