#[doc = "Register `RES0` reader"]
pub type R = crate::R<Res0Spec>;
#[doc = "Register `RES0` writer"]
pub type W = crate::W<Res0Spec>;
#[doc = "Field `MAU_RES0` reader - MAUWRAP Result Register 0"]
pub type MauRes0R = crate::FieldReader<u32>;
#[doc = "Field `MAU_RES0` writer - MAUWRAP Result Register 0"]
pub type MauRes0W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - MAUWRAP Result Register 0"]
    #[inline(always)]
    pub fn mau_res0(&self) -> MauRes0R {
        MauRes0R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - MAUWRAP Result Register 0"]
    #[inline(always)]
    pub fn mau_res0(&mut self) -> MauRes0W<Res0Spec> {
        MauRes0W::new(self, 0)
    }
}
#[doc = "Result Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`res0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`res0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Res0Spec;
impl crate::RegisterSpec for Res0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`res0::R`](R) reader structure"]
impl crate::Readable for Res0Spec {}
#[doc = "`write(|w| ..)` method takes [`res0::W`](W) writer structure"]
impl crate::Writable for Res0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RES0 to value 0"]
impl crate::Resettable for Res0Spec {}
