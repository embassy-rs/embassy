#[doc = "Register `CORR_ERR_CNT1` reader"]
pub type R = crate::R<CorrErrCnt1Spec>;
#[doc = "Register `CORR_ERR_CNT1` writer"]
pub type W = crate::W<CorrErrCnt1Spec>;
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
    pub fn count(&mut self) -> CountW<CorrErrCnt1Spec> {
        CountW::new(self, 0)
    }
}
#[doc = "ERM Memory 1 Correctable Error Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`corr_err_cnt1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`corr_err_cnt1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CorrErrCnt1Spec;
impl crate::RegisterSpec for CorrErrCnt1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`corr_err_cnt1::R`](R) reader structure"]
impl crate::Readable for CorrErrCnt1Spec {}
#[doc = "`write(|w| ..)` method takes [`corr_err_cnt1::W`](W) writer structure"]
impl crate::Writable for CorrErrCnt1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CORR_ERR_CNT1 to value 0"]
impl crate::Resettable for CorrErrCnt1Spec {}
