use core::cell::RefCell;
use core::marker::PhantomData;
use core::pin::Pin;

use usb_device::bus::UsbBus;
use usb_device::class::UsbClass;
use usb_device::device::UsbDevice;

use crate::interrupt;
use crate::usb_serial::{ReadInterface, UsbSerial, WriteInterface};
use embassy_extras::peripheral::{PeripheralMutex, PeripheralState};

pub struct State<'bus, B, T>
where
    B: UsbBus,
    T: ClassSet<B>,
{
    device: UsbDevice<'bus, B>,
    pub(crate) classes: T,
}

pub struct Usb<'bus, B, T>
where
    B: UsbBus,
    T: ClassSet<B>,
{
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
        irq: interrupt::OTG_FS,
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
        mutex.register_interrupt();
    }
}

impl<'bus, 'c, B, T> Usb<'bus, B, T>
where
    B: UsbBus,
    T: ClassSet<B> + SerialState<'bus, 'c, B, Index0>,
{
    pub fn take_serial_0<'a>(
        self: Pin<&'a Self>,
    ) -> (
        ReadInterface<'a, 'bus, 'c, Index0, B, T>,
        WriteInterface<'a, 'bus, 'c, Index0, B, T>,
    ) {
        let this = self.get_ref();

        let r = ReadInterface {
            inner: &this.inner,
            _buf_lifetime: PhantomData,
            _index: PhantomData,
        };

        let w = WriteInterface {
            inner: &this.inner,
            _buf_lifetime: PhantomData,
            _index: PhantomData,
        };
        (r, w)
    }
}

impl<'bus, 'c, B, T> Usb<'bus, B, T>
where
    B: UsbBus,
    T: ClassSet<B> + SerialState<'bus, 'c, B, Index1>,
{
    pub fn take_serial_1<'a>(
        self: Pin<&'a Self>,
    ) -> (
        ReadInterface<'a, 'bus, 'c, Index1, B, T>,
        WriteInterface<'a, 'bus, 'c, Index1, B, T>,
    ) {
        let this = self.get_ref();

        let r = ReadInterface {
            inner: &this.inner,
            _buf_lifetime: PhantomData,
            _index: PhantomData,
        };

        let w = WriteInterface {
            inner: &this.inner,
            _buf_lifetime: PhantomData,
            _index: PhantomData,
        };
        (r, w)
    }
}

impl<'bus, B, T> PeripheralState for State<'bus, B, T>
where
    B: UsbBus,
    T: ClassSet<B>,
{
    type Interrupt = interrupt::OTG_FS;
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

pub struct ClassSet1<B, C1>
where
    B: UsbBus,
    C1: UsbClass<B>,
{
    class: C1,
    _bus: PhantomData<B>,
}

pub struct ClassSet2<B, C1, C2>
where
    B: UsbBus,
    C1: UsbClass<B>,
    C2: UsbClass<B>,
{
    class1: C1,
    class2: C2,
    _bus: PhantomData<B>,
}

pub struct Index0;
pub struct Index1;

impl<B, C1> ClassSet<B> for ClassSet1<B, C1>
where
    B: UsbBus,
    C1: UsbClass<B>,
{
    fn poll_all(&mut self, device: &mut UsbDevice<'_, B>) -> bool {
        device.poll(&mut [&mut self.class])
    }
}

impl<B, C1, C2> ClassSet<B> for ClassSet2<B, C1, C2>
where
    B: UsbBus,
    C1: UsbClass<B>,
    C2: UsbClass<B>,
{
    fn poll_all(&mut self, device: &mut UsbDevice<'_, B>) -> bool {
        device.poll(&mut [&mut self.class1, &mut self.class2])
    }
}

impl<B, C1> IntoClassSet<B, ClassSet1<B, C1>> for C1
where
    B: UsbBus,
    C1: UsbClass<B>,
{
    fn into_class_set(self) -> ClassSet1<B, C1> {
        ClassSet1 {
            class: self,
            _bus: PhantomData,
        }
    }
}

impl<B, C1, C2> IntoClassSet<B, ClassSet2<B, C1, C2>> for (C1, C2)
where
    B: UsbBus,
    C1: UsbClass<B>,
    C2: UsbClass<B>,
{
    fn into_class_set(self) -> ClassSet2<B, C1, C2> {
        ClassSet2 {
            class1: self.0,
            class2: self.1,
            _bus: PhantomData,
        }
    }
}

pub trait SerialState<'bus, 'a, B: UsbBus, I> {
    fn get_serial(&mut self) -> &mut UsbSerial<'bus, 'a, B>;
}

impl<'bus, 'a, B: UsbBus> SerialState<'bus, 'a, B, Index0>
    for ClassSet1<B, UsbSerial<'bus, 'a, B>>
{
    fn get_serial(&mut self) -> &mut UsbSerial<'bus, 'a, B> {
        &mut self.class
    }
}

impl<'bus, 'a, B, C2> SerialState<'bus, 'a, B, Index0> for ClassSet2<B, UsbSerial<'bus, 'a, B>, C2>
where
    B: UsbBus,
    C2: UsbClass<B>,
{
    fn get_serial(&mut self) -> &mut UsbSerial<'bus, 'a, B> {
        &mut self.class1
    }
}

impl<'bus, 'a, B, C1> SerialState<'bus, 'a, B, Index1> for ClassSet2<B, C1, UsbSerial<'bus, 'a, B>>
where
    B: UsbBus,
    C1: UsbClass<B>,
{
    fn get_serial(&mut self) -> &mut UsbSerial<'bus, 'a, B> {
        &mut self.class2
    }
}
