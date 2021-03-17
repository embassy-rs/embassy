use crate::interrupt;
use core::future::Future;
use core::pin::Pin;
use embassy::interrupt::Interrupt;
use embassy::traits::qei::WaitForRotate;
use embedded_hal::Direction;
use embedded_hal::Qei as THQei;
use stm32f4xx_hal::pac::TIM2;
use stm32f4xx_hal::qei::{Pins, Qei as HalQei};

pub struct Qei<T: Instance, PINS> {
    qei: HalQei<T, PINS>,
    int: T::Interrupt,
}

impl<PINS: Pins<TIM2>> Qei<TIM2, PINS> {
    pub fn tim2(tim: TIM2, pins: PINS, interrupt: interrupt::TIM2) -> Self {
        let qei = HalQei::tim2(tim, pins);

        let tim = unsafe {
            &mut *(stm32f4xx_hal::stm32::TIM2::ptr()
                as *mut stm32f4xx_hal::stm32::tim2::RegisterBlock)
        };
        /*
            enable qei interrupt
        */
        tim.dier.write(|w| w.uie().set_bit());

        Qei {
            qei: qei,
            int: interrupt,
        }
    }
}

impl<PINS: Pins<TIM2> + 'static> WaitForRotate for Qei<TIM2, PINS> {
    type RotateFuture<'a> = impl Future<Output = Direction> + 'a;

    fn wait_for_rotate<'a>(
        self: Pin<&'a mut Self>,
        count_down: u16,
        count_up: u16,
    ) -> Self::RotateFuture<'a> {
        let s = unsafe { self.get_unchecked_mut() };

        let tim = unsafe {
            &mut *(stm32f4xx_hal::stm32::TIM2::ptr()
                as *mut stm32f4xx_hal::stm32::tim2::RegisterBlock)
        };

        /*
            the interrupt will be reached at zero or the max count
            write the total range to the qei.
        */
        tim.arr
            .write(|w| unsafe { w.bits((count_down + count_up) as u32) });

        /*
            set timer to the correct value in the range
        */
        tim.cnt.write(|w| unsafe { w.bits(count_down as u32) });

        /*
            clear interrupt flag
        */
        tim.sr.write(|w| w.uif().clear_bit());

        async move {
            embassy::util::InterruptFuture::new(&mut s.int).await;

            if tim.cnt.read().bits() == 0 {
                Direction::Downcounting
            } else if tim.cnt.read() == count_down + count_up {
                Direction::Upcounting
            } else {
                panic!("unexpected value")
            }
        }
    }
}

mod sealed {
    pub trait Sealed {}
}

pub trait Instance: sealed::Sealed {
    type Interrupt: interrupt::Interrupt;
}

#[cfg(feature = "stm32f405")]
impl sealed::Sealed for TIM2 {}
#[cfg(feature = "stm32f405")]
impl Instance for TIM2 {
    type Interrupt = interrupt::TIM2;
}
