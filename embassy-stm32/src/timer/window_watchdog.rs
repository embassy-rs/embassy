use core::cell::RefCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_cortex_m::peripheral::{PeripheralMutex, PeripheralState, StateStorage};
use embassy_sync::waitqueue::WakerRegistration;
use embassy_time::Duration;
use stm32_metapac::timer::vals;

use crate::gpio::sealed::AFType;
use crate::pwm::*;
use crate::Peripheral;

/* requires VALID_THRESHOLD edges within range before reporting ok */
const VALID_THRESHOLD: u8 = 2;

pub struct State<'a, T: CaptureCompare16bitInstance>(StateStorage<StateInner<'a, T>>);
impl<'a, T: CaptureCompare16bitInstance> State<'a, T> {
    pub const fn new() -> Self {
        Self(StateStorage::new())
    }
}

struct StateInner<'a, T: CaptureCompare16bitInstance> {
    phantom: PhantomData<&'a mut T>,
    waker: WakerRegistration,
    min: u16,
    count: u8,
}
unsafe impl<'a, T: CaptureCompare16bitInstance> Send for StateInner<'a, T> {}

pub struct WindowWatchdog<'a, T: CaptureCompare16bitInstance> {
    inner: RefCell<PeripheralMutex<'a, StateInner<'a, T>>>,
}
unsafe impl<'a, T: CaptureCompare16bitInstance> Send for WindowWatchdog<'a, T> {}

impl<'a, T: CaptureCompare16bitInstance> WindowWatchdog<'a, T> {
    pub fn new(
        state: &'a mut State<'a, T>,
        tim: impl Peripheral<P = T> + 'a,
        input: impl Peripheral<P = impl Channel1Pin<T>> + 'a,
        irq: impl Peripheral<P = T::Interrupt> + 'a,
        min_interval: Duration,
        max_interval: Duration,
    ) -> Self {
        assert!(min_interval < max_interval);

        let mut tim = tim.into_ref();
        let input = input.into_ref();

        T::enable();
        <T as crate::rcc::sealed::RccPeripheral>::reset();

        let clk = T::frequency().0 as u64;
        let psc: u16 = unwrap!(((max_interval.as_micros() * clk).div_ceil((1 << 16) * 1000000) - 1).try_into());
        let arr: u16 = unwrap!((max_interval.as_micros() * clk / ((psc as u64 + 1) * 1000000) - 1).try_into());
        let min: u16 = unwrap!(((min_interval.as_micros() * clk).div_ceil((psc as u64 + 1) * 1000000) - 1).try_into());

        let r = T::regs_gp16();
        unsafe {
            input.set_as_af(input.af_num(), AFType::Input);
            r.psc().write(|v| v.set_psc(psc));
            r.arr().write(|v| v.set_arr(arr));
            r.ccmr_input(0).write(|v| v.set_ccs(0, vals::CcmrInputCcs(1)));
	    r.cr1().write(|v| v.set_urs(vals::Urs::COUNTERONLY));
            r.ccer().write(|v| {
                v.set_ccnp(0, true);
                v.set_ccp(0, true);
                v.set_cce(0, true);
            });
            r.dier().write(|v| {
                v.set_ccie(0, true);
                v.set_uie(true);
            });
        }

        tim.start();

        Self {
            inner: RefCell::new(PeripheralMutex::new(irq, &mut state.0, move || StateInner {
                phantom: PhantomData,
                waker: WakerRegistration::new(),
                min,
                count: 0,
            })),
        }
    }

    pub fn ok(&self) -> bool {
        self.inner.borrow_mut().with(|state| state.ok())
    }

    pub async fn state_change(&self, old_state: bool) {
        poll_fn(|cx| {
            let mut inner = self.inner.borrow_mut();
            inner.with(|state| {
                if state.ok() != old_state {
                    return Poll::Ready(());
                }
                state.waker.register(cx.waker());
                Poll::Pending
            })
        })
        .await
    }
}

impl<'a, T: CaptureCompare16bitInstance> StateInner<'a, T> {
    fn isr(&mut self) {
        let r = T::regs_gp16();
        let sr = unsafe { r.sr().read() };

        if sr.0 == 0 {
            /* spurious interrupt */
            return;
        }

        let was_ok = self.ok();

        if sr.ccif(0) && unsafe { r.ccr(0).read().ccr() } >= self.min {
            /* got a valid pulse */
            self.count = self.count.saturating_add(1);

            /* reset timer */
            unsafe { r.egr().write(|r| r.set_ug(true)) };
        } else {
            self.count = 0;
        }

        /* ack interrupt */
        unsafe { r.sr().write(|w| w.0) };

        /* wakeup watchers on state change */
        if was_ok != self.ok() {
            self.waker.wake();
        }
    }

    fn ok(&self) -> bool {
        self.count >= VALID_THRESHOLD
    }
}

impl<'a, T: CaptureCompare16bitInstance> PeripheralState for StateInner<'a, T> {
    type Interrupt = T::Interrupt;
    fn on_interrupt(&mut self) {
        self.isr();
    }
}
