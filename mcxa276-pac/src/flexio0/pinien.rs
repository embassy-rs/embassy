#[doc = "Register `PINIEN` reader"]
pub type R = crate::R<PinienSpec>;
#[doc = "Register `PINIEN` writer"]
pub type W = crate::W<PinienSpec>;
#[doc = "Field `PSIE` reader - Pin Status Interrupt Enable"]
pub type PsieR = crate::FieldReader<u32>;
#[doc = "Field `PSIE` writer - Pin Status Interrupt Enable"]
pub type PsieW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Pin Status Interrupt Enable"]
    #[inline(always)]
    pub fn psie(&self) -> PsieR {
        PsieR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Pin Status Interrupt Enable"]
    #[inline(always)]
    pub fn psie(&mut self) -> PsieW<PinienSpec> {
        PsieW::new(self, 0)
    }
}
#[doc = "Pin Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`pinien::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinien::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PinienSpec;
impl crate::RegisterSpec for PinienSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pinien::R`](R) reader structure"]
impl crate::Readable for PinienSpec {}
#[doc = "`write(|w| ..)` method takes [`pinien::W`](W) writer structure"]
impl crate::Writable for PinienSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PINIEN to value 0"]
impl crate::Resettable for PinienSpec {}
