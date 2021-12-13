use core::cell::RefCell;
use core::marker::PhantomData;
use core::pin::Pin;

use usb_device::bus::UsbBus;
use usb_device::class::UsbClass;
use usb_device::device::UsbDevice;

mod cdc_acm;
pub mod usb_serial;

use crate::peripheral::{PeripheralMutex, PeripheralState, StateStorage};
use embassy::interrupt::Interrupt;
pub use usb_serial::{ReadInterface, UsbSerial, WriteInterface};

/// Marker trait to mark an interrupt to be used with the [`Usb`] abstraction.
pub unsafe trait USBInterrupt: Interrupt + Send {}

pub struct State<'bus, B, T, I>(StateStorage<StateInner<'bus, B, T, I>>)
where
    B: UsbBus,
    T: ClassSet<B>,
    I: USBInterrupt;

impl<'bus, B, T, I> State<'bus, B, T, I>
where
    B: UsbBus,
    T: ClassSet<B>,
    I: USBInterrupt,
{
    pub fn new() -> Self {
        Self(StateStorage::new())
    }
}

pub(crate) struct StateInner<'bus, B, T, I>
where
    B: UsbBus,
    T: ClassSet<B>,
    I: USBInterrupt,
{
    device: UsbDevice<'bus, B>,
    pub(crate) classes: T,
    _interrupt: PhantomData<I>,
}

pub struct Usb<'bus, B, T, I>
where
    B: UsbBus,
    T: ClassSet<B>,
    I: USBInterrupt,
{
    // Don't you dare moving out `PeripheralMutex`
    inner: RefCell<PeripheralMutex<'bus, StateInner<'bus, B, T, I>>>,
}

impl<'bus, B, T, I> Usb<'bus, B, T, I>
where
    B: UsbBus,
    T: ClassSet<B>,
    I: USBInterrupt,
{
    /// safety: the returned instance is not leak-safe
    pub unsafe fn new<S: IntoClassSet<B, T>>(
        state: &'bus mut State<'bus, B, T, I>,
        device: UsbDevice<'bus, B>,
        class_set: S,
        irq: I,
    ) -> Self {
        let mutex = PeripheralMutex::new_unchecked(irq, &mut state.0, || StateInner {
            device,
            classes: class_set.into_class_set(),
            _interrupt: PhantomData,
        });
        Self {
            inner: RefCell::new(mutex),
        }
    }
}

impl<'bus, 'c, B, T, I> Usb<'bus, B, T, I>
where
    B: UsbBus,
    T: ClassSet<B> + SerialState<'bus, 'c, B, Index0>,
    I: USBInterrupt,
{
    /// Take a serial class that was passed as the first class in a tuple
    pub fn take_serial_0<'a>(
        self: Pin<&'a Self>,
    ) -> (
        ReadInterface<'a, 'bus, 'c, Index0, B, T, I>,
        WriteInterface<'a, 'bus, 'c, Index0, B, T, I>,
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

impl<'bus, 'c, B, T, I> Usb<'bus, B, T, I>
where
    B: UsbBus,
    T: ClassSet<B> + SerialState<'bus, 'c, B, Index1>,
    I: USBInterrupt,
{
    /// Take a serial class that was passed as the second class in a tuple
    pub fn take_serial_1<'a>(
        self: Pin<&'a Self>,
    ) -> (
        ReadInterface<'a, 'bus, 'c, Index1, B, T, I>,
        WriteInterface<'a, 'bus, 'c, Index1, B, T, I>,
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

impl<'bus, B, T, I> PeripheralState for StateInner<'bus, B, T, I>
where
    B: UsbBus,
    T: ClassSet<B>,
    I: USBInterrupt,
{
    type Interrupt = I;
    fn on_interrupt(&mut self) {
        self.classes.poll_all(&mut self.device);
    }
}

pub trait ClassSet<B: UsbBus>: Send {
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

/// The first class into a [`ClassSet`]
pub struct Index0;

/// The second class into a [`ClassSet`]
pub struct Index1;

impl<B, C1> ClassSet<B> for ClassSet1<B, C1>
where
    B: UsbBus + Send,
    C1: UsbClass<B> + Send,
{
    fn poll_all(&mut self, device: &mut UsbDevice<'_, B>) -> bool {
        device.poll(&mut [&mut self.class])
    }
}

impl<B, C1, C2> ClassSet<B> for ClassSet2<B, C1, C2>
where
    B: UsbBus + Send,
    C1: UsbClass<B> + Send,
    C2: UsbClass<B> + Send,
{
    fn poll_all(&mut self, device: &mut UsbDevice<'_, B>) -> bool {
        device.poll(&mut [&mut self.class1, &mut self.class2])
    }
}

impl<B, C1> IntoClassSet<B, ClassSet1<B, C1>> for C1
where
    B: UsbBus + Send,
    C1: UsbClass<B> + Send,
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
    B: UsbBus + Send,
    C1: UsbClass<B> + Send,
    C2: UsbClass<B> + Send,
{
    fn into_class_set(self) -> ClassSet2<B, C1, C2> {
        ClassSet2 {
            class1: self.0,
            class2: self.1,
            _bus: PhantomData,
        }
    }
}

/// Trait for a USB State that has a serial class inside
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
