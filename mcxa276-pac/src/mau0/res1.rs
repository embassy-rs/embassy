#[doc = "Register `RES1` reader"]
pub type R = crate::R<Res1Spec>;
#[doc = "Register `RES1` writer"]
pub type W = crate::W<Res1Spec>;
#[doc = "Field `MAU_RES1` reader - MAUWRAP Result Register 1"]
pub type MauRes1R = crate::FieldReader<u32>;
#[doc = "Field `MAU_RES1` writer - MAUWRAP Result Register 1"]
pub type MauRes1W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - MAUWRAP Result Register 1"]
    #[inline(always)]
    pub fn mau_res1(&self) -> MauRes1R {
        MauRes1R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - MAUWRAP Result Register 1"]
    #[inline(always)]
    pub fn mau_res1(&mut self) -> MauRes1W<Res1Spec> {
        MauRes1W::new(self, 0)
    }
}
#[doc = "Result Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`res1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`res1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Res1Spec;
impl crate::RegisterSpec for Res1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`res1::R`](R) reader structure"]
impl crate::Readable for Res1Spec {}
#[doc = "`write(|w| ..)` method takes [`res1::W`](W) writer structure"]
impl crate::Writable for Res1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RES1 to value 0"]
impl crate::Resettable for Res1Spec {}
