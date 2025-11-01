#[doc = "Register `PINOUTCLR` reader"]
pub type R = crate::R<PinoutclrSpec>;
#[doc = "Register `PINOUTCLR` writer"]
pub type W = crate::W<PinoutclrSpec>;
#[doc = "Field `OUTCLR` reader - Output Clear"]
pub type OutclrR = crate::FieldReader<u32>;
#[doc = "Field `OUTCLR` writer - Output Clear"]
pub type OutclrW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Output Clear"]
    #[inline(always)]
    pub fn outclr(&self) -> OutclrR {
        OutclrR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Output Clear"]
    #[inline(always)]
    pub fn outclr(&mut self) -> OutclrW<PinoutclrSpec> {
        OutclrW::new(self, 0)
    }
}
#[doc = "Pin Output Clear\n\nYou can [`read`](crate::Reg::read) this register and get [`pinoutclr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinoutclr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PinoutclrSpec;
impl crate::RegisterSpec for PinoutclrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pinoutclr::R`](R) reader structure"]
impl crate::Readable for PinoutclrSpec {}
#[doc = "`write(|w| ..)` method takes [`pinoutclr::W`](W) writer structure"]
impl crate::Writable for PinoutclrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PINOUTCLR to value 0"]
impl crate::Resettable for PinoutclrSpec {}
