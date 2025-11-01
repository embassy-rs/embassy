#[doc = "Register `FREQMEAS_TAR` reader"]
pub type R = crate::R<FreqmeasTarSpec>;
#[doc = "Register `FREQMEAS_TAR` writer"]
pub type W = crate::W<FreqmeasTarSpec>;
#[doc = "Clock source number (binary value) for frequency measure function target clock.\n\nValue on reset: 63"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Inp {
    #[doc = "1: clk_in input is selected"]
    Val1 = 1,
    #[doc = "2: FRO_OSC_12M input is selected"]
    Val2 = 2,
    #[doc = "3: fro_hf_div input is selected"]
    Val3 = 3,
    #[doc = "5: clk_16k\\[1\\] input is selected"]
    Val5 = 5,
    #[doc = "6: SLOW_CLK input is selected"]
    Val6 = 6,
    #[doc = "7: FREQME_CLK_IN0 input is selected"]
    Val7 = 7,
    #[doc = "8: FREQME_CLK_IN1 input is selected input is selected"]
    Val8 = 8,
    #[doc = "9: AOI0_OUT0 input is selected"]
    Val9 = 9,
    #[doc = "10: AOI0_OUT1"]
    Val10 = 10,
    #[doc = "11: PWM0_SM0_MUX_TRIG0"]
    Val11 = 11,
    #[doc = "12: PWM0_SM0_MUX_TRIG1"]
    Val12 = 12,
    #[doc = "13: PWM0_SM1_MUX_TRIG0"]
    Val13 = 13,
    #[doc = "14: PWM0_SM1_MUX_TRIG1"]
    Val14 = 14,
    #[doc = "15: PWM0_SM2_MUX_TRIG0"]
    Val15 = 15,
    #[doc = "16: PWM0_SM2_MUX_TRIG1"]
    Val16 = 16,
    #[doc = "17: PWM0_SM3_MUX_TRIG0"]
    Val17 = 17,
    #[doc = "18: PWM0_SM3_MUX_TRIG1"]
    Val18 = 18,
    #[doc = "32: AOI1_OUT0 input is selected"]
    Val32 = 32,
    #[doc = "33: AOI1_OUT1 input is selected"]
    Val33 = 33,
    #[doc = "34: PWM1_SM0_MUX_TRIG0 input is selected"]
    Val34 = 34,
    #[doc = "35: PWM1_SM0_MUX_TRIG1 input is selected"]
    Val35 = 35,
    #[doc = "36: PWM1_SM1_MUX_TRIG0 input is selected"]
    Val36 = 36,
    #[doc = "37: PWM1_SM1_MUX_TRIG1 input is selected"]
    Val37 = 37,
    #[doc = "38: PWM1_SM2_MUX_TRIG0 input is selected"]
    Val38 = 38,
    #[doc = "39: PWM1_SM2_MUX_TRIG1 input is selected"]
    Val39 = 39,
    #[doc = "40: PWM1_SM3_MUX_TRIG0 input is selected"]
    Val40 = 40,
    #[doc = "41: PWM1_SM3_MUX_TRIG1 input is selected"]
    Val41 = 41,
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
#[doc = "Field `INP` reader - Clock source number (binary value) for frequency measure function target clock."]
pub type InpR = crate::FieldReader<Inp>;
impl InpR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Inp> {
        match self.bits {
            1 => Some(Inp::Val1),
            2 => Some(Inp::Val2),
            3 => Some(Inp::Val3),
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
            16 => Some(Inp::Val16),
            17 => Some(Inp::Val17),
            18 => Some(Inp::Val18),
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
            _ => None,
        }
    }
    #[doc = "clk_in input is selected"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == Inp::Val1
    }
    #[doc = "FRO_OSC_12M input is selected"]
    #[inline(always)]
    pub fn is_val2(&self) -> bool {
        *self == Inp::Val2
    }
    #[doc = "fro_hf_div input is selected"]
    #[inline(always)]
    pub fn is_val3(&self) -> bool {
        *self == Inp::Val3
    }
    #[doc = "clk_16k\\[1\\] input is selected"]
    #[inline(always)]
    pub fn is_val5(&self) -> bool {
        *self == Inp::Val5
    }
    #[doc = "SLOW_CLK input is selected"]
    #[inline(always)]
    pub fn is_val6(&self) -> bool {
        *self == Inp::Val6
    }
    #[doc = "FREQME_CLK_IN0 input is selected"]
    #[inline(always)]
    pub fn is_val7(&self) -> bool {
        *self == Inp::Val7
    }
    #[doc = "FREQME_CLK_IN1 input is selected input is selected"]
    #[inline(always)]
    pub fn is_val8(&self) -> bool {
        *self == Inp::Val8
    }
    #[doc = "AOI0_OUT0 input is selected"]
    #[inline(always)]
    pub fn is_val9(&self) -> bool {
        *self == Inp::Val9
    }
    #[doc = "AOI0_OUT1"]
    #[inline(always)]
    pub fn is_val10(&self) -> bool {
        *self == Inp::Val10
    }
    #[doc = "PWM0_SM0_MUX_TRIG0"]
    #[inline(always)]
    pub fn is_val11(&self) -> bool {
        *self == Inp::Val11
    }
    #[doc = "PWM0_SM0_MUX_TRIG1"]
    #[inline(always)]
    pub fn is_val12(&self) -> bool {
        *self == Inp::Val12
    }
    #[doc = "PWM0_SM1_MUX_TRIG0"]
    #[inline(always)]
    pub fn is_val13(&self) -> bool {
        *self == Inp::Val13
    }
    #[doc = "PWM0_SM1_MUX_TRIG1"]
    #[inline(always)]
    pub fn is_val14(&self) -> bool {
        *self == Inp::Val14
    }
    #[doc = "PWM0_SM2_MUX_TRIG0"]
    #[inline(always)]
    pub fn is_val15(&self) -> bool {
        *self == Inp::Val15
    }
    #[doc = "PWM0_SM2_MUX_TRIG1"]
    #[inline(always)]
    pub fn is_val16(&self) -> bool {
        *self == Inp::Val16
    }
    #[doc = "PWM0_SM3_MUX_TRIG0"]
    #[inline(always)]
    pub fn is_val17(&self) -> bool {
        *self == Inp::Val17
    }
    #[doc = "PWM0_SM3_MUX_TRIG1"]
    #[inline(always)]
    pub fn is_val18(&self) -> bool {
        *self == Inp::Val18
    }
    #[doc = "AOI1_OUT0 input is selected"]
    #[inline(always)]
    pub fn is_val32(&self) -> bool {
        *self == Inp::Val32
    }
    #[doc = "AOI1_OUT1 input is selected"]
    #[inline(always)]
    pub fn is_val33(&self) -> bool {
        *self == Inp::Val33
    }
    #[doc = "PWM1_SM0_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val34(&self) -> bool {
        *self == Inp::Val34
    }
    #[doc = "PWM1_SM0_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn is_val35(&self) -> bool {
        *self == Inp::Val35
    }
    #[doc = "PWM1_SM1_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val36(&self) -> bool {
        *self == Inp::Val36
    }
    #[doc = "PWM1_SM1_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn is_val37(&self) -> bool {
        *self == Inp::Val37
    }
    #[doc = "PWM1_SM2_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val38(&self) -> bool {
        *self == Inp::Val38
    }
    #[doc = "PWM1_SM2_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn is_val39(&self) -> bool {
        *self == Inp::Val39
    }
    #[doc = "PWM1_SM3_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn is_val40(&self) -> bool {
        *self == Inp::Val40
    }
    #[doc = "PWM1_SM3_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn is_val41(&self) -> bool {
        *self == Inp::Val41
    }
}
#[doc = "Field `INP` writer - Clock source number (binary value) for frequency measure function target clock."]
pub type InpW<'a, REG> = crate::FieldWriter<'a, REG, 7, Inp>;
impl<'a, REG> InpW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "clk_in input is selected"]
    #[inline(always)]
    pub fn val1(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val1)
    }
    #[doc = "FRO_OSC_12M input is selected"]
    #[inline(always)]
    pub fn val2(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val2)
    }
    #[doc = "fro_hf_div input is selected"]
    #[inline(always)]
    pub fn val3(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val3)
    }
    #[doc = "clk_16k\\[1\\] input is selected"]
    #[inline(always)]
    pub fn val5(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val5)
    }
    #[doc = "SLOW_CLK input is selected"]
    #[inline(always)]
    pub fn val6(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val6)
    }
    #[doc = "FREQME_CLK_IN0 input is selected"]
    #[inline(always)]
    pub fn val7(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val7)
    }
    #[doc = "FREQME_CLK_IN1 input is selected input is selected"]
    #[inline(always)]
    pub fn val8(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val8)
    }
    #[doc = "AOI0_OUT0 input is selected"]
    #[inline(always)]
    pub fn val9(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val9)
    }
    #[doc = "AOI0_OUT1"]
    #[inline(always)]
    pub fn val10(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val10)
    }
    #[doc = "PWM0_SM0_MUX_TRIG0"]
    #[inline(always)]
    pub fn val11(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val11)
    }
    #[doc = "PWM0_SM0_MUX_TRIG1"]
    #[inline(always)]
    pub fn val12(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val12)
    }
    #[doc = "PWM0_SM1_MUX_TRIG0"]
    #[inline(always)]
    pub fn val13(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val13)
    }
    #[doc = "PWM0_SM1_MUX_TRIG1"]
    #[inline(always)]
    pub fn val14(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val14)
    }
    #[doc = "PWM0_SM2_MUX_TRIG0"]
    #[inline(always)]
    pub fn val15(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val15)
    }
    #[doc = "PWM0_SM2_MUX_TRIG1"]
    #[inline(always)]
    pub fn val16(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val16)
    }
    #[doc = "PWM0_SM3_MUX_TRIG0"]
    #[inline(always)]
    pub fn val17(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val17)
    }
    #[doc = "PWM0_SM3_MUX_TRIG1"]
    #[inline(always)]
    pub fn val18(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val18)
    }
    #[doc = "AOI1_OUT0 input is selected"]
    #[inline(always)]
    pub fn val32(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val32)
    }
    #[doc = "AOI1_OUT1 input is selected"]
    #[inline(always)]
    pub fn val33(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val33)
    }
    #[doc = "PWM1_SM0_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val34(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val34)
    }
    #[doc = "PWM1_SM0_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn val35(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val35)
    }
    #[doc = "PWM1_SM1_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val36(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val36)
    }
    #[doc = "PWM1_SM1_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn val37(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val37)
    }
    #[doc = "PWM1_SM2_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val38(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val38)
    }
    #[doc = "PWM1_SM2_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn val39(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val39)
    }
    #[doc = "PWM1_SM3_MUX_TRIG0 input is selected"]
    #[inline(always)]
    pub fn val40(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val40)
    }
    #[doc = "PWM1_SM3_MUX_TRIG1 input is selected"]
    #[inline(always)]
    pub fn val41(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val41)
    }
}
impl R {
    #[doc = "Bits 0:6 - Clock source number (binary value) for frequency measure function target clock."]
    #[inline(always)]
    pub fn inp(&self) -> InpR {
        InpR::new((self.bits & 0x7f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:6 - Clock source number (binary value) for frequency measure function target clock."]
    #[inline(always)]
    pub fn inp(&mut self) -> InpW<FreqmeasTarSpec> {
        InpW::new(self, 0)
    }
}
#[doc = "Selection for frequency measurement reference clock\n\nYou can [`read`](crate::Reg::read) this register and get [`freqmeas_tar::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`freqmeas_tar::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FreqmeasTarSpec;
impl crate::RegisterSpec for FreqmeasTarSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`freqmeas_tar::R`](R) reader structure"]
impl crate::Readable for FreqmeasTarSpec {}
#[doc = "`write(|w| ..)` method takes [`freqmeas_tar::W`](W) writer structure"]
impl crate::Writable for FreqmeasTarSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FREQMEAS_TAR to value 0x3f"]
impl crate::Resettable for FreqmeasTarSpec {
    const RESET_VALUE: u32 = 0x3f;
}
