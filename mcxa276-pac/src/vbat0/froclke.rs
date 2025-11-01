#[doc = "Register `FROCLKE` reader"]
pub type R = crate::R<FroclkeSpec>;
#[doc = "Register `FROCLKE` writer"]
pub type W = crate::W<FroclkeSpec>;
#[doc = "Field `CLKE` reader - Clock Enable"]
pub type ClkeR = crate::FieldReader;
#[doc = "Field `CLKE` writer - Clock Enable"]
pub type ClkeW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
impl R {
    #[doc = "Bits 0:1 - Clock Enable"]
    #[inline(always)]
    pub fn clke(&self) -> ClkeR {
        ClkeR::new((self.bits & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Clock Enable"]
    #[inline(always)]
    pub fn clke(&mut self) -> ClkeW<FroclkeSpec> {
        ClkeW::new(self, 0)
    }
}
#[doc = "FRO16K Clock Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`froclke::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`froclke::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FroclkeSpec;
impl crate::RegisterSpec for FroclkeSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`froclke::R`](R) reader structure"]
impl crate::Readable for FroclkeSpec {}
#[doc = "`write(|w| ..)` method takes [`froclke::W`](W) writer structure"]
impl crate::Writable for FroclkeSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FROCLKE to value 0"]
impl crate::Resettable for FroclkeSpec {}
