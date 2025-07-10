// This must be imported so that __preinit is defined.
use imxrt_rt as _;
pub use nxp_pac as pac;

embassy_hal_internal::peripherals! {
    // External pins. These are not only GPIOs, they are multi-purpose pins and can be used by other
    // peripheral types (e.g. I2C).
    GPIO_00,
    GPIO_01,
    GPIO_02,
    GPIO_03,
    GPIO_04,
    GPIO_05,
    GPIO_06,
    GPIO_07,
    GPIO_08,
    GPIO_09,
    GPIO_10,
    GPIO_11,
    GPIO_12,
    GPIO_13,
    GPIO_AD_00,
    GPIO_AD_01,
    GPIO_AD_02,
    GPIO_AD_03,
    GPIO_AD_04,
    GPIO_AD_05,
    GPIO_AD_06,
    GPIO_AD_07,
    GPIO_AD_08,
    GPIO_AD_09,
    GPIO_AD_10,
    GPIO_AD_11,
    GPIO_AD_12,
    GPIO_AD_13,
    GPIO_AD_14,
    GPIO_SD_00,
    GPIO_SD_01,
    GPIO_SD_02,
    GPIO_SD_03,
    GPIO_SD_04,
    GPIO_SD_05,
    GPIO_SD_06,
    GPIO_SD_07,
    GPIO_SD_08,
    GPIO_SD_09,
    GPIO_SD_10,
    GPIO_SD_11,
    GPIO_SD_12,
    GPIO_SD_13,
    PMIC_ON_REQ,
}

impl_gpio! {
    // GPIO Bank 1
    GPIO_00(Gpio1, 0);
    GPIO_01(Gpio1, 1);
    GPIO_02(Gpio1, 2);
    GPIO_03(Gpio1, 3);
    GPIO_04(Gpio1, 4);
    GPIO_05(Gpio1, 5);
    GPIO_06(Gpio1, 6);
    GPIO_07(Gpio1, 7);
    GPIO_08(Gpio1, 8);
    GPIO_09(Gpio1, 9);
    GPIO_10(Gpio1, 10);
    GPIO_11(Gpio1, 11);
    GPIO_12(Gpio1, 12);
    GPIO_13(Gpio1, 13);
    GPIO_AD_00(Gpio1, 14);
    GPIO_AD_01(Gpio1, 15);
    GPIO_AD_02(Gpio1, 16);
    GPIO_AD_03(Gpio1, 17);
    GPIO_AD_04(Gpio1, 18);
    GPIO_AD_05(Gpio1, 19);
    GPIO_AD_06(Gpio1, 20);
    GPIO_AD_07(Gpio1, 21);
    GPIO_AD_08(Gpio1, 22);
    GPIO_AD_09(Gpio1, 23);
    GPIO_AD_10(Gpio1, 24);
    GPIO_AD_11(Gpio1, 25);
    GPIO_AD_12(Gpio1, 26);
    GPIO_AD_13(Gpio1, 27);
    GPIO_AD_14(Gpio1, 28);

    // GPIO Bank 2
    GPIO_SD_00(Gpio2, 0);
    GPIO_SD_01(Gpio2, 1);
    GPIO_SD_02(Gpio2, 2);
    GPIO_SD_03(Gpio2, 3);
    GPIO_SD_04(Gpio2, 4);
    GPIO_SD_05(Gpio2, 5);
    GPIO_SD_06(Gpio2, 6);
    GPIO_SD_07(Gpio2, 7);
    GPIO_SD_08(Gpio2, 8);
    GPIO_SD_09(Gpio2, 9);
    GPIO_SD_10(Gpio2, 10);
    GPIO_SD_11(Gpio2, 11);
    GPIO_SD_12(Gpio2, 12);
    GPIO_SD_13(Gpio2, 13);

    // GPIO Bank 5
    PMIC_ON_REQ(Gpio5, 0);
}

pub(crate) mod _generated {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    #![allow(non_snake_case)]
    #![allow(missing_docs)]

    include!(concat!(env!("OUT_DIR"), "/_generated.rs"));
}
