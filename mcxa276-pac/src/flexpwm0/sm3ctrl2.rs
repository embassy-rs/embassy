#[doc = "Register `SM3CTRL2` reader"]
pub type R = crate::R<Sm3ctrl2Spec>;
#[doc = "Register `SM3CTRL2` writer"]
pub type W = crate::W<Sm3ctrl2Spec>;
#[doc = "Clock Source Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ClkSel {
    #[doc = "0: The IPBus clock is used as the clock for the local prescaler and counter."]
    Ipbus = 0,
    #[doc = "1: EXT_CLK is used as the clock for the local prescaler and counter."]
    ExtClk = 1,
    #[doc = "2: Submodule 0's clock (AUX_CLK) is used as the source clock for the local prescaler and counter. This setting should not be used in submodule 0 as it forces the clock to logic 0."]
    AuxClk = 2,
}
impl From<ClkSel> for u8 {
    #[inline(always)]
    fn from(variant: ClkSel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for ClkSel {
    type Ux = u8;
}
impl crate::IsEnum for ClkSel {}
#[doc = "Field `CLK_SEL` reader - Clock Source Select"]
pub type ClkSelR = crate::FieldReader<ClkSel>;
impl ClkSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<ClkSel> {
        match self.bits {
            0 => Some(ClkSel::Ipbus),
            1 => Some(ClkSel::ExtClk),
            2 => Some(ClkSel::AuxClk),
            _ => None,
        }
    }
    #[doc = "The IPBus clock is used as the clock for the local prescaler and counter."]
    #[inline(always)]
    pub fn is_ipbus(&self) -> bool {
        *self == ClkSel::Ipbus
    }
    #[doc = "EXT_CLK is used as the clock for the local prescaler and counter."]
    #[inline(always)]
    pub fn is_ext_clk(&self) -> bool {
        *self == ClkSel::ExtClk
    }
    #[doc = "Submodule 0's clock (AUX_CLK) is used as the source clock for the local prescaler and counter. This setting should not be used in submodule 0 as it forces the clock to logic 0."]
    #[inline(always)]
    pub fn is_aux_clk(&self) -> bool {
        *self == ClkSel::AuxClk
    }
}
#[doc = "Field `CLK_SEL` writer - Clock Source Select"]
pub type ClkSelW<'a, REG> = crate::FieldWriter<'a, REG, 2, ClkSel>;
impl<'a, REG> ClkSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "The IPBus clock is used as the clock for the local prescaler and counter."]
    #[inline(always)]
    pub fn ipbus(self) -> &'a mut crate::W<REG> {
        self.variant(ClkSel::Ipbus)
    }
    #[doc = "EXT_CLK is used as the clock for the local prescaler and counter."]
    #[inline(always)]
    pub fn ext_clk(self) -> &'a mut crate::W<REG> {
        self.variant(ClkSel::ExtClk)
    }
    #[doc = "Submodule 0's clock (AUX_CLK) is used as the source clock for the local prescaler and counter. This setting should not be used in submodule 0 as it forces the clock to logic 0."]
    #[inline(always)]
    pub fn aux_clk(self) -> &'a mut crate::W<REG> {
        self.variant(ClkSel::AuxClk)
    }
}
#[doc = "Reload Source Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReloadSel {
    #[doc = "0: The local RELOAD signal is used to reload registers."]
    Local = 0,
    #[doc = "1: The master RELOAD signal (from submodule 0) is used to reload registers. This setting should not be used in submodule 0 as it forces the RELOAD signal to logic 0."]
    Master = 1,
}
impl From<ReloadSel> for bool {
    #[inline(always)]
    fn from(variant: ReloadSel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RELOAD_SEL` reader - Reload Source Select"]
pub type ReloadSelR = crate::BitReader<ReloadSel>;
impl ReloadSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> ReloadSel {
        match self.bits {
            false => ReloadSel::Local,
            true => ReloadSel::Master,
        }
    }
    #[doc = "The local RELOAD signal is used to reload registers."]
    #[inline(always)]
    pub fn is_local(&self) -> bool {
        *self == ReloadSel::Local
    }
    #[doc = "The master RELOAD signal (from submodule 0) is used to reload registers. This setting should not be used in submodule 0 as it forces the RELOAD signal to logic 0."]
    #[inline(always)]
    pub fn is_master(&self) -> bool {
        *self == ReloadSel::Master
    }
}
#[doc = "Field `RELOAD_SEL` writer - Reload Source Select"]
pub type ReloadSelW<'a, REG> = crate::BitWriter<'a, REG, ReloadSel>;
impl<'a, REG> ReloadSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The local RELOAD signal is used to reload registers."]
    #[inline(always)]
    pub fn local(self) -> &'a mut crate::W<REG> {
        self.variant(ReloadSel::Local)
    }
    #[doc = "The master RELOAD signal (from submodule 0) is used to reload registers. This setting should not be used in submodule 0 as it forces the RELOAD signal to logic 0."]
    #[inline(always)]
    pub fn master(self) -> &'a mut crate::W<REG> {
        self.variant(ReloadSel::Master)
    }
}
#[doc = "Force Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ForceSel {
    #[doc = "0: The local force signal, CTRL2\\[FORCE\\], from this submodule is used to force updates."]
    Local = 0,
    #[doc = "1: The master force signal from submodule 0 is used to force updates. This setting should not be used in submodule 0 as it holds the FORCE OUTPUT signal to logic 0."]
    Master = 1,
    #[doc = "2: The local reload signal from this submodule is used to force updates without regard to the state of LDOK."]
    LocalReload = 2,
    #[doc = "3: The master reload signal from submodule0 is used to force updates if LDOK is set. This setting should not be used in submodule0 as it holds the FORCE OUTPUT signal to logic 0."]
    MasterReload = 3,
    #[doc = "4: The local sync signal from this submodule is used to force updates."]
    LocalSync = 4,
    #[doc = "5: The master sync signal from submodule0 is used to force updates. This setting should not be used in submodule0 as it holds the FORCE OUTPUT signal to logic 0."]
    MasterSync = 5,
    #[doc = "6: The external force signal, EXT_FORCE, from outside the PWM module causes updates."]
    ExtForce = 6,
    #[doc = "7: The external sync signal, EXT_SYNC, from outside the PWM module causes updates."]
    ExtSync = 7,
}
impl From<ForceSel> for u8 {
    #[inline(always)]
    fn from(variant: ForceSel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for ForceSel {
    type Ux = u8;
}
impl crate::IsEnum for ForceSel {}
#[doc = "Field `FORCE_SEL` reader - Force Select"]
pub type ForceSelR = crate::FieldReader<ForceSel>;
impl ForceSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> ForceSel {
        match self.bits {
            0 => ForceSel::Local,
            1 => ForceSel::Master,
            2 => ForceSel::LocalReload,
            3 => ForceSel::MasterReload,
            4 => ForceSel::LocalSync,
            5 => ForceSel::MasterSync,
            6 => ForceSel::ExtForce,
            7 => ForceSel::ExtSync,
            _ => unreachable!(),
        }
    }
    #[doc = "The local force signal, CTRL2\\[FORCE\\], from this submodule is used to force updates."]
    #[inline(always)]
    pub fn is_local(&self) -> bool {
        *self == ForceSel::Local
    }
    #[doc = "The master force signal from submodule 0 is used to force updates. This setting should not be used in submodule 0 as it holds the FORCE OUTPUT signal to logic 0."]
    #[inline(always)]
    pub fn is_master(&self) -> bool {
        *self == ForceSel::Master
    }
    #[doc = "The local reload signal from this submodule is used to force updates without regard to the state of LDOK."]
    #[inline(always)]
    pub fn is_local_reload(&self) -> bool {
        *self == ForceSel::LocalReload
    }
    #[doc = "The master reload signal from submodule0 is used to force updates if LDOK is set. This setting should not be used in submodule0 as it holds the FORCE OUTPUT signal to logic 0."]
    #[inline(always)]
    pub fn is_master_reload(&self) -> bool {
        *self == ForceSel::MasterReload
    }
    #[doc = "The local sync signal from this submodule is used to force updates."]
    #[inline(always)]
    pub fn is_local_sync(&self) -> bool {
        *self == ForceSel::LocalSync
    }
    #[doc = "The master sync signal from submodule0 is used to force updates. This setting should not be used in submodule0 as it holds the FORCE OUTPUT signal to logic 0."]
    #[inline(always)]
    pub fn is_master_sync(&self) -> bool {
        *self == ForceSel::MasterSync
    }
    #[doc = "The external force signal, EXT_FORCE, from outside the PWM module causes updates."]
    #[inline(always)]
    pub fn is_ext_force(&self) -> bool {
        *self == ForceSel::ExtForce
    }
    #[doc = "The external sync signal, EXT_SYNC, from outside the PWM module causes updates."]
    #[inline(always)]
    pub fn is_ext_sync(&self) -> bool {
        *self == ForceSel::ExtSync
    }
}
#[doc = "Field `FORCE_SEL` writer - Force Select"]
pub type ForceSelW<'a, REG> = crate::FieldWriter<'a, REG, 3, ForceSel, crate::Safe>;
impl<'a, REG> ForceSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "The local force signal, CTRL2\\[FORCE\\], from this submodule is used to force updates."]
    #[inline(always)]
    pub fn local(self) -> &'a mut crate::W<REG> {
        self.variant(ForceSel::Local)
    }
    #[doc = "The master force signal from submodule 0 is used to force updates. This setting should not be used in submodule 0 as it holds the FORCE OUTPUT signal to logic 0."]
    #[inline(always)]
    pub fn master(self) -> &'a mut crate::W<REG> {
        self.variant(ForceSel::Master)
    }
    #[doc = "The local reload signal from this submodule is used to force updates without regard to the state of LDOK."]
    #[inline(always)]
    pub fn local_reload(self) -> &'a mut crate::W<REG> {
        self.variant(ForceSel::LocalReload)
    }
    #[doc = "The master reload signal from submodule0 is used to force updates if LDOK is set. This setting should not be used in submodule0 as it holds the FORCE OUTPUT signal to logic 0."]
    #[inline(always)]
    pub fn master_reload(self) -> &'a mut crate::W<REG> {
        self.variant(ForceSel::MasterReload)
    }
    #[doc = "The local sync signal from this submodule is used to force updates."]
    #[inline(always)]
    pub fn local_sync(self) -> &'a mut crate::W<REG> {
        self.variant(ForceSel::LocalSync)
    }
    #[doc = "The master sync signal from submodule0 is used to force updates. This setting should not be used in submodule0 as it holds the FORCE OUTPUT signal to logic 0."]
    #[inline(always)]
    pub fn master_sync(self) -> &'a mut crate::W<REG> {
        self.variant(ForceSel::MasterSync)
    }
    #[doc = "The external force signal, EXT_FORCE, from outside the PWM module causes updates."]
    #[inline(always)]
    pub fn ext_force(self) -> &'a mut crate::W<REG> {
        self.variant(ForceSel::ExtForce)
    }
    #[doc = "The external sync signal, EXT_SYNC, from outside the PWM module causes updates."]
    #[inline(always)]
    pub fn ext_sync(self) -> &'a mut crate::W<REG> {
        self.variant(ForceSel::ExtSync)
    }
}
#[doc = "Field `FORCE` reader - Force Initialization"]
pub type ForceR = crate::BitReader;
#[doc = "Field `FORCE` writer - Force Initialization"]
pub type ForceW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Force Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frcen {
    #[doc = "0: Initialization from a FORCE_OUT is disabled."]
    Disabled = 0,
    #[doc = "1: Initialization from a FORCE_OUT is enabled."]
    Enabled = 1,
}
impl From<Frcen> for bool {
    #[inline(always)]
    fn from(variant: Frcen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FRCEN` reader - Force Enable"]
pub type FrcenR = crate::BitReader<Frcen>;
impl FrcenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Frcen {
        match self.bits {
            false => Frcen::Disabled,
            true => Frcen::Enabled,
        }
    }
    #[doc = "Initialization from a FORCE_OUT is disabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Frcen::Disabled
    }
    #[doc = "Initialization from a FORCE_OUT is enabled."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Frcen::Enabled
    }
}
#[doc = "Field `FRCEN` writer - Force Enable"]
pub type FrcenW<'a, REG> = crate::BitWriter<'a, REG, Frcen>;
impl<'a, REG> FrcenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Initialization from a FORCE_OUT is disabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Frcen::Disabled)
    }
    #[doc = "Initialization from a FORCE_OUT is enabled."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Frcen::Enabled)
    }
}
#[doc = "Initialization Control Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum InitSel {
    #[doc = "0: Local sync (PWM_X) causes initialization."]
    PwmX = 0,
    #[doc = "1: Master reload from submodule 0 causes initialization. This setting should not be used in submodule 0 as it forces the INIT signal to logic 0. The submodule counter will only re-initialize when a master reload occurs."]
    MasterReload = 1,
    #[doc = "2: Master sync from submodule 0 causes initialization. This setting should not be used in submodule 0 as it forces the INIT signal to logic 0."]
    MasterSync = 2,
    #[doc = "3: EXT_SYNC causes initialization."]
    ExtSync = 3,
}
impl From<InitSel> for u8 {
    #[inline(always)]
    fn from(variant: InitSel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for InitSel {
    type Ux = u8;
}
impl crate::IsEnum for InitSel {}
#[doc = "Field `INIT_SEL` reader - Initialization Control Select"]
pub type InitSelR = crate::FieldReader<InitSel>;
impl InitSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> InitSel {
        match self.bits {
            0 => InitSel::PwmX,
            1 => InitSel::MasterReload,
            2 => InitSel::MasterSync,
            3 => InitSel::ExtSync,
            _ => unreachable!(),
        }
    }
    #[doc = "Local sync (PWM_X) causes initialization."]
    #[inline(always)]
    pub fn is_pwm_x(&self) -> bool {
        *self == InitSel::PwmX
    }
    #[doc = "Master reload from submodule 0 causes initialization. This setting should not be used in submodule 0 as it forces the INIT signal to logic 0. The submodule counter will only re-initialize when a master reload occurs."]
    #[inline(always)]
    pub fn is_master_reload(&self) -> bool {
        *self == InitSel::MasterReload
    }
    #[doc = "Master sync from submodule 0 causes initialization. This setting should not be used in submodule 0 as it forces the INIT signal to logic 0."]
    #[inline(always)]
    pub fn is_master_sync(&self) -> bool {
        *self == InitSel::MasterSync
    }
    #[doc = "EXT_SYNC causes initialization."]
    #[inline(always)]
    pub fn is_ext_sync(&self) -> bool {
        *self == InitSel::ExtSync
    }
}
#[doc = "Field `INIT_SEL` writer - Initialization Control Select"]
pub type InitSelW<'a, REG> = crate::FieldWriter<'a, REG, 2, InitSel, crate::Safe>;
impl<'a, REG> InitSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Local sync (PWM_X) causes initialization."]
    #[inline(always)]
    pub fn pwm_x(self) -> &'a mut crate::W<REG> {
        self.variant(InitSel::PwmX)
    }
    #[doc = "Master reload from submodule 0 causes initialization. This setting should not be used in submodule 0 as it forces the INIT signal to logic 0. The submodule counter will only re-initialize when a master reload occurs."]
    #[inline(always)]
    pub fn master_reload(self) -> &'a mut crate::W<REG> {
        self.variant(InitSel::MasterReload)
    }
    #[doc = "Master sync from submodule 0 causes initialization. This setting should not be used in submodule 0 as it forces the INIT signal to logic 0."]
    #[inline(always)]
    pub fn master_sync(self) -> &'a mut crate::W<REG> {
        self.variant(InitSel::MasterSync)
    }
    #[doc = "EXT_SYNC causes initialization."]
    #[inline(always)]
    pub fn ext_sync(self) -> &'a mut crate::W<REG> {
        self.variant(InitSel::ExtSync)
    }
}
#[doc = "Field `PWMX_INIT` reader - PWM_X Initial Value"]
pub type PwmxInitR = crate::BitReader;
#[doc = "Field `PWMX_INIT` writer - PWM_X Initial Value"]
pub type PwmxInitW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `PWM45_INIT` reader - PWM45 Initial Value"]
pub type Pwm45InitR = crate::BitReader;
#[doc = "Field `PWM45_INIT` writer - PWM45 Initial Value"]
pub type Pwm45InitW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `PWM23_INIT` reader - PWM23 Initial Value"]
pub type Pwm23InitR = crate::BitReader;
#[doc = "Field `PWM23_INIT` writer - PWM23 Initial Value"]
pub type Pwm23InitW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Independent or Complementary Pair Operation\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Indep {
    #[doc = "0: PWM_A and PWM_B form a complementary PWM pair."]
    Complementary = 0,
    #[doc = "1: PWM_A and PWM_B outputs are independent PWMs."]
    Independent = 1,
}
impl From<Indep> for bool {
    #[inline(always)]
    fn from(variant: Indep) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INDEP` reader - Independent or Complementary Pair Operation"]
pub type IndepR = crate::BitReader<Indep>;
impl IndepR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Indep {
        match self.bits {
            false => Indep::Complementary,
            true => Indep::Independent,
        }
    }
    #[doc = "PWM_A and PWM_B form a complementary PWM pair."]
    #[inline(always)]
    pub fn is_complementary(&self) -> bool {
        *self == Indep::Complementary
    }
    #[doc = "PWM_A and PWM_B outputs are independent PWMs."]
    #[inline(always)]
    pub fn is_independent(&self) -> bool {
        *self == Indep::Independent
    }
}
#[doc = "Field `INDEP` writer - Independent or Complementary Pair Operation"]
pub type IndepW<'a, REG> = crate::BitWriter<'a, REG, Indep>;
impl<'a, REG> IndepW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "PWM_A and PWM_B form a complementary PWM pair."]
    #[inline(always)]
    pub fn complementary(self) -> &'a mut crate::W<REG> {
        self.variant(Indep::Complementary)
    }
    #[doc = "PWM_A and PWM_B outputs are independent PWMs."]
    #[inline(always)]
    pub fn independent(self) -> &'a mut crate::W<REG> {
        self.variant(Indep::Independent)
    }
}
#[doc = "Field `DBGEN` reader - Debug Enable"]
pub type DbgenR = crate::BitReader;
#[doc = "Field `DBGEN` writer - Debug Enable"]
pub type DbgenW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bits 0:1 - Clock Source Select"]
    #[inline(always)]
    pub fn clk_sel(&self) -> ClkSelR {
        ClkSelR::new((self.bits & 3) as u8)
    }
    #[doc = "Bit 2 - Reload Source Select"]
    #[inline(always)]
    pub fn reload_sel(&self) -> ReloadSelR {
        ReloadSelR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bits 3:5 - Force Select"]
    #[inline(always)]
    pub fn force_sel(&self) -> ForceSelR {
        ForceSelR::new(((self.bits >> 3) & 7) as u8)
    }
    #[doc = "Bit 6 - Force Initialization"]
    #[inline(always)]
    pub fn force(&self) -> ForceR {
        ForceR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Force Enable"]
    #[inline(always)]
    pub fn frcen(&self) -> FrcenR {
        FrcenR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bits 8:9 - Initialization Control Select"]
    #[inline(always)]
    pub fn init_sel(&self) -> InitSelR {
        InitSelR::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bit 10 - PWM_X Initial Value"]
    #[inline(always)]
    pub fn pwmx_init(&self) -> PwmxInitR {
        PwmxInitR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - PWM45 Initial Value"]
    #[inline(always)]
    pub fn pwm45_init(&self) -> Pwm45InitR {
        Pwm45InitR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - PWM23 Initial Value"]
    #[inline(always)]
    pub fn pwm23_init(&self) -> Pwm23InitR {
        Pwm23InitR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Independent or Complementary Pair Operation"]
    #[inline(always)]
    pub fn indep(&self) -> IndepR {
        IndepR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 15 - Debug Enable"]
    #[inline(always)]
    pub fn dbgen(&self) -> DbgenR {
        DbgenR::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:1 - Clock Source Select"]
    #[inline(always)]
    pub fn clk_sel(&mut self) -> ClkSelW<Sm3ctrl2Spec> {
        ClkSelW::new(self, 0)
    }
    #[doc = "Bit 2 - Reload Source Select"]
    #[inline(always)]
    pub fn reload_sel(&mut self) -> ReloadSelW<Sm3ctrl2Spec> {
        ReloadSelW::new(self, 2)
    }
    #[doc = "Bits 3:5 - Force Select"]
    #[inline(always)]
    pub fn force_sel(&mut self) -> ForceSelW<Sm3ctrl2Spec> {
        ForceSelW::new(self, 3)
    }
    #[doc = "Bit 6 - Force Initialization"]
    #[inline(always)]
    pub fn force(&mut self) -> ForceW<Sm3ctrl2Spec> {
        ForceW::new(self, 6)
    }
    #[doc = "Bit 7 - Force Enable"]
    #[inline(always)]
    pub fn frcen(&mut self) -> FrcenW<Sm3ctrl2Spec> {
        FrcenW::new(self, 7)
    }
    #[doc = "Bits 8:9 - Initialization Control Select"]
    #[inline(always)]
    pub fn init_sel(&mut self) -> InitSelW<Sm3ctrl2Spec> {
        InitSelW::new(self, 8)
    }
    #[doc = "Bit 10 - PWM_X Initial Value"]
    #[inline(always)]
    pub fn pwmx_init(&mut self) -> PwmxInitW<Sm3ctrl2Spec> {
        PwmxInitW::new(self, 10)
    }
    #[doc = "Bit 11 - PWM45 Initial Value"]
    #[inline(always)]
    pub fn pwm45_init(&mut self) -> Pwm45InitW<Sm3ctrl2Spec> {
        Pwm45InitW::new(self, 11)
    }
    #[doc = "Bit 12 - PWM23 Initial Value"]
    #[inline(always)]
    pub fn pwm23_init(&mut self) -> Pwm23InitW<Sm3ctrl2Spec> {
        Pwm23InitW::new(self, 12)
    }
    #[doc = "Bit 13 - Independent or Complementary Pair Operation"]
    #[inline(always)]
    pub fn indep(&mut self) -> IndepW<Sm3ctrl2Spec> {
        IndepW::new(self, 13)
    }
    #[doc = "Bit 15 - Debug Enable"]
    #[inline(always)]
    pub fn dbgen(&mut self) -> DbgenW<Sm3ctrl2Spec> {
        DbgenW::new(self, 15)
    }
}
#[doc = "Control 2 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3ctrl2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3ctrl2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm3ctrl2Spec;
impl crate::RegisterSpec for Sm3ctrl2Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm3ctrl2::R`](R) reader structure"]
impl crate::Readable for Sm3ctrl2Spec {}
#[doc = "`write(|w| ..)` method takes [`sm3ctrl2::W`](W) writer structure"]
impl crate::Writable for Sm3ctrl2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM3CTRL2 to value 0"]
impl crate::Resettable for Sm3ctrl2Spec {}
