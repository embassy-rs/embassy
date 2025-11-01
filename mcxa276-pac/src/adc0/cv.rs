#[doc = "Register `CV%s` reader"]
pub type R = crate::R<CvSpec>;
#[doc = "Register `CV%s` writer"]
pub type W = crate::W<CvSpec>;
#[doc = "Field `CVL` reader - Compare Value Low"]
pub type CvlR = crate::FieldReader<u16>;
#[doc = "Field `CVL` writer - Compare Value Low"]
pub type CvlW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
#[doc = "Field `CVH` reader - Compare Value High"]
pub type CvhR = crate::FieldReader<u16>;
#[doc = "Field `CVH` writer - Compare Value High"]
pub type CvhW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Compare Value Low"]
    #[inline(always)]
    pub fn cvl(&self) -> CvlR {
        CvlR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - Compare Value High"]
    #[inline(always)]
    pub fn cvh(&self) -> CvhR {
        CvhR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Compare Value Low"]
    #[inline(always)]
    pub fn cvl(&mut self) -> CvlW<CvSpec> {
        CvlW::new(self, 0)
    }
    #[doc = "Bits 16:31 - Compare Value High"]
    #[inline(always)]
    pub fn cvh(&mut self) -> CvhW<CvSpec> {
        CvhW::new(self, 16)
    }
}
#[doc = "Compare Value Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cv::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cv::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CvSpec;
impl crate::RegisterSpec for CvSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cv::R`](R) reader structure"]
impl crate::Readable for CvSpec {}
#[doc = "`write(|w| ..)` method takes [`cv::W`](W) writer structure"]
impl crate::Writable for CvSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CV%s to value 0"]
impl crate::Resettable for CvSpec {}
