#![macro_use]

use crate::pac::dma_channels;
use crate::pac::dma_requests;
use crate::pac::peripherals;
use crate::peripherals;

use core::future::Future;

use crate::dma::{ReadDma, WriteDma};

#[allow(unused)]
pub(crate) async unsafe fn transfer_p2m(
    ch: &mut impl Channel,
    ch_func: u8,
    src: *const u8,
    dst: &mut [u8],
) {
    unimplemented!()
}

#[allow(unused)]
pub(crate) async unsafe fn transfer_m2p(
    ch: &mut impl Channel,
    ch_func: u8,
    src: &[u8],
    dst: *mut u8,
) {
    unimplemented!()
}

pub(crate) mod sealed {
    use super::*;

    pub trait DmaMux {}

    pub trait Channel {
        fn dmamux_ch_num(&self) -> u8;
    }

    pub trait PeripheralChannel<PERI, OP>: Channel {
        fn request(&self) -> u8;
    }
}

pub trait DmaMux: sealed::DmaMux {}
pub trait Channel: sealed::Channel {}
pub trait PeripheralChannel<PERI, OP>: sealed::Channel {}

pub struct P2M;
pub struct M2P;

macro_rules! impl_dma_channel {
    ($channel_peri:ident, $dmamux_peri:ident, $channel_num:expr, $dma_num:expr) => {
        impl Channel for peripherals::$channel_peri {}
        impl sealed::Channel for peripherals::$channel_peri {
            fn dmamux_ch_num(&self) -> u8 {
                ($dma_num * 8) + $channel_num
            }
        }

        impl<T> WriteDma<T> for peripherals::$channel_peri
        where
            Self: sealed::PeripheralChannel<T, M2P>,
            T: 'static,
        {
            type WriteDmaFuture<'a> = impl Future<Output = ()>;

            fn transfer<'a>(&'a mut self, buf: &'a [u8], dst: *mut u8) -> Self::WriteDmaFuture<'a>
            where
                T: 'a,
            {
                let request = sealed::PeripheralChannel::<T, M2P>::request(self);
                unsafe { transfer_m2p(self, request, buf, dst) }
            }
        }

        impl<T> ReadDma<T> for peripherals::$channel_peri
        where
            Self: sealed::PeripheralChannel<T, P2M>,
            T: 'static,
        {
            type ReadDmaFuture<'a> = impl Future<Output = ()>;

            fn transfer<'a>(
                &'a mut self,
                src: *const u8,
                buf: &'a mut [u8],
            ) -> Self::ReadDmaFuture<'a>
            where
                T: 'a,
            {
                let request = sealed::PeripheralChannel::<T, P2M>::request(self);
                unsafe { transfer_p2m(self, request, src, buf) }
            }
        }
    };
}

peripherals! {
    (bdma, DMA1) => {
        //impl_dma!(DMA1, 0);
        dma_channels! {
            ($channel_peri:ident, DMA1, $channel_num:expr) => {
                impl_dma_channel!($channel_peri, DMAMUX1, $channel_num, 0);
            };
        }
    };
    (bdma, DMA2) => {
        //impl_dma!(DMA2, 1);
        dma_channels! {
            ($channel_peri:ident, DMA2, $channel_num:expr) => {
                impl_dma_channel!($channel_peri, DMAMUX1, $channel_num, 1);
            };
        }
    };
}

macro_rules! impl_usart_dma_requests {
    ($channel_peri:ident, $dma_peri:ident, $channel_num:expr) => {
        dma_requests! {
            (usart, $peri:ident, RX, $request:expr) => {
                impl usart::RxDma<peripherals::$peri> for peripherals::$channel_peri { }
                impl usart::sealed::RxDma<peripherals::$peri> for peripherals::$channel_peri { }

                impl sealed::PeripheralChannel<peripherals::$peri, P2M> for peripherals::$channel_peri {
                    fn request(&self) -> u8 {
                        $request
                    }
                }

                impl PeripheralChannel<peripherals::$peri, P2M> for peripherals::$channel_peri { }

            };

            (usart, $peri:ident, TX, $request:expr) => {
                impl usart::TxDma<peripherals::$peri> for peripherals::$channel_peri { }
                impl usart::sealed::TxDma<peripherals::$peri> for peripherals::$channel_peri { }

                impl sealed::PeripheralChannel<peripherals::$peri, M2P> for peripherals::$channel_peri {
                    fn request(&self) -> u8 {
                        $request
                    }
                }

                impl PeripheralChannel<peripherals::$peri, M2P> for peripherals::$channel_peri { }

            };

            (uart, $peri:ident, TX, $request:expr) => {
                impl usart::RxDma<peripherals::$peri> for peripherals::$channel_peri { }
                impl usart::sealed::RxDma<peripherals::$peri> for peripherals::$channel_peri { }

                impl sealed::PeripheralChannel<peripherals::$peri, P2M> for peripherals::$channel_peri {
                    fn request(&self) -> u8 {
                        $request
                    }
                }

                impl PeripheralChannel<peripherals::$peri, P2M> for peripherals::$channel_peri { }
            };

            (uart, $peri:ident, RX, $request:expr) => {
                impl usart::TxDma<peripherals::$peri> for peripherals::$channel_peri { }
                impl usart::sealed::TxDma<peripherals::$peri> for peripherals::$channel_peri { }

                impl sealed::PeripheralChannel<peripherals::$peri, M2P> for peripherals::$channel_peri {
                    fn request(&self) -> u8 {
                        $request
                    }
                }

                impl PeripheralChannel<peripherals::$peri, M2P> for peripherals::$channel_peri { }
            };
        }

    };
}

#[cfg(usart)]
use crate::usart;

dma_channels! {
    ($channel_peri:ident, $dma_peri:ident, $channel_num:expr) => {
        impl_usart_dma_requests!($channel_peri, $dma_peri, $channel_num);
    };
}
