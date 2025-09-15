//! Pin Interrupt module.
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin as FuturePin;
use core::task::{Context, Poll};

use critical_section::Mutex;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{self, AnyPin, Level, SealedPin};
use crate::pac::{interrupt, INPUTMUX, PINT, SYSCON};
use crate::Peri;

struct PinInterrupt {
    assigned: bool,
    waker: AtomicWaker,
    /// If true, the interrupt was triggered due to this PinInterrupt. This is used to determine if
    /// an [InputFuture] should return Poll::Ready.
    at_fault: bool,
}

impl PinInterrupt {
    pub fn interrupt_active(&self) -> bool {
        self.assigned
    }

    /// Mark the interrupt as assigned to a pin.
    pub fn enable(&mut self) {
        self.assigned = true;
        self.at_fault = false;
    }

    /// Mark the interrupt as available.
    pub fn disable(&mut self) {
        self.assigned = false;
        self.at_fault = false;
    }

    /// Returns true if the interrupt was triggered due to this PinInterrupt.
    ///
    /// If this function returns true, it will also reset the at_fault flag.
    pub fn at_fault(&mut self) -> bool {
        let val = self.at_fault;
        self.at_fault = false;
        val
    }

    /// Set the at_fault flag to true.
    pub fn fault(&mut self) {
        self.at_fault = true;
    }
}

const INTERUPT_COUNT: usize = 8;
static PIN_INTERRUPTS: Mutex<RefCell<[PinInterrupt; INTERUPT_COUNT]>> = Mutex::new(RefCell::new(
    [const {
        PinInterrupt {
            assigned: false,
            waker: AtomicWaker::new(),
            at_fault: false,
        }
    }; INTERUPT_COUNT],
));

