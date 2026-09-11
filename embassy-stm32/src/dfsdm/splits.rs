//! Shape-generated types - the two halves of `Dfsdm::configure_pins`:
//! channel-selection bundles (handed to the closure) and channel/filter
//! split bundles (returned by it).
//!
//! Both are *generated* from two tables at the bottom of this file: the
//! [`dfsdm_split_shapes!`] split table (one entry per (channel-arity ×
//! filter-count) shape; per-arity wiring - pin-set pairing including the last
//! channel's wrap-around to `S0` - lives in the [`dfsdm_split_shape!`]
//! dispatch arms) and the [`dfsdm_selectors!`] selector table (one block per
//! transceiver count). The single master implementations are
//! [`dfsdm_split_shape_body!`] and [`dfsdm_selector_shape!`].

use super::*;

use core::marker::PhantomData;

// =============================================================================
// SplitBuild - filter-count dispatch, one trait per channel arity
// =============================================================================

/// Builds the actual split struct from already-extracted pin pairs.
///
/// One trait per channel arity (the pin-set parameter list is arity-shaped),
/// implemented for each filter-count capability by the shape table below.
pub trait Tcv2SplitBuild<T: Instance, C: ClockOutputMode, S0: PinSet, S1: PinSet> {
    /// The split struct for this (channel, filter) shape.
    type Out;

    /// Registers the pins with `common` and constructs the split.
    fn build<'d>(
        common: &mut DfsdmCommon<'d, T, Enabled>,
        ch0: (S0::Datin<'d>, S0::Ckin<'d>),
        ch1: (S1::Datin<'d>, S1::Ckin<'d>),
    ) -> Self::Out;
}

/// 4-channel twin of [`Tcv2SplitBuild`].
pub trait Tcv4SplitBuild<T: Instance, C: ClockOutputMode, S0: PinSet, S1: PinSet, S2: PinSet, S3: PinSet> {
    /// The split struct for this (channel, filter) shape.
    type Out;

    /// Registers the pins with `common` and constructs the split.
    fn build<'d>(
        common: &mut DfsdmCommon<'d, T, Enabled>,
        ch0: (S0::Datin<'d>, S0::Ckin<'d>),
        ch1: (S1::Datin<'d>, S1::Ckin<'d>),
        ch2: (S2::Datin<'d>, S2::Ckin<'d>),
        ch3: (S3::Datin<'d>, S3::Ckin<'d>),
    ) -> Self::Out;
}

/// 8-channel twin of [`Tcv2SplitBuild`].
pub trait Tcv8SplitBuild<
    T: Instance,
    C: ClockOutputMode,
    S0: PinSet,
    S1: PinSet,
    S2: PinSet,
    S3: PinSet,
    S4: PinSet,
    S5: PinSet,
    S6: PinSet,
    S7: PinSet,
>
{
    /// The split struct for this (channel, filter) shape.
    type Out;

    /// Registers the pins with `common` and constructs the split.
    fn build<'d>(
        common: &mut DfsdmCommon<'d, T, Enabled>,
        ch0: (S0::Datin<'d>, S0::Ckin<'d>),
        ch1: (S1::Datin<'d>, S1::Ckin<'d>),
        ch2: (S2::Datin<'d>, S2::Ckin<'d>),
        ch3: (S3::Datin<'d>, S3::Ckin<'d>),
        ch4: (S4::Datin<'d>, S4::Ckin<'d>),
        ch5: (S5::Datin<'d>, S5::Ckin<'d>),
        ch6: (S6::Datin<'d>, S6::Ckin<'d>),
        ch7: (S7::Datin<'d>, S7::Ckin<'d>),
    ) -> Self::Out;
}

// =============================================================================
// Channel-config tuples - the split entry point
// =============================================================================

