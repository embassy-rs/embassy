#![macro_use]

macro_rules! peri_trait {
    (
        $(irqs: [$($irq:ident),*],)?
    ) => {
        #[allow(private_interfaces)]
        pub(crate) trait SealedInstance {
            #[allow(unused)]
            fn info() -> &'static Info;
            #[allow(unused)]
            fn state() -> &'static State;
        }

        /// Peripheral instance trait.
        #[allow(private_bounds)]
        pub trait Instance: crate::Peripheral<P = Self> + SealedInstance + crate::rcc::RccPeripheral {
            $($(
                /// Interrupt for this peripheral.
                type $irq: crate::interrupt::typelevel::Interrupt;
            )*)?
        }
    };
}

macro_rules! peri_trait_impl {
    ($instance:ident, $info:expr) => {
        #[allow(private_interfaces)]
        impl SealedInstance for crate::peripherals::$instance {
            fn info() -> &'static Info {
                static INFO: Info = $info;
                &INFO
            }
            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        }
        impl Instance for crate::peripherals::$instance {}
    };
}

macro_rules! pin_trait {
    ($signal:ident, $instance:path $(, $mode:path)?) => {
        #[doc = concat!(stringify!($signal), " pin trait")]
        pub trait $signal<T: $instance $(, M: $mode)?>: crate::gpio::Pin {
            #[doc = concat!("Get the AF number needed to use this pin as ", stringify!($signal))]
            fn af_num(&self) -> u8;
        }
    };
}

macro_rules! pin_trait_impl {
    (crate::$mod:ident::$trait:ident$(<$mode:ident>)?, $instance:ident, $pin:ident, $af:expr) => {
        impl crate::$mod::$trait<crate::peripherals::$instance $(, crate::$mod::$mode)?> for crate::peripherals::$pin {
            fn af_num(&self) -> u8 {
                $af
            }
        }
    };
}

// ====================

macro_rules! dma_trait {
    ($signal:ident, $instance:path$(, $mode:path)?) => {
        #[doc = concat!(stringify!($signal), " DMA request trait")]
        pub trait $signal<T: $instance $(, M: $mode)?>: crate::dma::Channel {
            #[doc = concat!("Get the DMA request number needed to use this channel as", stringify!($signal))]
            /// Note: in some chips, ST calls this the "channel", and calls channels "streams".
            /// `embassy-stm32` always uses the "channel" and "request number" names.
            fn request(&self) -> crate::dma::Request;
        }
    };
}

#[allow(unused)]
macro_rules! dma_trait_impl {
    (crate::$mod:ident::$trait:ident$(<$mode:ident>)?, $instance:ident, $channel:ident, $request:expr) => {
        impl crate::$mod::$trait<crate::peripherals::$instance $(, crate::$mod::$mode)?> for crate::peripherals::$channel {
            fn request(&self) -> crate::dma::Request {
                $request
            }
        }
    };
}

macro_rules! new_dma {
    ($name:ident) => {{
        let dma = $name.into_ref();
        let request = dma.request();
        Some(crate::dma::ChannelAndRequest {
            channel: dma.map_into(),
            request,
        })
    }};
}

macro_rules! new_pin {
    ($name:ident, $aftype:expr) => {{
        new_pin!($name, $aftype, crate::gpio::Speed::Medium, crate::gpio::Pull::None)
    }};
    ($name:ident, $aftype:expr, $speed:expr) => {
        new_pin!($name, $aftype, $speed, crate::gpio::Pull::None)
    };
    ($name:ident, $aftype:expr, $speed:expr, $pull:expr) => {{
        let pin = $name.into_ref();
        pin.set_as_af_pull(pin.af_num(), $aftype, $pull);
        // Do not call set_speed on AFType::Input, as MODE and CNF bits are not independent
        // for gpio_v1
        match $aftype {
            crate::gpio::AFType::Input => {}
            _ => {
                pin.set_speed($speed);
            }
        };
        Some(pin.map_into())
    }};
}
