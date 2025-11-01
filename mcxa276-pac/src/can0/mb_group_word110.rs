#[doc = "Register `WORD110` reader"]
pub type R = crate::R<MbGroupWord110Spec>;
#[doc = "Register `WORD110` writer"]
pub type W = crate::W<MbGroupWord110Spec>;
#[doc = "Field `DATA_BYTE_7` reader - Data byte 0 of Rx/Tx frame."]
pub type DataByte7R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_7` writer - Data byte 0 of Rx/Tx frame."]
pub type DataByte7W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_6` reader - Data byte 1 of Rx/Tx frame."]
pub type DataByte6R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_6` writer - Data byte 1 of Rx/Tx frame."]
pub type DataByte6W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_5` reader - Data byte 2 of Rx/Tx frame."]
pub type DataByte5R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_5` writer - Data byte 2 of Rx/Tx frame."]
pub type DataByte5W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_4` reader - Data byte 3 of Rx/Tx frame."]
pub type DataByte4R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_4` writer - Data byte 3 of Rx/Tx frame."]
pub type DataByte4W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_7(&self) -> DataByte7R {
        DataByte7R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_6(&self) -> DataByte6R {
        DataByte6R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_5(&self) -> DataByte5R {
        DataByte5R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_4(&self) -> DataByte4R {
        DataByte4R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_7(&mut self) -> DataByte7W<MbGroupWord110Spec> {
        DataByte7W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_6(&mut self) -> DataByte6W<MbGroupWord110Spec> {
        DataByte6W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_5(&mut self) -> DataByte5W<MbGroupWord110Spec> {
        DataByte5W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_4(&mut self) -> DataByte4W<MbGroupWord110Spec> {
        DataByte4W::new(self, 24)
    }
}
#[doc = "Message Buffer 10 WORD1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word110::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word110::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MbGroupWord110Spec;
impl crate::RegisterSpec for MbGroupWord110Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_group_word110::R`](R) reader structure"]
impl crate::Readable for MbGroupWord110Spec {}
#[doc = "`write(|w| ..)` method takes [`mb_group_word110::W`](W) writer structure"]
impl crate::Writable for MbGroupWord110Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets WORD110 to value 0"]
impl crate::Resettable for MbGroupWord110Spec {}
