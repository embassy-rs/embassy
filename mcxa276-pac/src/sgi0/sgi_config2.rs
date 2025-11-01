#[doc = "Register `sgi_config2` reader"]
pub type R = crate::R<SgiConfig2Spec>;
#[doc = "Field `aes_used` reader - 0=Apollo; 1=Aegis; 2=Ayna; 3=Athenium; 4=Ajax;"]
pub type AesUsedR = crate::FieldReader;
#[doc = "Field `aes_num_sboxes` reader - Number of AES sboxes"]
pub type AesNumSboxesR = crate::FieldReader;
#[doc = "Field `aes_keysize` reader - 0=128-Only,1=192-Only, 2=256-Only, 3=All Keysizes"]
pub type AesKeysizeR = crate::FieldReader;
#[doc = "Field `config2b_rsvd` reader - reserved"]
pub type Config2bRsvdR = crate::FieldReader;
#[doc = "Field `des_used` reader - 0=Dakar; 1=Danube; 2=Depicta; 3=Digi; 4=Date;"]
pub type DesUsedR = crate::FieldReader;
#[doc = "Field `des_num_sboxes` reader - Number of DES sboxes"]
pub type DesNumSboxesR = crate::FieldReader;
#[doc = "Field `config2a_rsvd` reader - reserved"]
pub type Config2aRsvdR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:3 - 0=Apollo; 1=Aegis; 2=Ayna; 3=Athenium; 4=Ajax;"]
    #[inline(always)]
    pub fn aes_used(&self) -> AesUsedR {
        AesUsedR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 4:8 - Number of AES sboxes"]
    #[inline(always)]
    pub fn aes_num_sboxes(&self) -> AesNumSboxesR {
        AesNumSboxesR::new(((self.bits >> 4) & 0x1f) as u8)
    }
    #[doc = "Bits 9:10 - 0=128-Only,1=192-Only, 2=256-Only, 3=All Keysizes"]
    #[inline(always)]
    pub fn aes_keysize(&self) -> AesKeysizeR {
        AesKeysizeR::new(((self.bits >> 9) & 3) as u8)
    }
    #[doc = "Bits 11:15 - reserved"]
    #[inline(always)]
    pub fn config2b_rsvd(&self) -> Config2bRsvdR {
        Config2bRsvdR::new(((self.bits >> 11) & 0x1f) as u8)
    }
    #[doc = "Bits 16:19 - 0=Dakar; 1=Danube; 2=Depicta; 3=Digi; 4=Date;"]
    #[inline(always)]
    pub fn des_used(&self) -> DesUsedR {
        DesUsedR::new(((self.bits >> 16) & 0x0f) as u8)
    }
    #[doc = "Bits 20:24 - Number of DES sboxes"]
    #[inline(always)]
    pub fn des_num_sboxes(&self) -> DesNumSboxesR {
        DesNumSboxesR::new(((self.bits >> 20) & 0x1f) as u8)
    }
    #[doc = "Bits 25:31 - reserved"]
    #[inline(always)]
    pub fn config2a_rsvd(&self) -> Config2aRsvdR {
        Config2aRsvdR::new(((self.bits >> 25) & 0x7f) as u8)
    }
}
#[doc = "SHA Configuration 2 Reg\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_config2::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiConfig2Spec;
impl crate::RegisterSpec for SgiConfig2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_config2::R`](R) reader structure"]
impl crate::Readable for SgiConfig2Spec {}
#[doc = "`reset()` method sets sgi_config2 to value 0"]
impl crate::Resettable for SgiConfig2Spec {}
