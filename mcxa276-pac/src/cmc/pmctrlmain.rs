#[doc = "Register `PMCTRLMAIN` reader"]
pub type R = crate::R<PmctrlmainSpec>;
#[doc = "Register `PMCTRLMAIN` writer"]
pub type W = crate::W<PmctrlmainSpec>;
#[doc = "Low-Power Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Lpmode {
    #[doc = "0: Active/Sleep"]
    Lpmode0000 = 0,
    #[doc = "1: Deep Sleep"]
    Lpmode0001 = 1,
    #[doc = "3: Power Down"]
    Lpmode0011 = 3,
    #[doc = "15: Deep-Power Down"]
    Lpmode1111 = 15,
}
impl From<Lpmode> for u8 {
    #[inline(always)]
    fn from(variant: Lpmode) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Lpmode {
    type Ux = u8;
}
impl crate::IsEnum for Lpmode {}
#[doc = "Field `LPMODE` reader - Low-Power Mode"]
pub type LpmodeR = crate::FieldReader<Lpmode>;
impl LpmodeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Lpmode> {
        match self.bits {
            0 => Some(Lpmode::Lpmode0000),
            1 => Some(Lpmode::Lpmode0001),
            3 => Some(Lpmode::Lpmode0011),
            15 => Some(Lpmode::Lpmode1111),
            _ => None,
        }
    }
    #[doc = "Active/Sleep"]
    #[inline(always)]
    pub fn is_lpmode0000(&self) -> bool {
        *self == Lpmode::Lpmode0000
    }
    #[doc = "Deep Sleep"]
    #[inline(always)]
    pub fn is_lpmode0001(&self) -> bool {
        *self == Lpmode::Lpmode0001
    }
    #[doc = "Power Down"]
    #[inline(always)]
    pub fn is_lpmode0011(&self) -> bool {
        *self == Lpmode::Lpmode0011
    }
    #[doc = "Deep-Power Down"]
    #[inline(always)]
    pub fn is_lpmode1111(&self) -> bool {
        *self == Lpmode::Lpmode1111
    }
}
#[doc = "Field `LPMODE` writer - Low-Power Mode"]
pub type LpmodeW<'a, REG> = crate::FieldWriter<'a, REG, 4, Lpmode>;
impl<'a, REG> LpmodeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Active/Sleep"]
    #[inline(always)]
    pub fn lpmode0000(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::Lpmode0000)
    }
    #[doc = "Deep Sleep"]
    #[inline(always)]
    pub fn lpmode0001(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::Lpmode0001)
    }
    #[doc = "Power Down"]
    #[inline(always)]
    pub fn lpmode0011(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::Lpmode0011)
    }
    #[doc = "Deep-Power Down"]
    #[inline(always)]
    pub fn lpmode1111(self) -> &'a mut crate::W<REG> {
        self.variant(Lpmode::Lpmode1111)
    }
}
impl R {
    #[doc = "Bits 0:3 - Low-Power Mode"]
    #[inline(always)]
    pub fn lpmode(&self) -> LpmodeR {
        LpmodeR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Low-Power Mode"]
    #[inline(always)]
    pub fn lpmode(&mut self) -> LpmodeW<PmctrlmainSpec> {
        LpmodeW::new(self, 0)
    }
}
#[doc = "Power Mode Control\n\nYou can [`read`](crate::Reg::read) this register and get [`pmctrlmain::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pmctrlmain::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PmctrlmainSpec;
impl crate::RegisterSpec for PmctrlmainSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pmctrlmain::R`](R) reader structure"]
impl crate::Readable for PmctrlmainSpec {}
#[doc = "`write(|w| ..)` method takes [`pmctrlmain::W`](W) writer structure"]
impl crate::Writable for PmctrlmainSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PMCTRLMAIN to value 0"]
impl crate::Resettable for PmctrlmainSpec {}
