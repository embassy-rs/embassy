use super::types::*;
// use super::{Tcv0, Tcv1, Tcv2, Tcv3, Tcv4, Tcv5, Tcv6, Tcv7};

// =============================================================================
// Associate pin traits with channels
// =============================================================================

macro_rules! impl_ckin_bridge {
    ($marker:ty, $existing:ident) => {
        #[cfg(afio)]
        impl<T: Instance, A, P> CkinPin<T, $marker, A> for P
        where
            P: $existing<T, A>,
        {
            fn af_num(&self) -> u8 {
                $existing::af_num(self)
            }
        }

        #[cfg(not(afio))]
        impl<T: Instance, P> CkinPin<T, $marker> for P
        where
            P: $existing<T>,
        {
            fn af_num(&self) -> u8 {
                $existing::af_num(self)
            }
        }
    };
}

impl_ckin_bridge!(Tcv0, Ckin0Pin);
impl_ckin_bridge!(Tcv1, Ckin1Pin);
impl_ckin_bridge!(Tcv2, Ckin2Pin);
impl_ckin_bridge!(Tcv3, Ckin3Pin);
impl_ckin_bridge!(Tcv4, Ckin4Pin);
impl_ckin_bridge!(Tcv5, Ckin5Pin);
impl_ckin_bridge!(Tcv6, Ckin6Pin);
impl_ckin_bridge!(Tcv7, Ckin7Pin);

macro_rules! impl_datin_bridge {
    ($marker:ty, $existing:ident) => {
        #[cfg(afio)]
        impl<T: Instance, A, P> DatinPin<T, $marker, A> for P
        where
            P: $existing<T, A>,
        {
            fn af_num(&self) -> u8 {
                $existing::af_num(self)
            }
        }

        #[cfg(not(afio))]
        impl<T: Instance, P> DatinPin<T, $marker> for P
        where
            P: $existing<T>,
        {
            fn af_num(&self) -> u8 {
                $existing::af_num(self)
            }
        }
    };
}

impl_datin_bridge!(Tcv0, Datin0Pin);
impl_datin_bridge!(Tcv1, Datin1Pin);
impl_datin_bridge!(Tcv2, Datin2Pin);
impl_datin_bridge!(Tcv3, Datin3Pin);
impl_datin_bridge!(Tcv4, Datin4Pin);
impl_datin_bridge!(Tcv5, Datin5Pin);
impl_datin_bridge!(Tcv6, Datin6Pin);
impl_datin_bridge!(Tcv7, Datin7Pin);

// =============================================================================
// Associate capabilities
// =============================================================================

/// Implements `SealedInstance` + `Instance` for one DFSDM instance.
///
/// One template, so channel/filter/delay/version capability for a given
/// shape is only ever written once, next to the `Repr` it belongs to.
macro_rules! impl_dfsdm_instance {
    (
        $inst:ident,
        repr: $repr:ty,
        transceivers: $tcv:ty,
        filters: $flt:ty,
        $(delay: $delay:tt,)?
        $(hwid: $hwid:tt,)?
        $(adc_input: $adc:tt,)?
        $(,)?
    ) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> Registers {
                // SAFETY: siehe Trait-Doku — $inst zeigt auf einen realen DFSDM-Block,
                // dessen Basislayout mit Dfsdm8ch8fltDlyVer übereinstimmt. Aufrufer
                // dürfen nur die laut Instance::{Transceivers,Filters} und den
                // capability::Has*-Marker-Traits tatsächlich vorhandenen Register
                // lesen/schreiben.
                unsafe { Registers::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }

        impl Instance for crate::peripherals::$inst {
            type Repr = $repr;
            type Transceivers = $tcv;
            type Filters = $flt;
        }

        $(impl_dfsdm_instance!(@flag $inst, HasDelay, $delay);)?
        $(impl_dfsdm_instance!(@flag $inst, HasHwid, $hwid);)?
        $(impl_dfsdm_instance!(@flag $inst, AdcInput, $adc);)?
    };

    (@flag $inst:ident, $trait_name:ident, true) => {
        impl capability::$trait_name for crate::peripherals::$inst {}
    };
    (@flag $inst:ident, $trait_name:ident, false) => {};
}

// Mark specified variants with specified attributes
macro_rules! mark_dfsdm_instances {
    (
        $(
            $peripheral:ident => {
                repr: $repr:ident,
                transceivers: $transceivers:ident,
                filters: $filters:ident,
                delay: $delay:tt,
                hwid: $hwid:tt,
                adc_input: $adc:tt $(,)?
            }
        ),* $(,)?
    ) => {
        foreach_interrupt! {
            $(
                ($inst:ident, dfsdm, $peripheral, FLT0, $irq:ident) => {
                    impl_dfsdm_instance!(
                        $inst,
                        repr: crate::pac::dfsdm::$repr,
                        transceivers: capability::$transceivers,
                        filters: capability::$filters,
                        delay: $delay,
                        hwid: $hwid,
                        adc_input: $adc,
                    );
                };
            )*
        }
    };
}

