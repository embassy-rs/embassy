#[doc = "Register `SM2OCTRL` reader"]
pub type R = crate::R<Sm2octrlSpec>;
#[doc = "Register `SM2OCTRL` writer"]
pub type W = crate::W<Sm2octrlSpec>;
#[doc = "PWM_X Fault State\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pwmxfs {
    #[doc = "0: Output is forced to logic 0 state prior to consideration of output polarity control."]
    Logic0 = 0,
    #[doc = "1: Output is forced to logic 1 state prior to consideration of output polarity control."]
    Logic1 = 1,
    #[doc = "2: Output is put in a high-impedance state."]
    Tristated2 = 2,
    #[doc = "3: Output is put in a high-impedance state."]
    Tristated3 = 3,
}
impl From<Pwmxfs> for u8 {
    #[inline(always)]
    fn from(variant: Pwmxfs) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pwmxfs {
    type Ux = u8;
}
impl crate::IsEnum for Pwmxfs {}
#[doc = "Field `PWMXFS` reader - PWM_X Fault State"]
pub type PwmxfsR = crate::FieldReader<Pwmxfs>;
impl PwmxfsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pwmxfs {
        match self.bits {
            0 => Pwmxfs::Logic0,
            1 => Pwmxfs::Logic1,
            2 => Pwmxfs::Tristated2,
            3 => Pwmxfs::Tristated3,
            _ => unreachable!(),
        }
    }
    #[doc = "Output is forced to logic 0 state prior to consideration of output polarity control."]
    #[inline(always)]
    pub fn is_logic_0(&self) -> bool {
        *self == Pwmxfs::Logic0
    }
    #[doc = "Output is forced to logic 1 state prior to consideration of output polarity control."]
    #[inline(always)]
    pub fn is_logic_1(&self) -> bool {
        *self == Pwmxfs::Logic1
    }
    #[doc = "Output is put in a high-impedance state."]
    #[inline(always)]
    pub fn is_tristated_2(&self) -> bool {
        *self == Pwmxfs::Tristated2
    }
    #[doc = "Output is put in a high-impedance state."]
    #[inline(always)]
    pub fn is_tristated_3(&self) -> bool {
        *self == Pwmxfs::Tristated3
    }
}
#[doc = "Field `PWMXFS` writer - PWM_X Fault State"]
pub type PwmxfsW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pwmxfs, crate::Safe>;
impl<'a, REG> PwmxfsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Output is forced to logic 0 state prior to consideration of output polarity control."]
    #[inline(always)]
    pub fn logic_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmxfs::Logic0)
    }
    #[doc = "Output is forced to logic 1 state prior to consideration of output polarity control."]
    #[inline(always)]
    pub fn logic_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmxfs::Logic1)
    }
    #[doc = "Output is put in a high-impedance state."]
    #[inline(always)]
    pub fn tristated_2(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmxfs::Tristated2)
    }
    #[doc = "Output is put in a high-impedance state."]
    #[inline(always)]
    pub fn tristated_3(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmxfs::Tristated3)
    }
}
#[doc = "PWM_B Fault State\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pwmbfs {
    #[doc = "0: Output is forced to logic 0 state prior to consideration of output polarity control."]
    Logic0 = 0,
    #[doc = "1: Output is forced to logic 1 state prior to consideration of output polarity control."]
    Logic1 = 1,
    #[doc = "2: Output is put in a high-impedance state."]
    Tristated2 = 2,
    #[doc = "3: Output is put in a high-impedance state."]
    Tristated3 = 3,
}
impl From<Pwmbfs> for u8 {
    #[inline(always)]
    fn from(variant: Pwmbfs) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pwmbfs {
    type Ux = u8;
}
impl crate::IsEnum for Pwmbfs {}
#[doc = "Field `PWMBFS` reader - PWM_B Fault State"]
pub type PwmbfsR = crate::FieldReader<Pwmbfs>;
impl PwmbfsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pwmbfs {
        match self.bits {
            0 => Pwmbfs::Logic0,
            1 => Pwmbfs::Logic1,
            2 => Pwmbfs::Tristated2,
            3 => Pwmbfs::Tristated3,
            _ => unreachable!(),
        }
    }
    #[doc = "Output is forced to logic 0 state prior to consideration of output polarity control."]
    #[inline(always)]
    pub fn is_logic_0(&self) -> bool {
        *self == Pwmbfs::Logic0
    }
    #[doc = "Output is forced to logic 1 state prior to consideration of output polarity control."]
    #[inline(always)]
    pub fn is_logic_1(&self) -> bool {
        *self == Pwmbfs::Logic1
    }
    #[doc = "Output is put in a high-impedance state."]
    #[inline(always)]
    pub fn is_tristated_2(&self) -> bool {
        *self == Pwmbfs::Tristated2
    }
    #[doc = "Output is put in a high-impedance state."]
    #[inline(always)]
    pub fn is_tristated_3(&self) -> bool {
        *self == Pwmbfs::Tristated3
    }
}
#[doc = "Field `PWMBFS` writer - PWM_B Fault State"]
pub type PwmbfsW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pwmbfs, crate::Safe>;
impl<'a, REG> PwmbfsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Output is forced to logic 0 state prior to consideration of output polarity control."]
    #[inline(always)]
    pub fn logic_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmbfs::Logic0)
    }
    #[doc = "Output is forced to logic 1 state prior to consideration of output polarity control."]
    #[inline(always)]
    pub fn logic_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmbfs::Logic1)
    }
    #[doc = "Output is put in a high-impedance state."]
    #[inline(always)]
    pub fn tristated_2(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmbfs::Tristated2)
    }
    #[doc = "Output is put in a high-impedance state."]
    #[inline(always)]
    pub fn tristated_3(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmbfs::Tristated3)
    }
}
#[doc = "PWM_A Fault State\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pwmafs {
    #[doc = "0: Output is forced to logic 0 state prior to consideration of output polarity control."]
    Logic0 = 0,
    #[doc = "1: Output is forced to logic 1 state prior to consideration of output polarity control."]
    Logic1 = 1,
    #[doc = "2: Output is put in a high-impedance state."]
    Tristated2 = 2,
    #[doc = "3: Output is put in a high-impedance state."]
    Tristated3 = 3,
}
impl From<Pwmafs> for u8 {
    #[inline(always)]
    fn from(variant: Pwmafs) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pwmafs {
    type Ux = u8;
}
impl crate::IsEnum for Pwmafs {}
#[doc = "Field `PWMAFS` reader - PWM_A Fault State"]
pub type PwmafsR = crate::FieldReader<Pwmafs>;
impl PwmafsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pwmafs {
        match self.bits {
            0 => Pwmafs::Logic0,
            1 => Pwmafs::Logic1,
            2 => Pwmafs::Tristated2,
            3 => Pwmafs::Tristated3,
            _ => unreachable!(),
        }
    }
    #[doc = "Output is forced to logic 0 state prior to consideration of output polarity control."]
    #[inline(always)]
    pub fn is_logic_0(&self) -> bool {
        *self == Pwmafs::Logic0
    }
    #[doc = "Output is forced to logic 1 state prior to consideration of output polarity control."]
    #[inline(always)]
    pub fn is_logic_1(&self) -> bool {
        *self == Pwmafs::Logic1
    }
    #[doc = "Output is put in a high-impedance state."]
    #[inline(always)]
    pub fn is_tristated_2(&self) -> bool {
        *self == Pwmafs::Tristated2
    }
    #[doc = "Output is put in a high-impedance state."]
    #[inline(always)]
    pub fn is_tristated_3(&self) -> bool {
        *self == Pwmafs::Tristated3
    }
}
#[doc = "Field `PWMAFS` writer - PWM_A Fault State"]
pub type PwmafsW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pwmafs, crate::Safe>;
impl<'a, REG> PwmafsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Output is forced to logic 0 state prior to consideration of output polarity control."]
    #[inline(always)]
    pub fn logic_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmafs::Logic0)
    }
    #[doc = "Output is forced to logic 1 state prior to consideration of output polarity control."]
    #[inline(always)]
    pub fn logic_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmafs::Logic1)
    }
    #[doc = "Output is put in a high-impedance state."]
    #[inline(always)]
    pub fn tristated_2(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmafs::Tristated2)
    }
    #[doc = "Output is put in a high-impedance state."]
    #[inline(always)]
    pub fn tristated_3(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmafs::Tristated3)
    }
}
#[doc = "PWM_X Output Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Polx {
    #[doc = "0: PWM_X output not inverted. A high level on the PWM_X pin represents the \"on\" or \"active\" state."]
    NotInverted = 0,
    #[doc = "1: PWM_X output inverted. A low level on the PWM_X pin represents the \"on\" or \"active\" state."]
    Inverted = 1,
}
impl From<Polx> for bool {
    #[inline(always)]
    fn from(variant: Polx) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `POLX` reader - PWM_X Output Polarity"]
pub type PolxR = crate::BitReader<Polx>;
impl PolxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Polx {
        match self.bits {
            false => Polx::NotInverted,
            true => Polx::Inverted,
        }
    }
    #[doc = "PWM_X output not inverted. A high level on the PWM_X pin represents the \"on\" or \"active\" state."]
    #[inline(always)]
    pub fn is_not_inverted(&self) -> bool {
        *self == Polx::NotInverted
    }
    #[doc = "PWM_X output inverted. A low level on the PWM_X pin represents the \"on\" or \"active\" state."]
    #[inline(always)]
    pub fn is_inverted(&self) -> bool {
        *self == Polx::Inverted
    }
}
#[doc = "Field `POLX` writer - PWM_X Output Polarity"]
pub type PolxW<'a, REG> = crate::BitWriter<'a, REG, Polx>;
impl<'a, REG> PolxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "PWM_X output not inverted. A high level on the PWM_X pin represents the \"on\" or \"active\" state."]
    #[inline(always)]
    pub fn not_inverted(self) -> &'a mut crate::W<REG> {
        self.variant(Polx::NotInverted)
    }
    #[doc = "PWM_X output inverted. A low level on the PWM_X pin represents the \"on\" or \"active\" state."]
    #[inline(always)]
    pub fn inverted(self) -> &'a mut crate::W<REG> {
        self.variant(Polx::Inverted)
    }
}
#[doc = "PWM_B Output Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Polb {
    #[doc = "0: PWM_B output not inverted. A high level on the PWM_B pin represents the \"on\" or \"active\" state."]
    NotInverted = 0,
    #[doc = "1: PWM_B output inverted. A low level on the PWM_B pin represents the \"on\" or \"active\" state."]
    Inverted = 1,
}
impl From<Polb> for bool {
    #[inline(always)]
    fn from(variant: Polb) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `POLB` reader - PWM_B Output Polarity"]
pub type PolbR = crate::BitReader<Polb>;
impl PolbR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Polb {
        match self.bits {
            false => Polb::NotInverted,
            true => Polb::Inverted,
        }
    }
    #[doc = "PWM_B output not inverted. A high level on the PWM_B pin represents the \"on\" or \"active\" state."]
    #[inline(always)]
    pub fn is_not_inverted(&self) -> bool {
        *self == Polb::NotInverted
    }
    #[doc = "PWM_B output inverted. A low level on the PWM_B pin represents the \"on\" or \"active\" state."]
    #[inline(always)]
    pub fn is_inverted(&self) -> bool {
        *self == Polb::Inverted
    }
}
#[doc = "Field `POLB` writer - PWM_B Output Polarity"]
pub type PolbW<'a, REG> = crate::BitWriter<'a, REG, Polb>;
impl<'a, REG> PolbW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "PWM_B output not inverted. A high level on the PWM_B pin represents the \"on\" or \"active\" state."]
    #[inline(always)]
    pub fn not_inverted(self) -> &'a mut crate::W<REG> {
        self.variant(Polb::NotInverted)
    }
    #[doc = "PWM_B output inverted. A low level on the PWM_B pin represents the \"on\" or \"active\" state."]
    #[inline(always)]
    pub fn inverted(self) -> &'a mut crate::W<REG> {
        self.variant(Polb::Inverted)
    }
}
#[doc = "PWM_A Output Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pola {
    #[doc = "0: PWM_A output not inverted. A high level on the PWM_A pin represents the \"on\" or \"active\" state."]
    NotInverted = 0,
    #[doc = "1: PWM_A output inverted. A low level on the PWM_A pin represents the \"on\" or \"active\" state."]
    Inverted = 1,
}
impl From<Pola> for bool {
    #[inline(always)]
    fn from(variant: Pola) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `POLA` reader - PWM_A Output Polarity"]
pub type PolaR = crate::BitReader<Pola>;
impl PolaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pola {
        match self.bits {
            false => Pola::NotInverted,
            true => Pola::Inverted,
        }
    }
    #[doc = "PWM_A output not inverted. A high level on the PWM_A pin represents the \"on\" or \"active\" state."]
    #[inline(always)]
    pub fn is_not_inverted(&self) -> bool {
        *self == Pola::NotInverted
    }
    #[doc = "PWM_A output inverted. A low level on the PWM_A pin represents the \"on\" or \"active\" state."]
    #[inline(always)]
    pub fn is_inverted(&self) -> bool {
        *self == Pola::Inverted
    }
}
#[doc = "Field `POLA` writer - PWM_A Output Polarity"]
pub type PolaW<'a, REG> = crate::BitWriter<'a, REG, Pola>;
impl<'a, REG> PolaW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "PWM_A output not inverted. A high level on the PWM_A pin represents the \"on\" or \"active\" state."]
    #[inline(always)]
    pub fn not_inverted(self) -> &'a mut crate::W<REG> {
        self.variant(Pola::NotInverted)
    }
    #[doc = "PWM_A output inverted. A low level on the PWM_A pin represents the \"on\" or \"active\" state."]
    #[inline(always)]
    pub fn inverted(self) -> &'a mut crate::W<REG> {
        self.variant(Pola::Inverted)
    }
}
#[doc = "Field `PWMX_IN` reader - PWM_X Input"]
pub type PwmxInR = crate::BitReader;
#[doc = "Field `PWMB_IN` reader - PWM_B Input"]
pub type PwmbInR = crate::BitReader;
#[doc = "Field `PWMA_IN` reader - PWM_A Input"]
pub type PwmaInR = crate::BitReader;
impl R {
    #[doc = "Bits 0:1 - PWM_X Fault State"]
    #[inline(always)]
    pub fn pwmxfs(&self) -> PwmxfsR {
        PwmxfsR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:3 - PWM_B Fault State"]
    #[inline(always)]
    pub fn pwmbfs(&self) -> PwmbfsR {
        PwmbfsR::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:5 - PWM_A Fault State"]
    #[inline(always)]
    pub fn pwmafs(&self) -> PwmafsR {
        PwmafsR::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bit 8 - PWM_X Output Polarity"]
    #[inline(always)]
    pub fn polx(&self) -> PolxR {
        PolxR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - PWM_B Output Polarity"]
    #[inline(always)]
    pub fn polb(&self) -> PolbR {
        PolbR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - PWM_A Output Polarity"]
    #[inline(always)]
    pub fn pola(&self) -> PolaR {
        PolaR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 13 - PWM_X Input"]
    #[inline(always)]
    pub fn pwmx_in(&self) -> PwmxInR {
        PwmxInR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - PWM_B Input"]
    #[inline(always)]
    pub fn pwmb_in(&self) -> PwmbInR {
        PwmbInR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - PWM_A Input"]
    #[inline(always)]
    pub fn pwma_in(&self) -> PwmaInR {
        PwmaInR::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:1 - PWM_X Fault State"]
    #[inline(always)]
    pub fn pwmxfs(&mut self) -> PwmxfsW<Sm2octrlSpec> {
        PwmxfsW::new(self, 0)
    }
    #[doc = "Bits 2:3 - PWM_B Fault State"]
    #[inline(always)]
    pub fn pwmbfs(&mut self) -> PwmbfsW<Sm2octrlSpec> {
        PwmbfsW::new(self, 2)
    }
    #[doc = "Bits 4:5 - PWM_A Fault State"]
    #[inline(always)]
    pub fn pwmafs(&mut self) -> PwmafsW<Sm2octrlSpec> {
        PwmafsW::new(self, 4)
    }
    #[doc = "Bit 8 - PWM_X Output Polarity"]
    #[inline(always)]
    pub fn polx(&mut self) -> PolxW<Sm2octrlSpec> {
        PolxW::new(self, 8)
    }
    #[doc = "Bit 9 - PWM_B Output Polarity"]
    #[inline(always)]
    pub fn polb(&mut self) -> PolbW<Sm2octrlSpec> {
        PolbW::new(self, 9)
    }
    #[doc = "Bit 10 - PWM_A Output Polarity"]
    #[inline(always)]
    pub fn pola(&mut self) -> PolaW<Sm2octrlSpec> {
        PolaW::new(self, 10)
    }
}
#[doc = "Output Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2octrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2octrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm2octrlSpec;
impl crate::RegisterSpec for Sm2octrlSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm2octrl::R`](R) reader structure"]
impl crate::Readable for Sm2octrlSpec {}
#[doc = "`write(|w| ..)` method takes [`sm2octrl::W`](W) writer structure"]
impl crate::Writable for Sm2octrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM2OCTRL to value 0"]
impl crate::Resettable for Sm2octrlSpec {}
