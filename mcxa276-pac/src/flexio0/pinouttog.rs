#[doc = "Register `PINOUTTOG` reader"]
pub type R = crate::R<PinouttogSpec>;
#[doc = "Register `PINOUTTOG` writer"]
pub type W = crate::W<PinouttogSpec>;
#[doc = "Field `OUTTOG` reader - Output Toggle"]
pub type OuttogR = crate::FieldReader<u32>;
#[doc = "Field `OUTTOG` writer - Output Toggle"]
pub type OuttogW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Output Toggle"]
    #[inline(always)]
    pub fn outtog(&self) -> OuttogR {
        OuttogR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Output Toggle"]
    #[inline(always)]
    pub fn outtog(&mut self) -> OuttogW<PinouttogSpec> {
        OuttogW::new(self, 0)
    }
}
#[doc = "Pin Output Toggle\n\nYou can [`read`](crate::Reg::read) this register and get [`pinouttog::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinouttog::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PinouttogSpec;
impl crate::RegisterSpec for PinouttogSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pinouttog::R`](R) reader structure"]
impl crate::Readable for PinouttogSpec {}
#[doc = "`write(|w| ..)` method takes [`pinouttog::W`](W) writer structure"]
impl crate::Writable for PinouttogSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PINOUTTOG to value 0"]
impl crate::Resettable for PinouttogSpec {}