//TODO maybe move to variant-list and skip the repr.
// Mark variants with attributes
mark_dfsdm_instances! {

    DFSDM_2CH_1FLT_DLY_TRG5_ADC => {
        repr: Dfsdm2ch1fltDlyTrg5Adc,
        transceivers: Tcv2,
        filters: Flt1,
        delay: true,
        hwid: false,
        adc_input: true,
    },

    DFSDM_2CH_1FLT_TRG3_ADC => {
        repr: Dfsdm2ch1fltTrg3Adc,
        transceivers: Tcv2,
        filters: Flt1,
        delay: false,
        hwid: false,
        adc_input: true,
    },

    DFSDM_4CH_2FLT_TRG3 => {
        repr: Dfsdm4ch2fltTrg3,
        transceivers: Tcv4,
        filters: Flt2,
        delay: false,
        hwid: false,
        adc_input: false,
    },

    DFSDM_4CH_2FLT_TRG3_ADC => {
        repr: Dfsdm4ch2fltTrg3Adc,
        transceivers: Tcv4,
        filters: Flt2,
        delay: false,
        hwid: false,
        adc_input: true,
    },

    DFSDM_4CH_2FLT_DLY_TRG5_ADC => {
        repr: Dfsdm4ch2fltDlyTrg5Adc,
        transceivers: Tcv4,
        filters: Flt2,
        delay: true,
        hwid: false,
        adc_input: true,
    },

    DFSDM_4CH_2FLT_DLY_TRG5_ADC_HWID => {
        repr: Dfsdm4ch2fltDlyTrg5AdcHwid,
        transceivers: Tcv4,
        filters: Flt2,
        delay: true,
        hwid: true,
        adc_input: true,
    },

    DFSDM_4CH_4FLT_DLY_TRG5_ADC => {
        repr: Dfsdm4ch4fltDlyTrg5Adc,
        transceivers: Tcv4,
        filters: Flt4,
        delay: true,
        hwid: false,
        adc_input: true,
    },

    DFSDM_8CH_4FLT_TRG3 => {
        repr: Dfsdm8ch4fltTrg3,
        transceivers: Tcv8,
        filters: Flt4,
        delay: false,
        hwid: false,
        adc_input: false,
    },

    DFSDM_8CH_4FLT_TRG5 => {
        repr: Dfsdm8ch4fltTrg5,
        transceivers: Tcv8,
        filters: Flt4,
        delay: false,
        hwid: false,
        adc_input: false,
    },

    DFSDM_8CH_4FLT_TRG3_ADC => {
        repr: Dfsdm8ch4fltTrg3Adc,
        transceivers: Tcv8,
        filters: Flt4,
        delay: false,
        hwid: false,
        adc_input: true,
    },

    DFSDM_8CH_4FLT_TRG5_ADC => {
        repr: Dfsdm8ch4fltTrg5Adc,
        transceivers: Tcv8,
        filters: Flt4,
        delay: false,
        hwid: false,
        adc_input: true,
    },

    DFSDM_8CH_4FLT_DLY_TRG5_ADC => {
        repr: Dfsdm8ch4fltDlyTrg5Adc,
        transceivers: Tcv8,
        filters: Flt4,
        delay: true,
        hwid: false,
        adc_input: true,
    },

    DFSDM_8CH_6FLT_DLY_TRG5_ADC_HWID => {
        repr: Dfsdm8ch6fltDlyTrg5AdcHwid,
        transceivers: Tcv8,
        filters: Flt6,
        delay: true,
        hwid: true,
        adc_input: true,
    },

    DFSDM_8CH_8FLT_DLY_TRG5_ADC => {
        repr: Dfsdm8ch8fltDlyTrg5Adc,
        transceivers: Tcv8,
        filters: Flt8,
        delay: true,
        hwid: false,
        adc_input: true,
    },
}

// =============================================================================
// Associate interrupts
// =============================================================================

// Implement single IRQ for a single variants
macro_rules! impl_dfsdm_filter_irq {
    ($inst:ident, $filter:ty, $irq:ident) => {
        impl FilterInterrupt<$filter> for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;

            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        }
    };
}

// Implement IRQs in list for a single variant
macro_rules! impl_dfsdm_filter_irqs_variant {
    ($variant:ident, $( $flt:ident => $filter:ty ),+ $(,)?) => {
        foreach_interrupt! {
            $(
                ($inst:ident, dfsdm, $variant, $flt, $irq:ident) => {
                    impl_dfsdm_filter_irq!($inst, $filter, $irq);
                };
            )+
        }
    };
}

// Implement IRQs in list for a list of variants
macro_rules! impl_dfsdm_filter_irqs {
    (
        [ $variant:ident $(, $rest:ident )* $(,)? ],
        $( $flt:ident => $filter:ty ),+ $(,)?
    ) => {
        impl_dfsdm_filter_irqs_variant!($variant, $( $flt => $filter ),+);
        impl_dfsdm_filter_irqs!([ $( $rest ),* ], $( $flt => $filter ),+);
    };
    ([], $( $flt:ident => $filter:ty ),+ $(,)?) => {};
}

// Implement all single-channel IRQs
impl_dfsdm_filter_irqs! {
    [DFSDM_2CH_1FLT_TRG3_ADC],
    FLT0 => Flt0,
}

// Implement all eight-channel IRQs
impl_dfsdm_filter_irqs! {
    [DFSDM_8CH_8FLT_DLY_TRG5_ADC],
    FLT0 => Flt0,
    FLT1 => Flt1,
    FLT2 => Flt2,
    FLT3 => Flt3,
    FLT4 => Flt4,
    FLT5 => Flt5,
    FLT6 => Flt6,
    FLT7 => Flt7,
}

impl_dfsdm_filter_irqs! {
    [DFSDM_8CH_4FLT_TRG5_ADC],
    FLT0 => Flt0,
    FLT1 => Flt1,
    FLT2 => Flt2,
    FLT3 => Flt3,
}
