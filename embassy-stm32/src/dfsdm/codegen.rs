//! Build-time DFSDM shape data and codegen. Read by `build.rs` via `#[path]`.
//!
//! Must stay free of target-specific types (no PAC / `no_std`): this file is
//! compiled into the build script, which runs on the host.

use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};

// =============================================================================
// Shape data
// =============================================================================

/// A parsed DFSDM register-block shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shape {
    /// Transceiver (channel) count: 2, 4 or 8.
    pub ch: u8,
    /// Filter count: 1, 2, 4, 6 or 8.
    pub flt: u8,
    /// Has per-channel delay (DLY) registers.
    pub dly: bool,
    /// Has the HWID version-register block.
    pub hwid: bool,
    /// Has the internal parallel-ADC input path.
    pub adc: bool,
}

/// Parse a DFSDM register-block name like `DFSDM_8CH_4FLT_DLY_TRG5_ADC`.
///
/// The name grammar is `DFSDM_{2,4,8}CH_{1,2,4,6,8}FLT[_DLY]_TRG{3,5}[_ADC][_HWID]`.
pub fn parse(block: &str) -> Option<Shape> {
    let parts: Vec<&str> = block.split('_').collect();
    if parts.len() < 3 || parts[0] != "DFSDM" {
        return None;
    }

    let ch: u8 = parts[1].strip_suffix("CH")?.parse().ok()?;
    let flt: u8 = parts[2].strip_suffix("FLT")?.parse().ok()?;

    let mut dly = false;
    let mut adc = false;
    let mut hwid = false;
    let mut trg = false;

    for p in &parts[3..] {
        match *p {
            "DLY" => dly = true,
            "ADC" => adc = true,
            "HWID" => hwid = true,
            "TRG3" | "TRG5" => trg = true,
            _ => return None,
        }
    }

    if !trg {
        return None;
    }

    Some(Shape {
        ch,
        flt,
        dly,
        hwid,
        adc,
    })
}

/// Channel count -> transceiver capability ident.
pub fn tcv(ch: u8) -> &'static str {
    match ch {
        2 => "Tcv2",
        4 => "Tcv4",
        8 => "Tcv8",
        _ => unreachable!("invalid DFSDM channel count: {}", ch),
    }
}

/// Filter count -> filter capability ident.
pub fn flt(f: u8) -> &'static str {
    match f {
        1 => "Flt1",
        2 => "Flt2",
        4 => "Flt4",
        6 => "Flt6",
        8 => "Flt8",
        _ => unreachable!("invalid DFSDM filter count: {}", f),
    }
}

/// The six (channel count, filter count) shapes that exist in hardware.
pub const SHAPES: &[(u8, u8)] = &[(2, 1), (4, 2), (4, 4), (8, 4), (8, 6), (8, 8)];

// 3-bit JEXTSEL remap table for DFSDM injected triggers.
//
// On 3-bit-JEXTSEL parts the `DFSDM1_JTRGn` channel number is *not* the
// register value; the valid channels are compressed into 0..7 per filter.
// Each row is `(filter index, jtrg channel number, jextsel)`.
pub const DFSDM_TRG3_JEXTSEL: &[(u8, u8, u8)] = &[
    (0, 0, 0),
    (0, 1, 1),
    (0, 2, 2),
    (0, 3, 3),
    (0, 5, 4),
    (0, 7, 5),
    (0, 9, 6),
    (0, 10, 7),
    (1, 0, 0),
    (1, 1, 1),
    (1, 2, 2),
    (1, 3, 3),
    (1, 5, 4),
    (1, 7, 5),
    (1, 9, 6),
    (1, 10, 7),
    (2, 0, 0),
    (2, 1, 1),
    (2, 2, 2),
    (2, 3, 3),
    (2, 5, 4),
    (2, 8, 5),
    (2, 9, 6),
    (2, 10, 7),
    (3, 0, 0),
    (3, 1, 1),
    (3, 2, 2),
    (3, 4, 3),
    (3, 6, 4),
    (3, 8, 5),
    (3, 9, 6),
    (3, 10, 7),
];

// =============================================================================
// Instance codegen
// =============================================================================

