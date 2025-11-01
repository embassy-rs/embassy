#[doc = "Register `PINOUTD` reader"]
pub type R = crate::R<PinoutdSpec>;
#[doc = "Register `PINOUTD` writer"]
pub type W = crate::W<PinoutdSpec>;
#[doc = "Field `OUTD` reader - Output Data"]
pub type OutdR = crate::FieldReader<u32>;
#[doc = "Field `OUTD` writer - Output Data"]
pub type OutdW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Output Data"]
    #[inline(always)]
    pub fn outd(&self) -> OutdR {
        OutdR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Output Data"]
    #[inline(always)]
    pub fn outd(&mut self) -> OutdW<PinoutdSpec> {
        OutdW::new(self, 0)
    }
}
#[doc = "Pin Output Data\n\nYou can [`read`](crate::Reg::read) this register and get [`pinoutd::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinoutd::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PinoutdSpec;
impl crate::RegisterSpec for PinoutdSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pinoutd::R`](R) reader structure"]
impl crate::Readable for PinoutdSpec {}
#[doc = "`write(|w| ..)` method takes [`pinoutd::W`](W) writer structure"]
impl crate::Writable for PinoutdSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PINOUTD to value 0"]
impl crate::Resettable for PinoutdSpec {}
