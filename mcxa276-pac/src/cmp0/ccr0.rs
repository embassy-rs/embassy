#[doc = "Register `CCR0` reader"]
pub type R = crate::R<Ccr0Spec>;
#[doc = "Register `CCR0` writer"]
pub type W = crate::W<Ccr0Spec>;
#[doc = "Comparator Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmpEn {
    #[doc = "0: Disable (The analog logic remains off and consumes no power.)"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<CmpEn> for bool {
    #[inline(always)]
    fn from(variant: CmpEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP_EN` reader - Comparator Enable"]
pub type CmpEnR = crate::BitReader<CmpEn>;
impl CmpEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CmpEn {
        match self.bits {
            false => CmpEn::Disable,
            true => CmpEn::Enable,
        }
    }
    #[doc = "Disable (The analog logic remains off and consumes no power.)"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == CmpEn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == CmpEn::Enable
    }
}
#[doc = "Field `CMP_EN` writer - Comparator Enable"]
pub type CmpEnW<'a, REG> = crate::BitWriter<'a, REG, CmpEn>;
impl<'a, REG> CmpEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable (The analog logic remains off and consumes no power.)"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(CmpEn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(CmpEn::Enable)
    }
}
#[doc = "Comparator Deep Sleep Mode Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmpStopEn {
    #[doc = "0: Disables the analog comparator regardless of CMP_EN."]
    Disable = 0,
    #[doc = "1: Allows CMP_EN to enable the analog comparator."]
    Enable = 1,
}
impl From<CmpStopEn> for bool {
    #[inline(always)]
    fn from(variant: CmpStopEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP_STOP_EN` reader - Comparator Deep Sleep Mode Enable"]
pub type CmpStopEnR = crate::BitReader<CmpStopEn>;
impl CmpStopEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CmpStopEn {
        match self.bits {
            false => CmpStopEn::Disable,
            true => CmpStopEn::Enable,
        }
    }
    #[doc = "Disables the analog comparator regardless of CMP_EN."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == CmpStopEn::Disable
    }
    #[doc = "Allows CMP_EN to enable the analog comparator."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == CmpStopEn::Enable
    }
}
#[doc = "Field `CMP_STOP_EN` writer - Comparator Deep Sleep Mode Enable"]
pub type CmpStopEnW<'a, REG> = crate::BitWriter<'a, REG, CmpStopEn>;
impl<'a, REG> CmpStopEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables the analog comparator regardless of CMP_EN."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(CmpStopEn::Disable)
    }
    #[doc = "Allows CMP_EN to enable the analog comparator."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(CmpStopEn::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - Comparator Enable"]
    #[inline(always)]
    pub fn cmp_en(&self) -> CmpEnR {
        CmpEnR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Comparator Deep Sleep Mode Enable"]
    #[inline(always)]
    pub fn cmp_stop_en(&self) -> CmpStopEnR {
        CmpStopEnR::new(((self.bits >> 1) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Comparator Enable"]
    #[inline(always)]
    pub fn cmp_en(&mut self) -> CmpEnW<Ccr0Spec> {
        CmpEnW::new(self, 0)
    }
    #[doc = "Bit 1 - Comparator Deep Sleep Mode Enable"]
    #[inline(always)]
    pub fn cmp_stop_en(&mut self) -> CmpStopEnW<Ccr0Spec> {
        CmpStopEnW::new(self, 1)
    }
}
#[doc = "Comparator Control Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`ccr0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ccr0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ccr0Spec;
impl crate::RegisterSpec for Ccr0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ccr0::R`](R) reader structure"]
impl crate::Readable for Ccr0Spec {}
#[doc = "`write(|w| ..)` method takes [`ccr0::W`](W) writer structure"]
impl crate::Writable for Ccr0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CCR0 to value 0x02"]
impl crate::Resettable for Ccr0Spec {
    const RESET_VALUE: u32 = 0x02;
}
