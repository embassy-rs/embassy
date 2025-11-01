#[doc = "Register `MB4_64B_WORD2` reader"]
pub type R = crate::R<Mb64bGroupMb4_64bWord2Spec>;
#[doc = "Register `MB4_64B_WORD2` writer"]
pub type W = crate::W<Mb64bGroupMb4_64bWord2Spec>;
#[doc = "Field `DATA_BYTE_11` reader - Data byte 0 of Rx/Tx frame."]
pub type DataByte11R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_11` writer - Data byte 0 of Rx/Tx frame."]
pub type DataByte11W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_10` reader - Data byte 1 of Rx/Tx frame."]
pub type DataByte10R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_10` writer - Data byte 1 of Rx/Tx frame."]
pub type DataByte10W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_9` reader - Data byte 2 of Rx/Tx frame."]
pub type DataByte9R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_9` writer - Data byte 2 of Rx/Tx frame."]
pub type DataByte9W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_8` reader - Data byte 3 of Rx/Tx frame."]
pub type DataByte8R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_8` writer - Data byte 3 of Rx/Tx frame."]
pub type DataByte8W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_11(&self) -> DataByte11R {
        DataByte11R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_10(&self) -> DataByte10R {
        DataByte10R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_9(&self) -> DataByte9R {
        DataByte9R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_8(&self) -> DataByte8R {
        DataByte8R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_11(&mut self) -> DataByte11W<Mb64bGroupMb4_64bWord2Spec> {
        DataByte11W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_10(&mut self) -> DataByte10W<Mb64bGroupMb4_64bWord2Spec> {
        DataByte10W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_9(&mut self) -> DataByte9W<Mb64bGroupMb4_64bWord2Spec> {
        DataByte9W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_8(&mut self) -> DataByte8W<Mb64bGroupMb4_64bWord2Spec> {
        DataByte8W::new(self, 24)
    }
}
#[doc = "Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mb64bGroupMb4_64bWord2Spec;
impl crate::RegisterSpec for Mb64bGroupMb4_64bWord2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_64b_group_mb4_64b_word2::R`](R) reader structure"]
impl crate::Readable for Mb64bGroupMb4_64bWord2Spec {}
#[doc = "`write(|w| ..)` method takes [`mb_64b_group_mb4_64b_word2::W`](W) writer structure"]
impl crate::Writable for Mb64bGroupMb4_64bWord2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MB4_64B_WORD2 to value 0"]
impl crate::Resettable for Mb64bGroupMb4_64bWord2Spec {}
