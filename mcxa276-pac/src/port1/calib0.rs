#[doc = "Register `CALIB0` reader"]
pub type R = crate::R<Calib0Spec>;
#[doc = "Register `CALIB0` writer"]
pub type W = crate::W<Calib0Spec>;
#[doc = "Field `NCAL` reader - Calibration of NMOS Output Driver"]
pub type NcalR = crate::FieldReader;
#[doc = "Field `NCAL` writer - Calibration of NMOS Output Driver"]
pub type NcalW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `PCAL` reader - Calibration of PMOS Output Driver"]
pub type PcalR = crate::FieldReader;
#[doc = "Field `PCAL` writer - Calibration of PMOS Output Driver"]
pub type PcalW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
impl R {
    #[doc = "Bits 0:5 - Calibration of NMOS Output Driver"]
    #[inline(always)]
    pub fn ncal(&self) -> NcalR {
        NcalR::new((self.bits & 0x3f) as u8)
    }
    #[doc = "Bits 16:21 - Calibration of PMOS Output Driver"]
    #[inline(always)]
    pub fn pcal(&self) -> PcalR {
        PcalR::new(((self.bits >> 16) & 0x3f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:5 - Calibration of NMOS Output Driver"]
    #[inline(always)]
    pub fn ncal(&mut self) -> NcalW<Calib0Spec> {
        NcalW::new(self, 0)
    }
    #[doc = "Bits 16:21 - Calibration of PMOS Output Driver"]
    #[inline(always)]
    pub fn pcal(&mut self) -> PcalW<Calib0Spec> {
        PcalW::new(self, 16)
    }
}
#[doc = "Calibration 0\n\nYou can [`read`](crate::Reg::read) this register and get [`calib0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`calib0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Calib0Spec;
impl crate::RegisterSpec for Calib0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`calib0::R`](R) reader structure"]
impl crate::Readable for Calib0Spec {}
#[doc = "`write(|w| ..)` method takes [`calib0::W`](W) writer structure"]
impl crate::Writable for Calib0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CALIB0 to value 0"]
impl crate::Resettable for Calib0Spec {}
