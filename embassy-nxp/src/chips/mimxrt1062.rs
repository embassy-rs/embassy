// This must be imported so that __preinit is defined.
use imxrt_rt as _;
pub use nxp_pac as pac;

embassy_hal_internal::peripherals! {
    // External pins. These are not only GPIOs, they are multi-purpose pins and can be used by other
    // peripheral types (e.g. I2C).
    GPIO_AD_B0_00,
    GPIO_AD_B0_01,
    GPIO_AD_B0_02,
    GPIO_AD_B0_03,
    GPIO_AD_B0_04,
    GPIO_AD_B0_05,
    GPIO_AD_B0_06,
    GPIO_AD_B0_07,
    GPIO_AD_B0_08,
    GPIO_AD_B0_09,
    GPIO_AD_B0_10,
    GPIO_AD_B0_11,
    GPIO_AD_B0_12,
    GPIO_AD_B0_13,
    GPIO_AD_B0_14,
    GPIO_AD_B0_15,
    GPIO_AD_B1_00,
    GPIO_AD_B1_01,
    GPIO_AD_B1_02,
    GPIO_AD_B1_03,
    GPIO_AD_B1_04,
    GPIO_AD_B1_05,
    GPIO_AD_B1_06,
    GPIO_AD_B1_07,
    GPIO_AD_B1_08,
    GPIO_AD_B1_09,
    GPIO_AD_B1_10,
    GPIO_AD_B1_11,
    GPIO_AD_B1_12,
    GPIO_AD_B1_13,
    GPIO_AD_B1_14,
    GPIO_AD_B1_15,
    GPIO_B0_00,
    GPIO_B0_01,
    GPIO_B0_02,
    GPIO_B0_03,
    GPIO_B0_04,
    GPIO_B0_05,
    GPIO_B0_06,
    GPIO_B0_07,
    GPIO_B0_08,
    GPIO_B0_09,
    GPIO_B0_10,
    GPIO_B0_11,
    GPIO_B0_12,
    GPIO_B0_13,
    GPIO_B0_14,
    GPIO_B0_15,
    GPIO_B1_00,
    GPIO_B1_01,
    GPIO_B1_02,
    GPIO_B1_03,
    GPIO_B1_04,
    GPIO_B1_05,
    GPIO_B1_06,
    GPIO_B1_07,
    GPIO_B1_08,
    GPIO_B1_09,
    GPIO_B1_10,
    GPIO_B1_11,
    GPIO_B1_12,
    GPIO_B1_13,
    GPIO_B1_14,
    GPIO_B1_15,
    GPIO_EMC_00,
    GPIO_EMC_01,
    GPIO_EMC_02,
    GPIO_EMC_03,
    GPIO_EMC_04,
    GPIO_EMC_05,
    GPIO_EMC_06,
    GPIO_EMC_07,
    GPIO_EMC_08,
    GPIO_EMC_09,
    GPIO_EMC_10,
    GPIO_EMC_11,
    GPIO_EMC_12,
    GPIO_EMC_13,
    GPIO_EMC_14,
    GPIO_EMC_15,
    GPIO_EMC_16,
    GPIO_EMC_17,
    GPIO_EMC_18,
    GPIO_EMC_19,
    GPIO_EMC_20,
    GPIO_EMC_21,
    GPIO_EMC_22,
    GPIO_EMC_23,
    GPIO_EMC_24,
    GPIO_EMC_25,
    GPIO_EMC_26,
    GPIO_EMC_27,
    GPIO_EMC_28,
    GPIO_EMC_29,
    GPIO_EMC_30,
    GPIO_EMC_31,
    GPIO_EMC_32,
    GPIO_EMC_33,
    GPIO_EMC_34,
    GPIO_EMC_35,
    GPIO_EMC_36,
    GPIO_EMC_37,
    GPIO_EMC_38,
    GPIO_EMC_39,
    GPIO_EMC_40,
    GPIO_EMC_41,
    GPIO_SD_B0_00,
    GPIO_SD_B0_01,
    GPIO_SD_B0_02,
    GPIO_SD_B0_03,
    GPIO_SD_B0_04,
    GPIO_SD_B0_05,
    GPIO_SD_B1_00,
    GPIO_SD_B1_01,
    GPIO_SD_B1_02,
    GPIO_SD_B1_03,
    GPIO_SD_B1_04,
    GPIO_SD_B1_05,
    GPIO_SD_B1_06,
    GPIO_SD_B1_07,
    GPIO_SD_B1_08,
    GPIO_SD_B1_09,
    GPIO_SD_B1_10,
    GPIO_SD_B1_11,
    WAKEUP,
    PMIC_ON_REQ,
    PMIC_STBY_REQ,
}

