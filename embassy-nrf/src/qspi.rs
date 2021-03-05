use core::future::Future;
use core::pin::Pin;
use core::task::Poll;

use crate::fmt::{assert, assert_eq, *};
use crate::hal::gpio::{Output, Pin as GpioPin, PushPull};
use crate::interrupt::{self};
use crate::pac::QSPI;

pub use crate::pac::qspi::ifconfig0::ADDRMODE_A as AddressMode;
pub use crate::pac::qspi::ifconfig0::PPSIZE_A as WritePageSize;
pub use crate::pac::qspi::ifconfig0::READOC_A as ReadOpcode;
pub use crate::pac::qspi::ifconfig0::WRITEOC_A as WriteOpcode;
use crate::util::peripheral::{PeripheralMutex, PeripheralState};

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
use embassy::util::{DropBomb, WakerRegistration};
use futures::future::poll_fn;

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

struct State {
    inner: QSPI,
    waker: WakerRegistration,
}

pub struct Qspi {
    inner: PeripheralMutex<State>,
}

impl Qspi {
    pub fn new(qspi: QSPI, irq: interrupt::QSPI, config: Config) -> Self {
        qspi.psel.sck.write(|w| {
            let pin = &config.pins.sck;
            unsafe { w.bits(pin.psel_bits()) };
            w.connect().connected()
        });
        qspi.psel.csn.write(|w| {
            let pin = &config.pins.csn;
            unsafe { w.bits(pin.psel_bits()) };
            w.connect().connected()
        });
        qspi.psel.io0.write(|w| {
            let pin = &config.pins.io0;
            unsafe { w.bits(pin.psel_bits()) };
            w.connect().connected()
        });
        qspi.psel.io1.write(|w| {
            let pin = &config.pins.io1;
            unsafe { w.bits(pin.psel_bits()) };
            w.connect().connected()
        });
        qspi.psel.io2.write(|w| {
            if let Some(ref pin) = config.pins.io2 {
                unsafe { w.bits(pin.psel_bits()) };
                w.connect().connected()
            } else {
                w.connect().disconnected()
            }
        });
        qspi.psel.io3.write(|w| {
            if let Some(ref pin) = config.pins.io3 {
                unsafe { w.bits(pin.psel_bits()) };
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

        Self {
            inner: PeripheralMutex::new(
                State {
                    inner: qspi,
                    waker: WakerRegistration::new(),
                },
                irq,
            ),
        }
    }

    pub fn sleep(self: Pin<&mut Self>) {
        self.inner().with(|s, _| {
            info!("flash: sleeping");
            info!("flash: state = {:?}", s.inner.status.read().bits());
            s.inner.ifconfig1.modify(|_, w| w.dpmen().enter());
            info!("flash: state = {:?}", s.inner.status.read().bits());
            cortex_m::asm::delay(1000000);
            info!("flash: state = {:?}", s.inner.status.read().bits());

            s.inner
                .tasks_deactivate
                .write(|w| w.tasks_deactivate().set_bit());
        });
    }

    pub async fn custom_instruction<'a>(
        mut self: Pin<&'a mut Self>,
        opcode: u8,
        req: &'a [u8],
        resp: &'a mut [u8],
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

        self.as_mut().inner().with(|s, _| {
            s.inner.cinstrdat0.write(|w| unsafe { w.bits(dat0) });
            s.inner.cinstrdat1.write(|w| unsafe { w.bits(dat1) });

            s.inner.events_ready.reset();
            s.inner.intenset.write(|w| w.ready().set());

            s.inner.cinstrconf.write(|w| {
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
        });

        self.as_mut().wait_ready().await;

        self.as_mut().inner().with(|s, _| {
            let dat0 = s.inner.cinstrdat0.read().bits();
            let dat1 = s.inner.cinstrdat1.read().bits();
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
        });

        bomb.defuse();

        Ok(())
    }

    fn inner(self: Pin<&mut Self>) -> Pin<&mut PeripheralMutex<State>> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().inner) }
    }

    pub fn free(self: Pin<&mut Self>) -> (QSPI, interrupt::QSPI) {
        let (state, irq) = self.inner().free();
        (state.inner, irq)
    }

    fn wait_ready<'a>(mut self: Pin<&'a mut Self>) -> impl Future<Output = ()> + 'a {
        poll_fn(move |cx| {
            self.as_mut().inner().with(|s, _irq| {
                if s.inner.events_ready.read().bits() != 0 {
                    return Poll::Ready(());
                }
                s.waker.register(cx.waker());
                Poll::Pending
            })
        })
    }
}

impl Flash for Qspi {
    type ReadFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;
    type WriteFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;
    type ErasePageFuture<'a> = impl Future<Output = Result<(), Error>> + 'a;

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

            self.as_mut().inner().with(|s, _| {
                s.inner
                    .read
                    .src
                    .write(|w| unsafe { w.src().bits(address as u32) });
                s.inner
                    .read
                    .dst
                    .write(|w| unsafe { w.dst().bits(data.as_ptr() as u32) });
                s.inner
                    .read
                    .cnt
                    .write(|w| unsafe { w.cnt().bits(data.len() as u32) });

                s.inner.events_ready.reset();
                s.inner.intenset.write(|w| w.ready().set());
                s.inner
                    .tasks_readstart
                    .write(|w| w.tasks_readstart().bit(true));
            });

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

            self.as_mut().inner().with(|s, _| {
                s.inner
                    .write
                    .src
                    .write(|w| unsafe { w.src().bits(data.as_ptr() as u32) });
                s.inner
                    .write
                    .dst
                    .write(|w| unsafe { w.dst().bits(address as u32) });
                s.inner
                    .write
                    .cnt
                    .write(|w| unsafe { w.cnt().bits(data.len() as u32) });

                s.inner.events_ready.reset();
                s.inner.intenset.write(|w| w.ready().set());
                s.inner
                    .tasks_writestart
                    .write(|w| w.tasks_writestart().bit(true));
            });

            self.as_mut().wait_ready().await;

            bomb.defuse();

            Ok(())
        }
    }

    fn erase<'a>(mut self: Pin<&'a mut Self>, address: usize) -> Self::ErasePageFuture<'a> {
        async move {
            let bomb = DropBomb::new();

            assert_eq!(address as u32 % 4096, 0);

            self.as_mut().inner().with(|s, _| {
                s.inner
                    .erase
                    .ptr
                    .write(|w| unsafe { w.ptr().bits(address as u32) });
                s.inner.erase.len.write(|w| w.len()._4kb());

                s.inner.events_ready.reset();
                s.inner.intenset.write(|w| w.ready().set());
                s.inner
                    .tasks_erasestart
                    .write(|w| w.tasks_erasestart().bit(true));
            });

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

impl PeripheralState for State {
    type Interrupt = interrupt::QSPI;

    fn on_interrupt(&mut self) {
        if self.inner.events_ready.read().bits() != 0 {
            self.inner.intenclr.write(|w| w.ready().clear());
            self.waker.wake()
        }
    }
}
