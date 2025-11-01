#[doc = "Register `TYPE` reader"]
pub type R = crate::R<TypeSpec>;
#[doc = "Register `TYPE` writer"]
pub type W = crate::W<TypeSpec>;
#[doc = "Field `SREGION` reader - SAU regions. The number of implemented SAU regions."]
pub type SregionR = crate::FieldReader;
#[doc = "Field `SREGION` writer - SAU regions. The number of implemented SAU regions."]
pub type SregionW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - SAU regions. The number of implemented SAU regions."]
    #[inline(always)]
    pub fn sregion(&self) -> SregionR {
        SregionR::new((self.bits & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - SAU regions. The number of implemented SAU regions."]
    #[inline(always)]
    pub fn sregion(&mut self) -> SregionW<TypeSpec> {
        SregionW::new(self, 0)
    }
}
#[doc = "Security Attribution Unit Type Register\n\nYou can [`read`](crate::Reg::read) this register and get [`type_::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`type_::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TypeSpec;
impl crate::RegisterSpec for TypeSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`type_::R`](R) reader structure"]
impl crate::Readable for TypeSpec {}
#[doc = "`write(|w| ..)` method takes [`type_::W`](W) writer structure"]
impl crate::Writable for TypeSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TYPE to value 0"]
impl crate::Resettable for TypeSpec {}
