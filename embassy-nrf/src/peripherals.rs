embassy_extras::peripherals! {
    // RTC
    rtc0: RTC0,
    rtc1: RTC1,
    #[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
    rtc2: RTC2,

    // QSPI
    #[cfg(feature = "52840")]
    qspi: QSPI,

    // UARTE
    uarte0: UARTE0,
    #[cfg(any(feature = "52833", feature = "52840", feature = "9160"))]
    uarte1: UARTE1,

    // SPIM
    // TODO this is actually shared with SPI, SPIM, SPIS, TWI, TWIS, TWIS.
    // When they're all implemented, they should be only one peripheral here.
    spim0: SPIM0,
    #[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
    spim1: SPIM1,
    #[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
    spim2: SPIM2,
    #[cfg(any(feature = "52833", feature = "52840"))]
    spim3: SPIM3,

    // GPIOTE
    gpiote: GPIOTE,
    gpiote_ch_0: GPIOTE_CH0,
    gpiote_ch_1: GPIOTE_CH1,
    gpiote_ch_2: GPIOTE_CH2,
    gpiote_ch_3: GPIOTE_CH3,
    gpiote_ch_4: GPIOTE_CH4,
    gpiote_ch_5: GPIOTE_CH5,
    gpiote_ch_6: GPIOTE_CH6,
    gpiote_ch_7: GPIOTE_CH7,

    // GPIO port 0
    p0_00: P0_00,
    p0_01: P0_01,
    p0_02: P0_02,
    p0_03: P0_03,
    p0_04: P0_04,
    p0_05: P0_05,
    p0_06: P0_06,
    p0_07: P0_07,
    p0_08: P0_08,
    p0_09: P0_09,
    p0_10: P0_10,
    p0_11: P0_11,
    p0_12: P0_12,
    p0_13: P0_13,
    p0_14: P0_14,
    p0_15: P0_15,
    p0_16: P0_16,
    p0_17: P0_17,
    p0_18: P0_18,
    p0_19: P0_19,
    p0_20: P0_20,
    p0_21: P0_21,
    p0_22: P0_22,
    p0_23: P0_23,
    p0_24: P0_24,
    p0_25: P0_25,
    p0_26: P0_26,
    p0_27: P0_27,
    p0_28: P0_28,
    p0_29: P0_29,
    p0_30: P0_30,
    p0_31: P0_31,

    // GPIO port 1
    #[cfg(any(feature = "52833", feature = "52840"))]
    p1_00: P1_00,
    #[cfg(any(feature = "52833", feature = "52840"))]
    p1_01: P1_01,
    #[cfg(any(feature = "52833", feature = "52840"))]
    p1_02: P1_02,
    #[cfg(any(feature = "52833", feature = "52840"))]
    p1_03: P1_03,
    #[cfg(any(feature = "52833", feature = "52840"))]
    p1_04: P1_04,
    #[cfg(any(feature = "52833", feature = "52840"))]
    p1_05: P1_05,
    #[cfg(any(feature = "52833", feature = "52840"))]
    p1_06: P1_06,
    #[cfg(any(feature = "52833", feature = "52840"))]
    p1_07: P1_07,
    #[cfg(any(feature = "52833", feature = "52840"))]
    p1_08: P1_08,
    #[cfg(any(feature = "52833", feature = "52840"))]
    p1_09: P1_09,
    #[cfg(any(feature = "52833", feature = "52840"))]
    p1_10: P1_10,
    #[cfg(any(feature = "52833", feature = "52840"))]
    p1_11: P1_11,
    #[cfg(any(feature = "52833", feature = "52840"))]
    p1_12: P1_12,
    #[cfg(any(feature = "52833", feature = "52840"))]
    p1_13: P1_13,
    #[cfg(any(feature = "52833", feature = "52840"))]
    p1_14: P1_14,
    #[cfg(any(feature = "52833", feature = "52840"))]
    p1_15: P1_15,
}
