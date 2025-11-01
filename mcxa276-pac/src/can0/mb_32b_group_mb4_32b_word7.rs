#[doc = "Register `MB4_32B_WORD7` reader"]
pub type R = crate::R<Mb32bGroupMb4_32bWord7Spec>;
#[doc = "Register `MB4_32B_WORD7` writer"]
pub type W = crate::W<Mb32bGroupMb4_32bWord7Spec>;
#[doc = "Field `DATA_BYTE_31` reader - Data byte 0 of Rx/Tx frame."]
pub type DataByte31R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_31` writer - Data byte 0 of Rx/Tx frame."]
pub type DataByte31W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_30` reader - Data byte 1 of Rx/Tx frame."]
pub type DataByte30R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_30` writer - Data byte 1 of Rx/Tx frame."]
pub type DataByte30W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_29` reader - Data byte 2 of Rx/Tx frame."]
pub type DataByte29R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_29` writer - Data byte 2 of Rx/Tx frame."]
pub type DataByte29W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_28` reader - Data byte 3 of Rx/Tx frame."]
pub type DataByte28R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_28` writer - Data byte 3 of Rx/Tx frame."]
pub type DataByte28W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_31(&self) -> DataByte31R {
        DataByte31R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_30(&self) -> DataByte30R {
        DataByte30R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_29(&self) -> DataByte29R {
        DataByte29R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_28(&self) -> DataByte28R {
        DataByte28R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_31(&mut self) -> DataByte31W<Mb32bGroupMb4_32bWord7Spec> {
        DataByte31W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_30(&mut self) -> DataByte30W<Mb32bGroupMb4_32bWord7Spec> {
        DataByte30W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_29(&mut self) -> DataByte29W<Mb32bGroupMb4_32bWord7Spec> {
        DataByte29W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_28(&mut self) -> DataByte28W<Mb32bGroupMb4_32bWord7Spec> {
        DataByte28W::new(self, 24)
    }
}
#[doc = "Message Buffer 4 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb4_32b_word7::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb4_32b_word7::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mb32bGroupMb4_32bWord7Spec;
impl crate::RegisterSpec for Mb32bGroupMb4_32bWord7Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_32b_group_mb4_32b_word7::R`](R) reader structure"]
impl crate::Readable for Mb32bGroupMb4_32bWord7Spec {}
#[doc = "`write(|w| ..)` method takes [`mb_32b_group_mb4_32b_word7::W`](W) writer structure"]
impl crate::Writable for Mb32bGroupMb4_32bWord7Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MB4_32B_WORD7 to value 0"]
impl crate::Resettable for Mb32bGroupMb4_32bWord7Spec {}
