#[doc = "Register `MB2_64B_WORD15` reader"]
pub type R = crate::R<Mb64bGroupMb2_64bWord15Spec>;
#[doc = "Register `MB2_64B_WORD15` writer"]
pub type W = crate::W<Mb64bGroupMb2_64bWord15Spec>;
#[doc = "Field `DATA_BYTE_63` reader - Data byte 0 of Rx/Tx frame."]
pub type DataByte63R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_63` writer - Data byte 0 of Rx/Tx frame."]
pub type DataByte63W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_62` reader - Data byte 1 of Rx/Tx frame."]
pub type DataByte62R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_62` writer - Data byte 1 of Rx/Tx frame."]
pub type DataByte62W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_61` reader - Data byte 2 of Rx/Tx frame."]
pub type DataByte61R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_61` writer - Data byte 2 of Rx/Tx frame."]
pub type DataByte61W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_60` reader - Data byte 3 of Rx/Tx frame."]
pub type DataByte60R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_60` writer - Data byte 3 of Rx/Tx frame."]
pub type DataByte60W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_63(&self) -> DataByte63R {
        DataByte63R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_62(&self) -> DataByte62R {
        DataByte62R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_61(&self) -> DataByte61R {
        DataByte61R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_60(&self) -> DataByte60R {
        DataByte60R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_63(&mut self) -> DataByte63W<Mb64bGroupMb2_64bWord15Spec> {
        DataByte63W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_62(&mut self) -> DataByte62W<Mb64bGroupMb2_64bWord15Spec> {
        DataByte62W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_61(&mut self) -> DataByte61W<Mb64bGroupMb2_64bWord15Spec> {
        DataByte61W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_60(&mut self) -> DataByte60W<Mb64bGroupMb2_64bWord15Spec> {
        DataByte60W::new(self, 24)
    }
}
#[doc = "Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word15::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word15::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mb64bGroupMb2_64bWord15Spec;
impl crate::RegisterSpec for Mb64bGroupMb2_64bWord15Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_64b_group_mb2_64b_word15::R`](R) reader structure"]
impl crate::Readable for Mb64bGroupMb2_64bWord15Spec {}
#[doc = "`write(|w| ..)` method takes [`mb_64b_group_mb2_64b_word15::W`](W) writer structure"]
impl crate::Writable for Mb64bGroupMb2_64bWord15Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MB2_64B_WORD15 to value 0"]
impl crate::Resettable for Mb64bGroupMb2_64bWord15Spec {}
