#[doc = "Register `FlexPWM0_SM0_EXTA0` reader"]
pub type R = crate::R<FlexPwm0Sm0Exta0Spec>;
#[doc = "Register `FlexPWM0_SM0_EXTA0` writer"]
pub type W = crate::W<FlexPwm0Sm0Exta0Spec>;
#[doc = "EXTA input connections for PWM0\n\nValue on reset: 63"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Trigin {
    #[doc = "1: ARM_TXEV input is selected"]
    Val1 = 1,
    #[doc = "2: AOI0_OUT0 input is selected"]
    Val2 = 2,
    #[doc = "3: AOI0_OUT1 input is selected"]
    Val3 = 3,
    #[doc = "4: AOI0_OUT2 input is selected"]
    Val4 = 4,
    #[doc = "5: AOI0_OUT3 input is selected"]
    Val5 = 5,
    #[doc = "6: CMP0_OUT input is selected"]
    Val6 = 6,
    #[doc = "7: CMP1_OUT input is selected"]
    Val7 = 7,
    #[doc = "8: CMP2_OUT input is selected"]
    Val8 = 8,
    #[doc = "9: CTimer0_MAT2 input is selected"]
    Val9 = 9,
    #[doc = "10: CTimer0_MAT3 input is selected"]
    Val10 = 10,
    #[doc = "11: CTimer1_MAT2 input is selected"]
    Val11 = 11,
    #[doc = "12: CTimer1_MAT3 input is selected"]
    Val12 = 12,
    #[doc = "13: CTimer2_MAT2 input is selected"]
    Val13 = 13,
    #[doc = "14: CTimer2_MAT3 input is selected"]
    Val14 = 14,
    #[doc = "15: QDC0_CMP_FLAG0 input is selected"]
    Val15 = 15,
    #[doc = "16: QDC0_CMP_FLAG1 input is selected"]
    Val16 = 16,
    #[doc = "17: QDC0_CMP_FLAG2 input is selected"]
    Val17 = 17,
    #[doc = "18: QDC0_CMP_FLAG3 input is selected"]
    Val18 = 18,
    #[doc = "19: QDC0_POS_MATCH0 input is selected"]
    Val19 = 19,
    #[doc = "20: TRIG_IN0 input is selected"]
    Val20 = 20,
    #[doc = "21: TRIG_IN1 input is selected"]
    Val21 = 21,
    #[doc = "22: TRIG_IN2 input is selected"]
    Val22 = 22,
    #[doc = "23: TRIG_IN3 input is selected"]
    Val23 = 23,
    #[doc = "24: TRIG_IN4 input is selected"]
    Val24 = 24,
    #[doc = "25: TRIG_IN5 input is selected"]
    Val25 = 25,
    #[doc = "26: TRIG_IN6 input is selected"]
    Val26 = 26,
    #[doc = "27: TRIG_IN7 input is selected"]
    Val27 = 27,
    #[doc = "28: TRIG_IN8 input is selected"]
    Val28 = 28,
    #[doc = "29: TRIG_IN9 input is selected"]
    Val29 = 29,
    #[doc = "30: TRIG_IN10 input is selected"]
    Val30 = 30,
    #[doc = "31: TRIG_IN11 input is selected"]
    Val31 = 31,
    #[doc = "32: GPIO0 Pin Event Trig 0 input is selected"]
    Val32 = 32,
    #[doc = "33: GPIO1 Pin Event Trig 0 input is selected"]
    Val33 = 33,
    #[doc = "34: GPIO2 Pin Event Trig 0 input is selected"]
    Val34 = 34,
    #[doc = "35: GPIO3 Pin Event Trig 0 input is selected"]
    Val35 = 35,
    #[doc = "36: GPIO4 Pin Event Trig 0 input is selected"]
    Val36 = 36,
    #[doc = "37: AOI1_OUT0 input is selected"]
    Val37 = 37,
    #[doc = "38: AOI1_OUT1 input is selected"]
    Val38 = 38,
    #[doc = "39: AOI1_OUT2 input is selected"]
    Val39 = 39,
    #[doc = "40: AOI1_OUT3 input is selected"]
    Val40 = 40,
    #[doc = "45: CTimer3_MAT2 input is selected"]
    Val45 = 45,
    #[doc = "46: CTimer3_MAT3 input is selected"]
    Val46 = 46,
    #[doc = "47: CTimer4_MAT2 input is selected"]
    Val47 = 47,
    #[doc = "48: CTimer4_MAT3 input is selected"]
    Val48 = 48,
    #[doc = "49: QDC1_CMP_FLAG0 input is selected"]
    Val49 = 49,
    #[doc = "50: QDC1_CMP_FLAG1 input is selected"]
    Val50 = 50,
    #[doc = "51: QDC1_CMP_FLAG2 input is selected"]
    Val51 = 51,
    #[doc = "52: QDC1_CMP_FLAG3 input is selected"]
    Val52 = 52,
    #[doc = "53: QDC1_POS_MATCH0 input is selected"]
    Val53 = 53,
    #[doc = "54: PWM1_SM0_MUX_TRIG0 input is selected"]
    Val54 = 54,
    #[doc = "55: PWM1_SM0_MUX_TRIG1 input is selected"]
    Val55 = 55,
    #[doc = "56: PWM1_SM1_MUX_TRIG0 input is selected"]
    Val56 = 56,
    #[doc = "57: PWM1_SM1_MUX_TRIG1 input is selected"]
    Val57 = 57,
    #[doc = "58: PWM1_SM2_MUX_TRIG0 input is selected"]
    Val58 = 58,
    #[doc = "59: PWM1_SM2_MUX_TRIG1 input is selected"]
    Val59 = 59,
    #[doc = "60: PWM1_SM3_MUX_TRIG0 input is selected"]
    Val60 = 60,
    #[doc = "61: PWM1_SM3_MUX_TRIG1 input is selected"]
    Val61 = 61,
}
impl From<Trigin> for u8 {
    #[inline(always)]
    fn from(variant: Trigin) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Trigin {
    type Ux = u8;
}
impl crate::IsEnum for Trigin {}
#[doc = "Field `TRIGIN` reader - EXTA input connections for PWM0"]
pub type TriginR = crate::FieldReader<Trigin>;
impl TriginR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Trigin> {
        match self.bits {
            1 => Some(Trigin::Val1),
            2 => Some(Trigin::Val2),
            3 => Some(Trigin::Val3),
            4 => Some(Trigin::Val4),
            5 => Some(Trigin::Val5),
            6 => Some(Trigin::Val6),
            7 => Some(Trigin::Val7),
            8 => Some(Trigin::Val8),
            9 => Some(Trigin::Val9),
            10 => Some(Trigin::Val10),
            11 => Some(Trigin::Val11),
            12 => Some(Trigin::Val12),
            13 => Some(Trigin::Val13),
            14 => Some(Trigin::Val14),
            15 => Some(Trigin::Val15),
            16 => Some(Trigin::Val16),
            17 => Some(Trigin::Val17),
            18 => Some(Trigin::Val18),
            19 => Some(Trigin::Val19),
            20 => Some(Trigin::Val20),
            21 => Some(Trigin::Val21),
            22 => Some(Trigin::Val22),
            23 => Some(Trigin::Val23),
            24 => Some(Trigin::Val24),
            25 => Some(Trigin::Val25),
            26 => Some(Trigin::Val26),
            27 => Some(Trigin::Val27),
            28 => Some(Trigin::Val28),
            29 => Some(Trigin::Val29),
            30 => Some(Trigin::Val30),
            31 => Some(Trigin::Val31),
            32 => Some(Trigin::Val32),
            33 => Some(Trigin::Val33),
            34 => Some(Trigin::Val34),
            35 => Some(Trigin::Val35),
            36 => Some(Trigin::Val36),
            37 => Some(Trigin::Val37),
            38 => Some(Trigin::Val38),
            39 => Some(Trigin::Val39),
            40 => Some(Trigin::Val40),
            45 => Some(Trigin::Val45),
            46 => Some(Trigin::Val46),
            47 => Some(Trigin::Val47),
            48 => Some(Trigin::Val48),
            49 => Some(Trigin::Val49),
            50 => Some(Trigin::Val50),
            51 => Some(Trigin::Val51),
            52 => Some(Trigin::Val52),
            53 => Some(Trigin::Val53),
            54 => Some(Trigin::Val54),
            55 => Some(Trigin::Val55),
            56 => Some(Trigin::Val56),
            57 => Some(Trigin::Val57),
            58 => Some(Trigin::Val58),
            59 => Some(Trigin::Val59),
            60 => Some(Trigin::Val60),
            61 => Some(Trigin::Val61),
            _ => None,
        }
    }
    #[doc = "ARM_TXEV input is selected"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == Trigin::Val1
    }
    #[doc = "AOI0_OUT0 input is selected"]
    #[inline(always)]
    pub fn is_val2(&self) -> bool {
        *self == Trigin::Val2
    }
    #[doc = "AOI0_OUT1 input is selected"]
    #[inline(always)]
    pub fn is_val3(&self) -> bool {
        *self == Trigin::Val3
    }
    #[doc = "AOI0_OUT2 input is selected"]
    #[inline(always)]
    pub fn is_val4(&self) -> bool {
        *self == Trigin::Val4
    }
    #[doc = "AOI0_OUT3 input is selected"]
    #[inline(always)]
    pub fn is_val5(&self) -> bool {
        *self == Trigin::Val5
    }
    #[doc = "CMP0_OUT input is selected"]
    #[inline(always)]
    pub fn is_val6(&self) -> bool {
        *self == Trigin::Val6
    }
    #[doc = "CMP1_OUT input is selected"]
    #[inline(always)]
    pub fn is_val7(&self) -> bool {
        *self == Trigin::Val7
    }
    #[doc = "CMP2_OUT input is selected"]
    #[inline(always)]
    pub fn is_val8(&self) -> bool {
        *self == Trigin::Val8
    }
    #[doc = "CTimer0_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val9(&self) -> bool {
        *self == Trigin::Val9
    }
    #[doc = "CTimer0_MAT3 input is selected"]
    #[inline(always)]
    pub fn is_val10(&self) -> bool {
        *self == Trigin::Val10
    }
    #[doc = "CTimer1_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val11(&self) -> bool {
        *self == Trigin::Val11
    }
    #[doc = "CTimer1_MAT3 input is selected"]
    #[inline(always)]
    pub fn is_val12(&self) -> bool {
        *self == Trigin::Val12
    }
    #[doc = "CTimer2_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val13(&self) -> bool {
        *self == Trigin::Val13
    }
    #[doc = "CTimer2_MAT3 input is selected"]
    #[inline(always)]
    pub fn is_val14(&self) -> bool {
        *self == Trigin::Val14
    }
    #[doc = "QDC0_CMP_FLAG0 input is selected"]
    #[inline(always)]
    pub fn is_val15(&self) -> bool {
        *self == Trigin::Val15
    }
    #[doc = "QDC0_CMP_FLAG1 input is selected"]
    #[inline(always)]
    pub fn is_val16(&self) -> bool {
        *self == Trigin::Val16
    }
    #[doc = "QDC0_CMP_FLAG2 input is selected"]
    #[inline(always)]
    pub fn is_val17(&self) -> bool {
        *self == Trigin::Val17
    }
    #[doc = "QDC0_CMP_FLAG3 input is selected"]
    #[inline(always)]
    pub fn is_val18(&self) -> bool {
        *self == Trigin::Val18
    }
    #[doc = "QDC0_POS_MATCH0 input is selected"]
    #[inline(always)]
    pub fn is_val19(&self) -> bool {
        *self == Trigin::Val19
    }
    #[doc = "TRIG_IN0 input is selected"]
    #[inline(always)]
    pub fn is_val20(&self) -> bool {
        *self == Trigin::Val20
    }
    #[doc = "TRIG_IN1 input is selected"]
    #[inline(always)]
    pub fn is_val21(&self) -> bool {
        *self == Trigin::Val21
    }
    #[doc = "TRIG_IN2 input is selected"]
    #[inline(always)]
    pub fn is_val22(&self) -> bool {
        *self == Trigin::Val22
    }
    #[doc = "TRIG_IN3 input is selected"]
    #[inline(always)]
    pub fn is_val23(&self) -> bool {
        *self == Trigin::Val23
    }
    #[doc = "TRIG_IN4 input is selected"]
    #[inline(always)]
    pub fn is_val24(&self) -> bool {
        *self == Trigin::Val24
    }
    #[doc = "TRIG_IN5 input is selected"]
    #[inline(always)]
    pub fn is_val25(&self) -> bool {
        *self == Trigin::Val25
    }
    #[doc = "TRIG_IN6 input is selected"]
    #[inline(always)]
    pub fn is_val26(&self) -> bool {
        *self == Trigin::Val26
    }
    #[doc = "TRIG_IN7 input is selected"]
    #[inline(always)]
    pub fn is_val27(&self) -> bool {
        *self == Trigin::Val27
    }
    #[doc = "TRIG_IN8 input is selected"]
    #[inline(always)]
    pub fn is_val28(&self) -> bool {
        *self == Trigin::Val28
    }
    #[doc = "TRIG_IN9 input is selected"]
    #[inline(always)]
    pub fn is_val29(&self) -> bool {
        *self == Trigin::Val29
    }
    #[doc = "TRIG_IN10 input is selected"]
    #[inline(always)]
    pub fn is_val30(&self) -> bool {
        *self == Trigin::Val30
    }
    #[doc = "TRIG_IN11 input is selected"]
    #[inline(always)]
    pub fn is_val31(&self) -> bool {
        *self == Trigin::Val31
    }
    #[doc = "GPIO0 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn is_val32(&self) -> bool {
        *self == Trigin::Val32
    }
    #[doc = "GPIO1 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn is_val33(&self) -> bool {
        *self == Trigin::Val33
    }
    #[doc = "GPIO2 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn is_val34(&self) -> bool {
        *self == Trigin::Val34
    }
    #[doc = "GPIO3 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn is_val35(&self) -> bool {
        *self == Trigin::Val35
    }
    #[doc = "GPIO4 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn is_val36(&self) -> bool {
        *self == Trigin::Val36
    }
    #[doc = "AOI1_OUT0 input is selected"]
    #[inline(always)]
    pub fn is_val37(&self) -> bool {
        *self == Trigin::Val37
    }
    #[doc = "AOI1_OUT1 input is selected"]
    #[inline(always)]
    pub fn is_val38(&self) -> bool {
        *self == Trigin::Val38
    }
    #[doc = "AOI1_OUT2 input is selected"]
    #[inline(always)]
    pub fn is_val39(&self) -> bool {
        *self == Trigin::Val39
    }
    #[doc = "AOI1_OUT3 input is selected"]
    #[inline(always)]
    pub fn is_val40(&self) -> bool {
        *self == Trigin::Val40
    }
    #[doc = "CTimer3_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val45(&self) -> bool {
        *self == Trigin::Val45
    }
    #[doc = "CTimer3_MAT3 input is selected"]
    #[inline(always)]
    pub fn is_val46(&self) -> bool {
        *self == Trigin::Val46
    }
    #[doc = "CTimer4_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val47(&self) -> bool {
        *self == Trigin::Val47
    }
    #[doc = "CTimer4_MAT3 input is selected"]
    #[inline(always)]
    pub fn is_val48(&self) -> bool {
        *self == Trigin::Val48
    }
    #[doc = "QDC1_CMP_FLAG0 input is selected"]
    #[inline(always)]
    pub fn is_val49(&self) -> bool {
        *self == Trigin::Val49
    }
    #[doc = "QDC1_CMP_FLAG1 input is selected"]
    #[inline(always)]
    pub fn is_val50(&self) -> bool {
        *self == Trigin::Val50
    }
    #[doc = "QDC1_CMP_FLAG2 input is selected"]
    #[inline(always)]
    pub fn is_val51(&self) -> bool {
        *self == Trigin::Val51
    }
    #[doc = "QDC1_CMP_FLAG3 input is selected"]
    #[inline(always)]
    pub fn is_val52(&self) -> bool {
        *self == Trigin::Val52
    }
    #[doc = "QDC1_POS_MATCH0 input is selected"]
    #[inline(always)]
    pub fn is_val53(&self) -> bool {
        *self == Trigin::Val53
    }
    #[doc = "PWM1_SM0_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val54(&self) -> bool {
        *self == Trigin::Val54
    }
    #[doc = "PWM1_SM0_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn is_val55(&self) -> bool {
        *self == Trigin::Val55
    }
    #[doc = "PWM1_SM1_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val56(&self) -> bool {
        *self == Trigin::Val56
    }
    #[doc = "PWM1_SM1_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn is_val57(&self) -> bool {
        *self == Trigin::Val57
    }
    #[doc = "PWM1_SM2_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val58(&self) -> bool {
        *self == Trigin::Val58
    }
    #[doc = "PWM1_SM2_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn is_val59(&self) -> bool {
        *self == Trigin::Val59
    }
    #[doc = "PWM1_SM3_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val60(&self) -> bool {
        *self == Trigin::Val60
    }
    #[doc = "PWM1_SM3_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn is_val61(&self) -> bool {
        *self == Trigin::Val61
    }
}
#[doc = "Field `TRIGIN` writer - EXTA input connections for PWM0"]
pub type TriginW<'a, REG> = crate::FieldWriter<'a, REG, 6, Trigin>;
impl<'a, REG> TriginW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "ARM_TXEV input is selected"]
    #[inline(always)]
    pub fn val1(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val1)
    }
    #[doc = "AOI0_OUT0 input is selected"]
    #[inline(always)]
    pub fn val2(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val2)
    }
    #[doc = "AOI0_OUT1 input is selected"]
    #[inline(always)]
    pub fn val3(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val3)
    }
    #[doc = "AOI0_OUT2 input is selected"]
    #[inline(always)]
    pub fn val4(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val4)
    }
    #[doc = "AOI0_OUT3 input is selected"]
    #[inline(always)]
    pub fn val5(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val5)
    }
    #[doc = "CMP0_OUT input is selected"]
    #[inline(always)]
    pub fn val6(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val6)
    }
    #[doc = "CMP1_OUT input is selected"]
    #[inline(always)]
    pub fn val7(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val7)
    }
    #[doc = "CMP2_OUT input is selected"]
    #[inline(always)]
    pub fn val8(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val8)
    }
    #[doc = "CTimer0_MAT2 input is selected"]
    #[inline(always)]
    pub fn val9(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val9)
    }
    #[doc = "CTimer0_MAT3 input is selected"]
    #[inline(always)]
    pub fn val10(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val10)
    }
    #[doc = "CTimer1_MAT2 input is selected"]
    #[inline(always)]
    pub fn val11(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val11)
    }
    #[doc = "CTimer1_MAT3 input is selected"]
    #[inline(always)]
    pub fn val12(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val12)
    }
    #[doc = "CTimer2_MAT2 input is selected"]
    #[inline(always)]
    pub fn val13(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val13)
    }
    #[doc = "CTimer2_MAT3 input is selected"]
    #[inline(always)]
    pub fn val14(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val14)
    }
    #[doc = "QDC0_CMP_FLAG0 input is selected"]
    #[inline(always)]
    pub fn val15(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val15)
    }
    #[doc = "QDC0_CMP_FLAG1 input is selected"]
    #[inline(always)]
    pub fn val16(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val16)
    }
    #[doc = "QDC0_CMP_FLAG2 input is selected"]
    #[inline(always)]
    pub fn val17(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val17)
    }
    #[doc = "QDC0_CMP_FLAG3 input is selected"]
    #[inline(always)]
    pub fn val18(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val18)
    }
    #[doc = "QDC0_POS_MATCH0 input is selected"]
    #[inline(always)]
    pub fn val19(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val19)
    }
    #[doc = "TRIG_IN0 input is selected"]
    #[inline(always)]
    pub fn val20(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val20)
    }
    #[doc = "TRIG_IN1 input is selected"]
    #[inline(always)]
    pub fn val21(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val21)
    }
    #[doc = "TRIG_IN2 input is selected"]
    #[inline(always)]
    pub fn val22(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val22)
    }
    #[doc = "TRIG_IN3 input is selected"]
    #[inline(always)]
    pub fn val23(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val23)
    }
    #[doc = "TRIG_IN4 input is selected"]
    #[inline(always)]
    pub fn val24(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val24)
    }
    #[doc = "TRIG_IN5 input is selected"]
    #[inline(always)]
    pub fn val25(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val25)
    }
    #[doc = "TRIG_IN6 input is selected"]
    #[inline(always)]
    pub fn val26(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val26)
    }
    #[doc = "TRIG_IN7 input is selected"]
    #[inline(always)]
    pub fn val27(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val27)
    }
    #[doc = "TRIG_IN8 input is selected"]
    #[inline(always)]
    pub fn val28(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val28)
    }
    #[doc = "TRIG_IN9 input is selected"]
    #[inline(always)]
    pub fn val29(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val29)
    }
    #[doc = "TRIG_IN10 input is selected"]
    #[inline(always)]
    pub fn val30(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val30)
    }
    #[doc = "TRIG_IN11 input is selected"]
    #[inline(always)]
    pub fn val31(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val31)
    }
    #[doc = "GPIO0 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn val32(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val32)
    }
    #[doc = "GPIO1 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn val33(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val33)
    }
    #[doc = "GPIO2 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn val34(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val34)
    }
    #[doc = "GPIO3 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn val35(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val35)
    }
    #[doc = "GPIO4 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn val36(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val36)
    }
    #[doc = "AOI1_OUT0 input is selected"]
    #[inline(always)]
    pub fn val37(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val37)
    }
    #[doc = "AOI1_OUT1 input is selected"]
    #[inline(always)]
    pub fn val38(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val38)
    }
    #[doc = "AOI1_OUT2 input is selected"]
    #[inline(always)]
    pub fn val39(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val39)
    }
    #[doc = "AOI1_OUT3 input is selected"]
    #[inline(always)]
    pub fn val40(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val40)
    }
    #[doc = "CTimer3_MAT2 input is selected"]
    #[inline(always)]
    pub fn val45(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val45)
    }
    #[doc = "CTimer3_MAT3 input is selected"]
    #[inline(always)]
    pub fn val46(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val46)
    }
    #[doc = "CTimer4_MAT2 input is selected"]
    #[inline(always)]
    pub fn val47(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val47)
    }
    #[doc = "CTimer4_MAT3 input is selected"]
    #[inline(always)]
    pub fn val48(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val48)
    }
    #[doc = "QDC1_CMP_FLAG0 input is selected"]
    #[inline(always)]
    pub fn val49(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val49)
    }
    #[doc = "QDC1_CMP_FLAG1 input is selected"]
    #[inline(always)]
    pub fn val50(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val50)
    }
    #[doc = "QDC1_CMP_FLAG2 input is selected"]
    #[inline(always)]
    pub fn val51(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val51)
    }
    #[doc = "QDC1_CMP_FLAG3 input is selected"]
    #[inline(always)]
    pub fn val52(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val52)
    }
    #[doc = "QDC1_POS_MATCH0 input is selected"]
    #[inline(always)]
    pub fn val53(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val53)
    }
    #[doc = "PWM1_SM0_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val54(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val54)
    }
    #[doc = "PWM1_SM0_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn val55(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val55)
    }
    #[doc = "PWM1_SM1_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val56(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val56)
    }
    #[doc = "PWM1_SM1_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn val57(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val57)
    }
    #[doc = "PWM1_SM2_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val58(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val58)
    }
    #[doc = "PWM1_SM2_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn val59(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val59)
    }
    #[doc = "PWM1_SM3_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val60(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val60)
    }
    #[doc = "PWM1_SM3_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn val61(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val61)
    }
}
impl R {
    #[doc = "Bits 0:5 - EXTA input connections for PWM0"]
    #[inline(always)]
    pub fn trigin(&self) -> TriginR {
        TriginR::new((self.bits & 0x3f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:5 - EXTA input connections for PWM0"]
    #[inline(always)]
    pub fn trigin(&mut self) -> TriginW<FlexPwm0Sm0Exta0Spec> {
        TriginW::new(self, 0)
    }
}
#[doc = "PWM0 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm0_sm0_exta0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm0_sm0_exta0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FlexPwm0Sm0Exta0Spec;
impl crate::RegisterSpec for FlexPwm0Sm0Exta0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`flex_pwm0_sm0_exta0::R`](R) reader structure"]
impl crate::Readable for FlexPwm0Sm0Exta0Spec {}
#[doc = "`write(|w| ..)` method takes [`flex_pwm0_sm0_exta0::W`](W) writer structure"]
impl crate::Writable for FlexPwm0Sm0Exta0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FlexPWM0_SM0_EXTA0 to value 0x3f"]
impl crate::Resettable for FlexPwm0Sm0Exta0Spec {
    const RESET_VALUE: u32 = 0x3f;
}
