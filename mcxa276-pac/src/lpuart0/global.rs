#[doc = "Register `GLOBAL` reader"]
pub type R = crate::R<GlobalSpec>;
#[doc = "Register `GLOBAL` writer"]
pub type W = crate::W<GlobalSpec>;
#[doc = "Software Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rst {
    #[doc = "0: Not reset"]
    NoEffect = 0,
    #[doc = "1: Reset"]
    Reset = 1,
}
impl From<Rst> for bool {
    #[inline(always)]
    fn from(variant: Rst) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RST` reader - Software Reset"]
pub type RstR = crate::BitReader<Rst>;
impl RstR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rst {
        match self.bits {
            false => Rst::NoEffect,
            true => Rst::Reset,
        }
    }
    #[doc = "Not reset"]
    #[inline(always)]
    pub fn is_no_effect(&self) -> bool {
        *self == Rst::NoEffect
    }
    #[doc = "Reset"]
    #[inline(always)]
    pub fn is_reset(&self) -> bool {
        *self == Rst::Reset
    }
}
#[doc = "Field `RST` writer - Software Reset"]
pub type RstW<'a, REG> = crate::BitWriter<'a, REG, Rst>;
impl<'a, REG> RstW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not reset"]
    #[inline(always)]
    pub fn no_effect(self) -> &'a mut crate::W<REG> {
        self.variant(Rst::NoEffect)
    }
    #[doc = "Reset"]
    #[inline(always)]
    pub fn reset(self) -> &'a mut crate::W<REG> {
        self.variant(Rst::Reset)
    }
}
impl R {
    #[doc = "Bit 1 - Software Reset"]
    #[inline(always)]
    pub fn rst(&self) -> RstR {
        RstR::new(((self.bits >> 1) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 1 - Software Reset"]
    #[inline(always)]
    pub fn rst(&mut self) -> RstW<GlobalSpec> {
        RstW::new(self, 1)
    }
}
#[doc = "Global\n\nYou can [`read`](crate::Reg::read) this register and get [`global::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`global::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct GlobalSpec;
impl crate::RegisterSpec for GlobalSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`global::R`](R) reader structure"]
impl crate::Readable for GlobalSpec {}
#[doc = "`write(|w| ..)` method takes [`global::W`](W) writer structure"]
impl crate::Writable for GlobalSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets GLOBAL to value 0"]
impl crate::Resettable for GlobalSpec {}
