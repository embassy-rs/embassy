#[doc = "Register `MB2_64B_WORD8` reader"]
pub type R = crate::R<Mb64bGroupMb2_64bWord8Spec>;
#[doc = "Register `MB2_64B_WORD8` writer"]
pub type W = crate::W<Mb64bGroupMb2_64bWord8Spec>;
#[doc = "Field `DATA_BYTE_35` reader - Data byte 0 of Rx/Tx frame."]
pub type DataByte35R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_35` writer - Data byte 0 of Rx/Tx frame."]
pub type DataByte35W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_34` reader - Data byte 1 of Rx/Tx frame."]
pub type DataByte34R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_34` writer - Data byte 1 of Rx/Tx frame."]
pub type DataByte34W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_33` reader - Data byte 2 of Rx/Tx frame."]
pub type DataByte33R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_33` writer - Data byte 2 of Rx/Tx frame."]
pub type DataByte33W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_32` reader - Data byte 3 of Rx/Tx frame."]
pub type DataByte32R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_32` writer - Data byte 3 of Rx/Tx frame."]
pub type DataByte32W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_35(&self) -> DataByte35R {
        DataByte35R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_34(&self) -> DataByte34R {
        DataByte34R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_33(&self) -> DataByte33R {
        DataByte33R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_32(&self) -> DataByte32R {
        DataByte32R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_35(&mut self) -> DataByte35W<Mb64bGroupMb2_64bWord8Spec> {
        DataByte35W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_34(&mut self) -> DataByte34W<Mb64bGroupMb2_64bWord8Spec> {
        DataByte34W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_33(&mut self) -> DataByte33W<Mb64bGroupMb2_64bWord8Spec> {
        DataByte33W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_32(&mut self) -> DataByte32W<Mb64bGroupMb2_64bWord8Spec> {
        DataByte32W::new(self, 24)
    }
}
#[doc = "Message Buffer 2 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb2_64b_word8::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb2_64b_word8::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mb64bGroupMb2_64bWord8Spec;
impl crate::RegisterSpec for Mb64bGroupMb2_64bWord8Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_64b_group_mb2_64b_word8::R`](R) reader structure"]
impl crate::Readable for Mb64bGroupMb2_64bWord8Spec {}
#[doc = "`write(|w| ..)` method takes [`mb_64b_group_mb2_64b_word8::W`](W) writer structure"]
impl crate::Writable for Mb64bGroupMb2_64bWord8Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MB2_64B_WORD8 to value 0"]
impl crate::Resettable for Mb64bGroupMb2_64bWord8Spec {}
