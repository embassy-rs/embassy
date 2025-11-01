#[doc = "Register `MB3_64B_WORD13` reader"]
pub type R = crate::R<Mb64bGroupMb3_64bWord13Spec>;
#[doc = "Register `MB3_64B_WORD13` writer"]
pub type W = crate::W<Mb64bGroupMb3_64bWord13Spec>;
#[doc = "Field `DATA_BYTE_55` reader - Data byte 0 of Rx/Tx frame."]
pub type DataByte55R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_55` writer - Data byte 0 of Rx/Tx frame."]
pub type DataByte55W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_54` reader - Data byte 1 of Rx/Tx frame."]
pub type DataByte54R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_54` writer - Data byte 1 of Rx/Tx frame."]
pub type DataByte54W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_53` reader - Data byte 2 of Rx/Tx frame."]
pub type DataByte53R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_53` writer - Data byte 2 of Rx/Tx frame."]
pub type DataByte53W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_52` reader - Data byte 3 of Rx/Tx frame."]
pub type DataByte52R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_52` writer - Data byte 3 of Rx/Tx frame."]
pub type DataByte52W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_55(&self) -> DataByte55R {
        DataByte55R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_54(&self) -> DataByte54R {
        DataByte54R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_53(&self) -> DataByte53R {
        DataByte53R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_52(&self) -> DataByte52R {
        DataByte52R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_55(&mut self) -> DataByte55W<Mb64bGroupMb3_64bWord13Spec> {
        DataByte55W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_54(&mut self) -> DataByte54W<Mb64bGroupMb3_64bWord13Spec> {
        DataByte54W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_53(&mut self) -> DataByte53W<Mb64bGroupMb3_64bWord13Spec> {
        DataByte53W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_52(&mut self) -> DataByte52W<Mb64bGroupMb3_64bWord13Spec> {
        DataByte52W::new(self, 24)
    }
}
#[doc = "Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word13::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word13::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mb64bGroupMb3_64bWord13Spec;
impl crate::RegisterSpec for Mb64bGroupMb3_64bWord13Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_64b_group_mb3_64b_word13::R`](R) reader structure"]
impl crate::Readable for Mb64bGroupMb3_64bWord13Spec {}
#[doc = "`write(|w| ..)` method takes [`mb_64b_group_mb3_64b_word13::W`](W) writer structure"]
impl crate::Writable for Mb64bGroupMb3_64bWord13Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MB3_64B_WORD13 to value 0"]
impl crate::Resettable for Mb64bGroupMb3_64bWord13Spec {}
