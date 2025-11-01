#[doc = "Register `ADD` writer"]
pub type W = crate::W<AddSpec>;
#[doc = "Field `AD` writer - ADD Write Value"]
pub type AdW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - ADD Write Value"]
    #[inline(always)]
    pub fn ad(&mut self) -> AdW<AddSpec> {
        AdW::new(self, 0)
    }
}
#[doc = "ADD Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`add::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct AddSpec;
impl crate::RegisterSpec for AddSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`add::W`](W) writer structure"]
impl crate::Writable for AddSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ADD to value 0"]
impl crate::Resettable for AddSpec {}
