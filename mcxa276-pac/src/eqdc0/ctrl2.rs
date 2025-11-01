#[doc = "Register `CTRL2` reader"]
pub type R = crate::R<Ctrl2Spec>;
#[doc = "Register `CTRL2` writer"]
pub type W = crate::W<Ctrl2Spec>;
#[doc = "Field `UPDHLD` reader - Update Hold Registers"]
pub type UpdhldR = crate::BitReader;
#[doc = "Field `UPDHLD` writer - Update Hold Registers"]
pub type UpdhldW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `UPDPOS` reader - Update Position Registers"]
pub type UpdposR = crate::BitReader;
#[doc = "Field `UPDPOS` writer - Update Position Registers"]
pub type UpdposW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Operation Mode Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opmode {
    #[doc = "0: Decode Mode: Input nodes INDEX/PRESET and HOME/ENABLE are assigned to function of INDEX and HOME."]
    Opmode0 = 0,
    #[doc = "1: Count Mode: Input nodes INDEX/PRESET and HOME/ENABLE are assigned to functions of PRESET and ENABLE. In this mode: (1)only when ENABLE=1, all counters (position/position difference/revolution/watchdog) can run, when ENABLE=0, all counters (position/position difference/revolution/watchdog) can't run. (2) the rising edge of PRESET input can initialize position/revolution/watchdog counters (position counter initialization also need referring to bit CTRL\\[REV\\])."]
    Opmode1 = 1,
}
impl From<Opmode> for bool {
    #[inline(always)]
    fn from(variant: Opmode) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OPMODE` reader - Operation Mode Select"]
pub type OpmodeR = crate::BitReader<Opmode>;
impl OpmodeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Opmode {
        match self.bits {
            false => Opmode::Opmode0,
            true => Opmode::Opmode1,
        }
    }
    #[doc = "Decode Mode: Input nodes INDEX/PRESET and HOME/ENABLE are assigned to function of INDEX and HOME."]
    #[inline(always)]
    pub fn is_opmode0(&self) -> bool {
        *self == Opmode::Opmode0
    }
    #[doc = "Count Mode: Input nodes INDEX/PRESET and HOME/ENABLE are assigned to functions of PRESET and ENABLE. In this mode: (1)only when ENABLE=1, all counters (position/position difference/revolution/watchdog) can run, when ENABLE=0, all counters (position/position difference/revolution/watchdog) can't run. (2) the rising edge of PRESET input can initialize position/revolution/watchdog counters (position counter initialization also need referring to bit CTRL\\[REV\\])."]
    #[inline(always)]
    pub fn is_opmode1(&self) -> bool {
        *self == Opmode::Opmode1
    }
}
#[doc = "Field `OPMODE` writer - Operation Mode Select"]
pub type OpmodeW<'a, REG> = crate::BitWriter<'a, REG, Opmode>;
impl<'a, REG> OpmodeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Decode Mode: Input nodes INDEX/PRESET and HOME/ENABLE are assigned to function of INDEX and HOME."]
    #[inline(always)]
    pub fn opmode0(self) -> &'a mut crate::W<REG> {
        self.variant(Opmode::Opmode0)
    }
    #[doc = "Count Mode: Input nodes INDEX/PRESET and HOME/ENABLE are assigned to functions of PRESET and ENABLE. In this mode: (1)only when ENABLE=1, all counters (position/position difference/revolution/watchdog) can run, when ENABLE=0, all counters (position/position difference/revolution/watchdog) can't run. (2) the rising edge of PRESET input can initialize position/revolution/watchdog counters (position counter initialization also need referring to bit CTRL\\[REV\\])."]
    #[inline(always)]
    pub fn opmode1(self) -> &'a mut crate::W<REG> {
        self.variant(Opmode::Opmode1)
    }
}
#[doc = "Buffered Register Load (Update) Mode Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ldmod {
    #[doc = "0: Buffered registers are loaded and take effect immediately upon CTRL\\[LDOK\\] is set."]
    Ldmod0 = 0,
    #[doc = "1: Buffered registers are loaded and take effect at the next roll-over or roll-under if CTRL\\[LDOK\\] is set."]
    Ldmod1 = 1,
}
impl From<Ldmod> for bool {
    #[inline(always)]
    fn from(variant: Ldmod) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LDMOD` reader - Buffered Register Load (Update) Mode Select"]
pub type LdmodR = crate::BitReader<Ldmod>;
impl LdmodR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ldmod {
        match self.bits {
            false => Ldmod::Ldmod0,
            true => Ldmod::Ldmod1,
        }
    }
    #[doc = "Buffered registers are loaded and take effect immediately upon CTRL\\[LDOK\\] is set."]
    #[inline(always)]
    pub fn is_ldmod0(&self) -> bool {
        *self == Ldmod::Ldmod0
    }
    #[doc = "Buffered registers are loaded and take effect at the next roll-over or roll-under if CTRL\\[LDOK\\] is set."]
    #[inline(always)]
    pub fn is_ldmod1(&self) -> bool {
        *self == Ldmod::Ldmod1
    }
}
#[doc = "Field `LDMOD` writer - Buffered Register Load (Update) Mode Select"]
pub type LdmodW<'a, REG> = crate::BitWriter<'a, REG, Ldmod>;
impl<'a, REG> LdmodW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Buffered registers are loaded and take effect immediately upon CTRL\\[LDOK\\] is set."]
    #[inline(always)]
    pub fn ldmod0(self) -> &'a mut crate::W<REG> {
        self.variant(Ldmod::Ldmod0)
    }
    #[doc = "Buffered registers are loaded and take effect at the next roll-over or roll-under if CTRL\\[LDOK\\] is set."]
    #[inline(always)]
    pub fn ldmod1(self) -> &'a mut crate::W<REG> {
        self.variant(Ldmod::Ldmod1)
    }
}
#[doc = "Revolution Counter Modulus Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Revmod {
    #[doc = "0: Use INDEX pulse to increment/decrement revolution counter (REV)"]
    Revmod0 = 0,
    #[doc = "1: Use modulus counting roll-over/under to increment/decrement revolution counter (REV)"]
    Revmod1 = 1,
}
impl From<Revmod> for bool {
    #[inline(always)]
    fn from(variant: Revmod) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `REVMOD` reader - Revolution Counter Modulus Enable"]
pub type RevmodR = crate::BitReader<Revmod>;
impl RevmodR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Revmod {
        match self.bits {
            false => Revmod::Revmod0,
            true => Revmod::Revmod1,
        }
    }
    #[doc = "Use INDEX pulse to increment/decrement revolution counter (REV)"]
    #[inline(always)]
    pub fn is_revmod0(&self) -> bool {
        *self == Revmod::Revmod0
    }
    #[doc = "Use modulus counting roll-over/under to increment/decrement revolution counter (REV)"]
    #[inline(always)]
    pub fn is_revmod1(&self) -> bool {
        *self == Revmod::Revmod1
    }
}
#[doc = "Field `REVMOD` writer - Revolution Counter Modulus Enable"]
pub type RevmodW<'a, REG> = crate::BitWriter<'a, REG, Revmod>;
impl<'a, REG> RevmodW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Use INDEX pulse to increment/decrement revolution counter (REV)"]
    #[inline(always)]
    pub fn revmod0(self) -> &'a mut crate::W<REG> {
        self.variant(Revmod::Revmod0)
    }
    #[doc = "Use modulus counting roll-over/under to increment/decrement revolution counter (REV)"]
    #[inline(always)]
    pub fn revmod1(self) -> &'a mut crate::W<REG> {
        self.variant(Revmod::Revmod1)
    }
}
#[doc = "Output Control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outctl {
    #[doc = "0: POS_MATCH\\[x\\](x range is 0-3) is asserted when the Position Counter is equal to according compare value (UCOMPx/LCOMPx)(x range is 0-3), and de-asserted when the Position Counter not equal to the compare value (UCOMPx/LCOMPx)(x range is 0-3)"]
    Outctl0 = 0,
    #[doc = "1: All POS_MATCH\\[x\\](x range is 0-3) are asserted a pulse, when the UPOS, LPOS, REV, or POSD registers are read"]
    Outctl1 = 1,
}
impl From<Outctl> for bool {
    #[inline(always)]
    fn from(variant: Outctl) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OUTCTL` reader - Output Control"]
pub type OutctlR = crate::BitReader<Outctl>;
impl OutctlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Outctl {
        match self.bits {
            false => Outctl::Outctl0,
            true => Outctl::Outctl1,
        }
    }
    #[doc = "POS_MATCH\\[x\\](x range is 0-3) is asserted when the Position Counter is equal to according compare value (UCOMPx/LCOMPx)(x range is 0-3), and de-asserted when the Position Counter not equal to the compare value (UCOMPx/LCOMPx)(x range is 0-3)"]
    #[inline(always)]
    pub fn is_outctl0(&self) -> bool {
        *self == Outctl::Outctl0
    }
    #[doc = "All POS_MATCH\\[x\\](x range is 0-3) are asserted a pulse, when the UPOS, LPOS, REV, or POSD registers are read"]
    #[inline(always)]
    pub fn is_outctl1(&self) -> bool {
        *self == Outctl::Outctl1
    }
}
#[doc = "Field `OUTCTL` writer - Output Control"]
pub type OutctlW<'a, REG> = crate::BitWriter<'a, REG, Outctl>;
impl<'a, REG> OutctlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "POS_MATCH\\[x\\](x range is 0-3) is asserted when the Position Counter is equal to according compare value (UCOMPx/LCOMPx)(x range is 0-3), and de-asserted when the Position Counter not equal to the compare value (UCOMPx/LCOMPx)(x range is 0-3)"]
    #[inline(always)]
    pub fn outctl0(self) -> &'a mut crate::W<REG> {
        self.variant(Outctl::Outctl0)
    }
    #[doc = "All POS_MATCH\\[x\\](x range is 0-3) are asserted a pulse, when the UPOS, LPOS, REV, or POSD registers are read"]
    #[inline(always)]
    pub fn outctl1(self) -> &'a mut crate::W<REG> {
        self.variant(Outctl::Outctl1)
    }
}
#[doc = "Period measurement function enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pmen {
    #[doc = "0: Period measurement functions are not used. POSD is loaded to POSDH and then cleared whenever POSD, UPOS, LPOS or REV is read."]
    Pmen0 = 0,
    #[doc = "1: Period measurement functions are used. POSD is loaded into POSDH and then cleared only when POSD is read."]
    Pmen1 = 1,
}
impl From<Pmen> for bool {
    #[inline(always)]
    fn from(variant: Pmen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PMEN` reader - Period measurement function enable"]
pub type PmenR = crate::BitReader<Pmen>;
impl PmenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pmen {
        match self.bits {
            false => Pmen::Pmen0,
            true => Pmen::Pmen1,
        }
    }
    #[doc = "Period measurement functions are not used. POSD is loaded to POSDH and then cleared whenever POSD, UPOS, LPOS or REV is read."]
    #[inline(always)]
    pub fn is_pmen0(&self) -> bool {
        *self == Pmen::Pmen0
    }
    #[doc = "Period measurement functions are used. POSD is loaded into POSDH and then cleared only when POSD is read."]
    #[inline(always)]
    pub fn is_pmen1(&self) -> bool {
        *self == Pmen::Pmen1
    }
}
#[doc = "Field `PMEN` writer - Period measurement function enable"]
pub type PmenW<'a, REG> = crate::BitWriter<'a, REG, Pmen>;
impl<'a, REG> PmenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Period measurement functions are not used. POSD is loaded to POSDH and then cleared whenever POSD, UPOS, LPOS or REV is read."]
    #[inline(always)]
    pub fn pmen0(self) -> &'a mut crate::W<REG> {
        self.variant(Pmen::Pmen0)
    }
    #[doc = "Period measurement functions are used. POSD is loaded into POSDH and then cleared only when POSD is read."]
    #[inline(always)]
    pub fn pmen1(self) -> &'a mut crate::W<REG> {
        self.variant(Pmen::Pmen1)
    }
}
#[doc = "Enables/disables the position counter to be initialized by Index Event Edge Mark\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Emip {
    #[doc = "0: disables the position counter to be initialized by Index Event Edge Mark"]
    Emip0 = 0,
    #[doc = "1: enables the position counter to be initialized by Index Event Edge Mark."]
    Emip1 = 1,
}
impl From<Emip> for bool {
    #[inline(always)]
    fn from(variant: Emip) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EMIP` reader - Enables/disables the position counter to be initialized by Index Event Edge Mark"]
pub type EmipR = crate::BitReader<Emip>;
impl EmipR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Emip {
        match self.bits {
            false => Emip::Emip0,
            true => Emip::Emip1,
        }
    }
    #[doc = "disables the position counter to be initialized by Index Event Edge Mark"]
    #[inline(always)]
    pub fn is_emip0(&self) -> bool {
        *self == Emip::Emip0
    }
    #[doc = "enables the position counter to be initialized by Index Event Edge Mark."]
    #[inline(always)]
    pub fn is_emip1(&self) -> bool {
        *self == Emip::Emip1
    }
}
#[doc = "Field `EMIP` writer - Enables/disables the position counter to be initialized by Index Event Edge Mark"]
pub type EmipW<'a, REG> = crate::BitWriter<'a, REG, Emip>;
impl<'a, REG> EmipW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "disables the position counter to be initialized by Index Event Edge Mark"]
    #[inline(always)]
    pub fn emip0(self) -> &'a mut crate::W<REG> {
        self.variant(Emip::Emip0)
    }
    #[doc = "enables the position counter to be initialized by Index Event Edge Mark."]
    #[inline(always)]
    pub fn emip1(self) -> &'a mut crate::W<REG> {
        self.variant(Emip::Emip1)
    }
}
#[doc = "Initial Position Register\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Initpos {
    #[doc = "0: Don't initialize position counter on rising edge of TRIGGER"]
    Initpos0 = 0,
    #[doc = "1: Initialize position counter on rising edge of TRIGGER"]
    Initpos1 = 1,
}
impl From<Initpos> for bool {
    #[inline(always)]
    fn from(variant: Initpos) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INITPOS` reader - Initial Position Register"]
pub type InitposR = crate::BitReader<Initpos>;
impl InitposR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Initpos {
        match self.bits {
            false => Initpos::Initpos0,
            true => Initpos::Initpos1,
        }
    }
    #[doc = "Don't initialize position counter on rising edge of TRIGGER"]
    #[inline(always)]
    pub fn is_initpos0(&self) -> bool {
        *self == Initpos::Initpos0
    }
    #[doc = "Initialize position counter on rising edge of TRIGGER"]
    #[inline(always)]
    pub fn is_initpos1(&self) -> bool {
        *self == Initpos::Initpos1
    }
}
#[doc = "Field `INITPOS` writer - Initial Position Register"]
pub type InitposW<'a, REG> = crate::BitWriter<'a, REG, Initpos>;
impl<'a, REG> InitposW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Don't initialize position counter on rising edge of TRIGGER"]
    #[inline(always)]
    pub fn initpos0(self) -> &'a mut crate::W<REG> {
        self.variant(Initpos::Initpos0)
    }
    #[doc = "Initialize position counter on rising edge of TRIGGER"]
    #[inline(always)]
    pub fn initpos1(self) -> &'a mut crate::W<REG> {
        self.variant(Initpos::Initpos1)
    }
}
#[doc = "Count Once\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Once {
    #[doc = "0: Position counter counts repeatedly"]
    Once0 = 0,
    #[doc = "1: Position counter counts until roll-over or roll-under, then stop."]
    Once1 = 1,
}
impl From<Once> for bool {
    #[inline(always)]
    fn from(variant: Once) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ONCE` reader - Count Once"]
pub type OnceR = crate::BitReader<Once>;
impl OnceR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Once {
        match self.bits {
            false => Once::Once0,
            true => Once::Once1,
        }
    }
    #[doc = "Position counter counts repeatedly"]
    #[inline(always)]
    pub fn is_once0(&self) -> bool {
        *self == Once::Once0
    }
    #[doc = "Position counter counts until roll-over or roll-under, then stop."]
    #[inline(always)]
    pub fn is_once1(&self) -> bool {
        *self == Once::Once1
    }
}
#[doc = "Field `ONCE` writer - Count Once"]
pub type OnceW<'a, REG> = crate::BitWriter<'a, REG, Once>;
impl<'a, REG> OnceW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Position counter counts repeatedly"]
    #[inline(always)]
    pub fn once0(self) -> &'a mut crate::W<REG> {
        self.variant(Once::Once0)
    }
    #[doc = "Position counter counts until roll-over or roll-under, then stop."]
    #[inline(always)]
    pub fn once1(self) -> &'a mut crate::W<REG> {
        self.variant(Once::Once1)
    }
}
#[doc = "Field `CMODE` reader - Counting Mode"]
pub type CmodeR = crate::FieldReader;
#[doc = "Field `CMODE` writer - Counting Mode"]
pub type CmodeW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
impl R {
    #[doc = "Bit 0 - Update Hold Registers"]
    #[inline(always)]
    pub fn updhld(&self) -> UpdhldR {
        UpdhldR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Update Position Registers"]
    #[inline(always)]
    pub fn updpos(&self) -> UpdposR {
        UpdposR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Operation Mode Select"]
    #[inline(always)]
    pub fn opmode(&self) -> OpmodeR {
        OpmodeR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Buffered Register Load (Update) Mode Select"]
    #[inline(always)]
    pub fn ldmod(&self) -> LdmodR {
        LdmodR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 8 - Revolution Counter Modulus Enable"]
    #[inline(always)]
    pub fn revmod(&self) -> RevmodR {
        RevmodR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Output Control"]
    #[inline(always)]
    pub fn outctl(&self) -> OutctlR {
        OutctlR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Period measurement function enable"]
    #[inline(always)]
    pub fn pmen(&self) -> PmenR {
        PmenR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Enables/disables the position counter to be initialized by Index Event Edge Mark"]
    #[inline(always)]
    pub fn emip(&self) -> EmipR {
        EmipR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Initial Position Register"]
    #[inline(always)]
    pub fn initpos(&self) -> InitposR {
        InitposR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Count Once"]
    #[inline(always)]
    pub fn once(&self) -> OnceR {
        OnceR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bits 14:15 - Counting Mode"]
    #[inline(always)]
    pub fn cmode(&self) -> CmodeR {
        CmodeR::new(((self.bits >> 14) & 3) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Update Hold Registers"]
    #[inline(always)]
    pub fn updhld(&mut self) -> UpdhldW<Ctrl2Spec> {
        UpdhldW::new(self, 0)
    }
    #[doc = "Bit 1 - Update Position Registers"]
    #[inline(always)]
    pub fn updpos(&mut self) -> UpdposW<Ctrl2Spec> {
        UpdposW::new(self, 1)
    }
    #[doc = "Bit 2 - Operation Mode Select"]
    #[inline(always)]
    pub fn opmode(&mut self) -> OpmodeW<Ctrl2Spec> {
        OpmodeW::new(self, 2)
    }
    #[doc = "Bit 3 - Buffered Register Load (Update) Mode Select"]
    #[inline(always)]
    pub fn ldmod(&mut self) -> LdmodW<Ctrl2Spec> {
        LdmodW::new(self, 3)
    }
    #[doc = "Bit 8 - Revolution Counter Modulus Enable"]
    #[inline(always)]
    pub fn revmod(&mut self) -> RevmodW<Ctrl2Spec> {
        RevmodW::new(self, 8)
    }
    #[doc = "Bit 9 - Output Control"]
    #[inline(always)]
    pub fn outctl(&mut self) -> OutctlW<Ctrl2Spec> {
        OutctlW::new(self, 9)
    }
    #[doc = "Bit 10 - Period measurement function enable"]
    #[inline(always)]
    pub fn pmen(&mut self) -> PmenW<Ctrl2Spec> {
        PmenW::new(self, 10)
    }
    #[doc = "Bit 11 - Enables/disables the position counter to be initialized by Index Event Edge Mark"]
    #[inline(always)]
    pub fn emip(&mut self) -> EmipW<Ctrl2Spec> {
        EmipW::new(self, 11)
    }
    #[doc = "Bit 12 - Initial Position Register"]
    #[inline(always)]
    pub fn initpos(&mut self) -> InitposW<Ctrl2Spec> {
        InitposW::new(self, 12)
    }
    #[doc = "Bit 13 - Count Once"]
    #[inline(always)]
    pub fn once(&mut self) -> OnceW<Ctrl2Spec> {
        OnceW::new(self, 13)
    }
    #[doc = "Bits 14:15 - Counting Mode"]
    #[inline(always)]
    pub fn cmode(&mut self) -> CmodeW<Ctrl2Spec> {
        CmodeW::new(self, 14)
    }
}
#[doc = "Control 2 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ctrl2Spec;
impl crate::RegisterSpec for Ctrl2Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`ctrl2::R`](R) reader structure"]
impl crate::Readable for Ctrl2Spec {}
#[doc = "`write(|w| ..)` method takes [`ctrl2::W`](W) writer structure"]
impl crate::Writable for Ctrl2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CTRL2 to value 0"]
impl crate::Resettable for Ctrl2Spec {}