/// Implemented for the tuple a `configure_pins` closure returns.
/// The arity *is* the channel-count check: `(C0, C1)` only impls for
/// `Tcv2` instances, the 8-tuple only for `Tcv8`.
#[diagnostic::on_unimplemented(
    message = "the closure must return one pin token per channel of `{T}`",
    label = "tuple length doesn't match `{T}`'s transceiver count",
    note = "check `{T}`'s channel count and return a tuple of that length, one token per `creator.chN`"
)]
pub trait ChannelCfgTuple<'d, T: Instance, C: ClockOutputMode> {
    /// The fully-wired split (neighbor pin-sets already correct).
    type Split;

    /// Split helper-function
    fn split_parts(self, common: &mut DfsdmCommon<'d, T, Enabled>) -> Self::Split;
}

// =============================================================================
// Shape-table machinery
// =============================================================================

/// Master implementation for the split bundles - written once. Everything
/// is a repetition over the per-arity channel wiring and the per-shape
/// filter fields from the table. Not invoked directly.
macro_rules! dfsdm_split_shape_body {
    (
        $name:ident, $tcv:ident, $bt:ident,
        S: [$($s:ident),+ $(,)?],
        ch: [$(($idx:literal, $ch:ident, $tcvm:ident, $sa:ident, $sb:ident)),+ $(,)?],
        $flt:ident, $ready:ident,
        flt: [$($field:ident : $marker:ident),+ $(,)?]
    ) => {
        /// Per-shape split bundle: one [`TransceiverBuilder`] per channel,
        /// one [`FilterBuilder`] per filter, plus the shared
        /// [`DetectorsBuilder`].
        pub struct $name<T, C, $($s),*>
        where
            T: Instance + $ready,
            C: ClockOutputMode,
            $($s: PinSet,)*
        {
            /// Common detector (short-circuit / clock-absence) side.
            pub detectors: DetectorsBuilder<T>,
            // each channel pairs its own pin-set with its neighbor's;
            // the last channel wraps its neighbor back to `S0`
            $(
                /// Per-channel transceiver builder.
                pub $ch: TransceiverBuilder<T, $tcvm, C, $sa, $sb>,
            )+
            $(
                /// Per-filter builder.
                pub $field: FilterBuilder<T, $marker>,
            )+
        }

        impl<T, C, $($s),*> $bt<T, C, $($s),*> for capability::$flt
        where
            T: Instance<Transceivers = capability::$tcv, Filters = capability::$flt> + $ready,
            C: ClockOutputMode,
            $($s: PinSet,)*
        {
            type Out = $name<T, C, $($s),*>;

            fn build<'d>(
                common: &mut DfsdmCommon<'d, T, Enabled>,
                $($ch: ($sa::Datin<'d>, $sa::Ckin<'d>)),+
            ) -> Self::Out {
                $(
                    common.insert_pin($idx, PinKind::Datin, $sa::extract_datin($ch.0));
                    common.insert_pin($idx, PinKind::Ckin, $sa::extract_ckin($ch.1));
                )+
                $name {
                    detectors: DetectorsBuilder::new(),
                    $($ch: TransceiverBuilder::new(),)+
                    $($field: FilterBuilder::new(),)+
                }
            }
        }
    };
}

/// Arity body for the selectors - written once, invoked once per arity.
/// Emits the `ChannelSelectors*` struct handed to the `configure_pins`
/// closure plus its `Shape` impl. Not invoked directly.
macro_rules! dfsdm_selector_shape {
    ($tcv:ident, $sel:ident, ch: [$(($idx:literal, $ch:ident, $tcvm:ident)),+ $(,)?]) => {
        /// Per-channel pin selectors handed to the `configure_pins` closure.
        pub struct $sel<T: Instance> {
            $(
                #[doc = concat!("Selector for channel ", stringify!($idx), ".")]
                pub $ch: Sel<T, $tcvm>,
            )+
        }

        impl<T: Instance> $sel<T> {
            pub(crate) fn new() -> Self {
                Self { $($ch: Sel { _m: PhantomData },)+ }
            }
        }

        impl Shape for capability::$tcv {
            type Selectors<T: Instance> = $sel<T>;

            fn selectors<T: Instance>() -> Self::Selectors<T> {
                $sel::new()
            }
        }
    };
}

