#[doc = "Register `SFAR` reader"]
pub type R = crate::R<SfarSpec>;
#[doc = "Register `SFAR` writer"]
pub type W = crate::W<SfarSpec>;
#[doc = "Field `ADDRESS` reader - When the SFARVALID bit of the SFSR is set to 1, this field holds the address of an access that caused an SAU violation."]
pub type AddressR = crate::FieldReader<u32>;
#[doc = "Field `ADDRESS` writer - When the SFARVALID bit of the SFSR is set to 1, this field holds the address of an access that caused an SAU violation."]
pub type AddressW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - When the SFARVALID bit of the SFSR is set to 1, this field holds the address of an access that caused an SAU violation."]
    #[inline(always)]
    pub fn address(&self) -> AddressR {
        AddressR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - When the SFARVALID bit of the SFSR is set to 1, this field holds the address of an access that caused an SAU violation."]
    #[inline(always)]
    pub fn address(&mut self) -> AddressW<SfarSpec> {
        AddressW::new(self, 0)
    }
}
#[doc = "Secure Fault Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sfar::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sfar::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SfarSpec;
impl crate::RegisterSpec for SfarSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sfar::R`](R) reader structure"]
impl crate::Readable for SfarSpec {}
#[doc = "`write(|w| ..)` method takes [`sfar::W`](W) writer structure"]
impl crate::Writable for SfarSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SFAR to value 0"]
impl crate::Resettable for SfarSpec {}
