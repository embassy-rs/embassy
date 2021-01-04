use crate::fmt::{assert, assert_eq, panic, *};
use core::future::Future;

use crate::hal::gpio::{Output, Pin as GpioPin, Port as GpioPort, PushPull};
use crate::interrupt::{OwnedInterrupt, QSPIInterrupt};
use crate::pac::{Interrupt, QSPI};

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

use embassy::flash::{Error, Flash};
use embassy::util::{DropBomb, Signal};

pub struct Pins {
    pub sck: GpioPin<Output<PushPull>>,
    pub csn: GpioPin<Output<PushPull>>,
    pub io0: GpioPin<Output<PushPull>>,
    pub io1: GpioPin<Output<PushPull>>,
    pub io2: Option<GpioPin<Output<PushPull>>>,
    pub io3: Option<GpioPin<Output<PushPull>>>,
}

pub struct DeepPowerDownConfig {
    pub enter_time: u16,
    pub exit_time: u16,
}

pub struct Config {
    pub pins: Pins,
    pub xip_offset: u32,
    pub read_opcode: ReadOpcode,
    pub write_opcode: WriteOpcode,
    pub write_page_size: WritePageSize,
    pub deep_power_down: Option<DeepPowerDownConfig>,
}

pub struct Qspi {
    inner: QSPI,
}

fn port_bit(port: GpioPort) -> bool {
    match port {
        GpioPort::Port0 => false,
        GpioPort::Port1 => true,
    }
}

