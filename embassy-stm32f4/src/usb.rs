use core::cell::RefCell;
use core::marker::PhantomData;
use core::pin::Pin;

use usb_device::bus::UsbBus;
use usb_device::class::UsbClass;
use usb_device::device::UsbDevice;

use crate::interrupt;
use crate::usb_serial::{ReadInterface, UsbSerial, WriteInterface};
use crate::util::peripheral::{PeripheralMutex, PeripheralState};

pub struct State<'bus, B: UsbBus, T: ClassSet<B>> {
    device: UsbDevice<'bus, B>,
    pub(crate) classes: T,
}

pub struct Usb<'bus, B: UsbBus, T: ClassSet<B>> {
    // Don't you dare moving out `PeripheralMutex`
    inner: RefCell<PeripheralMutex<State<'bus, B, T>>>,
}

impl<'bus, B, T> Usb<'bus, B, T>
where
    B: UsbBus,
    T: ClassSet<B>,
{
    pub fn new<S: IntoClassSet<B, T>>(
        device: UsbDevice<'bus, B>,
        class_set: S,
        irq: interrupt::OTG_FSInterrupt,
    ) -> Self {
        let state = State {
            device,
            classes: class_set.into_class_set(),
        };
        let mutex = PeripheralMutex::new(state, irq);
        Self {
            inner: RefCell::new(mutex),
        }
    }

    pub fn start(self: Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        let mut mutex = this.inner.borrow_mut();
        let mutex = unsafe { Pin::new_unchecked(&mut *mutex) };

        // Use inner to register the irq
        mutex.with(|_, _| {});
    }
}

impl<'bus, 'c, B, T> Usb<'bus, B, T>
where
    B: UsbBus,
    T: ClassSet<B> + SerialState<'bus, 'c, B>,
{
    pub fn take_serial<'a>(
        self: Pin<&'a Self>,
    ) -> (
        ReadInterface<'a, 'bus, 'c, B, T>,
        WriteInterface<'a, 'bus, 'c, B, T>,
    ) {
        let this = self.get_ref();

        let r = ReadInterface {
            inner: &this.inner,
            _buf_lifetime: PhantomData,
        };

        let w = WriteInterface {
            inner: &this.inner,
            _buf_lifetime: PhantomData,
        };
        (r, w)
    }
}

impl<'bus, B, T> PeripheralState for State<'bus, B, T>
where
    B: UsbBus,
    T: ClassSet<B>,
{
    type Interrupt = interrupt::OTG_FSInterrupt;
    fn on_interrupt(&mut self) {
        self.classes.poll_all(&mut self.device);
    }
}

pub trait ClassSet<B: UsbBus> {
    fn poll_all(&mut self, device: &mut UsbDevice<'_, B>) -> bool;
}

pub trait IntoClassSet<B: UsbBus, C: ClassSet<B>> {
    fn into_class_set(self) -> C;
}

pub struct ClassSet1<B: UsbBus, T: UsbClass<B>> {
    class: T,
    _bus: PhantomData<B>,
}

impl<B, T> ClassSet<B> for ClassSet1<B, T>
where
    B: UsbBus,
    T: UsbClass<B>,
{
    fn poll_all(&mut self, device: &mut UsbDevice<'_, B>) -> bool {
        device.poll(&mut [&mut self.class])
    }
}

impl<B: UsbBus, T: UsbClass<B>> IntoClassSet<B, ClassSet1<B, T>> for T {
    fn into_class_set(self) -> ClassSet1<B, T> {
        ClassSet1 {
            class: self,
            _bus: PhantomData,
        }
    }
}

pub trait SerialState<'bus, 'a, B: UsbBus> {
    fn get_serial(&mut self) -> &mut UsbSerial<'bus, 'a, B>;
}

impl<'bus, 'a, B: UsbBus> SerialState<'bus, 'a, B> for ClassSet1<B, UsbSerial<'bus, 'a, B>> {
    fn get_serial(&mut self) -> &mut UsbSerial<'bus, 'a, B> {
        &mut self.class
    }
}
