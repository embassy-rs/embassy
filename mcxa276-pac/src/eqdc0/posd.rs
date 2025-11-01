#[doc = "Register `POSD` reader"]
pub type R = crate::R<PosdSpec>;
#[doc = "Register `POSD` writer"]
pub type W = crate::W<PosdSpec>;
#[doc = "Field `POSD` reader - POSD"]
pub type PosdR = crate::FieldReader<u16>;
#[doc = "Field `POSD` writer - POSD"]
pub type PosdW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - POSD"]
    #[inline(always)]
    pub fn posd(&self) -> PosdR {
        PosdR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - POSD"]
    #[inline(always)]
    pub fn posd(&mut self) -> PosdW<PosdSpec> {
        PosdW::new(self, 0)
    }
}
#[doc = "Position Difference Counter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`posd::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`posd::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PosdSpec;
impl crate::RegisterSpec for PosdSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`posd::R`](R) reader structure"]
impl crate::Readable for PosdSpec {}
#[doc = "`write(|w| ..)` method takes [`posd::W`](W) writer structure"]
impl crate::Writable for PosdSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets POSD to value 0"]
impl crate::Resettable for PosdSpec {}
