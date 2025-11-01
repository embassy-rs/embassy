#[doc = "Register `SOFTHLD` reader"]
pub type R = crate::R<SofthldSpec>;
#[doc = "Register `SOFTHLD` writer"]
pub type W = crate::W<SofthldSpec>;
#[doc = "Field `CNT` reader - SOF Count Threshold"]
pub type CntR = crate::FieldReader;
#[doc = "Field `CNT` writer - SOF Count Threshold"]
pub type CntW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - SOF Count Threshold"]
    #[inline(always)]
    pub fn cnt(&self) -> CntR {
        CntR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:7 - SOF Count Threshold"]
    #[inline(always)]
    pub fn cnt(&mut self) -> CntW<SofthldSpec> {
        CntW::new(self, 0)
    }
}
#[doc = "SOF Threshold\n\nYou can [`read`](crate::Reg::read) this register and get [`softhld::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`softhld::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SofthldSpec;
impl crate::RegisterSpec for SofthldSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`softhld::R`](R) reader structure"]
impl crate::Readable for SofthldSpec {}
#[doc = "`write(|w| ..)` method takes [`softhld::W`](W) writer structure"]
impl crate::Writable for SofthldSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SOFTHLD to value 0"]
impl crate::Resettable for SofthldSpec {}
