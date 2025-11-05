use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};

#[allow(unused_imports)]
use embassy_hal_internal::Peri;

use crate::adc::Adc;
#[allow(unused_imports)]
use crate::adc::Instance;

const NR_INJECTED_RANKS: usize = 4;

pub struct InjectedAdc<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> InjectedAdc<T> {
    pub(crate) fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn read_injected_samples(&mut self) -> [u16; NR_INJECTED_RANKS] {
        Adc::<T>::read_injected_samples()
    }
}


impl<T: Instance> Drop for InjectedAdc<T> {
    fn drop(&mut self) {
        Adc::<T>::teardown_adc();

        compiler_fence(Ordering::SeqCst);
    }
}
