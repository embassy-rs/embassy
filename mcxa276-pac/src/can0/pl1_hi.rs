#[doc = "Register `PL1_HI` reader"]
pub type R = crate::R<Pl1HiSpec>;
#[doc = "Register `PL1_HI` writer"]
pub type W = crate::W<Pl1HiSpec>;
#[doc = "Field `Data_byte_7` reader - Data byte 7"]
pub type DataByte7R = crate::FieldReader;
#[doc = "Field `Data_byte_7` writer - Data byte 7"]
pub type DataByte7W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `Data_byte_6` reader - Data byte 6"]
pub type DataByte6R = crate::FieldReader;
#[doc = "Field `Data_byte_6` writer - Data byte 6"]
pub type DataByte6W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `Data_byte_5` reader - Data byte 5"]
pub type DataByte5R = crate::FieldReader;
#[doc = "Field `Data_byte_5` writer - Data byte 5"]
pub type DataByte5W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `Data_byte_4` reader - Data byte 4"]
pub type DataByte4R = crate::FieldReader;
#[doc = "Field `Data_byte_4` writer - Data byte 4"]
pub type DataByte4W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Data byte 7"]
    #[inline(always)]
    pub fn data_byte_7(&self) -> DataByte7R {
        DataByte7R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data byte 6"]
    #[inline(always)]
    pub fn data_byte_6(&self) -> DataByte6R {
        DataByte6R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data byte 5"]
    #[inline(always)]
    pub fn data_byte_5(&self) -> DataByte5R {
        DataByte5R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data byte 4"]
    #[inline(always)]
    pub fn data_byte_4(&self) -> DataByte4R {
        DataByte4R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data byte 7"]
    #[inline(always)]
    pub fn data_byte_7(&mut self) -> DataByte7W<Pl1HiSpec> {
        DataByte7W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data byte 6"]
    #[inline(always)]
    pub fn data_byte_6(&mut self) -> DataByte6W<Pl1HiSpec> {
        DataByte6W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data byte 5"]
    #[inline(always)]
    pub fn data_byte_5(&mut self) -> DataByte5W<Pl1HiSpec> {
        DataByte5W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data byte 4"]
    #[inline(always)]
    pub fn data_byte_4(&mut self) -> DataByte4W<Pl1HiSpec> {
        DataByte4W::new(self, 24)
    }
}
#[doc = "Pretended Networking Payload High Filter 1\n\nYou can [`read`](crate::Reg::read) this register and get [`pl1_hi::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pl1_hi::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pl1HiSpec;
impl crate::RegisterSpec for Pl1HiSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pl1_hi::R`](R) reader structure"]
impl crate::Readable for Pl1HiSpec {}
#[doc = "`write(|w| ..)` method takes [`pl1_hi::W`](W) writer structure"]
impl crate::Writable for Pl1HiSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PL1_HI to value 0"]
impl crate::Resettable for Pl1HiSpec {}
