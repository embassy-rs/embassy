#[doc = "Register `MB3_64B_WORD12` reader"]
pub type R = crate::R<Mb64bGroupMb3_64bWord12Spec>;
#[doc = "Register `MB3_64B_WORD12` writer"]
pub type W = crate::W<Mb64bGroupMb3_64bWord12Spec>;
#[doc = "Field `DATA_BYTE_51` reader - Data byte 0 of Rx/Tx frame."]
pub type DataByte51R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_51` writer - Data byte 0 of Rx/Tx frame."]
pub type DataByte51W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_50` reader - Data byte 1 of Rx/Tx frame."]
pub type DataByte50R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_50` writer - Data byte 1 of Rx/Tx frame."]
pub type DataByte50W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_49` reader - Data byte 2 of Rx/Tx frame."]
pub type DataByte49R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_49` writer - Data byte 2 of Rx/Tx frame."]
pub type DataByte49W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_48` reader - Data byte 3 of Rx/Tx frame."]
pub type DataByte48R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_48` writer - Data byte 3 of Rx/Tx frame."]
pub type DataByte48W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_51(&self) -> DataByte51R {
        DataByte51R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_50(&self) -> DataByte50R {
        DataByte50R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_49(&self) -> DataByte49R {
        DataByte49R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_48(&self) -> DataByte48R {
        DataByte48R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_51(&mut self) -> DataByte51W<Mb64bGroupMb3_64bWord12Spec> {
        DataByte51W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_50(&mut self) -> DataByte50W<Mb64bGroupMb3_64bWord12Spec> {
        DataByte50W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_49(&mut self) -> DataByte49W<Mb64bGroupMb3_64bWord12Spec> {
        DataByte49W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_48(&mut self) -> DataByte48W<Mb64bGroupMb3_64bWord12Spec> {
        DataByte48W::new(self, 24)
    }
}
#[doc = "Message Buffer 3 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb3_64b_word12::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb3_64b_word12::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mb64bGroupMb3_64bWord12Spec;
impl crate::RegisterSpec for Mb64bGroupMb3_64bWord12Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_64b_group_mb3_64b_word12::R`](R) reader structure"]
impl crate::Readable for Mb64bGroupMb3_64bWord12Spec {}
#[doc = "`write(|w| ..)` method takes [`mb_64b_group_mb3_64b_word12::W`](W) writer structure"]
impl crate::Writable for Mb64bGroupMb3_64bWord12Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MB3_64B_WORD12 to value 0"]
impl crate::Resettable for Mb64bGroupMb3_64bWord12Spec {}
