#[doc = "Register `CCR` reader"]
pub type R = crate::R<CcrSpec>;
#[doc = "Register `CCR` writer"]
pub type W = crate::W<CcrSpec>;
#[doc = "Field `SCKDIV` reader - SCK Divider"]
pub type SckdivR = crate::FieldReader;
#[doc = "Field `SCKDIV` writer - SCK Divider"]
pub type SckdivW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `DBT` reader - Delay Between Transfers"]
pub type DbtR = crate::FieldReader;
#[doc = "Field `DBT` writer - Delay Between Transfers"]
pub type DbtW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `PCSSCK` reader - PCS-to-SCK Delay"]
pub type PcssckR = crate::FieldReader;
#[doc = "Field `PCSSCK` writer - PCS-to-SCK Delay"]
pub type PcssckW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `SCKPCS` reader - SCK-to-PCS Delay"]
pub type SckpcsR = crate::FieldReader;
#[doc = "Field `SCKPCS` writer - SCK-to-PCS Delay"]
pub type SckpcsW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - SCK Divider"]
    #[inline(always)]
    pub fn sckdiv(&self) -> SckdivR {
        SckdivR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Delay Between Transfers"]
    #[inline(always)]
    pub fn dbt(&self) -> DbtR {
        DbtR::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - PCS-to-SCK Delay"]
    #[inline(always)]
    pub fn pcssck(&self) -> PcssckR {
        PcssckR::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - SCK-to-PCS Delay"]
    #[inline(always)]
    pub fn sckpcs(&self) -> SckpcsR {
        SckpcsR::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - SCK Divider"]
    #[inline(always)]
    pub fn sckdiv(&mut self) -> SckdivW<CcrSpec> {
        SckdivW::new(self, 0)
    }
    #[doc = "Bits 8:15 - Delay Between Transfers"]
    #[inline(always)]
    pub fn dbt(&mut self) -> DbtW<CcrSpec> {
        DbtW::new(self, 8)
    }
    #[doc = "Bits 16:23 - PCS-to-SCK Delay"]
    #[inline(always)]
    pub fn pcssck(&mut self) -> PcssckW<CcrSpec> {
        PcssckW::new(self, 16)
    }
    #[doc = "Bits 24:31 - SCK-to-PCS Delay"]
    #[inline(always)]
    pub fn sckpcs(&mut self) -> SckpcsW<CcrSpec> {
        SckpcsW::new(self, 24)
    }
}
#[doc = "Clock Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`ccr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ccr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CcrSpec;
impl crate::RegisterSpec for CcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ccr::R`](R) reader structure"]
impl crate::Readable for CcrSpec {}
#[doc = "`write(|w| ..)` method takes [`ccr::W`](W) writer structure"]
impl crate::Writable for CcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CCR to value 0"]
impl crate::Resettable for CcrSpec {}
