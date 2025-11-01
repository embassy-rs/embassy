#[doc = "Register `WORD09` reader"]
pub type R = crate::R<MbGroupWord09Spec>;
#[doc = "Register `WORD09` writer"]
pub type W = crate::W<MbGroupWord09Spec>;
#[doc = "Field `DATA_BYTE_3` reader - Data byte 0 of Rx/Tx frame."]
pub type DataByte3R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_3` writer - Data byte 0 of Rx/Tx frame."]
pub type DataByte3W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_2` reader - Data byte 1 of Rx/Tx frame."]
pub type DataByte2R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_2` writer - Data byte 1 of Rx/Tx frame."]
pub type DataByte2W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_1` reader - Data byte 2 of Rx/Tx frame."]
pub type DataByte1R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_1` writer - Data byte 2 of Rx/Tx frame."]
pub type DataByte1W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_0` reader - Data byte 3 of Rx/Tx frame."]
pub type DataByte0R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_0` writer - Data byte 3 of Rx/Tx frame."]
pub type DataByte0W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_3(&self) -> DataByte3R {
        DataByte3R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_2(&self) -> DataByte2R {
        DataByte2R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_1(&self) -> DataByte1R {
        DataByte1R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_0(&self) -> DataByte0R {
        DataByte0R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_3(&mut self) -> DataByte3W<MbGroupWord09Spec> {
        DataByte3W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_2(&mut self) -> DataByte2W<MbGroupWord09Spec> {
        DataByte2W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_1(&mut self) -> DataByte1W<MbGroupWord09Spec> {
        DataByte1W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_0(&mut self) -> DataByte0W<MbGroupWord09Spec> {
        DataByte0W::new(self, 24)
    }
}
#[doc = "Message Buffer 9 WORD0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_word09::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_word09::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MbGroupWord09Spec;
impl crate::RegisterSpec for MbGroupWord09Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_group_word09::R`](R) reader structure"]
impl crate::Readable for MbGroupWord09Spec {}
#[doc = "`write(|w| ..)` method takes [`mb_group_word09::W`](W) writer structure"]
impl crate::Writable for MbGroupWord09Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets WORD09 to value 0"]
impl crate::Resettable for MbGroupWord09Spec {}
