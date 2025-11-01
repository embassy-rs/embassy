#[doc = "Register `RES3` reader"]
pub type R = crate::R<Res3Spec>;
#[doc = "Register `RES3` writer"]
pub type W = crate::W<Res3Spec>;
#[doc = "Field `MAU_RES3` reader - MAUWRAP Result Register 3"]
pub type MauRes3R = crate::FieldReader<u32>;
#[doc = "Field `MAU_RES3` writer - MAUWRAP Result Register 3"]
pub type MauRes3W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - MAUWRAP Result Register 3"]
    #[inline(always)]
    pub fn mau_res3(&self) -> MauRes3R {
        MauRes3R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - MAUWRAP Result Register 3"]
    #[inline(always)]
    pub fn mau_res3(&mut self) -> MauRes3W<Res3Spec> {
        MauRes3W::new(self, 0)
    }
}
#[doc = "Result Register 3\n\nYou can [`read`](crate::Reg::read) this register and get [`res3::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`res3::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Res3Spec;
impl crate::RegisterSpec for Res3Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`res3::R`](R) reader structure"]
impl crate::Readable for Res3Spec {}
#[doc = "`write(|w| ..)` method takes [`res3::W`](W) writer structure"]
impl crate::Writable for Res3Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RES3 to value 0"]
impl crate::Resettable for Res3Spec {}
