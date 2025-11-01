#[doc = "Register `MB5_64B_WORD9` reader"]
pub type R = crate::R<Mb64bGroupMb5_64bWord9Spec>;
#[doc = "Register `MB5_64B_WORD9` writer"]
pub type W = crate::W<Mb64bGroupMb5_64bWord9Spec>;
#[doc = "Field `DATA_BYTE_39` reader - Data byte 0 of Rx/Tx frame."]
pub type DataByte39R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_39` writer - Data byte 0 of Rx/Tx frame."]
pub type DataByte39W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_38` reader - Data byte 1 of Rx/Tx frame."]
pub type DataByte38R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_38` writer - Data byte 1 of Rx/Tx frame."]
pub type DataByte38W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_37` reader - Data byte 2 of Rx/Tx frame."]
pub type DataByte37R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_37` writer - Data byte 2 of Rx/Tx frame."]
pub type DataByte37W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_36` reader - Data byte 3 of Rx/Tx frame."]
pub type DataByte36R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_36` writer - Data byte 3 of Rx/Tx frame."]
pub type DataByte36W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_39(&self) -> DataByte39R {
        DataByte39R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_38(&self) -> DataByte38R {
        DataByte38R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_37(&self) -> DataByte37R {
        DataByte37R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_36(&self) -> DataByte36R {
        DataByte36R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_39(&mut self) -> DataByte39W<Mb64bGroupMb5_64bWord9Spec> {
        DataByte39W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_38(&mut self) -> DataByte38W<Mb64bGroupMb5_64bWord9Spec> {
        DataByte38W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_37(&mut self) -> DataByte37W<Mb64bGroupMb5_64bWord9Spec> {
        DataByte37W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_36(&mut self) -> DataByte36W<Mb64bGroupMb5_64bWord9Spec> {
        DataByte36W::new(self, 24)
    }
}
#[doc = "Message Buffer 5 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb5_64b_word9::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb5_64b_word9::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mb64bGroupMb5_64bWord9Spec;
impl crate::RegisterSpec for Mb64bGroupMb5_64bWord9Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_64b_group_mb5_64b_word9::R`](R) reader structure"]
impl crate::Readable for Mb64bGroupMb5_64bWord9Spec {}
#[doc = "`write(|w| ..)` method takes [`mb_64b_group_mb5_64b_word9::W`](W) writer structure"]
impl crate::Writable for Mb64bGroupMb5_64bWord9Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MB5_64B_WORD9 to value 0"]
impl crate::Resettable for Mb64bGroupMb5_64bWord9Spec {}
