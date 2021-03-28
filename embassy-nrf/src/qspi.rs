use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::Poll;

use embassy::interrupt::Interrupt;
use embassy_extras::peripheral::{PeripheralMutex, PeripheralState};
use embassy_extras::unborrow;

use crate::fmt::{assert, assert_eq, *};
use crate::gpio::Pin as GpioPin;
use crate::interrupt::{self};
use crate::{pac, peripherals};

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

use embassy::traits::flash::{Error, Flash};
use embassy::util::{wake_on_interrupt, DropBomb, PeripheralBorrow, WakerRegistration};
use futures::future::poll_fn;

pub struct DeepPowerDownConfig {
    pub enter_time: u16,
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

pub struct Qspi<'d, T: Instance> {
    peri: T,
    irq: T::Interrupt,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Qspi<'d, T> {
    pub fn new(
        qspi: impl PeripheralBorrow<Target = T> + 'd,
        irq: impl PeripheralBorrow<Target = T::Interrupt> + 'd,
        sck: impl PeripheralBorrow<Target = impl GpioPin> + 'd,
        csn: impl PeripheralBorrow<Target = impl GpioPin> + 'd,
        io0: impl PeripheralBorrow<Target = impl GpioPin> + 'd,
        io1: impl PeripheralBorrow<Target = impl GpioPin> + 'd,
        io2: impl PeripheralBorrow<Target = impl GpioPin> + 'd,
        io3: impl PeripheralBorrow<Target = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(qspi, irq, sck, csn, io0, io1, io2, io3);

        let r = qspi.regs();

        for cnf in &[
            sck.conf(),
            csn.conf(),
            io0.conf(),
            io1.conf(),
            io2.conf(),
            io3.conf(),
        ] {
            cnf.write(|w| w.dir().output().drive().h0h1());
        }

        r.psel.sck.write(|w| unsafe { w.bits(sck.psel_bits()) });
        r.psel.csn.write(|w| unsafe { w.bits(csn.psel_bits()) });
        r.psel.io0.write(|w| unsafe { w.bits(io0.psel_bits()) });
        r.psel.io1.write(|w| unsafe { w.bits(io1.psel_bits()) });
        r.psel.io2.write(|w| unsafe { w.bits(io2.psel_bits()) });
        r.psel.io3.write(|w| unsafe { w.bits(io3.psel_bits()) });

        r.ifconfig0.write(|mut w| {
            w = w.addrmode().variant(AddressMode::_24BIT);
            if config.deep_power_down.is_some() {
                w = w.dpmenable().enable();
            } else {
                w = w.dpmenable().disable();
            }
            w = w.ppsize().variant(config.write_page_size);
            w = w.readoc().variant(config.read_opcode);
            w = w.writeoc().variant(config.write_opcode);
            w
        });

        if let Some(dpd) = &config.deep_power_down {
            r.dpmdur.write(|mut w| unsafe {
                w = w.enter().bits(dpd.enter_time);
                w = w.exit().bits(dpd.exit_time);
                w
            })
        }

        r.ifconfig1.write(|w| {
            let w = unsafe { w.sckdelay().bits(80) };
            let w = w.dpmen().exit();
            let w = w.spimode().mode0();
            let w = unsafe { w.sckfreq().bits(3) };
            w
        });

        r.xipoffset
            .write(|w| unsafe { w.xipoffset().bits(config.xip_offset) });

        // Enable it
        r.enable.write(|w| w.enable().enabled());

        r.events_ready.reset();
        r.tasks_activate.write(|w| w.tasks_activate().bit(true));
        while r.events_ready.read().bits() == 0 {}
        r.events_ready.reset();

        Self {
            peri: qspi,
            irq,
            phantom: PhantomData,
        }
    }

    pub fn sleep(mut self: Pin<&mut Self>) {
        let r = unsafe { self.as_mut().get_unchecked_mut() }.peri.regs();

        info!("flash: sleeping");
        info!("flash: state = {:?}", r.status.read().bits());
        r.ifconfig1.modify(|_, w| w.dpmen().enter());
        info!("flash: state = {:?}", r.status.read().bits());
        cortex_m::asm::delay(1000000);
        info!("flash: state = {:?}", r.status.read().bits());

        r.tasks_deactivate.write(|w| w.tasks_deactivate().set_bit());
    }

