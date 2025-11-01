#[doc = "Register `DEBUG_FEATURES_DP` reader"]
pub type R = crate::R<DebugFeaturesDpSpec>;
#[doc = "Register `DEBUG_FEATURES_DP` writer"]
pub type W = crate::W<DebugFeaturesDpSpec>;
#[doc = "CPU0 invasive debug control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Cpu0Dbgen {
    #[doc = "1: Disables debug"]
    Disable = 1,
    #[doc = "2: Enables debug"]
    Enable = 2,
}
impl From<Cpu0Dbgen> for u8 {
    #[inline(always)]
    fn from(variant: Cpu0Dbgen) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Cpu0Dbgen {
    type Ux = u8;
}
impl crate::IsEnum for Cpu0Dbgen {}
#[doc = "Field `CPU0_DBGEN` reader - CPU0 invasive debug control"]
pub type Cpu0DbgenR = crate::FieldReader<Cpu0Dbgen>;
impl Cpu0DbgenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Cpu0Dbgen> {
        match self.bits {
            1 => Some(Cpu0Dbgen::Disable),
            2 => Some(Cpu0Dbgen::Enable),
            _ => None,
        }
    }
    #[doc = "Disables debug"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Cpu0Dbgen::Disable
    }
    #[doc = "Enables debug"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Cpu0Dbgen::Enable
    }
}
#[doc = "Field `CPU0_DBGEN` writer - CPU0 invasive debug control"]
pub type Cpu0DbgenW<'a, REG> = crate::FieldWriter<'a, REG, 2, Cpu0Dbgen>;
impl<'a, REG> Cpu0DbgenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disables debug"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Cpu0Dbgen::Disable)
    }
    #[doc = "Enables debug"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Cpu0Dbgen::Enable)
    }
}
#[doc = "CPU0 non-invasive debug control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Cpu0Niden {
    #[doc = "1: Disables debug"]
    Disable = 1,
    #[doc = "2: Enables debug"]
    Enable = 2,
}
impl From<Cpu0Niden> for u8 {
    #[inline(always)]
    fn from(variant: Cpu0Niden) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Cpu0Niden {
    type Ux = u8;
}
impl crate::IsEnum for Cpu0Niden {}
#[doc = "Field `CPU0_NIDEN` reader - CPU0 non-invasive debug control"]
pub type Cpu0NidenR = crate::FieldReader<Cpu0Niden>;
impl Cpu0NidenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Cpu0Niden> {
        match self.bits {
            1 => Some(Cpu0Niden::Disable),
            2 => Some(Cpu0Niden::Enable),
            _ => None,
        }
    }
    #[doc = "Disables debug"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Cpu0Niden::Disable
    }
    #[doc = "Enables debug"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Cpu0Niden::Enable
    }
}
#[doc = "Field `CPU0_NIDEN` writer - CPU0 non-invasive debug control"]
pub type Cpu0NidenW<'a, REG> = crate::FieldWriter<'a, REG, 2, Cpu0Niden>;
impl<'a, REG> Cpu0NidenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disables debug"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Cpu0Niden::Disable)
    }
    #[doc = "Enables debug"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Cpu0Niden::Enable)
    }
}
impl R {
    #[doc = "Bits 0:1 - CPU0 invasive debug control"]
    #[inline(always)]
    pub fn cpu0_dbgen(&self) -> Cpu0DbgenR {
        Cpu0DbgenR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:3 - CPU0 non-invasive debug control"]
    #[inline(always)]
    pub fn cpu0_niden(&self) -> Cpu0NidenR {
        Cpu0NidenR::new(((self.bits >> 2) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - CPU0 invasive debug control"]
    #[inline(always)]
    pub fn cpu0_dbgen(&mut self) -> Cpu0DbgenW<DebugFeaturesDpSpec> {
        Cpu0DbgenW::new(self, 0)
    }
    #[doc = "Bits 2:3 - CPU0 non-invasive debug control"]
    #[inline(always)]
    pub fn cpu0_niden(&mut self) -> Cpu0NidenW<DebugFeaturesDpSpec> {
        Cpu0NidenW::new(self, 2)
    }
}
#[doc = "Cortex Debug Features Control (Duplicate)\n\nYou can [`read`](crate::Reg::read) this register and get [`debug_features_dp::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`debug_features_dp::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DebugFeaturesDpSpec;
impl crate::RegisterSpec for DebugFeaturesDpSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`debug_features_dp::R`](R) reader structure"]
impl crate::Readable for DebugFeaturesDpSpec {}
#[doc = "`write(|w| ..)` method takes [`debug_features_dp::W`](W) writer structure"]
impl crate::Writable for DebugFeaturesDpSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DEBUG_FEATURES_DP to value 0"]
impl crate::Resettable for DebugFeaturesDpSpec {}
