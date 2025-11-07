use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};

#[allow(unused_imports)]
use embassy_hal_internal::Peri;

use super::{AnyAdcChannel, SampleTime};
use crate::adc::Adc;
#[allow(unused_imports)]
use crate::adc::Instance;

/// Injected ADC sequence with owned channels.
pub struct InjectedAdc<T: Instance, const N: usize> {
    _channels: [(AnyAdcChannel<T>, SampleTime); N],
    _phantom: PhantomData<T>,
}

impl<T: Instance, const N: usize> InjectedAdc<T, N> {
    pub(crate) fn new(channels: [(AnyAdcChannel<T>, SampleTime); N]) -> Self {
        Self {
            _channels: channels,
            _phantom: PhantomData,
        }
    }

    pub fn stop_injected_conversions(&mut self) {
        Adc::<T>::stop_injected_conversions()
    }

    pub fn start_injected_conversions(&mut self) {
        Adc::<T>::start_injected_conversions()
    }

    pub fn read_injected_samples(&mut self) -> [u16; N] {
        InjectedAdc::<T, N>::read_injected_data()
    }
}

impl<T: Instance, const N: usize> Drop for InjectedAdc<T, N> {
    fn drop(&mut self) {
        Adc::<T>::teardown_adc();
        compiler_fence(Ordering::SeqCst);
    }
}
