#[doc = "Register `PKC_XYPTR2` reader"]
pub type R = crate::R<PkcXyptr2Spec>;
#[doc = "Register `PKC_XYPTR2` writer"]
pub type W = crate::W<PkcXyptr2Spec>;
#[doc = "Field `XPTR` reader - Start address of X operand in PKCRAM with byte granularity"]
pub type XptrR = crate::FieldReader<u16>;
#[doc = "Field `XPTR` writer - Start address of X operand in PKCRAM with byte granularity"]
pub type XptrW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
#[doc = "Field `YPTR` reader - Start address of Y operand in PKCRAM with byte granularity"]
pub type YptrR = crate::FieldReader<u16>;
#[doc = "Field `YPTR` writer - Start address of Y operand in PKCRAM with byte granularity"]
pub type YptrW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Start address of X operand in PKCRAM with byte granularity"]
    #[inline(always)]
    pub fn xptr(&self) -> XptrR {
        XptrR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - Start address of Y operand in PKCRAM with byte granularity"]
    #[inline(always)]
    pub fn yptr(&self) -> YptrR {
        YptrR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Start address of X operand in PKCRAM with byte granularity"]
    #[inline(always)]
    pub fn xptr(&mut self) -> XptrW<PkcXyptr2Spec> {
        XptrW::new(self, 0)
    }
    #[doc = "Bits 16:31 - Start address of Y operand in PKCRAM with byte granularity"]
    #[inline(always)]
    pub fn yptr(&mut self) -> YptrW<PkcXyptr2Spec> {
        YptrW::new(self, 16)
    }
}
#[doc = "X+Y pointer register, parameter set 2\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_xyptr2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_xyptr2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcXyptr2Spec;
impl crate::RegisterSpec for PkcXyptr2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_xyptr2::R`](R) reader structure"]
impl crate::Readable for PkcXyptr2Spec {}
#[doc = "`write(|w| ..)` method takes [`pkc_xyptr2::W`](W) writer structure"]
impl crate::Writable for PkcXyptr2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_XYPTR2 to value 0"]
impl crate::Resettable for PkcXyptr2Spec {}
