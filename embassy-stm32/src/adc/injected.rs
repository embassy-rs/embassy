use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};

#[allow(unused_imports)]
use embassy_hal_internal::Peri;

use super::{AnyAdcChannel, SampleTime};
#[allow(unused_imports)]
use crate::adc::Instance;
use crate::adc::{Adc, AnyInstance};

/// Injected ADC sequence with owned channels.
pub struct InjectedAdc<'a, T: Instance, const N: usize> {
    _channels: [(AnyAdcChannel<'a, T>, SampleTime); N],
    _phantom: PhantomData<T>,
}

impl<'a, T: Instance, const N: usize> InjectedAdc<'a, T, N> {
    pub(crate) fn new(channels: [(AnyAdcChannel<'a, T>, SampleTime); N]) -> Self {
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

impl<'a, T: Instance + AnyInstance, const N: usize> Drop for InjectedAdc<'a, T, N> {
    fn drop(&mut self) {
        T::stop();
        compiler_fence(Ordering::SeqCst);
    }
}
