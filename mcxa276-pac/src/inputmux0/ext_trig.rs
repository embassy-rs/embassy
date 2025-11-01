#[doc = "Register `EXT_TRIG[%s]` reader"]
pub type R = crate::R<ExtTrigSpec>;
#[doc = "Register `EXT_TRIG[%s]` writer"]
pub type W = crate::W<ExtTrigSpec>;
#[doc = "EXT trigger input connections\n\nValue on reset: 31"]
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
    #[doc = "9: LPUART0 ipp_do_lpuart_txd input is selected"]
    Val9 = 9,
    #[doc = "10: LPUART1 ipp_do_lpuart_txd input is selected"]
    Val10 = 10,
    #[doc = "11: LPUART2 ipp_do_lpuart_txd input is selected"]
    Val11 = 11,
    #[doc = "12: LPUART3 ipp_do_lpuart_txd input is selected"]
    Val12 = 12,
    #[doc = "13: LPUART4 ipp_do_lpuart_txd input is selected"]
    Val13 = 13,
    #[doc = "14: AOI1_OUT0 input is selected"]
    Val14 = 14,
    #[doc = "15: AOI1_OUT1 input is selected"]
    Val15 = 15,
    #[doc = "16: AOI1_OUT2 input is selected"]
    Val16 = 16,
    #[doc = "17: RTC_1Hz_CLK input is selected"]
    Val17 = 17,
    #[doc = "18: LPUART5 ipp_do_lpuart_txd input is selected"]
    Val18 = 18,
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
#[doc = "Field `INP` reader - EXT trigger input connections"]
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
            16 => Some(Inp::Val16),
            17 => Some(Inp::Val17),
            18 => Some(Inp::Val18),
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
    #[doc = "LPUART0 ipp_do_lpuart_txd input is selected"]
    #[inline(always)]
    pub fn is_val9(&self) -> bool {
        *self == Inp::Val9
    }
    #[doc = "LPUART1 ipp_do_lpuart_txd input is selected"]
    #[inline(always)]
    pub fn is_val10(&self) -> bool {
        *self == Inp::Val10
    }
    #[doc = "LPUART2 ipp_do_lpuart_txd input is selected"]
    #[inline(always)]
    pub fn is_val11(&self) -> bool {
        *self == Inp::Val11
    }
    #[doc = "LPUART3 ipp_do_lpuart_txd input is selected"]
    #[inline(always)]
    pub fn is_val12(&self) -> bool {
        *self == Inp::Val12
    }
    #[doc = "LPUART4 ipp_do_lpuart_txd input is selected"]
    #[inline(always)]
    pub fn is_val13(&self) -> bool {
        *self == Inp::Val13
    }
    #[doc = "AOI1_OUT0 input is selected"]
    #[inline(always)]
    pub fn is_val14(&self) -> bool {
        *self == Inp::Val14
    }
    #[doc = "AOI1_OUT1 input is selected"]
    #[inline(always)]
    pub fn is_val15(&self) -> bool {
        *self == Inp::Val15
    }
    #[doc = "AOI1_OUT2 input is selected"]
    #[inline(always)]
    pub fn is_val16(&self) -> bool {
        *self == Inp::Val16
    }
    #[doc = "RTC_1Hz_CLK input is selected"]
    #[inline(always)]
    pub fn is_val17(&self) -> bool {
        *self == Inp::Val17
    }
    #[doc = "LPUART5 ipp_do_lpuart_txd input is selected"]
    #[inline(always)]
    pub fn is_val18(&self) -> bool {
        *self == Inp::Val18
    }
}
#[doc = "Field `INP` writer - EXT trigger input connections"]
pub type InpW<'a, REG> = crate::FieldWriter<'a, REG, 5, Inp>;
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
    #[doc = "LPUART0 ipp_do_lpuart_txd input is selected"]
    #[inline(always)]
    pub fn val9(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val9)
    }
    #[doc = "LPUART1 ipp_do_lpuart_txd input is selected"]
    #[inline(always)]
    pub fn val10(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val10)
    }
    #[doc = "LPUART2 ipp_do_lpuart_txd input is selected"]
    #[inline(always)]
    pub fn val11(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val11)
    }
    #[doc = "LPUART3 ipp_do_lpuart_txd input is selected"]
    #[inline(always)]
    pub fn val12(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val12)
    }
    #[doc = "LPUART4 ipp_do_lpuart_txd input is selected"]
    #[inline(always)]
    pub fn val13(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val13)
    }
    #[doc = "AOI1_OUT0 input is selected"]
    #[inline(always)]
    pub fn val14(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val14)
    }
    #[doc = "AOI1_OUT1 input is selected"]
    #[inline(always)]
    pub fn val15(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val15)
    }
    #[doc = "AOI1_OUT2 input is selected"]
    #[inline(always)]
    pub fn val16(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val16)
    }
    #[doc = "RTC_1Hz_CLK input is selected"]
    #[inline(always)]
    pub fn val17(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val17)
    }
    #[doc = "LPUART5 ipp_do_lpuart_txd input is selected"]
    #[inline(always)]
    pub fn val18(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val18)
    }
}
impl R {
    #[doc = "Bits 0:4 - EXT trigger input connections"]
    #[inline(always)]
    pub fn inp(&self) -> InpR {
        InpR::new((self.bits & 0x1f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:4 - EXT trigger input connections"]
    #[inline(always)]
    pub fn inp(&mut self) -> InpW<ExtTrigSpec> {
        InpW::new(self, 0)
    }
}
#[doc = "EXT trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`ext_trig::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ext_trig::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ExtTrigSpec;
impl crate::RegisterSpec for ExtTrigSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ext_trig::R`](R) reader structure"]
impl crate::Readable for ExtTrigSpec {}
#[doc = "`write(|w| ..)` method takes [`ext_trig::W`](W) writer structure"]
impl crate::Writable for ExtTrigSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets EXT_TRIG[%s] to value 0x1f"]
impl crate::Resettable for ExtTrigSpec {
    const RESET_VALUE: u32 = 0x1f;
}
