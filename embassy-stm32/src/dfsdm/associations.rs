use super::types::*;
// use super::{Tcv0, Tcv1, Tcv2, Tcv3, Tcv4, Tcv5, Tcv6, Tcv7};

// Channel- and instance-specific pin traits
//
//

pin_trait!(CkoutPin, Instance, @A);
pin_trait!(Datin0Pin, Instance, @A);
pin_trait!(Ckin0Pin, Instance, @A);
pin_trait!(Ckin1Pin, Instance, @A);
pin_trait!(Datin1Pin, Instance, @A);
pin_trait!(Datin2Pin, Instance, @A);
pin_trait!(Ckin2Pin, Instance, @A);
pin_trait!(Datin3Pin, Instance, @A);
pin_trait!(Ckin3Pin, Instance, @A);
pin_trait!(Datin4Pin, Instance, @A);
pin_trait!(Ckin4Pin, Instance, @A);
pin_trait!(Datin5Pin, Instance, @A);
pin_trait!(Ckin5Pin, Instance, @A);
pin_trait!(Datin6Pin, Instance, @A);
pin_trait!(Ckin6Pin, Instance, @A);
pin_trait!(Datin7Pin, Instance, @A);
pin_trait!(Ckin7Pin, Instance, @A);

// Associate pin traits with channels
//
//

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

// Associate Interrupts
//
//

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
        delay: $delay:ty,
        version: $version:ty,
        $(,)?// split: $split_ty:ident $(,)?        
    ) => {
        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> Registers {
                // SAFETY: siehe Trait-Doku — $inst zeigt auf einen realen DFSDM-Block,
                // dessen Basislayout mit Dfsdm8ch8fltDlyVer übereinstimmt. Aufrufer
                // dürfen nur die laut Instance::{Delay,Version,Transceivers,Filters}
                // tatsächlich vorhandenen Register lesen/schreiben.
                unsafe { Registers::from_ptr(crate::pac::$inst.as_ptr()) }
            }
        }

        impl Instance for crate::peripherals::$inst {
            type Repr = $repr;
            type Transceivers = $tcv;
            type Filters = $flt;
            type Delay = $delay;
            type Version = $version;

            // type Split<C: ClockOutputMode> = $split_ty<Self, C>;

            // fn split<C: ClockOutputMode>(dfsdm: Dfsdm<'_, Self, C>) -> Self::Split<C> {
            //     $split_ty::new(&dfsdm)
            // }
        }
    };
}

foreach_interrupt! {
    ($inst:ident, dfsdm, DFSDM_8CH_8FLT_DLY_TRG5_ADC, FLT0, $irq:ident) => {
        impl_dfsdm_instance!($inst,
            repr: crate::pac::dfsdm::Dfsdm8ch8fltDlyTrg5Adc,
            transceivers: capability::Tcv8,
            filters: capability::Flt8,
            delay: capability::WithDelay,
            version: capability::NoVersion,
            // split: TransceiverChannelBuilders8,
        );
    };
    // ($inst:ident, dfsdm, DFSDM_8CH_8FLT, FLT0, $irq:ident) => {
    //     impl_dfsdm_instance!($inst,
    //         repr: crate::pac::dfsdm::Dfsdm8ch8flt,
    //         transceivers: capability::Tcv8,
    //         filters: capability::Flt8,
    //         delay: capability::NoDelay,
    //         version: capability::NoVersion,
    //         // split: TransceiverChannelBuilders8,
    //     );
    // };
    ($inst:ident, dfsdm, DFSDM_2CH_1FLT_TRG3_ADC, FLT0, $irq:ident) => {
        impl_dfsdm_instance!($inst,
            repr: crate::pac::dfsdm::Dfsdm2ch1fltTrg3Adc,
            transceivers: capability::Tcv2,
            filters: capability::Flt1,
            delay: capability::NoDelay,
            version: capability::NoVersion,
            // split: TransceiverChannelBuilders2,
        );
    };
}

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