impl_gpio! {
    // GPIO Bank 1
    GPIO_AD_B0_00(Gpio1, 0);
    GPIO_AD_B0_01(Gpio1, 1);
    GPIO_AD_B0_02(Gpio1, 2);
    GPIO_AD_B0_03(Gpio1, 3);
    GPIO_AD_B0_04(Gpio1, 4);
    GPIO_AD_B0_05(Gpio1, 5);
    GPIO_AD_B0_06(Gpio1, 6);
    GPIO_AD_B0_07(Gpio1, 7);
    GPIO_AD_B0_08(Gpio1, 8);
    GPIO_AD_B0_09(Gpio1, 9);
    GPIO_AD_B0_10(Gpio1, 10);
    GPIO_AD_B0_11(Gpio1, 11);
    GPIO_AD_B0_12(Gpio1, 12);
    GPIO_AD_B0_13(Gpio1, 13);
    GPIO_AD_B0_14(Gpio1, 14);
    GPIO_AD_B0_15(Gpio1, 15);
    GPIO_AD_B1_00(Gpio1, 16);
    GPIO_AD_B1_01(Gpio1, 17);
    GPIO_AD_B1_02(Gpio1, 18);
    GPIO_AD_B1_03(Gpio1, 19);
    GPIO_AD_B1_04(Gpio1, 20);
    GPIO_AD_B1_05(Gpio1, 21);
    GPIO_AD_B1_06(Gpio1, 22);
    GPIO_AD_B1_07(Gpio1, 23);
    GPIO_AD_B1_08(Gpio1, 24);
    GPIO_AD_B1_09(Gpio1, 25);
    GPIO_AD_B1_10(Gpio1, 26);
    GPIO_AD_B1_11(Gpio1, 27);
    GPIO_AD_B1_12(Gpio1, 28);
    GPIO_AD_B1_13(Gpio1, 29);
    GPIO_AD_B1_14(Gpio1, 30);
    GPIO_AD_B1_15(Gpio1, 31);

    // GPIO Bank 2
    GPIO_B0_00(Gpio2, 0);
    GPIO_B0_01(Gpio2, 1);
    GPIO_B0_02(Gpio2, 2);
    GPIO_B0_03(Gpio2, 3);
    GPIO_B0_04(Gpio2, 4);
    GPIO_B0_05(Gpio2, 5);
    GPIO_B0_06(Gpio2, 6);
    GPIO_B0_07(Gpio2, 7);
    GPIO_B0_08(Gpio2, 8);
    GPIO_B0_09(Gpio2, 9);
    GPIO_B0_10(Gpio2, 10);
    GPIO_B0_11(Gpio2, 11);
    GPIO_B0_12(Gpio2, 12);
    GPIO_B0_13(Gpio2, 13);
    GPIO_B0_14(Gpio2, 14);
    GPIO_B0_15(Gpio2, 15);
    GPIO_B1_00(Gpio2, 16);
    GPIO_B1_01(Gpio2, 17);
    GPIO_B1_02(Gpio2, 18);
    GPIO_B1_03(Gpio2, 19);
    GPIO_B1_04(Gpio2, 20);
    GPIO_B1_05(Gpio2, 21);
    GPIO_B1_06(Gpio2, 22);
    GPIO_B1_07(Gpio2, 23);
    GPIO_B1_08(Gpio2, 24);
    GPIO_B1_09(Gpio2, 25);
    GPIO_B1_10(Gpio2, 26);
    GPIO_B1_11(Gpio2, 27);
    GPIO_B1_12(Gpio2, 28);
    GPIO_B1_13(Gpio2, 29);
    GPIO_B1_14(Gpio2, 30);
    GPIO_B1_15(Gpio2, 31);

    // GPIO Bank 4 (EMC is 4, then 3)
    GPIO_EMC_00(Gpio4, 0);
    GPIO_EMC_01(Gpio4, 1);
    GPIO_EMC_02(Gpio4, 2);
    GPIO_EMC_03(Gpio4, 3);
    GPIO_EMC_04(Gpio4, 4);
    GPIO_EMC_05(Gpio4, 5);
    GPIO_EMC_06(Gpio4, 6);
    GPIO_EMC_07(Gpio4, 7);
    GPIO_EMC_08(Gpio4, 8);
    GPIO_EMC_09(Gpio4, 9);
    GPIO_EMC_10(Gpio4, 10);
    GPIO_EMC_11(Gpio4, 11);
    GPIO_EMC_12(Gpio4, 12);
    GPIO_EMC_13(Gpio4, 13);
    GPIO_EMC_14(Gpio4, 14);
    GPIO_EMC_15(Gpio4, 15);
    GPIO_EMC_16(Gpio4, 16);
    GPIO_EMC_17(Gpio4, 17);
    GPIO_EMC_18(Gpio4, 18);
    GPIO_EMC_19(Gpio4, 19);
    GPIO_EMC_20(Gpio4, 20);
    GPIO_EMC_21(Gpio4, 21);
    GPIO_EMC_22(Gpio4, 22);
    GPIO_EMC_23(Gpio4, 23);
    GPIO_EMC_24(Gpio4, 24);
    GPIO_EMC_25(Gpio4, 25);
    GPIO_EMC_26(Gpio4, 26);
    GPIO_EMC_27(Gpio4, 27);
    GPIO_EMC_28(Gpio4, 28);
    GPIO_EMC_29(Gpio4, 29);
    GPIO_EMC_30(Gpio4, 30);
    GPIO_EMC_31(Gpio4, 31);

    // GPIO Bank 3
    GPIO_EMC_32(Gpio3, 18);
    GPIO_EMC_33(Gpio3, 19);
    GPIO_EMC_34(Gpio3, 20);
    GPIO_EMC_35(Gpio3, 21);
    GPIO_EMC_36(Gpio3, 22);
    GPIO_EMC_37(Gpio3, 23);
    GPIO_EMC_38(Gpio3, 24);
    GPIO_EMC_39(Gpio3, 25);
    GPIO_EMC_40(Gpio3, 26);
    GPIO_EMC_41(Gpio3, 27);
    GPIO_SD_B0_00(Gpio3, 12);
    GPIO_SD_B0_01(Gpio3, 13);
    GPIO_SD_B0_02(Gpio3, 14);
    GPIO_SD_B0_03(Gpio3, 15);
    GPIO_SD_B0_04(Gpio3, 16);
    GPIO_SD_B0_05(Gpio3, 17);
    GPIO_SD_B1_00(Gpio3, 0);
    GPIO_SD_B1_01(Gpio3, 1);
    GPIO_SD_B1_02(Gpio3, 2);
    GPIO_SD_B1_03(Gpio3, 3);
    GPIO_SD_B1_04(Gpio3, 4);
    GPIO_SD_B1_05(Gpio3, 5);
    GPIO_SD_B1_06(Gpio3, 6);
    GPIO_SD_B1_07(Gpio3, 7);
    GPIO_SD_B1_08(Gpio3, 8);
    GPIO_SD_B1_09(Gpio3, 9);
    GPIO_SD_B1_10(Gpio3, 10);
    GPIO_SD_B1_11(Gpio3, 11);

    WAKEUP(Gpio5, 0);
    PMIC_ON_REQ(Gpio5, 1);
    PMIC_STBY_REQ(Gpio5, 2);
}

pub(crate) mod _generated {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    #![allow(non_snake_case)]
    #![allow(missing_docs)]

    include!(concat!(env!("OUT_DIR"), "/_generated.rs"));
}
