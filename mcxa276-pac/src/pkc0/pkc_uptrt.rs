#[doc = "Register `PKC_UPTRT` reader"]
pub type R = crate::R<PkcUptrtSpec>;
#[doc = "Register `PKC_UPTRT` writer"]
pub type W = crate::W<PkcUptrtSpec>;
#[doc = "Field `PTR` reader - Pointer to start address of PKC FUP table"]
pub type PtrR = crate::FieldReader<u32>;
#[doc = "Field `PTR` writer - Pointer to start address of PKC FUP table"]
pub type PtrW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Pointer to start address of PKC FUP table"]
    #[inline(always)]
    pub fn ptr(&self) -> PtrR {
        PtrR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Pointer to start address of PKC FUP table"]
    #[inline(always)]
    pub fn ptr(&mut self) -> PtrW<PkcUptrtSpec> {
        PtrW::new(self, 0)
    }
}
#[doc = "Universal pointer FUP table\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_uptrt::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_uptrt::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcUptrtSpec;
impl crate::RegisterSpec for PkcUptrtSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_uptrt::R`](R) reader structure"]
impl crate::Readable for PkcUptrtSpec {}
#[doc = "`write(|w| ..)` method takes [`pkc_uptrt::W`](W) writer structure"]
impl crate::Writable for PkcUptrtSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_UPTRT to value 0"]
impl crate::Resettable for PkcUptrtSpec {}
