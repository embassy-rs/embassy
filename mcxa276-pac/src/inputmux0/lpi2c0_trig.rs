#[doc = "Register `LPI2C0_TRIG` reader"]
pub type R = crate::R<Lpi2c0TrigSpec>;
#[doc = "Register `LPI2C0_TRIG` writer"]
pub type W = crate::W<Lpi2c0TrigSpec>;
#[doc = "LPI2C0 trigger input connections\n\nValue on reset: 63"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Inp {
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
    #[doc = "9: CTimer0_MAT0 input is selected"]
    Val9 = 9,
    #[doc = "10: CTimer0_MAT1 input is selected"]
    Val10 = 10,
    #[doc = "11: CTimer1_MAT0 input is selected"]
    Val11 = 11,
    #[doc = "12: CTimer1_MAT1 input is selected"]
    Val12 = 12,
    #[doc = "13: CTimer2_MAT0 input is selected"]
    Val13 = 13,
    #[doc = "14: CTimer2_MAT1 input is selected"]
    Val14 = 14,
    #[doc = "15: LPTMR0 input is selected"]
    Val15 = 15,
    #[doc = "17: TRIG_IN0 input is selected"]
    Val17 = 17,
    #[doc = "18: TRIG_IN1 input is selected"]
    Val18 = 18,
    #[doc = "19: TRIG_IN2 input is selected"]
    Val19 = 19,
    #[doc = "20: TRIG_IN3 input is selected"]
    Val20 = 20,
    #[doc = "21: TRIG_IN4 input is selected"]
    Val21 = 21,
    #[doc = "22: TRIG_IN5 input is selected"]
    Val22 = 22,
    #[doc = "23: TRIG_IN6 input is selected"]
    Val23 = 23,
    #[doc = "24: TRIG_IN7 input is selected"]
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
    #[doc = "35: CTimer3_MAT2 input is selected"]
    Val35 = 35,
    #[doc = "36: CTimer3_MAT3 input is selected"]
    Val36 = 36,
    #[doc = "37: CTimer4_MAT2 input is selected"]
    Val37 = 37,
    #[doc = "38: CTimer4_MAT3 input is selected"]
    Val38 = 38,
    #[doc = "39: FlexIO CH0 input is selected"]
    Val39 = 39,
    #[doc = "40: FlexIO CH1 input is selected"]
    Val40 = 40,
    #[doc = "41: FlexIO CH2 input is selected"]
    Val41 = 41,
    #[doc = "42: FlexIO CH3 input is selected"]
    Val42 = 42,
}
impl From<Inp> for u8 {
    #[inline(always)]
    fn from(variant: Inp) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Inp {
    type Ux = u8;
}
impl crate::IsEnum for Inp {}
#[doc = "Field `INP` reader - LPI2C0 trigger input connections"]
pub type InpR = crate::FieldReader<Inp>;
impl InpR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Inp> {
        match self.bits {
            2 => Some(Inp::Val2),
            3 => Some(Inp::Val3),
            4 => Some(Inp::Val4),
            5 => Some(Inp::Val5),
            6 => Some(Inp::Val6),
            7 => Some(Inp::Val7),
            8 => Some(Inp::Val8),
            9 => Some(Inp::Val9),
            10 => Some(Inp::Val10),
            11 => Some(Inp::Val11),
            12 => Some(Inp::Val12),
            13 => Some(Inp::Val13),
            14 => Some(Inp::Val14),
            15 => Some(Inp::Val15),
            17 => Some(Inp::Val17),
            18 => Some(Inp::Val18),
            19 => Some(Inp::Val19),
            20 => Some(Inp::Val20),
            21 => Some(Inp::Val21),
            22 => Some(Inp::Val22),
            23 => Some(Inp::Val23),
            24 => Some(Inp::Val24),
            25 => Some(Inp::Val25),
            26 => Some(Inp::Val26),
            27 => Some(Inp::Val27),
            28 => Some(Inp::Val28),
            29 => Some(Inp::Val29),
            30 => Some(Inp::Val30),
            31 => Some(Inp::Val31),
            32 => Some(Inp::Val32),
            33 => Some(Inp::Val33),
            34 => Some(Inp::Val34),
            35 => Some(Inp::Val35),
            36 => Some(Inp::Val36),
            37 => Some(Inp::Val37),
            38 => Some(Inp::Val38),
            39 => Some(Inp::Val39),
            40 => Some(Inp::Val40),
            41 => Some(Inp::Val41),
            42 => Some(Inp::Val42),
            _ => None,
        }
    }
    #[doc = "AOI0_OUT0 input is selected"]
    #[inline(always)]
    pub fn is_val2(&self) -> bool {
        *self == Inp::Val2
    }
    #[doc = "AOI0_OUT1 input is selected"]
    #[inline(always)]
    pub fn is_val3(&self) -> bool {
        *self == Inp::Val3
    }
    #[doc = "AOI0_OUT2 input is selected"]
    #[inline(always)]
    pub fn is_val4(&self) -> bool {
        *self == Inp::Val4
    }
    #[doc = "AOI0_OUT3 input is selected"]
    #[inline(always)]
    pub fn is_val5(&self) -> bool {
        *self == Inp::Val5
    }
    #[doc = "CMP0_OUT input is selected"]
    #[inline(always)]
    pub fn is_val6(&self) -> bool {
        *self == Inp::Val6
    }
    #[doc = "CMP1_OUT input is selected"]
    #[inline(always)]
    pub fn is_val7(&self) -> bool {
        *self == Inp::Val7
    }
    #[doc = "CMP2_OUT input is selected"]
    #[inline(always)]
    pub fn is_val8(&self) -> bool {
        *self == Inp::Val8
    }
    #[doc = "CTimer0_MAT0 input is selected"]
    #[inline(always)]
    pub fn is_val9(&self) -> bool {
        *self == Inp::Val9
    }
    #[doc = "CTimer0_MAT1 input is selected"]
    #[inline(always)]
    pub fn is_val10(&self) -> bool {
        *self == Inp::Val10
    }
    #[doc = "CTimer1_MAT0 input is selected"]
    #[inline(always)]
    pub fn is_val11(&self) -> bool {
        *self == Inp::Val11
    }
    #[doc = "CTimer1_MAT1 input is selected"]
    #[inline(always)]
    pub fn is_val12(&self) -> bool {
        *self == Inp::Val12
    }
    #[doc = "CTimer2_MAT0 input is selected"]
    #[inline(always)]
    pub fn is_val13(&self) -> bool {
        *self == Inp::Val13
    }
    #[doc = "CTimer2_MAT1 input is selected"]
    #[inline(always)]
    pub fn is_val14(&self) -> bool {
        *self == Inp::Val14
    }
    #[doc = "LPTMR0 input is selected"]
    #[inline(always)]
    pub fn is_val15(&self) -> bool {
        *self == Inp::Val15
    }
    #[doc = "TRIG_IN0 input is selected"]
    #[inline(always)]
    pub fn is_val17(&self) -> bool {
        *self == Inp::Val17
    }
    #[doc = "TRIG_IN1 input is selected"]
    #[inline(always)]
    pub fn is_val18(&self) -> bool {
        *self == Inp::Val18
    }
    #[doc = "TRIG_IN2 input is selected"]
    #[inline(always)]
    pub fn is_val19(&self) -> bool {
        *self == Inp::Val19
    }
    #[doc = "TRIG_IN3 input is selected"]
    #[inline(always)]
    pub fn is_val20(&self) -> bool {
        *self == Inp::Val20
    }
    #[doc = "TRIG_IN4 input is selected"]
    #[inline(always)]
    pub fn is_val21(&self) -> bool {
        *self == Inp::Val21
    }
    #[doc = "TRIG_IN5 input is selected"]
    #[inline(always)]
    pub fn is_val22(&self) -> bool {
        *self == Inp::Val22
    }
    #[doc = "TRIG_IN6 input is selected"]
    #[inline(always)]
    pub fn is_val23(&self) -> bool {
        *self == Inp::Val23
    }
    #[doc = "TRIG_IN7 input is selected"]
    #[inline(always)]
    pub fn is_val24(&self) -> bool {
        *self == Inp::Val24
    }
    #[doc = "GPIO0 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn is_val25(&self) -> bool {
        *self == Inp::Val25
    }
    #[doc = "GPIO1 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn is_val26(&self) -> bool {
        *self == Inp::Val26
    }
    #[doc = "GPIO2 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn is_val27(&self) -> bool {
        *self == Inp::Val27
    }
    #[doc = "GPIO3 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn is_val28(&self) -> bool {
        *self == Inp::Val28
    }
    #[doc = "GPIO4 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn is_val29(&self) -> bool {
        *self == Inp::Val29
    }
    #[doc = "WUU input is selected"]
    #[inline(always)]
    pub fn is_val30(&self) -> bool {
        *self == Inp::Val30
    }
    #[doc = "AOI1_OUT0 input is selected"]
    #[inline(always)]
    pub fn is_val31(&self) -> bool {
        *self == Inp::Val31
    }
    #[doc = "AOI1_OUT1 input is selected"]
    #[inline(always)]
    pub fn is_val32(&self) -> bool {
        *self == Inp::Val32
    }
    #[doc = "AOI1_OUT2 input is selected"]
    #[inline(always)]
    pub fn is_val33(&self) -> bool {
        *self == Inp::Val33
    }
    #[doc = "AOI1_OUT3 input is selected"]
    #[inline(always)]
    pub fn is_val34(&self) -> bool {
        *self == Inp::Val34
    }
    #[doc = "CTimer3_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val35(&self) -> bool {
        *self == Inp::Val35
    }
    #[doc = "CTimer3_MAT3 input is selected"]
    #[inline(always)]
    pub fn is_val36(&self) -> bool {
        *self == Inp::Val36
    }
    #[doc = "CTimer4_MAT2 input is selected"]
    #[inline(always)]
    pub fn is_val37(&self) -> bool {
        *self == Inp::Val37
    }
    #[doc = "CTimer4_MAT3 input is selected"]
    #[inline(always)]
    pub fn is_val38(&self) -> bool {
        *self == Inp::Val38
    }
    #[doc = "FlexIO CH0 input is selected"]
    #[inline(always)]
    pub fn is_val39(&self) -> bool {
        *self == Inp::Val39
    }
    #[doc = "FlexIO CH1 input is selected"]
    #[inline(always)]
    pub fn is_val40(&self) -> bool {
        *self == Inp::Val40
    }
    #[doc = "FlexIO CH2 input is selected"]
    #[inline(always)]
    pub fn is_val41(&self) -> bool {
        *self == Inp::Val41
    }
    #[doc = "FlexIO CH3 input is selected"]
    #[inline(always)]
    pub fn is_val42(&self) -> bool {
        *self == Inp::Val42
    }
}
#[doc = "Field `INP` writer - LPI2C0 trigger input connections"]
pub type InpW<'a, REG> = crate::FieldWriter<'a, REG, 6, Inp>;
impl<'a, REG> InpW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "AOI0_OUT0 input is selected"]
    #[inline(always)]
    pub fn val2(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val2)
    }
    #[doc = "AOI0_OUT1 input is selected"]
    #[inline(always)]
    pub fn val3(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val3)
    }
    #[doc = "AOI0_OUT2 input is selected"]
    #[inline(always)]
    pub fn val4(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val4)
    }
    #[doc = "AOI0_OUT3 input is selected"]
    #[inline(always)]
    pub fn val5(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val5)
    }
    #[doc = "CMP0_OUT input is selected"]
    #[inline(always)]
    pub fn val6(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val6)
    }
    #[doc = "CMP1_OUT input is selected"]
    #[inline(always)]
    pub fn val7(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val7)
    }
    #[doc = "CMP2_OUT input is selected"]
    #[inline(always)]
    pub fn val8(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val8)
    }
    #[doc = "CTimer0_MAT0 input is selected"]
    #[inline(always)]
    pub fn val9(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val9)
    }
    #[doc = "CTimer0_MAT1 input is selected"]
    #[inline(always)]
    pub fn val10(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val10)
    }
    #[doc = "CTimer1_MAT0 input is selected"]
    #[inline(always)]
    pub fn val11(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val11)
    }
    #[doc = "CTimer1_MAT1 input is selected"]
    #[inline(always)]
    pub fn val12(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val12)
    }
    #[doc = "CTimer2_MAT0 input is selected"]
    #[inline(always)]
    pub fn val13(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val13)
    }
    #[doc = "CTimer2_MAT1 input is selected"]
    #[inline(always)]
    pub fn val14(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val14)
    }
    #[doc = "LPTMR0 input is selected"]
    #[inline(always)]
    pub fn val15(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val15)
    }
    #[doc = "TRIG_IN0 input is selected"]
    #[inline(always)]
    pub fn val17(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val17)
    }
    #[doc = "TRIG_IN1 input is selected"]
    #[inline(always)]
    pub fn val18(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val18)
    }
    #[doc = "TRIG_IN2 input is selected"]
    #[inline(always)]
    pub fn val19(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val19)
    }
    #[doc = "TRIG_IN3 input is selected"]
    #[inline(always)]
    pub fn val20(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val20)
    }
    #[doc = "TRIG_IN4 input is selected"]
    #[inline(always)]
    pub fn val21(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val21)
    }
    #[doc = "TRIG_IN5 input is selected"]
    #[inline(always)]
    pub fn val22(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val22)
    }
    #[doc = "TRIG_IN6 input is selected"]
    #[inline(always)]
    pub fn val23(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val23)
    }
    #[doc = "TRIG_IN7 input is selected"]
    #[inline(always)]
    pub fn val24(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val24)
    }
    #[doc = "GPIO0 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn val25(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val25)
    }
    #[doc = "GPIO1 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn val26(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val26)
    }
    #[doc = "GPIO2 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn val27(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val27)
    }
    #[doc = "GPIO3 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn val28(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val28)
    }
    #[doc = "GPIO4 Pin Event Trig 0 input is selected"]
    #[inline(always)]
    pub fn val29(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val29)
    }
    #[doc = "WUU input is selected"]
    #[inline(always)]
    pub fn val30(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val30)
    }
    #[doc = "AOI1_OUT0 input is selected"]
    #[inline(always)]
    pub fn val31(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val31)
    }
    #[doc = "AOI1_OUT1 input is selected"]
    #[inline(always)]
    pub fn val32(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val32)
    }
    #[doc = "AOI1_OUT2 input is selected"]
    #[inline(always)]
    pub fn val33(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val33)
    }
    #[doc = "AOI1_OUT3 input is selected"]
    #[inline(always)]
    pub fn val34(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val34)
    }
    #[doc = "CTimer3_MAT2 input is selected"]
    #[inline(always)]
    pub fn val35(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val35)
    }
    #[doc = "CTimer3_MAT3 input is selected"]
    #[inline(always)]
    pub fn val36(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val36)
    }
    #[doc = "CTimer4_MAT2 input is selected"]
    #[inline(always)]
    pub fn val37(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val37)
    }
    #[doc = "CTimer4_MAT3 input is selected"]
    #[inline(always)]
    pub fn val38(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val38)
    }
    #[doc = "FlexIO CH0 input is selected"]
    #[inline(always)]
    pub fn val39(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val39)
    }
    #[doc = "FlexIO CH1 input is selected"]
    #[inline(always)]
    pub fn val40(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val40)
    }
    #[doc = "FlexIO CH2 input is selected"]
    #[inline(always)]
    pub fn val41(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val41)
    }
    #[doc = "FlexIO CH3 input is selected"]
    #[inline(always)]
    pub fn val42(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val42)
    }
}
impl R {
    #[doc = "Bits 0:5 - LPI2C0 trigger input connections"]
    #[inline(always)]
    pub fn inp(&self) -> InpR {
        InpR::new((self.bits & 0x3f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:5 - LPI2C0 trigger input connections"]
    #[inline(always)]
    pub fn inp(&mut self) -> InpW<Lpi2c0TrigSpec> {
        InpW::new(self, 0)
    }
}
#[doc = "LPI2C0 trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`lpi2c0_trig::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpi2c0_trig::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Lpi2c0TrigSpec;
impl crate::RegisterSpec for Lpi2c0TrigSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`lpi2c0_trig::R`](R) reader structure"]
impl crate::Readable for Lpi2c0TrigSpec {}
#[doc = "`write(|w| ..)` method takes [`lpi2c0_trig::W`](W) writer structure"]
impl crate::Writable for Lpi2c0TrigSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LPI2C0_TRIG to value 0x3f"]
impl crate::Resettable for Lpi2c0TrigSpec {
    const RESET_VALUE: u32 = 0x3f;
}
