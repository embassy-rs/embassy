#[doc = "Register `MB6_64B_WORD11` reader"]
pub type R = crate::R<Mb64bGroupMb6_64bWord11Spec>;
#[doc = "Register `MB6_64B_WORD11` writer"]
pub type W = crate::W<Mb64bGroupMb6_64bWord11Spec>;
#[doc = "Field `DATA_BYTE_47` reader - Data byte 0 of Rx/Tx frame."]
pub type DataByte47R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_47` writer - Data byte 0 of Rx/Tx frame."]
pub type DataByte47W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_46` reader - Data byte 1 of Rx/Tx frame."]
pub type DataByte46R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_46` writer - Data byte 1 of Rx/Tx frame."]
pub type DataByte46W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_45` reader - Data byte 2 of Rx/Tx frame."]
pub type DataByte45R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_45` writer - Data byte 2 of Rx/Tx frame."]
pub type DataByte45W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_44` reader - Data byte 3 of Rx/Tx frame."]
pub type DataByte44R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_44` writer - Data byte 3 of Rx/Tx frame."]
pub type DataByte44W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_47(&self) -> DataByte47R {
        DataByte47R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_46(&self) -> DataByte46R {
        DataByte46R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_45(&self) -> DataByte45R {
        DataByte45R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_44(&self) -> DataByte44R {
        DataByte44R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_47(&mut self) -> DataByte47W<Mb64bGroupMb6_64bWord11Spec> {
        DataByte47W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_46(&mut self) -> DataByte46W<Mb64bGroupMb6_64bWord11Spec> {
        DataByte46W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_45(&mut self) -> DataByte45W<Mb64bGroupMb6_64bWord11Spec> {
        DataByte45W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_44(&mut self) -> DataByte44W<Mb64bGroupMb6_64bWord11Spec> {
        DataByte44W::new(self, 24)
    }
}
#[doc = "Message Buffer 6 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb6_64b_word11::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb6_64b_word11::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mb64bGroupMb6_64bWord11Spec;
impl crate::RegisterSpec for Mb64bGroupMb6_64bWord11Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_64b_group_mb6_64b_word11::R`](R) reader structure"]
impl crate::Readable for Mb64bGroupMb6_64bWord11Spec {}
#[doc = "`write(|w| ..)` method takes [`mb_64b_group_mb6_64b_word11::W`](W) writer structure"]
impl crate::Writable for Mb64bGroupMb6_64bWord11Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MB6_64B_WORD11 to value 0"]
impl crate::Resettable for Mb64bGroupMb6_64bWord11Spec {}
