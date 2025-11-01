#[doc = "Register `BREAK_VECT` reader"]
pub type R = crate::R<BreakVectSpec>;
#[doc = "Register `BREAK_VECT` writer"]
pub type W = crate::W<BreakVectSpec>;
#[doc = "Field `VEC` reader - Vector address of user debug routine."]
pub type VecR = crate::FieldReader<u32>;
#[doc = "Field `VEC` writer - Vector address of user debug routine."]
pub type VecW<'a, REG> = crate::FieldWriter<'a, REG, 30, u32>;
impl R {
    #[doc = "Bits 2:31 - Vector address of user debug routine."]
    #[inline(always)]
    pub fn vec(&self) -> VecR {
        VecR::new((self.bits >> 2) & 0x3fff_ffff)
    }
}
impl W {
    #[doc = "Bits 2:31 - Vector address of user debug routine."]
    #[inline(always)]
    pub fn vec(&mut self) -> VecW<BreakVectSpec> {
        VecW::new(self, 2)
    }
}
#[doc = "Breakpoint Vector\n\nYou can [`read`](crate::Reg::read) this register and get [`break_vect::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`break_vect::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct BreakVectSpec;
impl crate::RegisterSpec for BreakVectSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`break_vect::R`](R) reader structure"]
impl crate::Readable for BreakVectSpec {}
#[doc = "`write(|w| ..)` method takes [`break_vect::W`](W) writer structure"]
impl crate::Writable for BreakVectSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets BREAK_VECT to value 0"]
impl crate::Resettable for BreakVectSpec {}