    pub async fn custom_instruction(
        mut self: Pin<&mut Self>,
        opcode: u8,
        req: &[u8],
        resp: &mut [u8],
    ) -> Result<(), Error> {
        let bomb = DropBomb::new();

        assert!(req.len() <= 8);
        assert!(resp.len() <= 8);

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

        let len = core::cmp::max(req.len(), resp.len()) as u8;

        let r = unsafe { self.as_mut().get_unchecked_mut() }.peri.regs();
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

        self.as_mut().wait_ready().await;

        let r = unsafe { self.as_mut().get_unchecked_mut() }.peri.regs();

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

        bomb.defuse();

        Ok(())
    }

    async fn wait_ready(self: Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };

        poll_fn(move |cx| {
            let r = this.peri.regs();

            if r.events_ready.read().bits() != 0 {
                r.events_ready.reset();
                return Poll::Ready(());
            }

            wake_on_interrupt(&mut this.irq, cx.waker());

            Poll::Pending
        })
        .await
    }
}

impl<'d, T: Instance> Flash for Qspi<'d, T> {
    #[rustfmt::skip]
    type ReadFuture<'a> where Self: 'a = impl Future<Output = Result<(), Error>> + 'a;
    #[rustfmt::skip]
    type WriteFuture<'a> where Self: 'a = impl Future<Output = Result<(), Error>> + 'a;
    #[rustfmt::skip]
    type ErasePageFuture<'a> where Self: 'a = impl Future<Output = Result<(), Error>> + 'a;

    fn read<'a>(
        mut self: Pin<&'a mut Self>,
        address: usize,
        data: &'a mut [u8],
    ) -> Self::ReadFuture<'a> {
        async move {
            let bomb = DropBomb::new();

            assert_eq!(data.as_ptr() as u32 % 4, 0);
            assert_eq!(data.len() as u32 % 4, 0);
            assert_eq!(address as u32 % 4, 0);

            let r = unsafe { self.as_mut().get_unchecked_mut() }.peri.regs();

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

            self.as_mut().wait_ready().await;

            bomb.defuse();

            Ok(())
        }
    }

    fn write<'a>(
        mut self: Pin<&'a mut Self>,
        address: usize,
        data: &'a [u8],
    ) -> Self::WriteFuture<'a> {
        async move {
            let bomb = DropBomb::new();

            assert_eq!(data.as_ptr() as u32 % 4, 0);
            assert_eq!(data.len() as u32 % 4, 0);
            assert_eq!(address as u32 % 4, 0);

            let r = unsafe { self.as_mut().get_unchecked_mut() }.peri.regs();
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

            self.as_mut().wait_ready().await;

            bomb.defuse();

            Ok(())
        }
    }

    fn erase<'a>(mut self: Pin<&'a mut Self>, address: usize) -> Self::ErasePageFuture<'a> {
        async move {
            let bomb = DropBomb::new();

            assert_eq!(address as u32 % 4096, 0);

            let r = unsafe { self.as_mut().get_unchecked_mut() }.peri.regs();
            r.erase
                .ptr
                .write(|w| unsafe { w.ptr().bits(address as u32) });
            r.erase.len.write(|w| w.len()._4kb());

            r.events_ready.reset();
            r.intenset.write(|w| w.ready().set());
            r.tasks_erasestart.write(|w| w.tasks_erasestart().bit(true));

            self.as_mut().wait_ready().await;

            bomb.defuse();

            Ok(())
        }
    }

    fn size(&self) -> usize {
        256 * 4096 // TODO
    }

    fn read_size(&self) -> usize {
        4 // TODO
    }

    fn write_size(&self) -> usize {
        4 // TODO
    }

    fn erase_size(&self) -> usize {
        4096 // TODO
    }
}

mod sealed {
    use super::*;

    pub trait Instance {
        fn regs(&self) -> &pac::qspi::RegisterBlock;
    }
}

pub trait Instance: sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

macro_rules! impl_instance {
    ($type:ident, $irq:ident) => {
        impl sealed::Instance for peripherals::$type {
            fn regs(&self) -> &pac::qspi::RegisterBlock {
                unsafe { &*pac::$type::ptr() }
            }
        }
        impl Instance for peripherals::$type {
            type Interrupt = interrupt::$irq;
        }
    };
}

impl_instance!(QSPI, QSPI);
