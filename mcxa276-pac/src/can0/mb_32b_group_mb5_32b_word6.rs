#[doc = "Register `MB5_32B_WORD6` reader"]
pub type R = crate::R<Mb32bGroupMb5_32bWord6Spec>;
#[doc = "Register `MB5_32B_WORD6` writer"]
pub type W = crate::W<Mb32bGroupMb5_32bWord6Spec>;
#[doc = "Field `DATA_BYTE_27` reader - Data byte 0 of Rx/Tx frame."]
pub type DataByte27R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_27` writer - Data byte 0 of Rx/Tx frame."]
pub type DataByte27W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_26` reader - Data byte 1 of Rx/Tx frame."]
pub type DataByte26R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_26` writer - Data byte 1 of Rx/Tx frame."]
pub type DataByte26W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_25` reader - Data byte 2 of Rx/Tx frame."]
pub type DataByte25R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_25` writer - Data byte 2 of Rx/Tx frame."]
pub type DataByte25W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_24` reader - Data byte 3 of Rx/Tx frame."]
pub type DataByte24R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_24` writer - Data byte 3 of Rx/Tx frame."]
pub type DataByte24W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_27(&self) -> DataByte27R {
        DataByte27R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_26(&self) -> DataByte26R {
        DataByte26R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_25(&self) -> DataByte25R {
        DataByte25R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_24(&self) -> DataByte24R {
        DataByte24R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_27(&mut self) -> DataByte27W<Mb32bGroupMb5_32bWord6Spec> {
        DataByte27W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_26(&mut self) -> DataByte26W<Mb32bGroupMb5_32bWord6Spec> {
        DataByte26W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_25(&mut self) -> DataByte25W<Mb32bGroupMb5_32bWord6Spec> {
        DataByte25W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_24(&mut self) -> DataByte24W<Mb32bGroupMb5_32bWord6Spec> {
        DataByte24W::new(self, 24)
    }
}
#[doc = "Message Buffer 5 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb5_32b_word6::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb5_32b_word6::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mb32bGroupMb5_32bWord6Spec;
impl crate::RegisterSpec for Mb32bGroupMb5_32bWord6Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_32b_group_mb5_32b_word6::R`](R) reader structure"]
impl crate::Readable for Mb32bGroupMb5_32bWord6Spec {}
#[doc = "`write(|w| ..)` method takes [`mb_32b_group_mb5_32b_word6::W`](W) writer structure"]
impl crate::Writable for Mb32bGroupMb5_32bWord6Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MB5_32B_WORD6 to value 0"]
impl crate::Resettable for Mb32bGroupMb5_32bWord6Spec {}
