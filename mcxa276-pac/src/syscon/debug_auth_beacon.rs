#[doc = "Register `DEBUG_AUTH_BEACON` reader"]
pub type R = crate::R<DebugAuthBeaconSpec>;
#[doc = "Register `DEBUG_AUTH_BEACON` writer"]
pub type W = crate::W<DebugAuthBeaconSpec>;
#[doc = "Field `BEACON` reader - Sets by the debug authentication code in ROM to pass the debug beacons (Credential Beacon and Authentication Beacon) to the application code."]
pub type BeaconR = crate::FieldReader<u32>;
#[doc = "Field `BEACON` writer - Sets by the debug authentication code in ROM to pass the debug beacons (Credential Beacon and Authentication Beacon) to the application code."]
pub type BeaconW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Sets by the debug authentication code in ROM to pass the debug beacons (Credential Beacon and Authentication Beacon) to the application code."]
    #[inline(always)]
    pub fn beacon(&self) -> BeaconR {
        BeaconR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Sets by the debug authentication code in ROM to pass the debug beacons (Credential Beacon and Authentication Beacon) to the application code."]
    #[inline(always)]
    pub fn beacon(&mut self) -> BeaconW<DebugAuthBeaconSpec> {
        BeaconW::new(self, 0)
    }
}
#[doc = "Debug Authentication BEACON\n\nYou can [`read`](crate::Reg::read) this register and get [`debug_auth_beacon::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`debug_auth_beacon::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DebugAuthBeaconSpec;
impl crate::RegisterSpec for DebugAuthBeaconSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`debug_auth_beacon::R`](R) reader structure"]
impl crate::Readable for DebugAuthBeaconSpec {}
#[doc = "`write(|w| ..)` method takes [`debug_auth_beacon::W`](W) writer structure"]
impl crate::Writable for DebugAuthBeaconSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DEBUG_AUTH_BEACON to value 0"]
impl crate::Resettable for DebugAuthBeaconSpec {}
