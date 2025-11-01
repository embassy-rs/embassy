#[doc = "Register `PINOUTDIS` reader"]
pub type R = crate::R<PinoutdisSpec>;
#[doc = "Register `PINOUTDIS` writer"]
pub type W = crate::W<PinoutdisSpec>;
#[doc = "Field `OUTDIS` reader - Output Disable"]
pub type OutdisR = crate::FieldReader<u32>;
#[doc = "Field `OUTDIS` writer - Output Disable"]
pub type OutdisW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Output Disable"]
    #[inline(always)]
    pub fn outdis(&self) -> OutdisR {
        OutdisR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Output Disable"]
    #[inline(always)]
    pub fn outdis(&mut self) -> OutdisW<PinoutdisSpec> {
        OutdisW::new(self, 0)
    }
}
#[doc = "Pin Output Disable\n\nYou can [`read`](crate::Reg::read) this register and get [`pinoutdis::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinoutdis::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PinoutdisSpec;
impl crate::RegisterSpec for PinoutdisSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pinoutdis::R`](R) reader structure"]
impl crate::Readable for PinoutdisSpec {}
#[doc = "`write(|w| ..)` method takes [`pinoutdis::W`](W) writer structure"]
impl crate::Writable for PinoutdisSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PINOUTDIS to value 0"]
impl crate::Resettable for PinoutdisSpec {}