/// The selector table - one arity block per transceiver count: the
/// `ChannelSelectors*` struct plus its `Shape` impl. The channel list here is
/// count + marker mapping only (`Sel` needs no pin-set pairing); the wiring
/// with S-neighbor pairs lives in the split table's dispatch arms.
macro_rules! dfsdm_selectors {
    (
        $(
            $tcv:ident, $sel:ident,
            ch: [$(($idx:literal, $ch:ident, $tcvm:ident)),+ $(,)?]
        ),+ $(,)?
    ) => {
        $(dfsdm_selector_shape!($tcv, $sel, ch: [$(($idx, $ch, $tcvm)),+]);)+
    };
}

// =============================================================================
// Split table machinery
// =============================================================================

/// Case-select: injects the per-arity channel wiring (S-list + neighbor
/// pairing, last channel wraps to `S0`) and forwards to the master body.
/// The arms hold the only genuinely per-arity data for the split products.
macro_rules! dfsdm_split_shape {
    ($name:ident, Tcv2, $($rest:tt)*) => {
        dfsdm_split_shape_body!(
            $name, Tcv2, Tcv2SplitBuild,
            S: [S0, S1],
            ch: [
                (0, ch0, Tcv0, S0, S1),
                (1, ch1, Tcv1, S1, S0), // wrap!
            ],
            $($rest)*
        );
    };
    ($name:ident, Tcv4, $($rest:tt)*) => {
        dfsdm_split_shape_body!(
            $name, Tcv4, Tcv4SplitBuild,
            S: [S0, S1, S2, S3],
            ch: [
                (0, ch0, Tcv0, S0, S1),
                (1, ch1, Tcv1, S1, S2),
                (2, ch2, Tcv2, S2, S3),
                (3, ch3, Tcv3, S3, S0), // wrap!
            ],
            $($rest)*
        );
    };
    ($name:ident, Tcv8, $($rest:tt)*) => {
        dfsdm_split_shape_body!(
            $name, Tcv8, Tcv8SplitBuild,
            S: [S0, S1, S2, S3, S4, S5, S6, S7],
            ch: [
                (0, ch0, Tcv0, S0, S1),
                (1, ch1, Tcv1, S1, S2),
                (2, ch2, Tcv2, S2, S3),
                (3, ch3, Tcv3, S3, S4),
                (4, ch4, Tcv4, S4, S5),
                (5, ch5, Tcv5, S5, S6),
                (6, ch6, Tcv6, S6, S7),
                (7, ch7, Tcv7, S7, S0), // wrap!
            ],
            $($rest)*
        );
    };
}

/// The split table - one entry per (channel-arity × filter-count) DFSDM shape
/// that exists in hardware. Each line expands to the split struct plus its
/// `Tcv*SplitBuild` impl.
macro_rules! dfsdm_split_shapes {
    (
        $(
            $name:ident : $tcv:ident, $flt:ident, $ready:ident,
            flt: [$($field:ident : $marker:ident),+ $(,)?]
        ),+ $(,)?
    ) => {
        $(dfsdm_split_shape!($name, $tcv, $flt, $ready, flt: [$($field : $marker),+]);)+
    };
}

dfsdm_split_shapes! {
    DfsdmSplit2Ch1Flt: Tcv2, Flt1, Flt1Ready, flt: [flt0: Flt0],
    DfsdmSplit4Ch2Flt: Tcv4, Flt2, Flt2Ready, flt: [flt0: Flt0, flt1: Flt1],
    DfsdmSplit4Ch4Flt: Tcv4, Flt4, Flt4Ready, flt: [flt0: Flt0, flt1: Flt1, flt2: Flt2, flt3: Flt3],
    DfsdmSplit8Ch4Flt: Tcv8, Flt4, Flt4Ready, flt: [flt0: Flt0, flt1: Flt1, flt2: Flt2, flt3: Flt3],
    DfsdmSplit8Ch6Flt: Tcv8, Flt6, Flt6Ready,
        flt: [flt0: Flt0, flt1: Flt1, flt2: Flt2, flt3: Flt3, flt4: Flt4, flt5: Flt5],
    DfsdmSplit8Ch8Flt: Tcv8, Flt8, Flt8Ready,
        flt: [flt0: Flt0, flt1: Flt1, flt2: Flt2, flt3: Flt3, flt4: Flt4, flt5: Flt5, flt6: Flt6, flt7: Flt7],
}

