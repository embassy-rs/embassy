#[doc = "Register `PKC_LEN1` reader"]
pub type R = crate::R<PkcLen1Spec>;
#[doc = "Register `PKC_LEN1` writer"]
pub type W = crate::W<PkcLen1Spec>;
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
    pub fn len(&mut self) -> LenW<PkcLen1Spec> {
        LenW::new(self, 0)
    }
    #[doc = "Bits 16:31 - Loop counter for microcode pattern"]
    #[inline(always)]
    pub fn mclen(&mut self) -> MclenW<PkcLen1Spec> {
        MclenW::new(self, 16)
    }
}
#[doc = "Length register, parameter set 1\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_len1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_len1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcLen1Spec;
impl crate::RegisterSpec for PkcLen1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_len1::R`](R) reader structure"]
impl crate::Readable for PkcLen1Spec {}
#[doc = "`write(|w| ..)` method takes [`pkc_len1::W`](W) writer structure"]
impl crate::Writable for PkcLen1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_LEN1 to value 0"]
impl crate::Resettable for PkcLen1Spec {}
