#[doc = "Register `FROCTLA` reader"]
pub type R = crate::R<FroctlaSpec>;
#[doc = "Register `FROCTLA` writer"]
pub type W = crate::W<FroctlaSpec>;
#[doc = "FRO16K Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FroEn {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<FroEn> for bool {
    #[inline(always)]
    fn from(variant: FroEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FRO_EN` reader - FRO16K Enable"]
pub type FroEnR = crate::BitReader<FroEn>;
impl FroEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FroEn {
        match self.bits {
            false => FroEn::Disable,
            true => FroEn::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == FroEn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == FroEn::Enable
    }
}
#[doc = "Field `FRO_EN` writer - FRO16K Enable"]
pub type FroEnW<'a, REG> = crate::BitWriter<'a, REG, FroEn>;
impl<'a, REG> FroEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(FroEn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(FroEn::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - FRO16K Enable"]
    #[inline(always)]
    pub fn fro_en(&self) -> FroEnR {
        FroEnR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - FRO16K Enable"]
    #[inline(always)]
    pub fn fro_en(&mut self) -> FroEnW<FroctlaSpec> {
        FroEnW::new(self, 0)
    }
}
#[doc = "FRO16K Control A\n\nYou can [`read`](crate::Reg::read) this register and get [`froctla::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`froctla::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FroctlaSpec;
impl crate::RegisterSpec for FroctlaSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`froctla::R`](R) reader structure"]
impl crate::Readable for FroctlaSpec {}
#[doc = "`write(|w| ..)` method takes [`froctla::W`](W) writer structure"]
impl crate::Writable for FroctlaSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FROCTLA to value 0x01"]
impl crate::Resettable for FroctlaSpec {
    const RESET_VALUE: u32 = 0x01;
}
