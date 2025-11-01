#[doc = "Register `PINREN` reader"]
pub type R = crate::R<PinrenSpec>;
#[doc = "Register `PINREN` writer"]
pub type W = crate::W<PinrenSpec>;
#[doc = "Field `PRE` reader - Pin Rising Edge"]
pub type PreR = crate::FieldReader<u32>;
#[doc = "Field `PRE` writer - Pin Rising Edge"]
pub type PreW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Pin Rising Edge"]
    #[inline(always)]
    pub fn pre(&self) -> PreR {
        PreR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Pin Rising Edge"]
    #[inline(always)]
    pub fn pre(&mut self) -> PreW<PinrenSpec> {
        PreW::new(self, 0)
    }
}
#[doc = "Pin Rising Edge Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`pinren::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pinren::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PinrenSpec;
impl crate::RegisterSpec for PinrenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pinren::R`](R) reader structure"]
impl crate::Readable for PinrenSpec {}
#[doc = "`write(|w| ..)` method takes [`pinren::W`](W) writer structure"]
impl crate::Writable for PinrenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PINREN to value 0"]
impl crate::Resettable for PinrenSpec {}
