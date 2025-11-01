#[doc = "Register `PINOUTE` reader"]
pub type R = crate::R<PinouteSpec>;
#[doc = "Register `PINOUTE` writer"]
pub type W = crate::W<PinouteSpec>;
#[doc = "Field `OUTE` reader - Output Enable"]
pub type OuteR = crate::FieldReader<u32>;
#[doc = "Field `OUTE` writer - Output Enable"]
pub type OuteW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Output Enable"]
    #[inline(always)]
    pub fn oute(&self) -> OuteR {
        OuteR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Output Enable"]
    #[inline(always)]
    pub fn oute(&mut self) -> OuteW<PinouteSpec> {
        OuteW::new(self, 0)
    }
}
#[doc = "Pin Output Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`pinoute::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinoute::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PinouteSpec;
impl crate::RegisterSpec for PinouteSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pinoute::R`](R) reader structure"]
impl crate::Readable for PinouteSpec {}
#[doc = "`write(|w| ..)` method takes [`pinoute::W`](W) writer structure"]
impl crate::Writable for PinouteSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PINOUTE to value 0"]
impl crate::Resettable for PinouteSpec {}
