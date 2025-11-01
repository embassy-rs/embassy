#[doc = "Register `EMER_VECT` reader"]
pub type R = crate::R<EmerVectSpec>;
#[doc = "Register `EMER_VECT` writer"]
pub type W = crate::W<EmerVectSpec>;
#[doc = "Field `VEC` reader - Vector address of emergency code routine"]
pub type VecR = crate::FieldReader<u32>;
#[doc = "Field `VEC` writer - Vector address of emergency code routine"]
pub type VecW<'a, REG> = crate::FieldWriter<'a, REG, 30, u32>;
impl R {
    #[doc = "Bits 2:31 - Vector address of emergency code routine"]
    #[inline(always)]
    pub fn vec(&self) -> VecR {
        VecR::new((self.bits >> 2) & 0x3fff_ffff)
    }
}
impl W {
    #[doc = "Bits 2:31 - Vector address of emergency code routine"]
    #[inline(always)]
    pub fn vec(&mut self) -> VecW<EmerVectSpec> {
        VecW::new(self, 2)
    }
}
#[doc = "Emergency Vector\n\nYou can [`read`](crate::Reg::read) this register and get [`emer_vect::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`emer_vect::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct EmerVectSpec;
impl crate::RegisterSpec for EmerVectSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`emer_vect::R`](R) reader structure"]
impl crate::Readable for EmerVectSpec {}
#[doc = "`write(|w| ..)` method takes [`emer_vect::W`](W) writer structure"]
impl crate::Writable for EmerVectSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets EMER_VECT to value 0"]
impl crate::Resettable for EmerVectSpec {}
