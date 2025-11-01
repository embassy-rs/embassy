#[doc = "Register `SM2CTRL` reader"]
pub type R = crate::R<Sm2ctrlSpec>;
#[doc = "Register `SM2CTRL` writer"]
pub type W = crate::W<Sm2ctrlSpec>;
#[doc = "Double Switching Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dblen {
    #[doc = "0: Double switching disabled."]
    Disabled = 0,
    #[doc = "1: Double switching enabled."]
    Enabled = 1,
}
impl From<Dblen> for bool {
    #[inline(always)]
    fn from(variant: Dblen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DBLEN` reader - Double Switching Enable"]
pub type DblenR = crate::BitReader<Dblen>;
impl DblenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dblen {
        match self.bits {
            false => Dblen::Disabled,
            true => Dblen::Enabled,
        }
    }
    #[doc = "Double switching disabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Dblen::Disabled
    }
    #[doc = "Double switching enabled."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Dblen::Enabled
    }
}
#[doc = "Field `DBLEN` writer - Double Switching Enable"]
pub type DblenW<'a, REG> = crate::BitWriter<'a, REG, Dblen>;
impl<'a, REG> DblenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Double switching disabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dblen::Disabled)
    }
    #[doc = "Double switching enabled."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dblen::Enabled)
    }
}
#[doc = "PWM_X Double Switching Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dblx {
    #[doc = "0: PWM_X double pulse disabled."]
    Disabled = 0,
    #[doc = "1: PWM_X double pulse enabled."]
    Enabled = 1,
}
impl From<Dblx> for bool {
    #[inline(always)]
    fn from(variant: Dblx) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DBLX` reader - PWM_X Double Switching Enable"]
pub type DblxR = crate::BitReader<Dblx>;
impl DblxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dblx {
        match self.bits {
            false => Dblx::Disabled,
            true => Dblx::Enabled,
        }
    }
    #[doc = "PWM_X double pulse disabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Dblx::Disabled
    }
    #[doc = "PWM_X double pulse enabled."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Dblx::Enabled
    }
}
#[doc = "Field `DBLX` writer - PWM_X Double Switching Enable"]
pub type DblxW<'a, REG> = crate::BitWriter<'a, REG, Dblx>;
impl<'a, REG> DblxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "PWM_X double pulse disabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dblx::Disabled)
    }
    #[doc = "PWM_X double pulse enabled."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dblx::Enabled)
    }
}
#[doc = "Load Mode Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ldmod {
    #[doc = "0: Buffered registers of this submodule are loaded and take effect at the next PWM reload if MCTRL\\[LDOK\\] is set."]
    NextPwmReload = 0,
    #[doc = "1: Buffered registers of this submodule are loaded and take effect immediately upon MCTRL\\[LDOK\\] being set. In this case, it is not necessary to set CTRL\\[FULL\\] or CTRL\\[HALF\\]."]
    MtctrlLdokSet = 1,
}
impl From<Ldmod> for bool {
    #[inline(always)]
    fn from(variant: Ldmod) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LDMOD` reader - Load Mode Select"]
pub type LdmodR = crate::BitReader<Ldmod>;
impl LdmodR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ldmod {
        match self.bits {
            false => Ldmod::NextPwmReload,
            true => Ldmod::MtctrlLdokSet,
        }
    }
    #[doc = "Buffered registers of this submodule are loaded and take effect at the next PWM reload if MCTRL\\[LDOK\\] is set."]
    #[inline(always)]
    pub fn is_next_pwm_reload(&self) -> bool {
        *self == Ldmod::NextPwmReload
    }
    #[doc = "Buffered registers of this submodule are loaded and take effect immediately upon MCTRL\\[LDOK\\] being set. In this case, it is not necessary to set CTRL\\[FULL\\] or CTRL\\[HALF\\]."]
    #[inline(always)]
    pub fn is_mtctrl_ldok_set(&self) -> bool {
        *self == Ldmod::MtctrlLdokSet
    }
}
#[doc = "Field `LDMOD` writer - Load Mode Select"]
pub type LdmodW<'a, REG> = crate::BitWriter<'a, REG, Ldmod>;
impl<'a, REG> LdmodW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Buffered registers of this submodule are loaded and take effect at the next PWM reload if MCTRL\\[LDOK\\] is set."]
    #[inline(always)]
    pub fn next_pwm_reload(self) -> &'a mut crate::W<REG> {
        self.variant(Ldmod::NextPwmReload)
    }
    #[doc = "Buffered registers of this submodule are loaded and take effect immediately upon MCTRL\\[LDOK\\] being set. In this case, it is not necessary to set CTRL\\[FULL\\] or CTRL\\[HALF\\]."]
    #[inline(always)]
    pub fn mtctrl_ldok_set(self) -> &'a mut crate::W<REG> {
        self.variant(Ldmod::MtctrlLdokSet)
    }
}
#[doc = "Split the DBLPWM signal to PWM_A and PWM_B\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Split {
    #[doc = "0: DBLPWM is not split. PWM_A and PWM_B each have double pulses."]
    Disabled = 0,
    #[doc = "1: DBLPWM is split to PWM_A and PWM_B."]
    Enabled = 1,
}
impl From<Split> for bool {
    #[inline(always)]
    fn from(variant: Split) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SPLIT` reader - Split the DBLPWM signal to PWM_A and PWM_B"]
pub type SplitR = crate::BitReader<Split>;
impl SplitR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Split {
        match self.bits {
            false => Split::Disabled,
            true => Split::Enabled,
        }
    }
    #[doc = "DBLPWM is not split. PWM_A and PWM_B each have double pulses."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Split::Disabled
    }
    #[doc = "DBLPWM is split to PWM_A and PWM_B."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Split::Enabled
    }
}
#[doc = "Field `SPLIT` writer - Split the DBLPWM signal to PWM_A and PWM_B"]
pub type SplitW<'a, REG> = crate::BitWriter<'a, REG, Split>;
impl<'a, REG> SplitW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "DBLPWM is not split. PWM_A and PWM_B each have double pulses."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Split::Disabled)
    }
    #[doc = "DBLPWM is split to PWM_A and PWM_B."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Split::Enabled)
    }
}
#[doc = "Prescaler\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Prsc {
    #[doc = "0: Prescaler 1"]
    One = 0,
    #[doc = "1: Prescaler 2"]
    Two = 1,
    #[doc = "2: Prescaler 4"]
    Four = 2,
    #[doc = "3: Prescaler 8"]
    Eight = 3,
    #[doc = "4: Prescaler 16"]
    Sixteen = 4,
    #[doc = "5: Prescaler 32"]
    Thirtytwo = 5,
    #[doc = "6: Prescaler 64"]
    Sixtyfour = 6,
    #[doc = "7: Prescaler 128"]
    Hundredtwentyeight = 7,
}
impl From<Prsc> for u8 {
    #[inline(always)]
    fn from(variant: Prsc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Prsc {
    type Ux = u8;
}
impl crate::IsEnum for Prsc {}
#[doc = "Field `PRSC` reader - Prescaler"]
pub type PrscR = crate::FieldReader<Prsc>;
impl PrscR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Prsc {
        match self.bits {
            0 => Prsc::One,
            1 => Prsc::Two,
            2 => Prsc::Four,
            3 => Prsc::Eight,
            4 => Prsc::Sixteen,
            5 => Prsc::Thirtytwo,
            6 => Prsc::Sixtyfour,
            7 => Prsc::Hundredtwentyeight,
            _ => unreachable!(),
        }
    }
    #[doc = "Prescaler 1"]
    #[inline(always)]
    pub fn is_one(&self) -> bool {
        *self == Prsc::One
    }
    #[doc = "Prescaler 2"]
    #[inline(always)]
    pub fn is_two(&self) -> bool {
        *self == Prsc::Two
    }
    #[doc = "Prescaler 4"]
    #[inline(always)]
    pub fn is_four(&self) -> bool {
        *self == Prsc::Four
    }
    #[doc = "Prescaler 8"]
    #[inline(always)]
    pub fn is_eight(&self) -> bool {
        *self == Prsc::Eight
    }
    #[doc = "Prescaler 16"]
    #[inline(always)]
    pub fn is_sixteen(&self) -> bool {
        *self == Prsc::Sixteen
    }
    #[doc = "Prescaler 32"]
    #[inline(always)]
    pub fn is_thirtytwo(&self) -> bool {
        *self == Prsc::Thirtytwo
    }
    #[doc = "Prescaler 64"]
    #[inline(always)]
    pub fn is_sixtyfour(&self) -> bool {
        *self == Prsc::Sixtyfour
    }
    #[doc = "Prescaler 128"]
    #[inline(always)]
    pub fn is_hundredtwentyeight(&self) -> bool {
        *self == Prsc::Hundredtwentyeight
    }
}
#[doc = "Field `PRSC` writer - Prescaler"]
pub type PrscW<'a, REG> = crate::FieldWriter<'a, REG, 3, Prsc, crate::Safe>;
impl<'a, REG> PrscW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Prescaler 1"]
    #[inline(always)]
    pub fn one(self) -> &'a mut crate::W<REG> {
        self.variant(Prsc::One)
    }
    #[doc = "Prescaler 2"]
    #[inline(always)]
    pub fn two(self) -> &'a mut crate::W<REG> {
        self.variant(Prsc::Two)
    }
    #[doc = "Prescaler 4"]
    #[inline(always)]
    pub fn four(self) -> &'a mut crate::W<REG> {
        self.variant(Prsc::Four)
    }
    #[doc = "Prescaler 8"]
    #[inline(always)]
    pub fn eight(self) -> &'a mut crate::W<REG> {
        self.variant(Prsc::Eight)
    }
    #[doc = "Prescaler 16"]
    #[inline(always)]
    pub fn sixteen(self) -> &'a mut crate::W<REG> {
        self.variant(Prsc::Sixteen)
    }
    #[doc = "Prescaler 32"]
    #[inline(always)]
    pub fn thirtytwo(self) -> &'a mut crate::W<REG> {
        self.variant(Prsc::Thirtytwo)
    }
    #[doc = "Prescaler 64"]
    #[inline(always)]
    pub fn sixtyfour(self) -> &'a mut crate::W<REG> {
        self.variant(Prsc::Sixtyfour)
    }
    #[doc = "Prescaler 128"]
    #[inline(always)]
    pub fn hundredtwentyeight(self) -> &'a mut crate::W<REG> {
        self.variant(Prsc::Hundredtwentyeight)
    }
}
#[doc = "Compare Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Compmode {
    #[doc = "0: The VAL* registers and the PWM counter are compared using an \"equal to\" method. This means that PWM edges are only produced when the counter is equal to one of the VAL* register values. This implies that a PWM_A output that is high at the end of a period maintains this state until a match with VAL3 clears the output in the following period."]
    EqualTo = 0,
    #[doc = "1: The VAL* registers and the PWM counter are compared using an \"equal to or greater than\" method. This means that PWM edges are produced when the counter is equal to or greater than one of the VAL* register values. This implies that a PWM_A output that is high at the end of a period could go low at the start of the next period if the starting counter value is greater than (but not necessarily equal to) the new VAL3 value."]
    EqualToOrGreaterThan = 1,
}
impl From<Compmode> for bool {
    #[inline(always)]
    fn from(variant: Compmode) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `COMPMODE` reader - Compare Mode"]
pub type CompmodeR = crate::BitReader<Compmode>;
impl CompmodeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Compmode {
        match self.bits {
            false => Compmode::EqualTo,
            true => Compmode::EqualToOrGreaterThan,
        }
    }
    #[doc = "The VAL* registers and the PWM counter are compared using an \"equal to\" method. This means that PWM edges are only produced when the counter is equal to one of the VAL* register values. This implies that a PWM_A output that is high at the end of a period maintains this state until a match with VAL3 clears the output in the following period."]
    #[inline(always)]
    pub fn is_equal_to(&self) -> bool {
        *self == Compmode::EqualTo
    }
    #[doc = "The VAL* registers and the PWM counter are compared using an \"equal to or greater than\" method. This means that PWM edges are produced when the counter is equal to or greater than one of the VAL* register values. This implies that a PWM_A output that is high at the end of a period could go low at the start of the next period if the starting counter value is greater than (but not necessarily equal to) the new VAL3 value."]
    #[inline(always)]
    pub fn is_equal_to_or_greater_than(&self) -> bool {
        *self == Compmode::EqualToOrGreaterThan
    }
}
#[doc = "Field `COMPMODE` writer - Compare Mode"]
pub type CompmodeW<'a, REG> = crate::BitWriter<'a, REG, Compmode>;
impl<'a, REG> CompmodeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The VAL* registers and the PWM counter are compared using an \"equal to\" method. This means that PWM edges are only produced when the counter is equal to one of the VAL* register values. This implies that a PWM_A output that is high at the end of a period maintains this state until a match with VAL3 clears the output in the following period."]
    #[inline(always)]
    pub fn equal_to(self) -> &'a mut crate::W<REG> {
        self.variant(Compmode::EqualTo)
    }
    #[doc = "The VAL* registers and the PWM counter are compared using an \"equal to or greater than\" method. This means that PWM edges are produced when the counter is equal to or greater than one of the VAL* register values. This implies that a PWM_A output that is high at the end of a period could go low at the start of the next period if the starting counter value is greater than (but not necessarily equal to) the new VAL3 value."]
    #[inline(always)]
    pub fn equal_to_or_greater_than(self) -> &'a mut crate::W<REG> {
        self.variant(Compmode::EqualToOrGreaterThan)
    }
}
#[doc = "Field `DT` reader - Deadtime"]
pub type DtR = crate::FieldReader;
#[doc = "Full Cycle Reload\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Full {
    #[doc = "0: Full-cycle reloads disabled."]
    Disabled = 0,
    #[doc = "1: Full-cycle reloads enabled."]
    Enabled = 1,
}
impl From<Full> for bool {
    #[inline(always)]
    fn from(variant: Full) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FULL` reader - Full Cycle Reload"]
pub type FullR = crate::BitReader<Full>;
impl FullR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Full {
        match self.bits {
            false => Full::Disabled,
            true => Full::Enabled,
        }
    }
    #[doc = "Full-cycle reloads disabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Full::Disabled
    }
    #[doc = "Full-cycle reloads enabled."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Full::Enabled
    }
}
#[doc = "Field `FULL` writer - Full Cycle Reload"]
pub type FullW<'a, REG> = crate::BitWriter<'a, REG, Full>;
impl<'a, REG> FullW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Full-cycle reloads disabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Full::Disabled)
    }
    #[doc = "Full-cycle reloads enabled."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Full::Enabled)
    }
}
#[doc = "Half Cycle Reload\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Half {
    #[doc = "0: Half-cycle reloads disabled."]
    Disabled = 0,
    #[doc = "1: Half-cycle reloads enabled."]
    Enabled = 1,
}
impl From<Half> for bool {
    #[inline(always)]
    fn from(variant: Half) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HALF` reader - Half Cycle Reload"]
pub type HalfR = crate::BitReader<Half>;
impl HalfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Half {
        match self.bits {
            false => Half::Disabled,
            true => Half::Enabled,
        }
    }
    #[doc = "Half-cycle reloads disabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Half::Disabled
    }
    #[doc = "Half-cycle reloads enabled."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Half::Enabled
    }
}
#[doc = "Field `HALF` writer - Half Cycle Reload"]
pub type HalfW<'a, REG> = crate::BitWriter<'a, REG, Half>;
impl<'a, REG> HalfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Half-cycle reloads disabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Half::Disabled)
    }
    #[doc = "Half-cycle reloads enabled."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Half::Enabled)
    }
}
#[doc = "Load Frequency\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Ldfq {
    #[doc = "0: Every PWM opportunity"]
    Everypwm = 0,
    #[doc = "1: Every 2 PWM opportunities"]
    Every2pwm = 1,
    #[doc = "2: Every 3 PWM opportunities"]
    Every3pwm = 2,
    #[doc = "3: Every 4 PWM opportunities"]
    Every4pwm = 3,
    #[doc = "4: Every 5 PWM opportunities"]
    Every5pwm = 4,
    #[doc = "5: Every 6 PWM opportunities"]
    Every6pwm = 5,
    #[doc = "6: Every 7 PWM opportunities"]
    Every7pwm = 6,
    #[doc = "7: Every 8 PWM opportunities"]
    Every8pwm = 7,
    #[doc = "8: Every 9 PWM opportunities"]
    Every9pwm = 8,
    #[doc = "9: Every 10 PWM opportunities"]
    Every10pwm = 9,
    #[doc = "10: Every 11 PWM opportunities"]
    Every11pwm = 10,
    #[doc = "11: Every 12 PWM opportunities"]
    Every12pwm = 11,
    #[doc = "12: Every 13 PWM opportunities"]
    Every13pwm = 12,
    #[doc = "13: Every 14 PWM opportunities"]
    Every14pwm = 13,
    #[doc = "14: Every 15 PWM opportunities"]
    Every15pwm = 14,
    #[doc = "15: Every 16 PWM opportunities"]
    Every16pwm = 15,
}
impl From<Ldfq> for u8 {
    #[inline(always)]
    fn from(variant: Ldfq) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Ldfq {
    type Ux = u8;
}
impl crate::IsEnum for Ldfq {}
#[doc = "Field `LDFQ` reader - Load Frequency"]
pub type LdfqR = crate::FieldReader<Ldfq>;
impl LdfqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ldfq {
        match self.bits {
            0 => Ldfq::Everypwm,
            1 => Ldfq::Every2pwm,
            2 => Ldfq::Every3pwm,
            3 => Ldfq::Every4pwm,
            4 => Ldfq::Every5pwm,
            5 => Ldfq::Every6pwm,
            6 => Ldfq::Every7pwm,
            7 => Ldfq::Every8pwm,
            8 => Ldfq::Every9pwm,
            9 => Ldfq::Every10pwm,
            10 => Ldfq::Every11pwm,
            11 => Ldfq::Every12pwm,
            12 => Ldfq::Every13pwm,
            13 => Ldfq::Every14pwm,
            14 => Ldfq::Every15pwm,
            15 => Ldfq::Every16pwm,
            _ => unreachable!(),
        }
    }
    #[doc = "Every PWM opportunity"]
    #[inline(always)]
    pub fn is_everypwm(&self) -> bool {
        *self == Ldfq::Everypwm
    }
    #[doc = "Every 2 PWM opportunities"]
    #[inline(always)]
    pub fn is_every2pwm(&self) -> bool {
        *self == Ldfq::Every2pwm
    }
    #[doc = "Every 3 PWM opportunities"]
    #[inline(always)]
    pub fn is_every3pwm(&self) -> bool {
        *self == Ldfq::Every3pwm
    }
    #[doc = "Every 4 PWM opportunities"]
    #[inline(always)]
    pub fn is_every4pwm(&self) -> bool {
        *self == Ldfq::Every4pwm
    }
    #[doc = "Every 5 PWM opportunities"]
    #[inline(always)]
    pub fn is_every5pwm(&self) -> bool {
        *self == Ldfq::Every5pwm
    }
    #[doc = "Every 6 PWM opportunities"]
    #[inline(always)]
    pub fn is_every6pwm(&self) -> bool {
        *self == Ldfq::Every6pwm
    }
    #[doc = "Every 7 PWM opportunities"]
    #[inline(always)]
    pub fn is_every7pwm(&self) -> bool {
        *self == Ldfq::Every7pwm
    }
    #[doc = "Every 8 PWM opportunities"]
    #[inline(always)]
    pub fn is_every8pwm(&self) -> bool {
        *self == Ldfq::Every8pwm
    }
    #[doc = "Every 9 PWM opportunities"]
    #[inline(always)]
    pub fn is_every9pwm(&self) -> bool {
        *self == Ldfq::Every9pwm
    }
    #[doc = "Every 10 PWM opportunities"]
    #[inline(always)]
    pub fn is_every10pwm(&self) -> bool {
        *self == Ldfq::Every10pwm
    }
    #[doc = "Every 11 PWM opportunities"]
    #[inline(always)]
    pub fn is_every11pwm(&self) -> bool {
        *self == Ldfq::Every11pwm
    }
    #[doc = "Every 12 PWM opportunities"]
    #[inline(always)]
    pub fn is_every12pwm(&self) -> bool {
        *self == Ldfq::Every12pwm
    }
    #[doc = "Every 13 PWM opportunities"]
    #[inline(always)]
    pub fn is_every13pwm(&self) -> bool {
        *self == Ldfq::Every13pwm
    }
    #[doc = "Every 14 PWM opportunities"]
    #[inline(always)]
    pub fn is_every14pwm(&self) -> bool {
        *self == Ldfq::Every14pwm
    }
    #[doc = "Every 15 PWM opportunities"]
    #[inline(always)]
    pub fn is_every15pwm(&self) -> bool {
        *self == Ldfq::Every15pwm
    }
    #[doc = "Every 16 PWM opportunities"]
    #[inline(always)]
    pub fn is_every16pwm(&self) -> bool {
        *self == Ldfq::Every16pwm
    }
}
#[doc = "Field `LDFQ` writer - Load Frequency"]
pub type LdfqW<'a, REG> = crate::FieldWriter<'a, REG, 4, Ldfq, crate::Safe>;
impl<'a, REG> LdfqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Every PWM opportunity"]
    #[inline(always)]
    pub fn everypwm(self) -> &'a mut crate::W<REG> {
        self.variant(Ldfq::Everypwm)
    }
    #[doc = "Every 2 PWM opportunities"]
    #[inline(always)]
    pub fn every2pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Ldfq::Every2pwm)
    }
    #[doc = "Every 3 PWM opportunities"]
    #[inline(always)]
    pub fn every3pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Ldfq::Every3pwm)
    }
    #[doc = "Every 4 PWM opportunities"]
    #[inline(always)]
    pub fn every4pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Ldfq::Every4pwm)
    }
    #[doc = "Every 5 PWM opportunities"]
    #[inline(always)]
    pub fn every5pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Ldfq::Every5pwm)
    }
    #[doc = "Every 6 PWM opportunities"]
    #[inline(always)]
    pub fn every6pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Ldfq::Every6pwm)
    }
    #[doc = "Every 7 PWM opportunities"]
    #[inline(always)]
    pub fn every7pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Ldfq::Every7pwm)
    }
    #[doc = "Every 8 PWM opportunities"]
    #[inline(always)]
    pub fn every8pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Ldfq::Every8pwm)
    }
    #[doc = "Every 9 PWM opportunities"]
    #[inline(always)]
    pub fn every9pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Ldfq::Every9pwm)
    }
    #[doc = "Every 10 PWM opportunities"]
    #[inline(always)]
    pub fn every10pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Ldfq::Every10pwm)
    }
    #[doc = "Every 11 PWM opportunities"]
    #[inline(always)]
    pub fn every11pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Ldfq::Every11pwm)
    }
    #[doc = "Every 12 PWM opportunities"]
    #[inline(always)]
    pub fn every12pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Ldfq::Every12pwm)
    }
    #[doc = "Every 13 PWM opportunities"]
    #[inline(always)]
    pub fn every13pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Ldfq::Every13pwm)
    }
    #[doc = "Every 14 PWM opportunities"]
    #[inline(always)]
    pub fn every14pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Ldfq::Every14pwm)
    }
    #[doc = "Every 15 PWM opportunities"]
    #[inline(always)]
    pub fn every15pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Ldfq::Every15pwm)
    }
    #[doc = "Every 16 PWM opportunities"]
    #[inline(always)]
    pub fn every16pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Ldfq::Every16pwm)
    }
}
impl R {
    #[doc = "Bit 0 - Double Switching Enable"]
    #[inline(always)]
    pub fn dblen(&self) -> DblenR {
        DblenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - PWM_X Double Switching Enable"]
    #[inline(always)]
    pub fn dblx(&self) -> DblxR {
        DblxR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Load Mode Select"]
    #[inline(always)]
    pub fn ldmod(&self) -> LdmodR {
        LdmodR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Split the DBLPWM signal to PWM_A and PWM_B"]
    #[inline(always)]
    pub fn split(&self) -> SplitR {
        SplitR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bits 4:6 - Prescaler"]
    #[inline(always)]
    pub fn prsc(&self) -> PrscR {
        PrscR::new(((self.bits >> 4) & 7) as u8)
    }
    #[doc = "Bit 7 - Compare Mode"]
    #[inline(always)]
    pub fn compmode(&self) -> CompmodeR {
        CompmodeR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bits 8:9 - Deadtime"]
    #[inline(always)]
    pub fn dt(&self) -> DtR {
        DtR::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bit 10 - Full Cycle Reload"]
    #[inline(always)]
    pub fn full(&self) -> FullR {
        FullR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Half Cycle Reload"]
    #[inline(always)]
    pub fn half(&self) -> HalfR {
        HalfR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bits 12:15 - Load Frequency"]
    #[inline(always)]
    pub fn ldfq(&self) -> LdfqR {
        LdfqR::new(((self.bits >> 12) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Double Switching Enable"]
    #[inline(always)]
    pub fn dblen(&mut self) -> DblenW<Sm2ctrlSpec> {
        DblenW::new(self, 0)
    }
    #[doc = "Bit 1 - PWM_X Double Switching Enable"]
    #[inline(always)]
    pub fn dblx(&mut self) -> DblxW<Sm2ctrlSpec> {
        DblxW::new(self, 1)
    }
    #[doc = "Bit 2 - Load Mode Select"]
    #[inline(always)]
    pub fn ldmod(&mut self) -> LdmodW<Sm2ctrlSpec> {
        LdmodW::new(self, 2)
    }
    #[doc = "Bit 3 - Split the DBLPWM signal to PWM_A and PWM_B"]
    #[inline(always)]
    pub fn split(&mut self) -> SplitW<Sm2ctrlSpec> {
        SplitW::new(self, 3)
    }
    #[doc = "Bits 4:6 - Prescaler"]
    #[inline(always)]
    pub fn prsc(&mut self) -> PrscW<Sm2ctrlSpec> {
        PrscW::new(self, 4)
    }
    #[doc = "Bit 7 - Compare Mode"]
    #[inline(always)]
    pub fn compmode(&mut self) -> CompmodeW<Sm2ctrlSpec> {
        CompmodeW::new(self, 7)
    }
    #[doc = "Bit 10 - Full Cycle Reload"]
    #[inline(always)]
    pub fn full(&mut self) -> FullW<Sm2ctrlSpec> {
        FullW::new(self, 10)
    }
    #[doc = "Bit 11 - Half Cycle Reload"]
    #[inline(always)]
    pub fn half(&mut self) -> HalfW<Sm2ctrlSpec> {
        HalfW::new(self, 11)
    }
    #[doc = "Bits 12:15 - Load Frequency"]
    #[inline(always)]
    pub fn ldfq(&mut self) -> LdfqW<Sm2ctrlSpec> {
        LdfqW::new(self, 12)
    }
}
#[doc = "Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm2ctrlSpec;
impl crate::RegisterSpec for Sm2ctrlSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm2ctrl::R`](R) reader structure"]
impl crate::Readable for Sm2ctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`sm2ctrl::W`](W) writer structure"]
impl crate::Writable for Sm2ctrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM2CTRL to value 0x0400"]
impl crate::Resettable for Sm2ctrlSpec {
    const RESET_VALUE: u16 = 0x0400;
}
