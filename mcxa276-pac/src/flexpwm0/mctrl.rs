#[doc = "Register `MCTRL` reader"]
pub type R = crate::R<MctrlSpec>;
#[doc = "Register `MCTRL` writer"]
pub type W = crate::W<MctrlSpec>;
#[doc = "Load Okay\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Ldok {
    #[doc = "0: Do not load new values."]
    Disabled = 0,
    #[doc = "1: Load prescaler, modulus, and PWM values of the corresponding submodule."]
    Enabled = 1,
}
impl From<Ldok> for u8 {
    #[inline(always)]
    fn from(variant: Ldok) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Ldok {
    type Ux = u8;
}
impl crate::IsEnum for Ldok {}
#[doc = "Field `LDOK` reader - Load Okay"]
pub type LdokR = crate::FieldReader<Ldok>;
impl LdokR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Ldok> {
        match self.bits {
            0 => Some(Ldok::Disabled),
            1 => Some(Ldok::Enabled),
            _ => None,
        }
    }
    #[doc = "Do not load new values."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ldok::Disabled
    }
    #[doc = "Load prescaler, modulus, and PWM values of the corresponding submodule."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ldok::Enabled
    }
}
#[doc = "Field `LDOK` writer - Load Okay"]
pub type LdokW<'a, REG> = crate::FieldWriter<'a, REG, 4, Ldok>;
impl<'a, REG> LdokW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Do not load new values."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ldok::Disabled)
    }
    #[doc = "Load prescaler, modulus, and PWM values of the corresponding submodule."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ldok::Enabled)
    }
}
#[doc = "Field `CLDOK` reader - Clear Load Okay"]
pub type CldokR = crate::FieldReader;
#[doc = "Field `CLDOK` writer - Clear Load Okay"]
pub type CldokW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Run\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Run {
    #[doc = "0: PWM counter is stopped, but PWM outputs hold the current state."]
    Disabled = 0,
    #[doc = "1: PWM counter is started in the corresponding submodule."]
    Enabled = 1,
}
impl From<Run> for u8 {
    #[inline(always)]
    fn from(variant: Run) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Run {
    type Ux = u8;
}
impl crate::IsEnum for Run {}
#[doc = "Field `RUN` reader - Run"]
pub type RunR = crate::FieldReader<Run>;
impl RunR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Run> {
        match self.bits {
            0 => Some(Run::Disabled),
            1 => Some(Run::Enabled),
            _ => None,
        }
    }
    #[doc = "PWM counter is stopped, but PWM outputs hold the current state."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Run::Disabled
    }
    #[doc = "PWM counter is started in the corresponding submodule."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Run::Enabled
    }
}
#[doc = "Field `RUN` writer - Run"]
pub type RunW<'a, REG> = crate::FieldWriter<'a, REG, 4, Run>;
impl<'a, REG> RunW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "PWM counter is stopped, but PWM outputs hold the current state."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Run::Disabled)
    }
    #[doc = "PWM counter is started in the corresponding submodule."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Run::Enabled)
    }
}
#[doc = "Current Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Ipol {
    #[doc = "0: PWM23 is used to generate complementary PWM pair in the corresponding submodule."]
    Pwm23 = 0,
    #[doc = "1: PWM45 is used to generate complementary PWM pair in the corresponding submodule."]
    Pwm45 = 1,
}
impl From<Ipol> for u8 {
    #[inline(always)]
    fn from(variant: Ipol) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Ipol {
    type Ux = u8;
}
impl crate::IsEnum for Ipol {}
#[doc = "Field `IPOL` reader - Current Polarity"]
pub type IpolR = crate::FieldReader<Ipol>;
impl IpolR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Ipol> {
        match self.bits {
            0 => Some(Ipol::Pwm23),
            1 => Some(Ipol::Pwm45),
            _ => None,
        }
    }
    #[doc = "PWM23 is used to generate complementary PWM pair in the corresponding submodule."]
    #[inline(always)]
    pub fn is_pwm23(&self) -> bool {
        *self == Ipol::Pwm23
    }
    #[doc = "PWM45 is used to generate complementary PWM pair in the corresponding submodule."]
    #[inline(always)]
    pub fn is_pwm45(&self) -> bool {
        *self == Ipol::Pwm45
    }
}
#[doc = "Field `IPOL` writer - Current Polarity"]
pub type IpolW<'a, REG> = crate::FieldWriter<'a, REG, 4, Ipol>;
impl<'a, REG> IpolW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "PWM23 is used to generate complementary PWM pair in the corresponding submodule."]
    #[inline(always)]
    pub fn pwm23(self) -> &'a mut crate::W<REG> {
        self.variant(Ipol::Pwm23)
    }
    #[doc = "PWM45 is used to generate complementary PWM pair in the corresponding submodule."]
    #[inline(always)]
    pub fn pwm45(self) -> &'a mut crate::W<REG> {
        self.variant(Ipol::Pwm45)
    }
}
impl R {
    #[doc = "Bits 0:3 - Load Okay"]
    #[inline(always)]
    pub fn ldok(&self) -> LdokR {
        LdokR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 4:7 - Clear Load Okay"]
    #[inline(always)]
    pub fn cldok(&self) -> CldokR {
        CldokR::new(((self.bits >> 4) & 0x0f) as u8)
    }
    #[doc = "Bits 8:11 - Run"]
    #[inline(always)]
    pub fn run(&self) -> RunR {
        RunR::new(((self.bits >> 8) & 0x0f) as u8)
    }
    #[doc = "Bits 12:15 - Current Polarity"]
    #[inline(always)]
    pub fn ipol(&self) -> IpolR {
        IpolR::new(((self.bits >> 12) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Load Okay"]
    #[inline(always)]
    pub fn ldok(&mut self) -> LdokW<MctrlSpec> {
        LdokW::new(self, 0)
    }
    #[doc = "Bits 4:7 - Clear Load Okay"]
    #[inline(always)]
    pub fn cldok(&mut self) -> CldokW<MctrlSpec> {
        CldokW::new(self, 4)
    }
    #[doc = "Bits 8:11 - Run"]
    #[inline(always)]
    pub fn run(&mut self) -> RunW<MctrlSpec> {
        RunW::new(self, 8)
    }
    #[doc = "Bits 12:15 - Current Polarity"]
    #[inline(always)]
    pub fn ipol(&mut self) -> IpolW<MctrlSpec> {
        IpolW::new(self, 12)
    }
}
#[doc = "Master Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MctrlSpec;
impl crate::RegisterSpec for MctrlSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`mctrl::R`](R) reader structure"]
impl crate::Readable for MctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`mctrl::W`](W) writer structure"]
impl crate::Writable for MctrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MCTRL to value 0"]
impl crate::Resettable for MctrlSpec {}
