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
        pub trait Instance: SealedInstance + crate::PeripheralType  + crate::rcc::RccPeripheral {
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
    ($signal:ident, $instance:path $(, $mode:path)? $(, @$afio:ident)?) => {
        #[doc = concat!(stringify!($signal), " pin trait")]
        pub trait $signal<T: $instance $(, M: $mode)? $(, $afio)?>: crate::gpio::Pin {
            #[doc = concat!("Get the AF number needed to use this pin as ", stringify!($signal))]
            fn af_num(&self) -> u8;

            #[cfg(afio)]
            #[doc = concat!("Configures AFIO_MAPR to use this pin as ", stringify!($signal))]
            fn afio_remap(&self);
        }
    };
}

macro_rules! pin_trait_impl {
    (crate::$mod:ident::$trait:ident$(<$mode:ident>)?, $instance:ident, $pin:ident, $af:expr $(, $afio:path)?) => {
        impl crate::$mod::$trait<crate::peripherals::$instance $(, crate::$mod::$mode)? $(, $afio)?> for crate::peripherals::$pin {
            fn af_num(&self) -> u8 {
                $af
            }

            #[cfg(afio)]
            fn afio_remap(&self) {
                // nothing
            }
        }
    };
}

#[cfg(afio)]
macro_rules! pin_trait_afio_impl {
    (crate::$mod:ident::$trait:ident, $instance:ident, $pin:ident, {$setter:ident, $type:ident, [$($val:expr),+]}) => {
        $(
            impl crate::$mod::$trait<crate::peripherals::$instance, crate::gpio::$type<$val>> for crate::peripherals::$pin {
                fn af_num(&self) -> u8 {
                    0
                }

                fn afio_remap(&self) {
                    crate::pac::AFIO.mapr().modify(|w| {
                        w.set_swj_cfg(crate::pac::afio::vals::SwjCfg::NO_OP);
                        w.$setter($val);
                    });
                }
            }
        )+
    };
}

#[allow(unused_macros)]
macro_rules! sel_trait_impl {
    (crate::$mod:ident::$trait:ident$(<$mode:ident>)?, $instance:ident, $pin:ident, $sel:expr) => {
        impl crate::$mod::$trait<crate::peripherals::$instance $(, crate::$mod::$mode)?> for crate::peripherals::$pin {
            fn sel(&self) -> u8 {
                $sel
            }
        }
    };
}

// ====================

macro_rules! timer_afio_impl {
    ($instance:ident $(, $set_afio:expr)? $(,{$mapr_value:literal, [$($pin:literal),*]})*) => {
        impl crate::timer::Afio for crate::peripherals::$instance {
            fn afio_mappings() -> &'static [crate::timer::AfioMapping] {
                &[
                    $(
                        crate::timer::AfioMapping {
                            value: $mapr_value,
                            pins: &[$($pin),*],
                        }
                    ),*
                ]
            }

            #[allow(unused)]
            fn set_afio(value: u8) {
                $($set_afio(value);)?
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
            #[doc = "Remap the DMA channel"]
            fn remap(&self);
        }
    };
}

#[allow(unused)]
macro_rules! dma_trait_impl {
    (crate::$mod:ident::$trait:ident$(<$mode:ident>)?, $instance:ident, $channel:ident, $request:expr, $remap:expr) => {
        impl crate::$mod::$trait<crate::peripherals::$instance $(, crate::$mod::$mode)?> for crate::peripherals::$channel {
            fn request(&self) -> crate::dma::Request {
                $request
            }

            fn remap(&self) {
                critical_section::with(|_| {
                    #[allow(unused_unsafe)]
                    unsafe {
                        $remap;
                    }
                });
            }
        }
    };
}

#[allow(unused)]
macro_rules! new_dma_nonopt {
    ($name:ident) => {{
        let dma = $name;
        let request = dma.request();
        crate::dma::ChannelAndRequest {
            channel: dma.into(),
            request,
        }
    }};
}

macro_rules! new_dma {
    ($name:ident) => {{
        let dma = $name;
        dma.remap();
        let request = dma.request();
        Some(crate::dma::ChannelAndRequest {
            channel: dma.into(),
            request,
        })
    }};
}

macro_rules! new_pin {
    ($name:ident, $af_type:expr) => {{
        let pin = $name;
        #[cfg(afio)]
        pin.afio_remap();
        pin.set_as_af(pin.af_num(), $af_type);
        Some(pin.into())
    }};
}
