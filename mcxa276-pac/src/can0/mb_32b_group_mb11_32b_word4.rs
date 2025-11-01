#[doc = "Register `MB11_32B_WORD4` reader"]
pub type R = crate::R<Mb32bGroupMb11_32bWord4Spec>;
#[doc = "Register `MB11_32B_WORD4` writer"]
pub type W = crate::W<Mb32bGroupMb11_32bWord4Spec>;
#[doc = "Field `DATA_BYTE_19` reader - Data byte 0 of Rx/Tx frame."]
pub type DataByte19R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_19` writer - Data byte 0 of Rx/Tx frame."]
pub type DataByte19W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_18` reader - Data byte 1 of Rx/Tx frame."]
pub type DataByte18R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_18` writer - Data byte 1 of Rx/Tx frame."]
pub type DataByte18W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_17` reader - Data byte 2 of Rx/Tx frame."]
pub type DataByte17R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_17` writer - Data byte 2 of Rx/Tx frame."]
pub type DataByte17W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_16` reader - Data byte 3 of Rx/Tx frame."]
pub type DataByte16R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_16` writer - Data byte 3 of Rx/Tx frame."]
pub type DataByte16W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_19(&self) -> DataByte19R {
        DataByte19R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_18(&self) -> DataByte18R {
        DataByte18R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_17(&self) -> DataByte17R {
        DataByte17R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_16(&self) -> DataByte16R {
        DataByte16R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_19(&mut self) -> DataByte19W<Mb32bGroupMb11_32bWord4Spec> {
        DataByte19W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_18(&mut self) -> DataByte18W<Mb32bGroupMb11_32bWord4Spec> {
        DataByte18W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_17(&mut self) -> DataByte17W<Mb32bGroupMb11_32bWord4Spec> {
        DataByte17W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_16(&mut self) -> DataByte16W<Mb32bGroupMb11_32bWord4Spec> {
        DataByte16W::new(self, 24)
    }
}
#[doc = "Message Buffer 11 WORD_32B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_32b_group_mb11_32b_word4::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_32b_group_mb11_32b_word4::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mb32bGroupMb11_32bWord4Spec;
impl crate::RegisterSpec for Mb32bGroupMb11_32bWord4Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_32b_group_mb11_32b_word4::R`](R) reader structure"]
impl crate::Readable for Mb32bGroupMb11_32bWord4Spec {}
#[doc = "`write(|w| ..)` method takes [`mb_32b_group_mb11_32b_word4::W`](W) writer structure"]
impl crate::Writable for Mb32bGroupMb11_32bWord4Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MB11_32B_WORD4 to value 0"]
impl crate::Resettable for Mb32bGroupMb11_32bWord4Spec {}