impl Qspi {
    pub fn new(qspi: QSPI, irq: QSPIInterrupt, config: Config) -> Self {
        qspi.psel.sck.write(|w| {
            let pin = &config.pins.sck;
            let w = unsafe { w.pin().bits(pin.pin()) };
            let w = w.port().bit(port_bit(pin.port()));
            w.connect().connected()
        });
        qspi.psel.csn.write(|w| {
            let pin = &config.pins.csn;
            let w = unsafe { w.pin().bits(pin.pin()) };
            let w = w.port().bit(port_bit(pin.port()));
            w.connect().connected()
        });
        qspi.psel.io0.write(|w| {
            let pin = &config.pins.io0;
            let w = unsafe { w.pin().bits(pin.pin()) };
            let w = w.port().bit(port_bit(pin.port()));
            w.connect().connected()
        });
        qspi.psel.io1.write(|w| {
            let pin = &config.pins.io1;
            let w = unsafe { w.pin().bits(pin.pin()) };
            let w = w.port().bit(port_bit(pin.port()));
            w.connect().connected()
        });
        qspi.psel.io2.write(|w| {
            if let Some(ref pin) = config.pins.io2 {
                let w = unsafe { w.pin().bits(pin.pin()) };
                let w = w.port().bit(port_bit(pin.port()));
                w.connect().connected()
            } else {
                w.connect().disconnected()
            }
        });
        qspi.psel.io3.write(|w| {
            if let Some(ref pin) = config.pins.io3 {
                let w = unsafe { w.pin().bits(pin.pin()) };
                let w = w.port().bit(port_bit(pin.port()));
                w.connect().connected()
            } else {
                w.connect().disconnected()
            }
        });

        qspi.ifconfig0.write(|mut w| {
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
            qspi.dpmdur.write(|mut w| unsafe {
                w = w.enter().bits(dpd.enter_time);
                w = w.exit().bits(dpd.exit_time);
                w
            })
        }

        qspi.ifconfig1.write(|w| {
            let w = unsafe { w.sckdelay().bits(80) };
            let w = w.dpmen().exit();
            let w = w.spimode().mode0();
            let w = unsafe { w.sckfreq().bits(3) };
            w
        });

        qspi.xipoffset
            .write(|w| unsafe { w.xipoffset().bits(config.xip_offset) });

        // Enable it
        qspi.enable.write(|w| w.enable().enabled());

        qspi.events_ready.reset();
        qspi.tasks_activate.write(|w| w.tasks_activate().bit(true));
        while qspi.events_ready.read().bits() == 0 {}
        qspi.events_ready.reset();

        // Enable READY interrupt
        SIGNAL.reset();
        qspi.intenset.write(|w| w.ready().set());

        irq.set_handler(irq_handler, core::ptr::null_mut());
        irq.unpend();
        irq.enable();

        Self { inner: qspi }
    }

    pub fn sleep(&mut self) {
        info!("flash: sleeping");
        info!("flash: state = {:?}", self.inner.status.read().bits());
        self.inner.ifconfig1.modify(|r, w| w.dpmen().enter());
        info!("flash: state = {:?}", self.inner.status.read().bits());
        cortex_m::asm::delay(1000000);
        info!("flash: state = {:?}", self.inner.status.read().bits());

        self.inner
            .tasks_deactivate
            .write(|w| w.tasks_deactivate().set_bit());
    }

    pub fn custom_instruction<'a>(
        &'a mut self,
        opcode: u8,
        req: &'a [u8],
        resp: &'a mut [u8],
    ) -> impl Future<Output = Result<(), Error>> + 'a {
        async move {
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

            self.inner.cinstrdat0.write(|w| unsafe { w.bits(dat0) });
            self.inner.cinstrdat1.write(|w| unsafe { w.bits(dat1) });
            self.inner.events_ready.reset();
            self.inner.cinstrconf.write(|w| {
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

            SIGNAL.wait().await;

            let dat0 = self.inner.cinstrdat0.read().bits();
            let dat1 = self.inner.cinstrdat1.read().bits();
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
    }
}

impl Flash for Qspi {
    type ReadFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;
    type WriteFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;
    type ErasePageFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;

    fn read<'a>(&'a mut self, address: usize, data: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move {
            let bomb = DropBomb::new();

            assert_eq!(data.as_ptr() as u32 % 4, 0);
            assert_eq!(data.len() as u32 % 4, 0);
            assert_eq!(address as u32 % 4, 0);

            self.inner
                .read
                .src
                .write(|w| unsafe { w.src().bits(address as u32) });
            self.inner
                .read
                .dst
                .write(|w| unsafe { w.dst().bits(data.as_ptr() as u32) });
            self.inner
                .read
                .cnt
                .write(|w| unsafe { w.cnt().bits(data.len() as u32) });

            self.inner.events_ready.reset();
            self.inner
                .tasks_readstart
                .write(|w| w.tasks_readstart().bit(true));

            SIGNAL.wait().await;

            bomb.defuse();

            Ok(())
        }
    }

    fn write<'a>(&'a mut self, address: usize, data: &'a [u8]) -> Self::WriteFuture<'a> {
        async move {
            let bomb = DropBomb::new();

            assert_eq!(data.as_ptr() as u32 % 4, 0);
            assert_eq!(data.len() as u32 % 4, 0);
            assert_eq!(address as u32 % 4, 0);

            self.inner
                .write
                .src
                .write(|w| unsafe { w.src().bits(data.as_ptr() as u32) });
            self.inner
                .write
                .dst
                .write(|w| unsafe { w.dst().bits(address as u32) });
            self.inner
                .write
                .cnt
                .write(|w| unsafe { w.cnt().bits(data.len() as u32) });

            self.inner.events_ready.reset();
            self.inner
                .tasks_writestart
                .write(|w| w.tasks_writestart().bit(true));

            SIGNAL.wait().await;

            bomb.defuse();

            Ok(())
        }
    }

    fn erase<'a>(&'a mut self, address: usize) -> Self::ErasePageFuture<'a> {
        async move {
            let bomb = DropBomb::new();

            assert_eq!(address as u32 % 4096, 0);

            self.inner
                .erase
                .ptr
                .write(|w| unsafe { w.ptr().bits(address as u32) });
            self.inner.erase.len.write(|w| w.len()._4kb());
            self.inner.events_ready.reset();
            self.inner
                .tasks_erasestart
                .write(|w| w.tasks_erasestart().bit(true));

            SIGNAL.wait().await;

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

static SIGNAL: Signal<()> = Signal::new();

unsafe fn irq_handler(_ctx: *mut ()) {
    let p = crate::pac::Peripherals::steal().QSPI;
    if p.events_ready.read().events_ready().bit_is_set() {
        p.events_ready.reset();
        info!("qspi ready");
        SIGNAL.signal(());
    }
}
