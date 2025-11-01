#[doc = "Register `SIDEXT` reader"]
pub type R = crate::R<SidextSpec>;
#[doc = "Register `SIDEXT` writer"]
pub type W = crate::W<SidextSpec>;
#[doc = "Field `DCR` reader - Device Characteristic Register"]
pub type DcrR = crate::FieldReader;
#[doc = "Field `DCR` writer - Device Characteristic Register"]
pub type DcrW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `BCR` reader - Bus Characteristics Register"]
pub type BcrR = crate::FieldReader;
#[doc = "Field `BCR` writer - Bus Characteristics Register"]
pub type BcrW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 8:15 - Device Characteristic Register"]
    #[inline(always)]
    pub fn dcr(&self) -> DcrR {
        DcrR::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Bus Characteristics Register"]
    #[inline(always)]
    pub fn bcr(&self) -> BcrR {
        BcrR::new(((self.bits >> 16) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 8:15 - Device Characteristic Register"]
    #[inline(always)]
    pub fn dcr(&mut self) -> DcrW<SidextSpec> {
        DcrW::new(self, 8)
    }
    #[doc = "Bits 16:23 - Bus Characteristics Register"]
    #[inline(always)]
    pub fn bcr(&mut self) -> BcrW<SidextSpec> {
        BcrW::new(self, 16)
    }
}
#[doc = "Target ID Extension\n\nYou can [`read`](crate::Reg::read) this register and get [`sidext::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sidext::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SidextSpec;
impl crate::RegisterSpec for SidextSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sidext::R`](R) reader structure"]
impl crate::Readable for SidextSpec {}
#[doc = "`write(|w| ..)` method takes [`sidext::W`](W) writer structure"]
impl crate::Writable for SidextSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SIDEXT to value 0x0066_ef00"]
impl crate::Resettable for SidextSpec {
    const RESET_VALUE: u32 = 0x0066_ef00;
}
