#[doc = "Register `PKC_ZRPTR2` reader"]
pub type R = crate::R<PkcZrptr2Spec>;
#[doc = "Register `PKC_ZRPTR2` writer"]
pub type W = crate::W<PkcZrptr2Spec>;
#[doc = "Field `ZPTR` reader - Start address of Z operand in PKCRAM with byte granularity or constant for calculation modes using CONST"]
pub type ZptrR = crate::FieldReader<u16>;
#[doc = "Field `ZPTR` writer - Start address of Z operand in PKCRAM with byte granularity or constant for calculation modes using CONST"]
pub type ZptrW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
#[doc = "Field `RPTR` reader - Start address of R result in PKCRAM with byte granularity"]
pub type RptrR = crate::FieldReader<u16>;
#[doc = "Field `RPTR` writer - Start address of R result in PKCRAM with byte granularity"]
pub type RptrW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Start address of Z operand in PKCRAM with byte granularity or constant for calculation modes using CONST"]
    #[inline(always)]
    pub fn zptr(&self) -> ZptrR {
        ZptrR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - Start address of R result in PKCRAM with byte granularity"]
    #[inline(always)]
    pub fn rptr(&self) -> RptrR {
        RptrR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Start address of Z operand in PKCRAM with byte granularity or constant for calculation modes using CONST"]
    #[inline(always)]
    pub fn zptr(&mut self) -> ZptrW<PkcZrptr2Spec> {
        ZptrW::new(self, 0)
    }
    #[doc = "Bits 16:31 - Start address of R result in PKCRAM with byte granularity"]
    #[inline(always)]
    pub fn rptr(&mut self) -> RptrW<PkcZrptr2Spec> {
        RptrW::new(self, 16)
    }
}
#[doc = "Z+R pointer register, parameter set 2\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_zrptr2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_zrptr2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcZrptr2Spec;
impl crate::RegisterSpec for PkcZrptr2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_zrptr2::R`](R) reader structure"]
impl crate::Readable for PkcZrptr2Spec {}
#[doc = "`write(|w| ..)` method takes [`pkc_zrptr2::W`](W) writer structure"]
impl crate::Writable for PkcZrptr2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_ZRPTR2 to value 0"]
impl crate::Resettable for PkcZrptr2Spec {}
