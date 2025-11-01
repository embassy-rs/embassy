#[doc = "Register `CORR_ERR_CNT0` reader"]
pub type R = crate::R<CorrErrCnt0Spec>;
#[doc = "Register `CORR_ERR_CNT0` writer"]
pub type W = crate::W<CorrErrCnt0Spec>;
#[doc = "Field `COUNT` reader - Memory n Correctable Error Count"]
pub type CountR = crate::FieldReader;
#[doc = "Field `COUNT` writer - Memory n Correctable Error Count"]
pub type CountW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Memory n Correctable Error Count"]
    #[inline(always)]
    pub fn count(&self) -> CountR {
        CountR::new((self.bits & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Memory n Correctable Error Count"]
    #[inline(always)]
    pub fn count(&mut self) -> CountW<CorrErrCnt0Spec> {
        CountW::new(self, 0)
    }
}
#[doc = "ERM Memory 0 Correctable Error Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`corr_err_cnt0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`corr_err_cnt0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CorrErrCnt0Spec;
impl crate::RegisterSpec for CorrErrCnt0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`corr_err_cnt0::R`](R) reader structure"]
impl crate::Readable for CorrErrCnt0Spec {}
#[doc = "`write(|w| ..)` method takes [`corr_err_cnt0::W`](W) writer structure"]
impl crate::Writable for CorrErrCnt0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CORR_ERR_CNT0 to value 0"]
impl crate::Resettable for CorrErrCnt0Spec {}
