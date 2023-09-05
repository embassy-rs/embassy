#[cfg(not(adc_f3_v2))]
macro_rules! impl_sample_time {
    ($default_doc:expr, $default:ident, ($(($doc:expr, $variant:ident, $pac_variant:ident)),*)) => {
        #[doc = concat!("ADC sample time\n\nThe default setting is ", $default_doc, " ADC clock cycles.")]
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
        pub enum SampleTime {
            $(
                #[doc = concat!($doc, " ADC clock cycles.")]
                $variant,
            )*
        }

        impl From<SampleTime> for crate::pac::adc::vals::SampleTime {
            fn from(sample_time: SampleTime) -> crate::pac::adc::vals::SampleTime {
                match sample_time {
                    $(SampleTime::$variant => crate::pac::adc::vals::SampleTime::$pac_variant),*
                }
            }
        }

        impl Default for SampleTime {
            fn default() -> Self {
                Self::$default
            }
        }
    };
}

#[cfg(any(adc_f1, adc_v1))]
impl_sample_time!(
    "1.5",
    Cycles1_5,
    (
        ("1.5", Cycles1_5, CYCLES1_5),
        ("7.5", Cycles7_5, CYCLES7_5),
        ("13.5", Cycles13_5, CYCLES13_5),
        ("28.5", Cycles28_5, CYCLES28_5),
        ("41.5", Cycles41_5, CYCLES41_5),
        ("55.5", Cycles55_5, CYCLES55_5),
        ("71.5", Cycles71_5, CYCLES71_5),
        ("239.5", Cycles239_5, CYCLES239_5)
    )
);

#[cfg(adc_v2)]
impl_sample_time!(
    "3",
    Cycles3,
    (
        ("3", Cycles3, CYCLES3),
        ("15", Cycles15, CYCLES15),
        ("28", Cycles28, CYCLES28),
        ("56", Cycles56, CYCLES56),
        ("84", Cycles84, CYCLES84),
        ("112", Cycles112, CYCLES112),
        ("144", Cycles144, CYCLES144),
        ("480", Cycles480, CYCLES480)
    )
);

#[cfg(adc_v3)]
impl_sample_time!(
    "2.5",
    Cycles2_5,
    (
        ("2.5", Cycles2_5, CYCLES2_5),
        ("6.5", Cycles6_5, CYCLES6_5),
        ("12.5", Cycles12_5, CYCLES12_5),
        ("24.5", Cycles24_5, CYCLES24_5),
        ("47.5", Cycles47_5, CYCLES47_5),
        ("92.5", Cycles92_5, CYCLES92_5),
        ("247.5", Cycles247_5, CYCLES247_5),
        ("640.5", Cycles640_5, CYCLES640_5)
    )
);

#[cfg(adc_g0)]
impl_sample_time!(
    "1.5",
    Cycles1_5,
    (
        ("1.5", Cycles1_5, CYCLES1_5),
        ("3.5", Cycles3_5, CYCLES3_5),
        ("7.5", Cycles7_5, CYCLES7_5),
        ("12.5", Cycles12_5, CYCLES12_5),
        ("19.5", Cycles19_5, CYCLES19_5),
        ("39.5", Cycles39_5, CYCLES39_5),
        ("79.5", Cycles79_5, CYCLES79_5),
        ("160.5", Cycles160_5, CYCLES160_5)
    )
);

#[cfg(adc_v4)]
impl_sample_time!(
    "1.5",
    Cycles1_5,
    (
        ("1.5", Cycles1_5, CYCLES1_5),
        ("2.5", Cycles2_5, CYCLES2_5),
        ("8.5", Cycles8_5, CYCLES8_5),
        ("16.5", Cycles16_5, CYCLES16_5),
        ("32.5", Cycles32_5, CYCLES32_5),
        ("64.5", Cycles64_5, CYCLES64_5),
        ("387.5", Cycles387_5, CYCLES387_5),
        ("810.5", Cycles810_5, CYCLES810_5)
    )
);

#[cfg(adc_f3)]
impl_sample_time!(
    "1.5",
    Cycles1_5,
    (
        ("1.5", Cycles1_5, CYCLES1_5),
        ("2.5", Cycles2_5, CYCLES2_5),
        ("4.5", Cycles4_5, CYCLES4_5),
        ("7.5", Cycles7_5, CYCLES7_5),
        ("19.5", Cycles19_5, CYCLES19_5),
        ("61.5", Cycles61_5, CYCLES61_5),
        ("181.5", Cycles181_5, CYCLES181_5),
        ("601.5", Cycles601_5, CYCLES601_5)
    )
);
