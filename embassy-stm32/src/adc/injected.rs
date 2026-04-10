use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};

use super::AnyAdcChannel;
use crate::adc::{BasicAdcRegs, InjectedAdcRegs, Instance};

/// Injected ADC sequence with owned channels.
pub struct InjectedAdc<'d, R: InjectedAdcRegs> {
    regs: R,
    len: usize,
    _typ: PhantomData<&'d mut ()>,
}

impl<'d, R: InjectedAdcRegs> InjectedAdc<'d, R> {
    pub(crate) fn new<T: Instance<Regs = R>, const N: usize>(
        _channels: [(AnyAdcChannel<'d, T>, <T::Regs as BasicAdcRegs>::SampleTime); N],
    ) -> Self {
        Self {
            regs: T::regs(),
            len: N,
            _typ: PhantomData,
        }
    }

    pub fn stop_injected_conversions(&mut self) {
        self.regs.stop_injected();
    }

    pub fn start_injected_conversions(&mut self) {
        self.regs.start_injected();
    }

    pub fn read_injected_samples(&mut self, buf: &mut [u16]) {
        assert!(
            buf.len() == self.len,
            "Buffer must have as many entries as the sequence"
        );

        self.regs.read_injected(buf);
    }
}

impl<'d, R: InjectedAdcRegs> Drop for InjectedAdc<'d, R> {
    fn drop(&mut self) {
        self.regs.stop_injected();
        compiler_fence(Ordering::SeqCst);
    }
}