dfsdm_selectors! {
    Tcv2, ChannelSelectors2,
    ch: [
        (0, ch0, Tcv0),
        (1, ch1, Tcv1),
    ],
    Tcv4, ChannelSelectors4,
    ch: [
        (0, ch0, Tcv0),
        (1, ch1, Tcv1),
        (2, ch2, Tcv2),
        (3, ch3, Tcv3),
    ],
    Tcv8, ChannelSelectors8,
    ch: [
        (0, ch0, Tcv0),
        (1, ch1, Tcv1),
        (2, ch2, Tcv2),
        (3, ch3, Tcv3),
        (4, ch4, Tcv4),
        (5, ch5, Tcv5),
        (6, ch6, Tcv6),
        (7, ch7, Tcv7),
    ],
}

// =============================================================================
// ChannelCfgTuple impls - one per tuple arity, dispatching the filter
// dimension through `T::Filters: Tcv*SplitBuild`
// =============================================================================

impl<'d, T, C, C0, C1> ChannelCfgTuple<'d, T, C> for (C0, C1)
where
    T: Instance<Transceivers = capability::Tcv2>,
    C: ClockOutputMode,
    C0: ChannelCfg<'d, T, Tcv0>,
    C1: ChannelCfg<'d, T, Tcv1>,
    T::Filters:
        Tcv2SplitBuild<T, C, <C0 as ChannelCfg<'d, T, Tcv0>>::Presence, <C1 as ChannelCfg<'d, T, Tcv1>>::Presence>,
{
    type Split = <T::Filters as Tcv2SplitBuild<
        T,
        C,
        <C0 as ChannelCfg<'d, T, Tcv0>>::Presence,
        <C1 as ChannelCfg<'d, T, Tcv1>>::Presence,
    >>::Out;

    fn split_parts(self, common: &mut DfsdmCommon<'d, T, Enabled>) -> Self::Split {
        let (d0, k0) = self.0.into_parts();
        let (d1, k1) = self.1.into_parts();
        <T::Filters as Tcv2SplitBuild<
            T,
            C,
            <C0 as ChannelCfg<'d, T, Tcv0>>::Presence,
            <C1 as ChannelCfg<'d, T, Tcv1>>::Presence,
        >>::build(common, (d0, k0), (d1, k1))
    }
}

impl<'d, T, C, C0, C1, C2, C3> ChannelCfgTuple<'d, T, C> for (C0, C1, C2, C3)
where
    T: Instance<Transceivers = capability::Tcv4>,
    C: ClockOutputMode,
    C0: ChannelCfg<'d, T, Tcv0>,
    C1: ChannelCfg<'d, T, Tcv1>,
    C2: ChannelCfg<'d, T, Tcv2>,
    C3: ChannelCfg<'d, T, Tcv3>,
    T::Filters: Tcv4SplitBuild<
            T,
            C,
            <C0 as ChannelCfg<'d, T, Tcv0>>::Presence,
            <C1 as ChannelCfg<'d, T, Tcv1>>::Presence,
            <C2 as ChannelCfg<'d, T, Tcv2>>::Presence,
            <C3 as ChannelCfg<'d, T, Tcv3>>::Presence,
        >,
{
    type Split = <T::Filters as Tcv4SplitBuild<
        T,
        C,
        <C0 as ChannelCfg<'d, T, Tcv0>>::Presence,
        <C1 as ChannelCfg<'d, T, Tcv1>>::Presence,
        <C2 as ChannelCfg<'d, T, Tcv2>>::Presence,
        <C3 as ChannelCfg<'d, T, Tcv3>>::Presence,
    >>::Out;

    fn split_parts(self, common: &mut DfsdmCommon<'d, T, Enabled>) -> Self::Split {
        let (d0, k0) = self.0.into_parts();
        let (d1, k1) = self.1.into_parts();
        let (d2, k2) = self.2.into_parts();
        let (d3, k3) = self.3.into_parts();
        <T::Filters as Tcv4SplitBuild<
            T,
            C,
            <C0 as ChannelCfg<'d, T, Tcv0>>::Presence,
            <C1 as ChannelCfg<'d, T, Tcv1>>::Presence,
            <C2 as ChannelCfg<'d, T, Tcv2>>::Presence,
            <C3 as ChannelCfg<'d, T, Tcv3>>::Presence,
        >>::build(common, (d0, k0), (d1, k1), (d2, k2), (d3, k3))
    }
}

