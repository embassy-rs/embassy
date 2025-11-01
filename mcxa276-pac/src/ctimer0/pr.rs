#[doc = "Register `PR` reader"]
pub type R = crate::R<PrSpec>;
#[doc = "Register `PR` writer"]
pub type W = crate::W<PrSpec>;
#[doc = "Field `PRVAL` reader - Prescale Reload Value"]
pub type PrvalR = crate::FieldReader<u32>;
#[doc = "Field `PRVAL` writer - Prescale Reload Value"]
pub type PrvalW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Prescale Reload Value"]
    #[inline(always)]
    pub fn prval(&self) -> PrvalR {
        PrvalR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Prescale Reload Value"]
    #[inline(always)]
    pub fn prval(&mut self) -> PrvalW<PrSpec> {
        PrvalW::new(self, 0)
    }
}
#[doc = "Prescale\n\nYou can [`read`](crate::Reg::read) this register and get [`pr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PrSpec;
impl crate::RegisterSpec for PrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pr::R`](R) reader structure"]
impl crate::Readable for PrSpec {}
#[doc = "`write(|w| ..)` method takes [`pr::W`](W) writer structure"]
impl crate::Writable for PrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PR to value 0"]
impl crate::Resettable for PrSpec {}
