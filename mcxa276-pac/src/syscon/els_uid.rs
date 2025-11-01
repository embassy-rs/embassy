#[doc = "Register `ELS_UID[%s]` reader"]
pub type R = crate::R<ElsUidSpec>;
#[doc = "Register `ELS_UID[%s]` writer"]
pub type W = crate::W<ElsUidSpec>;
#[doc = "Field `UID0` reader - UID"]
pub type Uid0R = crate::FieldReader<u32>;
#[doc = "Field `UID0` writer - UID"]
pub type Uid0W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - UID"]
    #[inline(always)]
    pub fn uid0(&self) -> Uid0R {
        Uid0R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - UID"]
    #[inline(always)]
    pub fn uid0(&mut self) -> Uid0W<ElsUidSpec> {
        Uid0W::new(self, 0)
    }
}
#[doc = "Device UID n\n\nYou can [`read`](crate::Reg::read) this register and get [`els_uid::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`els_uid::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ElsUidSpec;
impl crate::RegisterSpec for ElsUidSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`els_uid::R`](R) reader structure"]
impl crate::Readable for ElsUidSpec {}
#[doc = "`write(|w| ..)` method takes [`els_uid::W`](W) writer structure"]
impl crate::Writable for ElsUidSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ELS_UID[%s] to value 0"]
impl crate::Resettable for ElsUidSpec {}
