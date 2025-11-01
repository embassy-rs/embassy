#[doc = "Register `PINFEN` reader"]
pub type R = crate::R<PinfenSpec>;
#[doc = "Register `PINFEN` writer"]
pub type W = crate::W<PinfenSpec>;
#[doc = "Field `PFE` reader - Pin Falling Edge"]
pub type PfeR = crate::FieldReader<u32>;
#[doc = "Field `PFE` writer - Pin Falling Edge"]
pub type PfeW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Pin Falling Edge"]
    #[inline(always)]
    pub fn pfe(&self) -> PfeR {
        PfeR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Pin Falling Edge"]
    #[inline(always)]
    pub fn pfe(&mut self) -> PfeW<PinfenSpec> {
        PfeW::new(self, 0)
    }
}
#[doc = "Pin Falling Edge Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`pinfen::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinfen::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PinfenSpec;
impl crate::RegisterSpec for PinfenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pinfen::R`](R) reader structure"]
impl crate::Readable for PinfenSpec {}
#[doc = "`write(|w| ..)` method takes [`pinfen::W`](W) writer structure"]
impl crate::Writable for PinfenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PINFEN to value 0"]
impl crate::Resettable for PinfenSpec {}
