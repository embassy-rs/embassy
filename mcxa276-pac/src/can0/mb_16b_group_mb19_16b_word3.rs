#[doc = "Register `MB19_16B_WORD3` reader"]
pub type R = crate::R<Mb16bGroupMb19_16bWord3Spec>;
#[doc = "Register `MB19_16B_WORD3` writer"]
pub type W = crate::W<Mb16bGroupMb19_16bWord3Spec>;
#[doc = "Field `DATA_BYTE_15` reader - Data byte 0 of Rx/Tx frame."]
pub type DataByte15R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_15` writer - Data byte 0 of Rx/Tx frame."]
pub type DataByte15W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_14` reader - Data byte 1 of Rx/Tx frame."]
pub type DataByte14R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_14` writer - Data byte 1 of Rx/Tx frame."]
pub type DataByte14W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_13` reader - Data byte 2 of Rx/Tx frame."]
pub type DataByte13R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_13` writer - Data byte 2 of Rx/Tx frame."]
pub type DataByte13W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_12` reader - Data byte 3 of Rx/Tx frame."]
pub type DataByte12R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_12` writer - Data byte 3 of Rx/Tx frame."]
pub type DataByte12W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_15(&self) -> DataByte15R {
        DataByte15R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_14(&self) -> DataByte14R {
        DataByte14R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_13(&self) -> DataByte13R {
        DataByte13R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_12(&self) -> DataByte12R {
        DataByte12R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_15(&mut self) -> DataByte15W<Mb16bGroupMb19_16bWord3Spec> {
        DataByte15W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_14(&mut self) -> DataByte14W<Mb16bGroupMb19_16bWord3Spec> {
        DataByte14W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_13(&mut self) -> DataByte13W<Mb16bGroupMb19_16bWord3Spec> {
        DataByte13W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_12(&mut self) -> DataByte12W<Mb16bGroupMb19_16bWord3Spec> {
        DataByte12W::new(self, 24)
    }
}
#[doc = "Message Buffer 19 WORD_16B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb19_16b_word3::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb19_16b_word3::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mb16bGroupMb19_16bWord3Spec;
impl crate::RegisterSpec for Mb16bGroupMb19_16bWord3Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_16b_group_mb19_16b_word3::R`](R) reader structure"]
impl crate::Readable for Mb16bGroupMb19_16bWord3Spec {}
#[doc = "`write(|w| ..)` method takes [`mb_16b_group_mb19_16b_word3::W`](W) writer structure"]
impl crate::Writable for Mb16bGroupMb19_16bWord3Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MB19_16B_WORD3 to value 0"]
impl crate::Resettable for Mb16bGroupMb19_16bWord3Spec {}