/// Emit `SealedInstance` + `Instance` + capability-flag impls for one DFSDM
/// instance, derived entirely from its register-block name.
pub fn gen_instance(inst: &str, block: &str) -> TokenStream {
    let inst = format_ident!("{}", inst);
    let shape = parse(block).unwrap_or_else(|| panic!("unrecognized DFSDM block name: {}", block));
    let tcv = format_ident!("{}", tcv(shape.ch));
    let flt = format_ident!("{}", flt(shape.flt));

    let mut ts = quote! {
        impl crate::dfsdm::SealedInstance for crate::peripherals::#inst {
            fn regs() -> crate::dfsdm::Registers {
                unsafe { crate::dfsdm::Registers::from_ptr(crate::pac::#inst.as_ptr()) }
            }
        }

        impl crate::dfsdm::Instance for crate::peripherals::#inst {
            type Transceivers = crate::dfsdm::capability::#tcv;
            type Filters = crate::dfsdm::capability::#flt;

            fn instance_state() -> &'static crate::dfsdm::InstanceState {
                static INSTANCE_STATE: crate::dfsdm::InstanceState = crate::dfsdm::InstanceState::new();
                &INSTANCE_STATE
            }
        }
    };

    if shape.dly {
        ts.extend(quote! { impl crate::dfsdm::capability::HasDelay for crate::peripherals::#inst {} });
    }
    if shape.hwid {
        ts.extend(quote! { impl crate::dfsdm::capability::HasHwid for crate::peripherals::#inst {} });
    }
    if shape.adc {
        ts.extend(quote! { impl crate::dfsdm::capability::AdcInput for crate::peripherals::#inst {} });
    }

    ts
}

// =============================================================================
// Trigger codegen
// =============================================================================

/// Emit the `TriggerSource` impls for one injected-trigger source on one DFSDM
/// instance. 5-bit parts get an identity blanket over all filters; 3-bit parts
/// get the per-filter remap from [`DFSDM_TRG3_JEXTSEL`].
pub fn gen_trigger_source(inst: &str, block: &str, source: &Ident, idx: u8) -> TokenStream {
    let inst = format_ident!("{}", inst);

    if block.contains("TRG5") {
        // 5-bit JEXTSEL: the signal number *is* the JEXTSEL value, and every
        // source can drive every filter.
        quote! {
            impl<M: crate::dfsdm::FilterMarker> crate::dfsdm::TriggerSource<crate::peripherals::#inst, M>
                for crate::triggers::#source {
                fn jextsel(&self) -> u8 { #idx }
            }
        }
    } else {
        // 3-bit JEXTSEL: remap the channel number per filter.
        let impls = DFSDM_TRG3_JEXTSEL
            .iter()
            .filter(|(_, n, _)| *n == idx)
            .map(|&(flt, _, jextsel)| {
                let flt = format_ident!("Flt{}", flt);
                quote! {
                    impl crate::dfsdm::TriggerSource<crate::peripherals::#inst, crate::dfsdm::#flt>
                        for crate::triggers::#source {
                        fn jextsel(&self) -> u8 { #jextsel }
                    }
                }
            });
        quote! { #(#impls)* }
    }
}

// =============================================================================
// Shape codegen
// =============================================================================

/// Emit the whole shape-dependent surface: the six split structs, the three
/// channel-selector structs and the three `ChannelCfgTuple` impls, all inside
/// a `pub mod dfsdm` that the driver glob-re-exports.
pub fn gen_shapes() -> TokenStream {
    let splits = SHAPES.iter().map(|&(ch, flt)| gen_split(ch, flt));
    let selectors = [2u8, 4, 8].map(gen_selector);
    let tuples = [2u8, 4, 8].map(gen_tuple);

    quote! {
        #[cfg(dfsdm)]
        pub mod dfsdm {
            #(#splits)*
            #(#selectors)*
            #(#tuples)*
        }
    }
}

/// Emit one split struct + its `Tcv*SplitBuild` impl for a (channel, filter) shape.
fn gen_split(ch: u8, flt_n: u8) -> TokenStream {
    let ch = ch as usize;
    let flt_count = flt_n as usize;

    let name = format_ident!("DfsdmSplit{}Ch{}Flt", ch, flt_count);
    let ready = format_ident!("Flt{}Ready", flt_count);
    let tcv_trait = format_ident!("Tcv{}SplitBuild", ch);
    let tcv_cap = format_ident!("{}", tcv(ch as u8));
    let flt_cap = format_ident!("{}", flt(flt_count as u8));

    let s: Vec<Ident> = (0..ch).map(|i| format_ident!("S{}", i)).collect();
    let ch_idents: Vec<Ident> = (0..ch).map(|i| format_ident!("ch{}", i)).collect();
    let tcv_idents: Vec<Ident> = (0..ch).map(|i| format_ident!("Tcv{}", i)).collect();
    let flt_idents: Vec<Ident> = (0..flt_count).map(|i| format_ident!("flt{}", i)).collect();
    let flt_markers: Vec<Ident> = (0..flt_count).map(|i| format_ident!("Flt{}", i)).collect();

    let struct_channels = (0..ch).map(|i| {
        let c = &ch_idents[i];
        let t = &tcv_idents[i];
        let own = &s[i];
        let neighbor = &s[(i + 1) % ch];
        let doc = format!("Builder for [`crate::dfsdm::Transceiver`] {}.", i);
        quote! { #[doc = #doc] pub #c: crate::dfsdm::TransceiverBuilder<T, crate::dfsdm::#t, C, #own, #neighbor>, }
    });
    let struct_filters = (0..flt_count).map(|i| {
        let f = &flt_idents[i];
        let m = &flt_markers[i];
        let doc = format!("Builder for [`crate::dfsdm::Filter`] {}.", i);
        quote! { #[doc = #doc] pub #f: crate::dfsdm::FilterBuilder<T, crate::dfsdm::#m>, }
    });

    let build_args = (0..ch).map(|i| {
        let c = &ch_idents[i];
        let own = &s[i];
        quote! { #c: (#own::Datin<'d>, #own::Ckin<'d>), }
    });

    let inserts = (0..ch).flat_map(|i| {
        let c = &ch_idents[i];
        let own = &s[i];
        let idx = Literal::usize_unsuffixed(i);
        [
            quote! { common.insert_pin(#idx, crate::dfsdm::PinKind::Datin, #own::extract_datin(#c.0)); },
            quote! { common.insert_pin(#idx, crate::dfsdm::PinKind::Ckin, #own::extract_ckin(#c.1)); },
        ]
    });

    let construct_channels = (0..ch).map(|i| {
        let c = &ch_idents[i];
        quote! { #c: crate::dfsdm::TransceiverBuilder::new(), }
    });
    let construct_filters = (0..flt_count).map(|i| {
        let f = &flt_idents[i];
        quote! { #f: crate::dfsdm::FilterBuilder::new(), }
    });

    quote! {
        /// One [`crate::dfsdm::TransceiverBuilder`] per transceiver, one
        /// [`crate::dfsdm::FilterBuilder`] per filter, and the shared
        /// [`crate::dfsdm::DetectorsBuilder`].
        pub struct #name<T, C, #(#s),*>
        where
            T: crate::dfsdm::Instance + crate::dfsdm::#ready,
            C: crate::dfsdm::ClockOutputMode,
            #(#s: crate::dfsdm::PinSet,)*
        {
            /// Builds the instance-level [`crate::dfsdm::ShortCircuitDetector`] and
            /// [`crate::dfsdm::ClockAbsenceDetector`].
            pub detectors: crate::dfsdm::DetectorsBuilder<T>,
            #(#struct_channels)*
            #(#struct_filters)*
        }

        impl<T, C, #(#s),*> crate::dfsdm::#tcv_trait<T, C, #(#s),*> for crate::dfsdm::capability::#flt_cap
        where
            T: crate::dfsdm::Instance<
                    Transceivers = crate::dfsdm::capability::#tcv_cap,
                    Filters = crate::dfsdm::capability::#flt_cap,
                > + crate::dfsdm::#ready,
            C: crate::dfsdm::ClockOutputMode,
            #(#s: crate::dfsdm::PinSet,)*
        {
            type Out = #name<T, C, #(#s),*>;

            fn build<'d>(
                common: &mut crate::dfsdm::DfsdmCommon<'d, T, crate::dfsdm::Enabled>,
                #(#build_args)*
            ) -> Self::Out {
                #(#inserts)*
                #name {
                    detectors: crate::dfsdm::DetectorsBuilder::new(),
                    #(#construct_channels)*
                    #(#construct_filters)*
                }
            }
        }
    }
}

/// Emit one channel-selector struct + its `Shape` impl for a channel arity.
fn gen_selector(ch: u8) -> TokenStream {
    let ch = ch as usize;
    let name = format_ident!("ChannelSelectors{}", ch);
    let tcv_cap = format_ident!("{}", tcv(ch as u8));

    let ch_idents: Vec<Ident> = (0..ch).map(|i| format_ident!("ch{}", i)).collect();
    let tcv_idents: Vec<Ident> = (0..ch).map(|i| format_ident!("Tcv{}", i)).collect();

    let fields = (0..ch).map(|i| {
        let c = &ch_idents[i];
        let t = &tcv_idents[i];
        let doc = format!("Pin selector for transceiver {}.", i);
        quote! { #[doc = #doc] pub #c: crate::dfsdm::Sel<T, crate::dfsdm::#t>, }
    });
    let constructs = (0..ch).map(|i| {
        let c = &ch_idents[i];
        quote! { #c: crate::dfsdm::Sel::new(), }
    });

    quote! {
        /// The pin selectors handed to the [`crate::dfsdm::Dfsdm::configure_pins`]
        /// closure. Declare each transceiver's pins with [`crate::dfsdm::Sel::datin`],
        /// [`crate::dfsdm::Sel::datin_ckin`] or [`crate::dfsdm::Sel::none`].
        pub struct #name<T: crate::dfsdm::Instance> {
            #(#fields)*
        }
        impl<T: crate::dfsdm::Instance> #name<T> {
            pub(crate) fn new() -> Self {
                Self { #(#constructs)* }
            }
        }
        impl crate::dfsdm::Shape for crate::dfsdm::capability::#tcv_cap {
            type Selectors<T: crate::dfsdm::Instance> = #name<T>;
            fn selectors<T: crate::dfsdm::Instance>() -> Self::Selectors<T> {
                #name::new()
            }
        }
    }
}

/// Emit the `ChannelCfgTuple` impl for a channel-arity tuple `(C0, .., C{ch-1})`.
fn gen_tuple(ch: u8) -> TokenStream {
    let ch = ch as usize;
    let tcv_cap = format_ident!("{}", tcv(ch as u8));
    let tcv_trait = format_ident!("Tcv{}SplitBuild", ch);

    let c: Vec<Ident> = (0..ch).map(|i| format_ident!("C{}", i)).collect();
    let tcv_idents: Vec<Ident> = (0..ch).map(|i| format_ident!("Tcv{}", i)).collect();
    let d: Vec<Ident> = (0..ch).map(|i| format_ident!("d{}", i)).collect();
    let k: Vec<Ident> = (0..ch).map(|i| format_ident!("k{}", i)).collect();

    let where_c = (0..ch).map(|i| {
        let ci = &c[i];
        let t = &tcv_idents[i];
        quote! { #ci: crate::dfsdm::ChannelCfg<'d, T, crate::dfsdm::#t>, }
    });
    let presence: Vec<TokenStream> = (0..ch)
        .map(|i| {
            let ci = &c[i];
            let t = &tcv_idents[i];
            quote! { <#ci as crate::dfsdm::ChannelCfg<'d, T, crate::dfsdm::#t>>::Presence }
        })
        .collect();
    let into_parts = (0..ch).map(|i| {
        let di = &d[i];
        let ki = &k[i];
        let idx = Literal::usize_unsuffixed(i);
        quote! { let (#di, #ki) = self.#idx.into_parts(); }
    });
    let build_args = (0..ch).map(|i| {
        let di = &d[i];
        let ki = &k[i];
        quote! { (#di, #ki) }
    });

    quote! {
        impl<'d, T, C, #(#c),*> crate::dfsdm::ChannelCfgTuple<'d, T, C> for (#(#c),*)
        where
            T: crate::dfsdm::Instance<Transceivers = crate::dfsdm::capability::#tcv_cap>,
            C: crate::dfsdm::ClockOutputMode,
            #(#where_c)*
            T::Filters: crate::dfsdm::#tcv_trait<T, C, #(#presence),*>,
        {
            type Split = <T::Filters as crate::dfsdm::#tcv_trait<T, C, #(#presence),*>>::Out;

            fn split_parts(
                self,
                common: &mut crate::dfsdm::DfsdmCommon<'d, T, crate::dfsdm::Enabled>,
            ) -> Self::Split {
                #(#into_parts)*
                <T::Filters as crate::dfsdm::#tcv_trait<T, C, #(#presence),*>>::build(common, #(#build_args),*)
            }
        }
    }
}
