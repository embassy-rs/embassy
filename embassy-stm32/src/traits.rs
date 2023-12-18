#![macro_use]

macro_rules! pin_trait {
    ($signal:ident, $instance:path) => {
        #[doc = concat!(stringify!($signal), " pin trait")]
        pub trait $signal<T: $instance>: crate::gpio::Pin {
            #[doc = concat!("Get the AF number needed to use this pin as", stringify!($signal))]
            fn af_num(&self) -> u8;
        }
    };
}

macro_rules! pin_trait_impl {
    (crate::$mod:ident::$trait:ident, $instance:ident, $pin:ident, $af:expr) => {
        impl crate::$mod::$trait<crate::peripherals::$instance> for crate::peripherals::$pin {
            fn af_num(&self) -> u8 {
                $af
            }
        }
    };
}

// ====================

macro_rules! dma_trait {
    ($signal:ident, $instance:path) => {
        #[doc = concat!(stringify!($signal), " DMA request trait")]
        pub trait $signal<T: $instance>: crate::dma::Channel {
            #[doc = concat!("Get the DMA request number needed to use this channel as", stringify!($signal))]
            /// Note: in some chips, ST calls this the "channel", and calls channels "streams".
            /// `embassy-stm32` always uses the "channel" and "request number" names.
            fn request(&self) -> crate::dma::Request;
        }
    };
}

#[allow(unused)]
macro_rules! dma_trait_impl {
    // DMAMUX
    (crate::$mod:ident::$trait:ident, $instance:ident, {dmamux: $dmamux:ident}, $request:expr) => {
        impl<T> crate::$mod::$trait<crate::peripherals::$instance> for T
        where
            T: crate::dma::Channel + crate::dma::MuxChannel<Mux = crate::dma::$dmamux>,
        {
            fn request(&self) -> crate::dma::Request {
                $request
            }
        }
    };

    // DMAMUX
    (crate::$mod:ident::$trait:ident, $instance:ident, {dma: $dma:ident}, $request:expr) => {
        impl<T> crate::$mod::$trait<crate::peripherals::$instance> for T
        where
            T: crate::dma::Channel,
        {
            fn request(&self) -> crate::dma::Request {
                $request
            }
        }
    };

    // DMA/GPDMA, without DMAMUX
    (crate::$mod:ident::$trait:ident, $instance:ident, {channel: $channel:ident}, $request:expr) => {
        impl crate::$mod::$trait<crate::peripherals::$instance> for crate::peripherals::$channel {
            fn request(&self) -> crate::dma::Request {
                $request
            }
        }
    };
}
