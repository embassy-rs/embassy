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
        pub trait $signal<T: $instance $(, M: $mode)? $(, #[cfg(afio)] $afio)?>: crate::gpio::Pin {
            #[cfg(not(afio))]
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
        #[cfg(afio)]
        impl crate::$mod::$trait<crate::peripherals::$instance $(, crate::$mod::$mode)? $(, $afio)?> for crate::peripherals::$pin {
            fn afio_remap(&self) {
                // nothing
            }
        }

        #[cfg(not(afio))]
        impl crate::$mod::$trait<crate::peripherals::$instance $(, crate::$mod::$mode)?> for crate::peripherals::$pin {
            fn af_num(&self) -> u8 {
                $af
            }
        }
    };
}

#[cfg(afio)]
macro_rules! pin_trait_afio_impl {
    (@set mapr, $setter:ident, $val:expr) => {
        crate::pac::AFIO.mapr().modify(|w| {
            w.set_swj_cfg(crate::pac::afio::vals::SwjCfg::NO_OP);
            w.$setter($val);
        });
    };
    (@set mapr2, $setter:ident, $val:expr) => {
        crate::pac::AFIO.mapr2().modify(|w| {
            w.$setter($val);
        });
    };
    (crate::$mod:ident::$trait:ident<$mode:ident>, $instance:ident, $pin:ident, {$reg:ident, $setter:ident, $type:ident, [$($val:expr),+]}) => {
        $(
            impl crate::$mod::$trait<crate::peripherals::$instance, crate::$mod::$mode, crate::gpio::$type<$val>> for crate::peripherals::$pin {
                fn afio_remap(&self) {
                    pin_trait_afio_impl!(@set $reg, $setter, $val);
                }
            }
        )+
    };
    (crate::$mod:ident::$trait:ident, $instance:ident, $pin:ident, {$reg:ident, $setter:ident, $type:ident, [$($val:expr),+]}) => {
        $(
            impl crate::$mod::$trait<crate::peripherals::$instance, crate::gpio::$type<$val>> for crate::peripherals::$pin {
                fn afio_remap(&self) {
                    pin_trait_afio_impl!(@set $reg, $setter, $val);
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

macro_rules! dma_trait {
    ($signal:ident, $instance:path$(, $mode:path)?) => {
        #[doc = concat!(stringify!($signal), " DMA request trait")]
        pub trait $signal<T: $instance $(, M: $mode)?>: crate::dma::Channel + crate::dma::TypedChannel {
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
    ($name:ident, $irqs:expr) => {{
        let dma = $name;
        let request = dma.request();
        crate::dma::ChannelAndRequest {
            channel: crate::dma::dma_into(dma, &$irqs),
            request,
        }
    }};
}

macro_rules! new_dma {
    ($name:ident, $irqs:expr) => {{
        let dma = $name;
        dma.remap();
        let request = dma.request();
        Some(crate::dma::ChannelAndRequest {
            channel: crate::dma::dma_into(dma, &$irqs),
            request,
        })
    }};
}

macro_rules! new_pin {
    ($name:ident, $af_type:expr) => {{
        let pin = $name;
        #[cfg(afio)]
        pin.afio_remap();
        pin.set_as_af(
            #[cfg(not(afio))]
            pin.af_num(),
            $af_type,
        );
        Some(pin.into())
    }};
}

/// Macro to configure a pin for alternate function use.
/// For AFIO chips (STM32F1), it calls afio_remap().
/// For non-AFIO chips, it calls set_as_af() with the pin's af_num().
macro_rules! set_as_af {
    ($pin:expr, $af_type:expr) => {
        #[cfg(afio)]
        {
            $pin.set_as_af($af_type);
            $pin.afio_remap();
        }
        #[cfg(not(afio))]
        {
            $pin.set_as_af($pin.af_num(), $af_type);
        }
    };
}

#[cfg(afio)]
macro_rules! if_afio {
    ($($t:tt)*) => {
        $($t)*
    }
}
#[cfg(not(afio))]
macro_rules! if_afio {
    (($a:ty, A)) => {
        ($a,)
    };
    (($a:ty, $b:ty, A)) => {
        ($a,$b)
    };
    (($a:ty, $b:ty, $c:ty, A)) => {
        ($a,$b, $c)
    };
    ($type:ident<$lt:lifetime, $a:ty, $b:ty, A>) => {
        $type<$lt, $a, $b>
    };
    ($type:ident<$lt:lifetime, $a:ty, $b:ty, $c:ty, A>) => {
        $type<$lt, $a, $b, $c>
    };
    ($type:ident<$a:ty, A>) => {
        $type<$a>
    };
    ($type:ident<$a:ty, $b:ty, A>) => {
        $type<$a, $b>
    };
    ($type:ident<$a:ty, $b:ty, $c:ty, A>) => {
        $type<$a, $b, $c>
    };
    (impl $trait:ident<$a:ty, A>) => {
        impl $trait<$a>
    };
    (impl $trait:ident<$a:ty, $b:ty, A>) => {
        impl $trait<$a, $b>
    };
    (impl $trait:ident<$a:ty, $b:ty, $c:ty, A>) => {
        impl $trait<$a, $b, $c>
    };
}
