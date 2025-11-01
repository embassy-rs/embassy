#[doc = "Register `MB4_64B_WORD14` reader"]
pub type R = crate::R<Mb64bGroupMb4_64bWord14Spec>;
#[doc = "Register `MB4_64B_WORD14` writer"]
pub type W = crate::W<Mb64bGroupMb4_64bWord14Spec>;
#[doc = "Field `DATA_BYTE_59` reader - Data byte 0 of Rx/Tx frame."]
pub type DataByte59R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_59` writer - Data byte 0 of Rx/Tx frame."]
pub type DataByte59W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_58` reader - Data byte 1 of Rx/Tx frame."]
pub type DataByte58R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_58` writer - Data byte 1 of Rx/Tx frame."]
pub type DataByte58W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_57` reader - Data byte 2 of Rx/Tx frame."]
pub type DataByte57R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_57` writer - Data byte 2 of Rx/Tx frame."]
pub type DataByte57W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DATA_BYTE_56` reader - Data byte 3 of Rx/Tx frame."]
pub type DataByte56R = crate::FieldReader;
#[doc = "Field `DATA_BYTE_56` writer - Data byte 3 of Rx/Tx frame."]
pub type DataByte56W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_59(&self) -> DataByte59R {
        DataByte59R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_58(&self) -> DataByte58R {
        DataByte58R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_57(&self) -> DataByte57R {
        DataByte57R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_56(&self) -> DataByte56R {
        DataByte56R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data byte 0 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_59(&mut self) -> DataByte59W<Mb64bGroupMb4_64bWord14Spec> {
        DataByte59W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data byte 1 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_58(&mut self) -> DataByte58W<Mb64bGroupMb4_64bWord14Spec> {
        DataByte58W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data byte 2 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_57(&mut self) -> DataByte57W<Mb64bGroupMb4_64bWord14Spec> {
        DataByte57W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data byte 3 of Rx/Tx frame."]
    #[inline(always)]
    pub fn data_byte_56(&mut self) -> DataByte56W<Mb64bGroupMb4_64bWord14Spec> {
        DataByte56W::new(self, 24)
    }
}
#[doc = "Message Buffer 4 WORD_64B Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_64b_group_mb4_64b_word14::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_64b_group_mb4_64b_word14::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mb64bGroupMb4_64bWord14Spec;
impl crate::RegisterSpec for Mb64bGroupMb4_64bWord14Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_64b_group_mb4_64b_word14::R`](R) reader structure"]
impl crate::Readable for Mb64bGroupMb4_64bWord14Spec {}
#[doc = "`write(|w| ..)` method takes [`mb_64b_group_mb4_64b_word14::W`](W) writer structure"]
impl crate::Writable for Mb64bGroupMb4_64bWord14Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MB4_64B_WORD14 to value 0"]
impl crate::Resettable for Mb64bGroupMb4_64bWord14Spec {}
