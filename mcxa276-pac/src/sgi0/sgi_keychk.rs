#[doc = "Register `sgi_keychk` reader"]
pub type R = crate::R<SgiKeychkSpec>;
#[doc = "Register `sgi_keychk` writer"]
pub type W = crate::W<SgiKeychkSpec>;
#[doc = "Field `keychksum` reader - Key checksum (32-bit)."]
pub type KeychksumR = crate::FieldReader<u32>;
#[doc = "Field `keychksum` writer - Key checksum (32-bit)."]
pub type KeychksumW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Key checksum (32-bit)."]
    #[inline(always)]
    pub fn keychksum(&self) -> KeychksumR {
        KeychksumR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Key checksum (32-bit)."]
    #[inline(always)]
    pub fn keychksum(&mut self) -> KeychksumW<SgiKeychkSpec> {
        KeychksumW::new(self, 0)
    }
}
#[doc = "Key checksum register\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_keychk::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_keychk::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKeychkSpec;
impl crate::RegisterSpec for SgiKeychkSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_keychk::R`](R) reader structure"]
impl crate::Readable for SgiKeychkSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_keychk::W`](W) writer structure"]
impl crate::Writable for SgiKeychkSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_keychk to value 0"]
impl crate::Resettable for SgiKeychkSpec {}
