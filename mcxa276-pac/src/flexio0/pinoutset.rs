#[doc = "Register `PINOUTSET` reader"]
pub type R = crate::R<PinoutsetSpec>;
#[doc = "Register `PINOUTSET` writer"]
pub type W = crate::W<PinoutsetSpec>;
#[doc = "Field `OUTSET` reader - Output Set"]
pub type OutsetR = crate::FieldReader<u32>;
#[doc = "Field `OUTSET` writer - Output Set"]
pub type OutsetW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Output Set"]
    #[inline(always)]
    pub fn outset(&self) -> OutsetR {
        OutsetR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Output Set"]
    #[inline(always)]
    pub fn outset(&mut self) -> OutsetW<PinoutsetSpec> {
        OutsetW::new(self, 0)
    }
}
#[doc = "Pin Output Set\n\nYou can [`read`](crate::Reg::read) this register and get [`pinoutset::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinoutset::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PinoutsetSpec;
impl crate::RegisterSpec for PinoutsetSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pinoutset::R`](R) reader structure"]
impl crate::Readable for PinoutsetSpec {}
#[doc = "`write(|w| ..)` method takes [`pinoutset::W`](W) writer structure"]
impl crate::Writable for PinoutsetSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PINOUTSET to value 0"]
impl crate::Resettable for PinoutsetSpec {}
