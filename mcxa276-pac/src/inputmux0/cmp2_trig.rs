#[doc = "Register `CMP2_TRIG` reader"]
pub type R = crate::R<Cmp2TrigSpec>;
#[doc = "Register `CMP2_TRIG` writer"]
pub type W = crate::W<Cmp2TrigSpec>;
#[doc = "CMP2 input trigger\n\nValue on reset: 63"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Trigin {
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
    #[doc = "8: CTimer0_MAT0 input is selected"]
    Val8 = 8,
    #[doc = "9: CTimer0_MAT2 input is selected"]
    Val9 = 9,
    #[doc = "10: CTimer1_MAT0"]
    Val10 = 10,
    #[doc = "11: CTimer1_MAT2 input is selected"]
    Val11 = 11,
    #[doc = "12: CTimer2_MAT0 input is selected"]
    Val12 = 12,
    #[doc = "13: CTimer2_MAT2 input is selected"]
    Val13 = 13,
    #[doc = "14: LPTMR0 input is selected"]
    Val14 = 14,
    #[doc = "16: QDC0_POS_MATCH0 input is selected"]
    Val16 = 16,
    #[doc = "17: PWM0_SM0_MUX_TRIG0 input is selected"]
    Val17 = 17,
    #[doc = "18: PWM0_SM0_MUX_TRIG1 input is selected"]
    Val18 = 18,
    #[doc = "19: PWM0_SM1_MUX_TRIG0 input is selected"]
    Val19 = 19,
    #[doc = "20: PWM0_SM1_MUX_TRIG1 input is selected"]
    Val20 = 20,
    #[doc = "21: PWM0_SM2_MUX_TRIG0 input is selected"]
    Val21 = 21,
    #[doc = "22: PWM0_SM2_MUX_TRIG1 input is selected"]
    Val22 = 22,
    #[doc = "23: PWM0_SM3_MUX_TRIG0 input is selected"]
    Val23 = 23,
    #[doc = "24: PWM0_SM3_MUX_TRIG1 input is selected"]
    Val24 = 24,
    #[doc = "25: GPIO0 Pin Event Trig 0 input is selected"]
    Val25 = 25,
    #[doc = "26: GPIO1 Pin Event Trig 0 input is selected"]
    Val26 = 26,
    #[doc = "27: GPIO2 Pin Event Trig 0 input is selected"]
    Val27 = 27,
    #[doc = "28: GPIO3 Pin Event Trig 0 input is selected"]
    Val28 = 28,
    #[doc = "29: GPIO4 Pin Event Trig 0 input is selected"]
    Val29 = 29,
    #[doc = "30: WUU input is selected"]
    Val30 = 30,
    #[doc = "31: AOI1_OUT0 input is selected"]
    Val31 = 31,
    #[doc = "32: AOI1_OUT1 input is selected"]
    Val32 = 32,
    #[doc = "33: AOI1_OUT2 input is selected"]
    Val33 = 33,
    #[doc = "34: AOI1_OUT3 input is selected"]
    Val34 = 34,
    #[doc = "39: CTimer3_MAT0"]
    Val39 = 39,
    #[doc = "40: CTimer3_MAT1"]
    Val40 = 40,
    #[doc = "41: CTimer4_MAT0 input is selected"]
    Val41 = 41,
    #[doc = "42: CTimer4_MAT1 input is selected"]
    Val42 = 42,
    #[doc = "47: QDC1_POS_MATCH0 input is selected"]
    Val47 = 47,
    #[doc = "48: PWM1_SM0_MUX_TRIG0 input is selected"]
    Val48 = 48,
    #[doc = "49: PWM1_SM0_MUX_TRIG1 input is selected"]
    Val49 = 49,
    #[doc = "50: PWM1_SM1_MUX_TRIG0 input is selected"]
    Val50 = 50,
    #[doc = "51: PWM1_SM1_MUX_TRIG1 input is selected"]
    Val51 = 51,
    #[doc = "52: PWM1_SM2_MUX_TRIG0 input is selected"]
    Val52 = 52,
    #[doc = "53: PWM1_SM2_MUX_TRIG1 input is selected"]
    Val53 = 53,
    #[doc = "54: PWM1_SM3_MUX_TRIG0 input is selected"]
    Val54 = 54,
    #[doc = "55: PWM1_SM2_MUX_TRIG1 input is selected"]
    Val55 = 55,
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
#[doc = "Field `TRIGIN` reader - CMP2 input trigger"]
pub type TriginR = crate::FieldReader<Trigin>;
impl TriginR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Trigin> {
        match self.bits {
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
            39 => Some(Trigin::Val39),
            40 => Some(Trigin::Val40),
            41 => Some(Trigin::Val41),
            42 => Some(Trigin::Val42),
            47 => Some(Trigin::Val47),
            48 => Some(Trigin::Val48),
            49 => Some(Trigin::Val49),
            50 => Some(Trigin::Val50),
            51 => Some(Trigin::Val51),
            52 => Some(Trigin::Val52),
            53 => Some(Trigin::Val53),
            54 => Some(Trigin::Val54),
            55 => Some(Trigin::Val55),
            _ => None,
        }
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
    #[doc = "CTimer0_MAT0 input is selected"]
    #[inline(always)]
    pub fn is_val8(&self) -> bool {
        *self == Trigin::Val8
    }
    #[doc = "CTimer0_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val9(&self) -> bool {
        *self == Trigin::Val9
    }
    #[doc = "CTimer1_MAT0"]
    #[inline(always)]
    pub fn is_val10(&self) -> bool {
        *self == Trigin::Val10
    }
    #[doc = "CTimer1_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val11(&self) -> bool {
        *self == Trigin::Val11
    }
    #[doc = "CTimer2_MAT0 input is selected"]
    #[inline(always)]
    pub fn is_val12(&self) -> bool {
        *self == Trigin::Val12
    }
    #[doc = "CTimer2_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val13(&self) -> bool {
        *self == Trigin::Val13
    }
    #[doc = "LPTMR0 input is selected"]
    #[inline(always)]
    pub fn is_val14(&self) -> bool {
        *self == Trigin::Val14
    }
    #[doc = "QDC0_POS_MATCH0 input is selected"]
    #[inline(always)]
    pub fn is_val16(&self) -> bool {
        *self == Trigin::Val16
    }
    #[doc = "PWM0_SM0_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val17(&self) -> bool {
        *self == Trigin::Val17
    }
    #[doc = "PWM0_SM0_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn is_val18(&self) -> bool {
        *self == Trigin::Val18
    }
    #[doc = "PWM0_SM1_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val19(&self) -> bool {
        *self == Trigin::Val19
    }
    #[doc = "PWM0_SM1_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn is_val20(&self) -> bool {
        *self == Trigin::Val20
    }
    #[doc = "PWM0_SM2_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val21(&self) -> bool {
        *self == Trigin::Val21
    }
    #[doc = "PWM0_SM2_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn is_val22(&self) -> bool {
        *self == Trigin::Val22
    }
    #[doc = "PWM0_SM3_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val23(&self) -> bool {
        *self == Trigin::Val23
    }
    #[doc = "PWM0_SM3_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn is_val24(&self) -> bool {
        *self == Trigin::Val24
    }
    #[doc = "GPIO0 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn is_val25(&self) -> bool {
        *self == Trigin::Val25
    }
    #[doc = "GPIO1 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn is_val26(&self) -> bool {
        *self == Trigin::Val26
    }
    #[doc = "GPIO2 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn is_val27(&self) -> bool {
        *self == Trigin::Val27
    }
    #[doc = "GPIO3 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn is_val28(&self) -> bool {
        *self == Trigin::Val28
    }
    #[doc = "GPIO4 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn is_val29(&self) -> bool {
        *self == Trigin::Val29
    }
    #[doc = "WUU input is selected"]
    #[inline(always)]
    pub fn is_val30(&self) -> bool {
        *self == Trigin::Val30
    }
    #[doc = "AOI1_OUT0 input is selected"]
    #[inline(always)]
    pub fn is_val31(&self) -> bool {
        *self == Trigin::Val31
    }
    #[doc = "AOI1_OUT1 input is selected"]
    #[inline(always)]
    pub fn is_val32(&self) -> bool {
        *self == Trigin::Val32
    }
    #[doc = "AOI1_OUT2 input is selected"]
    #[inline(always)]
    pub fn is_val33(&self) -> bool {
        *self == Trigin::Val33
    }
    #[doc = "AOI1_OUT3 input is selected"]
    #[inline(always)]
    pub fn is_val34(&self) -> bool {
        *self == Trigin::Val34
    }
    #[doc = "CTimer3_MAT0"]
    #[inline(always)]
    pub fn is_val39(&self) -> bool {
        *self == Trigin::Val39
    }
    #[doc = "CTimer3_MAT1"]
    #[inline(always)]
    pub fn is_val40(&self) -> bool {
        *self == Trigin::Val40
    }
    #[doc = "CTimer4_MAT0 input is selected"]
    #[inline(always)]
    pub fn is_val41(&self) -> bool {
        *self == Trigin::Val41
    }
    #[doc = "CTimer4_MAT1 input is selected"]
    #[inline(always)]
    pub fn is_val42(&self) -> bool {
        *self == Trigin::Val42
    }
    #[doc = "QDC1_POS_MATCH0 input is selected"]
    #[inline(always)]
    pub fn is_val47(&self) -> bool {
        *self == Trigin::Val47
    }
    #[doc = "PWM1_SM0_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val48(&self) -> bool {
        *self == Trigin::Val48
    }
    #[doc = "PWM1_SM0_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn is_val49(&self) -> bool {
        *self == Trigin::Val49
    }
    #[doc = "PWM1_SM1_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val50(&self) -> bool {
        *self == Trigin::Val50
    }
    #[doc = "PWM1_SM1_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn is_val51(&self) -> bool {
        *self == Trigin::Val51
    }
    #[doc = "PWM1_SM2_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val52(&self) -> bool {
        *self == Trigin::Val52
    }
    #[doc = "PWM1_SM2_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn is_val53(&self) -> bool {
        *self == Trigin::Val53
    }
    #[doc = "PWM1_SM3_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val54(&self) -> bool {
        *self == Trigin::Val54
    }
    #[doc = "PWM1_SM2_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn is_val55(&self) -> bool {
        *self == Trigin::Val55
    }
}
#[doc = "Field `TRIGIN` writer - CMP2 input trigger"]
pub type TriginW<'a, REG> = crate::FieldWriter<'a, REG, 6, Trigin>;
impl<'a, REG> TriginW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
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
    #[doc = "CTimer0_MAT0 input is selected"]
    #[inline(always)]
    pub fn val8(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val8)
    }
    #[doc = "CTimer0_MAT2 input is selected"]
    #[inline(always)]
    pub fn val9(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val9)
    }
    #[doc = "CTimer1_MAT0"]
    #[inline(always)]
    pub fn val10(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val10)
    }
    #[doc = "CTimer1_MAT2 input is selected"]
    #[inline(always)]
    pub fn val11(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val11)
    }
    #[doc = "CTimer2_MAT0 input is selected"]
    #[inline(always)]
    pub fn val12(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val12)
    }
    #[doc = "CTimer2_MAT2 input is selected"]
    #[inline(always)]
    pub fn val13(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val13)
    }
    #[doc = "LPTMR0 input is selected"]
    #[inline(always)]
    pub fn val14(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val14)
    }
    #[doc = "QDC0_POS_MATCH0 input is selected"]
    #[inline(always)]
    pub fn val16(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val16)
    }
    #[doc = "PWM0_SM0_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val17(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val17)
    }
    #[doc = "PWM0_SM0_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn val18(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val18)
    }
    #[doc = "PWM0_SM1_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val19(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val19)
    }
    #[doc = "PWM0_SM1_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn val20(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val20)
    }
    #[doc = "PWM0_SM2_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val21(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val21)
    }
    #[doc = "PWM0_SM2_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn val22(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val22)
    }
    #[doc = "PWM0_SM3_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val23(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val23)
    }
    #[doc = "PWM0_SM3_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn val24(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val24)
    }
    #[doc = "GPIO0 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn val25(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val25)
    }
    #[doc = "GPIO1 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn val26(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val26)
    }
    #[doc = "GPIO2 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn val27(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val27)
    }
    #[doc = "GPIO3 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn val28(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val28)
    }
    #[doc = "GPIO4 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn val29(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val29)
    }
    #[doc = "WUU input is selected"]
    #[inline(always)]
    pub fn val30(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val30)
    }
    #[doc = "AOI1_OUT0 input is selected"]
    #[inline(always)]
    pub fn val31(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val31)
    }
    #[doc = "AOI1_OUT1 input is selected"]
    #[inline(always)]
    pub fn val32(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val32)
    }
    #[doc = "AOI1_OUT2 input is selected"]
    #[inline(always)]
    pub fn val33(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val33)
    }
    #[doc = "AOI1_OUT3 input is selected"]
    #[inline(always)]
    pub fn val34(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val34)
    }
    #[doc = "CTimer3_MAT0"]
    #[inline(always)]
    pub fn val39(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val39)
    }
    #[doc = "CTimer3_MAT1"]
    #[inline(always)]
    pub fn val40(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val40)
    }
    #[doc = "CTimer4_MAT0 input is selected"]
    #[inline(always)]
    pub fn val41(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val41)
    }
    #[doc = "CTimer4_MAT1 input is selected"]
    #[inline(always)]
    pub fn val42(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val42)
    }
    #[doc = "QDC1_POS_MATCH0 input is selected"]
    #[inline(always)]
    pub fn val47(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val47)
    }
    #[doc = "PWM1_SM0_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val48(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val48)
    }
    #[doc = "PWM1_SM0_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn val49(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val49)
    }
    #[doc = "PWM1_SM1_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val50(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val50)
    }
    #[doc = "PWM1_SM1_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn val51(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val51)
    }
    #[doc = "PWM1_SM2_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val52(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val52)
    }
    #[doc = "PWM1_SM2_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn val53(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val53)
    }
    #[doc = "PWM1_SM3_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val54(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val54)
    }
    #[doc = "PWM1_SM2_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn val55(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val55)
    }
}
impl R {
    #[doc = "Bits 0:5 - CMP2 input trigger"]
    #[inline(always)]
    pub fn trigin(&self) -> TriginR {
        TriginR::new((self.bits & 0x3f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:5 - CMP2 input trigger"]
    #[inline(always)]
    pub fn trigin(&mut self) -> TriginW<Cmp2TrigSpec> {
        TriginW::new(self, 0)
    }
}
#[doc = "CMP2 input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`cmp2_trig::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmp2_trig::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Cmp2TrigSpec;
impl crate::RegisterSpec for Cmp2TrigSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cmp2_trig::R`](R) reader structure"]
impl crate::Readable for Cmp2TrigSpec {}
#[doc = "`write(|w| ..)` method takes [`cmp2_trig::W`](W) writer structure"]
impl crate::Writable for Cmp2TrigSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CMP2_TRIG to value 0x3f"]
impl crate::Resettable for Cmp2TrigSpec {
    const RESET_VALUE: u32 = 0x3f;
}
