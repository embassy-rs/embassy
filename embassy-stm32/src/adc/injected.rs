use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};

use super::AnyAdcChannel;
use crate::adc::{BasicAdcRegs, InjectedAdcRegs, Instance, SealedInjectedAdcRegs};

/// Injected ADC sequence with owned channels.
pub struct InjectedAdc<'a, T: Instance<Regs: InjectedAdcRegs>, const N: usize> {
    _channels: [(AnyAdcChannel<'a, T>, <T::Regs as BasicAdcRegs>::SampleTime); N],
    _phantom: PhantomData<T>,
}

impl<'a, T: Instance<Regs: InjectedAdcRegs>, const N: usize> InjectedAdc<'a, T, N> {
    pub(crate) fn new(channels: [(AnyAdcChannel<'a, T>, <T::Regs as BasicAdcRegs>::SampleTime); N]) -> Self {
        Self {
            _channels: channels,
            _phantom: PhantomData,
        }
    }

    pub fn stop_injected_conversions(&mut self) {
        T::regs().stop_injected();
    }

    pub fn start_injected_conversions(&mut self) {
        T::regs().start_injected();
    }

    pub fn read_injected_samples(&mut self, data: &mut [u16; N]) {
        T::regs().read_injected(&mut data[..]);
    }
}

impl<'a, T: Instance<Regs: InjectedAdcRegs>, const N: usize> Drop for InjectedAdc<'a, T, N> {
    fn drop(&mut self) {
        T::regs().stop_injected();
        compiler_fence(Ordering::SeqCst);
    }
}
