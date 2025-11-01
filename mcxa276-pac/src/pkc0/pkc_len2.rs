#[doc = "Register `PKC_LEN2` reader"]
pub type R = crate::R<PkcLen2Spec>;
#[doc = "Register `PKC_LEN2` writer"]
pub type W = crate::W<PkcLen2Spec>;
#[doc = "Field `LEN` reader - Operand length"]
pub type LenR = crate::FieldReader<u16>;
#[doc = "Field `LEN` writer - Operand length"]
pub type LenW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
#[doc = "Field `MCLEN` reader - Loop counter for microcode pattern"]
pub type MclenR = crate::FieldReader<u16>;
#[doc = "Field `MCLEN` writer - Loop counter for microcode pattern"]
pub type MclenW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Operand length"]
    #[inline(always)]
    pub fn len(&self) -> LenR {
        LenR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - Loop counter for microcode pattern"]
    #[inline(always)]
    pub fn mclen(&self) -> MclenR {
        MclenR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Operand length"]
    #[inline(always)]
    pub fn len(&mut self) -> LenW<PkcLen2Spec> {
        LenW::new(self, 0)
    }
    #[doc = "Bits 16:31 - Loop counter for microcode pattern"]
    #[inline(always)]
    pub fn mclen(&mut self) -> MclenW<PkcLen2Spec> {
        MclenW::new(self, 16)
    }
}
#[doc = "Length register, parameter set 2\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_len2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_len2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcLen2Spec;
impl crate::RegisterSpec for PkcLen2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_len2::R`](R) reader structure"]
impl crate::Readable for PkcLen2Spec {}
#[doc = "`write(|w| ..)` method takes [`pkc_len2::W`](W) writer structure"]
impl crate::Writable for PkcLen2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_LEN2 to value 0"]
impl crate::Resettable for PkcLen2Spec {}