impl<'d, T, C, C0, C1, C2, C3, C4, C5, C6, C7> ChannelCfgTuple<'d, T, C> for (C0, C1, C2, C3, C4, C5, C6, C7)
where
    T: Instance<Transceivers = capability::Tcv8>,
    C: ClockOutputMode,
    C0: ChannelCfg<'d, T, Tcv0>,
    C1: ChannelCfg<'d, T, Tcv1>,
    C2: ChannelCfg<'d, T, Tcv2>,
    C3: ChannelCfg<'d, T, Tcv3>,
    C4: ChannelCfg<'d, T, Tcv4>,
    C5: ChannelCfg<'d, T, Tcv5>,
    C6: ChannelCfg<'d, T, Tcv6>,
    C7: ChannelCfg<'d, T, Tcv7>,
    T::Filters: Tcv8SplitBuild<
            T,
            C,
            <C0 as ChannelCfg<'d, T, Tcv0>>::Presence,
            <C1 as ChannelCfg<'d, T, Tcv1>>::Presence,
            <C2 as ChannelCfg<'d, T, Tcv2>>::Presence,
            <C3 as ChannelCfg<'d, T, Tcv3>>::Presence,
            <C4 as ChannelCfg<'d, T, Tcv4>>::Presence,
            <C5 as ChannelCfg<'d, T, Tcv5>>::Presence,
            <C6 as ChannelCfg<'d, T, Tcv6>>::Presence,
            <C7 as ChannelCfg<'d, T, Tcv7>>::Presence,
        >,
{
    type Split = <T::Filters as Tcv8SplitBuild<
        T,
        C,
        <C0 as ChannelCfg<'d, T, Tcv0>>::Presence,
        <C1 as ChannelCfg<'d, T, Tcv1>>::Presence,
        <C2 as ChannelCfg<'d, T, Tcv2>>::Presence,
        <C3 as ChannelCfg<'d, T, Tcv3>>::Presence,
        <C4 as ChannelCfg<'d, T, Tcv4>>::Presence,
        <C5 as ChannelCfg<'d, T, Tcv5>>::Presence,
        <C6 as ChannelCfg<'d, T, Tcv6>>::Presence,
        <C7 as ChannelCfg<'d, T, Tcv7>>::Presence,
    >>::Out;

    fn split_parts(self, common: &mut DfsdmCommon<'d, T, Enabled>) -> Self::Split {
        let (d0, k0) = self.0.into_parts();
        let (d1, k1) = self.1.into_parts();
        let (d2, k2) = self.2.into_parts();
        let (d3, k3) = self.3.into_parts();
        let (d4, k4) = self.4.into_parts();
        let (d5, k5) = self.5.into_parts();
        let (d6, k6) = self.6.into_parts();
        let (d7, k7) = self.7.into_parts();
        <T::Filters as Tcv8SplitBuild<
            T,
            C,
            <C0 as ChannelCfg<'d, T, Tcv0>>::Presence,
            <C1 as ChannelCfg<'d, T, Tcv1>>::Presence,
            <C2 as ChannelCfg<'d, T, Tcv2>>::Presence,
            <C3 as ChannelCfg<'d, T, Tcv3>>::Presence,
            <C4 as ChannelCfg<'d, T, Tcv4>>::Presence,
            <C5 as ChannelCfg<'d, T, Tcv5>>::Presence,
            <C6 as ChannelCfg<'d, T, Tcv6>>::Presence,
            <C7 as ChannelCfg<'d, T, Tcv7>>::Presence,
        >>::build(
            common,
            (d0, k0),
            (d1, k1),
            (d2, k2),
            (d3, k3),
            (d4, k4),
            (d5, k5),
            (d6, k6),
            (d7, k7),
        )
    }
}
