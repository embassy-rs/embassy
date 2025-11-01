#[doc = "Register `DATA` reader"]
pub type R = crate::R<DataSpec>;
#[doc = "Register `DATA` writer"]
pub type W = crate::W<DataSpec>;
#[doc = "Field `LL` reader - Lower Part of Low Byte"]
pub type LlR = crate::FieldReader;
#[doc = "Field `LL` writer - Lower Part of Low Byte"]
pub type LlW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `LU` reader - Upper Part of Low Byte"]
pub type LuR = crate::FieldReader;
#[doc = "Field `LU` writer - Upper Part of Low Byte"]
pub type LuW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `HL` reader - Lower Part of High Byte"]
pub type HlR = crate::FieldReader;
#[doc = "Field `HL` writer - Lower Part of High Byte"]
pub type HlW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `HU` reader - Upper Part of High Byte"]
pub type HuR = crate::FieldReader;
#[doc = "Field `HU` writer - Upper Part of High Byte"]
pub type HuW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Lower Part of Low Byte"]
    #[inline(always)]
    pub fn ll(&self) -> LlR {
        LlR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Upper Part of Low Byte"]
    #[inline(always)]
    pub fn lu(&self) -> LuR {
        LuR::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Lower Part of High Byte"]
    #[inline(always)]
    pub fn hl(&self) -> HlR {
        HlR::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Upper Part of High Byte"]
    #[inline(always)]
    pub fn hu(&self) -> HuR {
        HuR::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Lower Part of Low Byte"]
    #[inline(always)]
    pub fn ll(&mut self) -> LlW<DataSpec> {
        LlW::new(self, 0)
    }
    #[doc = "Bits 8:15 - Upper Part of Low Byte"]
    #[inline(always)]
    pub fn lu(&mut self) -> LuW<DataSpec> {
        LuW::new(self, 8)
    }
    #[doc = "Bits 16:23 - Lower Part of High Byte"]
    #[inline(always)]
    pub fn hl(&mut self) -> HlW<DataSpec> {
        HlW::new(self, 16)
    }
    #[doc = "Bits 24:31 - Upper Part of High Byte"]
    #[inline(always)]
    pub fn hu(&mut self) -> HuW<DataSpec> {
        HuW::new(self, 24)
    }
}
#[doc = "Data\n\nYou can [`read`](crate::Reg::read) this register and get [`data::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`data::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DataSpec;
impl crate::RegisterSpec for DataSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`data::R`](R) reader structure"]
impl crate::Readable for DataSpec {}
#[doc = "`write(|w| ..)` method takes [`data::W`](W) writer structure"]
impl crate::Writable for DataSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DATA to value 0xffff_ffff"]
impl crate::Resettable for DataSpec {
    const RESET_VALUE: u32 = 0xffff_ffff;
}
