#[doc = "Register `PKC_UPTR` reader"]
pub type R = crate::R<PkcUptrSpec>;
#[doc = "Register `PKC_UPTR` writer"]
pub type W = crate::W<PkcUptrSpec>;
#[doc = "Field `PTR` reader - Pointer to start address of PKC FUP program"]
pub type PtrR = crate::FieldReader<u32>;
#[doc = "Field `PTR` writer - Pointer to start address of PKC FUP program"]
pub type PtrW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Pointer to start address of PKC FUP program"]
    #[inline(always)]
    pub fn ptr(&self) -> PtrR {
        PtrR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Pointer to start address of PKC FUP program"]
    #[inline(always)]
    pub fn ptr(&mut self) -> PtrW<PkcUptrSpec> {
        PtrW::new(self, 0)
    }
}
#[doc = "Universal pointer FUP program\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_uptr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_uptr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcUptrSpec;
impl crate::RegisterSpec for PkcUptrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_uptr::R`](R) reader structure"]
impl crate::Readable for PkcUptrSpec {}
#[doc = "`write(|w| ..)` method takes [`pkc_uptr::W`](W) writer structure"]
impl crate::Writable for PkcUptrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_UPTR to value 0"]
impl crate::Resettable for PkcUptrSpec {}
