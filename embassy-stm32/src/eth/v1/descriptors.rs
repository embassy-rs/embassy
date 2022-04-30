use crate::eth::_version::rx_desc::RDesRing;
use crate::eth::_version::tx_desc::TDesRing;

pub struct DescriptorRing<const T: usize, const R: usize> {
    pub(crate) tx: TDesRing<T>,
    pub(crate) rx: RDesRing<R>,
}

impl<const T: usize, const R: usize> DescriptorRing<T, R> {
    pub const fn new() -> Self {
        Self {
            tx: TDesRing::new(),
            rx: RDesRing::new(),
        }
    }

    pub fn init(&mut self) {
        self.tx.init();
        self.rx.init();
    }
}
