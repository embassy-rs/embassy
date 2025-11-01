#[doc = "Register `SM2TCTRL` reader"]
pub type R = crate::R<Sm2tctrlSpec>;
#[doc = "Register `SM2TCTRL` writer"]
pub type W = crate::W<Sm2tctrlSpec>;
#[doc = "Output Trigger Enables\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OutTrigEn {
    #[doc = "1: PWM_OUT_TRIG0 will set when the counter value matches the VAL0 value."]
    Val0 = 1,
}
impl From<OutTrigEn> for u8 {
    #[inline(always)]
    fn from(variant: OutTrigEn) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for OutTrigEn {
    type Ux = u8;
}
impl crate::IsEnum for OutTrigEn {}
#[doc = "Field `OUT_TRIG_EN` reader - Output Trigger Enables"]
pub type OutTrigEnR = crate::FieldReader<OutTrigEn>;
impl OutTrigEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<OutTrigEn> {
        match self.bits {
            1 => Some(OutTrigEn::Val0),
            _ => None,
        }
    }
    #[doc = "PWM_OUT_TRIG0 will set when the counter value matches the VAL0 value."]
    #[inline(always)]
    pub fn is_val0(&self) -> bool {
        *self == OutTrigEn::Val0
    }
}
#[doc = "Field `OUT_TRIG_EN` writer - Output Trigger Enables"]
pub type OutTrigEnW<'a, REG> = crate::FieldWriter<'a, REG, 6, OutTrigEn>;
impl<'a, REG> OutTrigEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "PWM_OUT_TRIG0 will set when the counter value matches the VAL0 value."]
    #[inline(always)]
    pub fn val0(self) -> &'a mut crate::W<REG> {
        self.variant(OutTrigEn::Val0)
    }
}
#[doc = "Trigger Frequency\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trgfrq {
    #[doc = "0: Trigger outputs are generated during every PWM period even if the PWM is not reloaded every period due to CTRL\\[LDFQ\\] being non-zero."]
    Everypwm = 0,
    #[doc = "1: Trigger outputs are generated only during the final PWM period prior to a reload opportunity when the PWM is not reloaded every period due to CTRL\\[LDFQ\\] being non-zero."]
    Finalpwm = 1,
}
impl From<Trgfrq> for bool {
    #[inline(always)]
    fn from(variant: Trgfrq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRGFRQ` reader - Trigger Frequency"]
pub type TrgfrqR = crate::BitReader<Trgfrq>;
impl TrgfrqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Trgfrq {
        match self.bits {
            false => Trgfrq::Everypwm,
            true => Trgfrq::Finalpwm,
        }
    }
    #[doc = "Trigger outputs are generated during every PWM period even if the PWM is not reloaded every period due to CTRL\\[LDFQ\\] being non-zero."]
    #[inline(always)]
    pub fn is_everypwm(&self) -> bool {
        *self == Trgfrq::Everypwm
    }
    #[doc = "Trigger outputs are generated only during the final PWM period prior to a reload opportunity when the PWM is not reloaded every period due to CTRL\\[LDFQ\\] being non-zero."]
    #[inline(always)]
    pub fn is_finalpwm(&self) -> bool {
        *self == Trgfrq::Finalpwm
    }
}
#[doc = "Field `TRGFRQ` writer - Trigger Frequency"]
pub type TrgfrqW<'a, REG> = crate::BitWriter<'a, REG, Trgfrq>;
impl<'a, REG> TrgfrqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Trigger outputs are generated during every PWM period even if the PWM is not reloaded every period due to CTRL\\[LDFQ\\] being non-zero."]
    #[inline(always)]
    pub fn everypwm(self) -> &'a mut crate::W<REG> {
        self.variant(Trgfrq::Everypwm)
    }
    #[doc = "Trigger outputs are generated only during the final PWM period prior to a reload opportunity when the PWM is not reloaded every period due to CTRL\\[LDFQ\\] being non-zero."]
    #[inline(always)]
    pub fn finalpwm(self) -> &'a mut crate::W<REG> {
        self.variant(Trgfrq::Finalpwm)
    }
}
#[doc = "Mux Output Trigger 1 Source Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pwbot1 {
    #[doc = "0: Route the PWM_OUT_TRIG1 signal to PWM_MUX_TRIG1 port."]
    PwmOutTrig1Signal = 0,
    #[doc = "1: Route the PWM_B output to the PWM_MUX_TRIG1 port."]
    PwmbOutput = 1,
}
impl From<Pwbot1> for bool {
    #[inline(always)]
    fn from(variant: Pwbot1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PWBOT1` reader - Mux Output Trigger 1 Source Select"]
pub type Pwbot1R = crate::BitReader<Pwbot1>;
impl Pwbot1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pwbot1 {
        match self.bits {
            false => Pwbot1::PwmOutTrig1Signal,
            true => Pwbot1::PwmbOutput,
        }
    }
    #[doc = "Route the PWM_OUT_TRIG1 signal to PWM_MUX_TRIG1 port."]
    #[inline(always)]
    pub fn is_pwm_out_trig1_signal(&self) -> bool {
        *self == Pwbot1::PwmOutTrig1Signal
    }
    #[doc = "Route the PWM_B output to the PWM_MUX_TRIG1 port."]
    #[inline(always)]
    pub fn is_pwmb_output(&self) -> bool {
        *self == Pwbot1::PwmbOutput
    }
}
#[doc = "Field `PWBOT1` writer - Mux Output Trigger 1 Source Select"]
pub type Pwbot1W<'a, REG> = crate::BitWriter<'a, REG, Pwbot1>;
impl<'a, REG> Pwbot1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Route the PWM_OUT_TRIG1 signal to PWM_MUX_TRIG1 port."]
    #[inline(always)]
    pub fn pwm_out_trig1_signal(self) -> &'a mut crate::W<REG> {
        self.variant(Pwbot1::PwmOutTrig1Signal)
    }
    #[doc = "Route the PWM_B output to the PWM_MUX_TRIG1 port."]
    #[inline(always)]
    pub fn pwmb_output(self) -> &'a mut crate::W<REG> {
        self.variant(Pwbot1::PwmbOutput)
    }
}
#[doc = "Mux Output Trigger 0 Source Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pwaot0 {
    #[doc = "0: Route the PWM_OUT_TRIG0 signal to PWM_MUX_TRIG0 port."]
    PwmOutTrig0Signal = 0,
    #[doc = "1: Route the PWM_A output to the PWM_MUX_TRIG0 port."]
    PwmaOutput = 1,
}
impl From<Pwaot0> for bool {
    #[inline(always)]
    fn from(variant: Pwaot0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PWAOT0` reader - Mux Output Trigger 0 Source Select"]
pub type Pwaot0R = crate::BitReader<Pwaot0>;
impl Pwaot0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pwaot0 {
        match self.bits {
            false => Pwaot0::PwmOutTrig0Signal,
            true => Pwaot0::PwmaOutput,
        }
    }
    #[doc = "Route the PWM_OUT_TRIG0 signal to PWM_MUX_TRIG0 port."]
    #[inline(always)]
    pub fn is_pwm_out_trig0_signal(&self) -> bool {
        *self == Pwaot0::PwmOutTrig0Signal
    }
    #[doc = "Route the PWM_A output to the PWM_MUX_TRIG0 port."]
    #[inline(always)]
    pub fn is_pwma_output(&self) -> bool {
        *self == Pwaot0::PwmaOutput
    }
}
#[doc = "Field `PWAOT0` writer - Mux Output Trigger 0 Source Select"]
pub type Pwaot0W<'a, REG> = crate::BitWriter<'a, REG, Pwaot0>;
impl<'a, REG> Pwaot0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Route the PWM_OUT_TRIG0 signal to PWM_MUX_TRIG0 port."]
    #[inline(always)]
    pub fn pwm_out_trig0_signal(self) -> &'a mut crate::W<REG> {
        self.variant(Pwaot0::PwmOutTrig0Signal)
    }
    #[doc = "Route the PWM_A output to the PWM_MUX_TRIG0 port."]
    #[inline(always)]
    pub fn pwma_output(self) -> &'a mut crate::W<REG> {
        self.variant(Pwaot0::PwmaOutput)
    }
}
impl R {
    #[doc = "Bits 0:5 - Output Trigger Enables"]
    #[inline(always)]
    pub fn out_trig_en(&self) -> OutTrigEnR {
        OutTrigEnR::new((self.bits & 0x3f) as u8)
    }
    #[doc = "Bit 12 - Trigger Frequency"]
    #[inline(always)]
    pub fn trgfrq(&self) -> TrgfrqR {
        TrgfrqR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 14 - Mux Output Trigger 1 Source Select"]
    #[inline(always)]
    pub fn pwbot1(&self) -> Pwbot1R {
        Pwbot1R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Mux Output Trigger 0 Source Select"]
    #[inline(always)]
    pub fn pwaot0(&self) -> Pwaot0R {
        Pwaot0R::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:5 - Output Trigger Enables"]
    #[inline(always)]
    pub fn out_trig_en(&mut self) -> OutTrigEnW<Sm2tctrlSpec> {
        OutTrigEnW::new(self, 0)
    }
    #[doc = "Bit 12 - Trigger Frequency"]
    #[inline(always)]
    pub fn trgfrq(&mut self) -> TrgfrqW<Sm2tctrlSpec> {
        TrgfrqW::new(self, 12)
    }
    #[doc = "Bit 14 - Mux Output Trigger 1 Source Select"]
    #[inline(always)]
    pub fn pwbot1(&mut self) -> Pwbot1W<Sm2tctrlSpec> {
        Pwbot1W::new(self, 14)
    }
    #[doc = "Bit 15 - Mux Output Trigger 0 Source Select"]
    #[inline(always)]
    pub fn pwaot0(&mut self) -> Pwaot0W<Sm2tctrlSpec> {
        Pwaot0W::new(self, 15)
    }
}
#[doc = "Output Trigger Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2tctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2tctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm2tctrlSpec;
impl crate::RegisterSpec for Sm2tctrlSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm2tctrl::R`](R) reader structure"]
impl crate::Readable for Sm2tctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`sm2tctrl::W`](W) writer structure"]
impl crate::Writable for Sm2tctrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM2TCTRL to value 0"]
impl crate::Resettable for Sm2tctrlSpec {}