fn next_available_interrupt() -> Option<usize> {
    critical_section::with(|cs| {
        for (i, pin_interrupt) in PIN_INTERRUPTS.borrow(cs).borrow().iter().enumerate() {
            if !pin_interrupt.interrupt_active() {
                return Some(i);
            }
        }

        None
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Edge {
    Rising,
    Falling,
    Both,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InterruptOn {
    Level(Level),
    Edge(Edge),
}

pub(crate) fn init() {
    SYSCON.ahbclkctrl0().modify(|w| w.set_pint(true));

    // Enable interrupts
    unsafe {
        interrupt::PIN_INT0.enable();
        interrupt::PIN_INT1.enable();
        interrupt::PIN_INT2.enable();
        interrupt::PIN_INT3.enable();
        interrupt::PIN_INT4.enable();
        interrupt::PIN_INT5.enable();
        interrupt::PIN_INT6.enable();
        interrupt::PIN_INT7.enable();
    };

    info!("Pin interrupts initialized");
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct InputFuture<'d> {
    #[allow(dead_code)]
    pin: Peri<'d, AnyPin>,
    interrupt_number: usize,
}

impl<'d> InputFuture<'d> {
    /// Create a new input future. Returns None if all interrupts are in use.
    fn new(pin: Peri<'d, impl gpio::Pin>, interrupt_on: InterruptOn) -> Option<Self> {
        let pin = pin.into();
        let interrupt_number = next_available_interrupt()?;

        // Clear interrupt, just in case
        PINT.rise().write(|w| w.set_rdet(1 << interrupt_number));
        PINT.fall().write(|w| w.set_fdet(1 << interrupt_number));

        // Enable input multiplexing on pin interrupt register 0 for pin (32*bank + pin_number)
        INPUTMUX
            .pintsel(interrupt_number as usize)
            .write(|w| w.set_intpin(32 * pin.pin_bank() as u8 + pin.pin_number()));

        match interrupt_on {
            InterruptOn::Level(level) => {
                // Set pin interrupt register to edge sensitive or level sensitive
                // 0 = edge sensitive, 1 = level sensitive
                PINT.isel().modify(|w| w.set_pmode(w.pmode() | (1 << interrupt_number)));

                // Enable level interrupt.
                //
                // Note: Level sensitive interrupts are enabled by the same register as rising edge
                // is activated.

                // 0 = no-op, 1 = enable
                PINT.sienr().write(|w| w.set_setenrl(1 << interrupt_number));

                // Set active level
                match level {
                    Level::Low => {
                        // 0 = no-op, 1 = select LOW
                        PINT.cienf().write(|w| w.set_cenaf(1 << interrupt_number));
                    }
                    Level::High => {
                        // 0 = no-op, 1 = select HIGH
                        PINT.sienf().write(|w| w.set_setenaf(1 << interrupt_number));
                    }
                }
            }
            InterruptOn::Edge(edge) => {
                // Set pin interrupt register to edge sensitive or level sensitive
                // 0 = edge sensitive, 1 = level sensitive
                PINT.isel()
                    .modify(|w| w.set_pmode(w.pmode() & !(1 << interrupt_number)));

                // Enable rising/falling edge detection
                match edge {
                    Edge::Rising => {
                        // 0 = no-op, 1 = enable rising edge
                        PINT.sienr().write(|w| w.set_setenrl(1 << interrupt_number));
                        // 0 = no-op, 1 = disable falling edge
                        PINT.cienf().write(|w| w.set_cenaf(1 << interrupt_number));
                    }
                    Edge::Falling => {
                        // 0 = no-op, 1 = enable falling edge
                        PINT.sienf().write(|w| w.set_setenaf(1 << interrupt_number));
                        // 0 = no-op, 1 = disable rising edge
                        PINT.cienr().write(|w| w.set_cenrl(1 << interrupt_number));
                    }
                    Edge::Both => {
                        // 0 = no-op, 1 = enable
                        PINT.sienr().write(|w| w.set_setenrl(1 << interrupt_number));
                        PINT.sienf().write(|w| w.set_setenaf(1 << interrupt_number));
                    }
                }
            }
        }

        critical_section::with(|cs| {
            let mut pin_interrupts = PIN_INTERRUPTS.borrow(cs).borrow_mut();
            let pin_interrupt = &mut pin_interrupts[interrupt_number];

            pin_interrupt.enable();
        });

        Some(Self { pin, interrupt_number })
    }

    /// Returns true if the interrupt was triggered for this pin.
    fn interrupt_triggered(&self) -> bool {
        let interrupt_number = self.interrupt_number;

        // Initially, we determine if the interrupt was triggered by this InputFuture by checking
        // the flags of the interrupt_number. However, by the time we get to this point, the
        // interrupt may have been triggered again, so we needed to clear the cpu flags immediately.
        // As a solution, we mark which [PinInterrupt] is responsible for the interrupt ("at fault")
        critical_section::with(|cs| {
            let mut pin_interrupts = PIN_INTERRUPTS.borrow(cs).borrow_mut();
            let pin_interrupt = &mut pin_interrupts[interrupt_number];

            pin_interrupt.at_fault()
        })
    }
}

impl<'d> Drop for InputFuture<'d> {
    fn drop(&mut self) {
        let interrupt_number = self.interrupt_number;

        // Disable pin interrupt
        // 0 = no-op, 1 = disable
        PINT.cienr().write(|w| w.set_cenrl(1 << interrupt_number));
        PINT.cienf().write(|w| w.set_cenaf(1 << interrupt_number));

        critical_section::with(|cs| {
            let mut pin_interrupts = PIN_INTERRUPTS.borrow(cs).borrow_mut();
            let pin_interrupt = &mut pin_interrupts[interrupt_number];

            pin_interrupt.disable();
        });
    }
}

impl<'d> Future for InputFuture<'d> {
    type Output = ();

    fn poll(self: FuturePin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let interrupt_number = self.interrupt_number;

        critical_section::with(|cs| {
            let mut pin_interrupts = PIN_INTERRUPTS.borrow(cs).borrow_mut();
            let pin_interrupt = &mut pin_interrupts[interrupt_number];

            pin_interrupt.waker.register(cx.waker());
        });

        if self.interrupt_triggered() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

fn handle_interrupt(interrupt_number: usize) {
    PINT.rise().write(|w| w.set_rdet(1 << interrupt_number));
    PINT.fall().write(|w| w.set_fdet(1 << interrupt_number));

    critical_section::with(|cs| {
        let mut pin_interrupts = PIN_INTERRUPTS.borrow(cs).borrow_mut();
        let pin_interrupt = &mut pin_interrupts[interrupt_number];

        pin_interrupt.fault();
        pin_interrupt.waker.wake();
    });
}

#[allow(non_snake_case)]
#[interrupt]
fn PIN_INT0() {
    handle_interrupt(0);
}

#[allow(non_snake_case)]
#[interrupt]
fn PIN_INT1() {
    handle_interrupt(1);
}

#[allow(non_snake_case)]
#[interrupt]
fn PIN_INT2() {
    handle_interrupt(2);
}

#[allow(non_snake_case)]
#[interrupt]
fn PIN_INT3() {
    handle_interrupt(3);
}

#[allow(non_snake_case)]
#[interrupt]
fn PIN_INT4() {
    handle_interrupt(4);
}

#[allow(non_snake_case)]
#[interrupt]
fn PIN_INT5() {
    handle_interrupt(5);
}

#[allow(non_snake_case)]
#[interrupt]
fn PIN_INT6() {
    handle_interrupt(6);
}

#[allow(non_snake_case)]
#[interrupt]
fn PIN_INT7() {
    handle_interrupt(7);
}

impl gpio::Flex<'_> {
    /// Wait for a falling or rising edge on the pin. You can have at most 8 pins waiting. If you
    /// try to wait for more than 8 pins, this function will return `None`.
    pub async fn wait_for_any_edge(&mut self) -> Option<()> {
        InputFuture::new(self.pin.reborrow(), InterruptOn::Edge(Edge::Both))?.await;
        Some(())
    }

    /// Wait for a falling edge on the pin. You can have at most 8 pins waiting. If you try to wait
    /// for more than 8 pins, this function will return `None`.
    pub async fn wait_for_falling_edge(&mut self) -> Option<()> {
        InputFuture::new(self.pin.reborrow(), InterruptOn::Edge(Edge::Falling))?.await;
        Some(())
    }

    /// Wait for a rising edge on the pin. You can have at most 8 pins waiting. If you try to wait
    /// for more than 8 pins, this function will return `None`.
    pub async fn wait_for_rising_edge(&mut self) -> Option<()> {
        InputFuture::new(self.pin.reborrow(), InterruptOn::Edge(Edge::Rising))?.await;
        Some(())
    }

    /// Wait for a low level on the pin. You can have at most 8 pins waiting. If you try to wait for
    /// more than 8 pins, this function will return `None`.
    pub async fn wait_for_low(&mut self) -> Option<()> {
        InputFuture::new(self.pin.reborrow(), InterruptOn::Level(Level::Low))?.await;
        Some(())
    }

    /// Wait for a high level on the pin. You can have at most 8 pins waiting. If you try to wait for
    /// more than 8 pins, this function will return `None`.
    pub async fn wait_for_high(&mut self) -> Option<()> {
        InputFuture::new(self.pin.reborrow(), InterruptOn::Level(Level::High))?.await;
        Some(())
    }
}

impl gpio::Input<'_> {
    /// Wait for a falling or rising edge on the pin. You can have at most 8 pins waiting. If you
    /// try to wait for more than 8 pins, this function will return `None`.
    pub async fn wait_for_any_edge(&mut self) -> Option<()> {
        self.pin.wait_for_any_edge().await
    }

    /// Wait for a falling edge on the pin. You can have at most 8 pins waiting. If you try to wait
    /// for more than 8 pins, this function will return `None`.
    pub async fn wait_for_falling_edge(&mut self) -> Option<()> {
        self.pin.wait_for_falling_edge().await
    }

    /// Wait for a rising edge on the pin. You can have at most 8 pins waiting. If you try to wait
    /// for more than 8 pins, this function will return `None`.
    pub async fn wait_for_rising_edge(&mut self) -> Option<()> {
        self.pin.wait_for_rising_edge().await
    }

    /// Wait for a low level on the pin. You can have at most 8 pins waiting. If you try to wait for
    /// more than 8 pins, this function will return `None`.
    pub async fn wait_for_low(&mut self) -> Option<()> {
        self.pin.wait_for_low().await
    }

    /// Wait for a high level on the pin. You can have at most 8 pins waiting. If you try to wait for
    /// more than 8 pins, this function will return `None`.
    pub async fn wait_for_high(&mut self) -> Option<()> {
        self.pin.wait_for_high().await
    }
}

impl gpio::Output<'_> {
    /// Wait for a falling or rising edge on the pin. You can have at most 8 pins waiting. If you
    /// try to wait for more than 8 pins, this function will return `None`.
    pub async fn wait_for_any_edge(&mut self) -> Option<()> {
        self.pin.wait_for_any_edge().await
    }

    /// Wait for a falling edge on the pin. You can have at most 8 pins waiting. If you try to wait
    /// for more than 8 pins, this function will return `None`.
    pub async fn wait_for_falling_edge(&mut self) -> Option<()> {
        self.pin.wait_for_falling_edge().await
    }

    /// Wait for a rising edge on the pin. You can have at most 8 pins waiting. If you try to wait
    /// for more than 8 pins, this function will return `None`.
    pub async fn wait_for_rising_edge(&mut self) -> Option<()> {
        self.pin.wait_for_rising_edge().await
    }

    /// Wait for a low level on the pin. You can have at most 8 pins waiting. If you try to wait for
    /// more than 8 pins, this function will return `None`.
    pub async fn wait_for_low(&mut self) -> Option<()> {
        self.pin.wait_for_low().await
    }

    /// Wait for a high level on the pin. You can have at most 8 pins waiting. If you try to wait for
    /// more than 8 pins, this function will return `None`.
    pub async fn wait_for_high(&mut self) -> Option<()> {
        self.pin.wait_for_high().await
    }
}
