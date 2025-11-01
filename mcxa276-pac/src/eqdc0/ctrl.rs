#[doc = "Register `CTRL` reader"]
pub type R = crate::R<CtrlSpec>;
#[doc = "Register `CTRL` writer"]
pub type W = crate::W<CtrlSpec>;
#[doc = "Load Okay\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ldok {
    #[doc = "0: No loading action taken. Users can write new values to buffered registers (writing into outer-set of these buffered registers)"]
    Ldok0 = 0,
    #[doc = "1: Outer-set values are ready to be loaded into inner-set and take effect. The loading time point depends on CTRL2\\[LDMOD\\]."]
    Ldok1 = 1,
}
impl From<Ldok> for bool {
    #[inline(always)]
    fn from(variant: Ldok) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LDOK` reader - Load Okay"]
pub type LdokR = crate::BitReader<Ldok>;
impl LdokR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ldok {
        match self.bits {
            false => Ldok::Ldok0,
            true => Ldok::Ldok1,
        }
    }
    #[doc = "No loading action taken. Users can write new values to buffered registers (writing into outer-set of these buffered registers)"]
    #[inline(always)]
    pub fn is_ldok0(&self) -> bool {
        *self == Ldok::Ldok0
    }
    #[doc = "Outer-set values are ready to be loaded into inner-set and take effect. The loading time point depends on CTRL2\\[LDMOD\\]."]
    #[inline(always)]
    pub fn is_ldok1(&self) -> bool {
        *self == Ldok::Ldok1
    }
}
#[doc = "Field `LDOK` writer - Load Okay"]
pub type LdokW<'a, REG> = crate::BitWriter<'a, REG, Ldok>;
impl<'a, REG> LdokW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No loading action taken. Users can write new values to buffered registers (writing into outer-set of these buffered registers)"]
    #[inline(always)]
    pub fn ldok0(self) -> &'a mut crate::W<REG> {
        self.variant(Ldok::Ldok0)
    }
    #[doc = "Outer-set values are ready to be loaded into inner-set and take effect. The loading time point depends on CTRL2\\[LDMOD\\]."]
    #[inline(always)]
    pub fn ldok1(self) -> &'a mut crate::W<REG> {
        self.variant(Ldok::Ldok1)
    }
}
#[doc = "DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dmaen {
    #[doc = "0: DMA is disabled"]
    Dmaen0 = 0,
    #[doc = "1: DMA is enabled. DMA request asserts automatically when the values in the outer-set of buffered compare registers (UCOMP0/LCOMP0;UCOMP1/LCOMP1;UCOMP2/LCOMP2;UCOMP3/LCOMP3), initial registers(UINIT/LINIT) and modulus registers (UMOD/LMOD) are loaded into the inner-set of buffer and then LDOK is cleared automatically. After the completion of this DMA transfer, LDOK is set automatically, it ensures outer-set values can be loaded into inner-set which in turn triggers DMA again."]
    Dmaen1 = 1,
}
impl From<Dmaen> for bool {
    #[inline(always)]
    fn from(variant: Dmaen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DMAEN` reader - DMA Enable"]
pub type DmaenR = crate::BitReader<Dmaen>;
impl DmaenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dmaen {
        match self.bits {
            false => Dmaen::Dmaen0,
            true => Dmaen::Dmaen1,
        }
    }
    #[doc = "DMA is disabled"]
    #[inline(always)]
    pub fn is_dmaen_0(&self) -> bool {
        *self == Dmaen::Dmaen0
    }
    #[doc = "DMA is enabled. DMA request asserts automatically when the values in the outer-set of buffered compare registers (UCOMP0/LCOMP0;UCOMP1/LCOMP1;UCOMP2/LCOMP2;UCOMP3/LCOMP3), initial registers(UINIT/LINIT) and modulus registers (UMOD/LMOD) are loaded into the inner-set of buffer and then LDOK is cleared automatically. After the completion of this DMA transfer, LDOK is set automatically, it ensures outer-set values can be loaded into inner-set which in turn triggers DMA again."]
    #[inline(always)]
    pub fn is_dmaen_1(&self) -> bool {
        *self == Dmaen::Dmaen1
    }
}
#[doc = "Field `DMAEN` writer - DMA Enable"]
pub type DmaenW<'a, REG> = crate::BitWriter<'a, REG, Dmaen>;
impl<'a, REG> DmaenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "DMA is disabled"]
    #[inline(always)]
    pub fn dmaen_0(self) -> &'a mut crate::W<REG> {
        self.variant(Dmaen::Dmaen0)
    }
    #[doc = "DMA is enabled. DMA request asserts automatically when the values in the outer-set of buffered compare registers (UCOMP0/LCOMP0;UCOMP1/LCOMP1;UCOMP2/LCOMP2;UCOMP3/LCOMP3), initial registers(UINIT/LINIT) and modulus registers (UMOD/LMOD) are loaded into the inner-set of buffer and then LDOK is cleared automatically. After the completion of this DMA transfer, LDOK is set automatically, it ensures outer-set values can be loaded into inner-set which in turn triggers DMA again."]
    #[inline(always)]
    pub fn dmaen_1(self) -> &'a mut crate::W<REG> {
        self.variant(Dmaen::Dmaen1)
    }
}
#[doc = "Watchdog Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wde {
    #[doc = "0: Disabled"]
    Wde0 = 0,
    #[doc = "1: Enabled"]
    Wde1 = 1,
}
impl From<Wde> for bool {
    #[inline(always)]
    fn from(variant: Wde) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WDE` reader - Watchdog Enable"]
pub type WdeR = crate::BitReader<Wde>;
impl WdeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wde {
        match self.bits {
            false => Wde::Wde0,
            true => Wde::Wde1,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_wde0(&self) -> bool {
        *self == Wde::Wde0
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_wde1(&self) -> bool {
        *self == Wde::Wde1
    }
}
#[doc = "Field `WDE` writer - Watchdog Enable"]
pub type WdeW<'a, REG> = crate::BitWriter<'a, REG, Wde>;
impl<'a, REG> WdeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn wde0(self) -> &'a mut crate::W<REG> {
        self.variant(Wde::Wde0)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn wde1(self) -> &'a mut crate::W<REG> {
        self.variant(Wde::Wde1)
    }
}
#[doc = "Watchdog Timeout Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wdie {
    #[doc = "0: Disabled"]
    Wdie0 = 0,
    #[doc = "1: Enabled"]
    Wdie1 = 1,
}
impl From<Wdie> for bool {
    #[inline(always)]
    fn from(variant: Wdie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WDIE` reader - Watchdog Timeout Interrupt Enable"]
pub type WdieR = crate::BitReader<Wdie>;
impl WdieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wdie {
        match self.bits {
            false => Wdie::Wdie0,
            true => Wdie::Wdie1,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_wdie0(&self) -> bool {
        *self == Wdie::Wdie0
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_wdie1(&self) -> bool {
        *self == Wdie::Wdie1
    }
}
#[doc = "Field `WDIE` writer - Watchdog Timeout Interrupt Enable"]
pub type WdieW<'a, REG> = crate::BitWriter<'a, REG, Wdie>;
impl<'a, REG> WdieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn wdie0(self) -> &'a mut crate::W<REG> {
        self.variant(Wdie::Wdie0)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn wdie1(self) -> &'a mut crate::W<REG> {
        self.variant(Wdie::Wdie1)
    }
}
#[doc = "Watchdog Timeout Interrupt Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wdirq {
    #[doc = "0: No Watchdog timeout interrupt has occurred"]
    Wdirq0 = 0,
    #[doc = "1: Watchdog timeout interrupt has occurred"]
    Wdirq1 = 1,
}
impl From<Wdirq> for bool {
    #[inline(always)]
    fn from(variant: Wdirq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WDIRQ` reader - Watchdog Timeout Interrupt Request"]
pub type WdirqR = crate::BitReader<Wdirq>;
impl WdirqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wdirq {
        match self.bits {
            false => Wdirq::Wdirq0,
            true => Wdirq::Wdirq1,
        }
    }
    #[doc = "No Watchdog timeout interrupt has occurred"]
    #[inline(always)]
    pub fn is_wdirq0(&self) -> bool {
        *self == Wdirq::Wdirq0
    }
    #[doc = "Watchdog timeout interrupt has occurred"]
    #[inline(always)]
    pub fn is_wdirq1(&self) -> bool {
        *self == Wdirq::Wdirq1
    }
}
#[doc = "Field `WDIRQ` writer - Watchdog Timeout Interrupt Request"]
pub type WdirqW<'a, REG> = crate::BitWriter1C<'a, REG, Wdirq>;
impl<'a, REG> WdirqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No Watchdog timeout interrupt has occurred"]
    #[inline(always)]
    pub fn wdirq0(self) -> &'a mut crate::W<REG> {
        self.variant(Wdirq::Wdirq0)
    }
    #[doc = "Watchdog timeout interrupt has occurred"]
    #[inline(always)]
    pub fn wdirq1(self) -> &'a mut crate::W<REG> {
        self.variant(Wdirq::Wdirq1)
    }
}
#[doc = "Select Positive/Negative Edge of INDEX/PRESET Pulse\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Xne {
    #[doc = "0: Use positive edge of INDEX/PRESET pulse"]
    Xne0 = 0,
    #[doc = "1: Use negative edge of INDEX/PRESET pulse"]
    Xne1 = 1,
}
impl From<Xne> for bool {
    #[inline(always)]
    fn from(variant: Xne) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `XNE` reader - Select Positive/Negative Edge of INDEX/PRESET Pulse"]
pub type XneR = crate::BitReader<Xne>;
impl XneR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Xne {
        match self.bits {
            false => Xne::Xne0,
            true => Xne::Xne1,
        }
    }
    #[doc = "Use positive edge of INDEX/PRESET pulse"]
    #[inline(always)]
    pub fn is_xne0(&self) -> bool {
        *self == Xne::Xne0
    }
    #[doc = "Use negative edge of INDEX/PRESET pulse"]
    #[inline(always)]
    pub fn is_xne1(&self) -> bool {
        *self == Xne::Xne1
    }
}
#[doc = "Field `XNE` writer - Select Positive/Negative Edge of INDEX/PRESET Pulse"]
pub type XneW<'a, REG> = crate::BitWriter<'a, REG, Xne>;
impl<'a, REG> XneW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Use positive edge of INDEX/PRESET pulse"]
    #[inline(always)]
    pub fn xne0(self) -> &'a mut crate::W<REG> {
        self.variant(Xne::Xne0)
    }
    #[doc = "Use negative edge of INDEX/PRESET pulse"]
    #[inline(always)]
    pub fn xne1(self) -> &'a mut crate::W<REG> {
        self.variant(Xne::Xne1)
    }
}
#[doc = "INDEX Triggered Initialization of Position Counters UPOS and LPOS\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Xip {
    #[doc = "0: INDEX pulse does not initialize the position counter"]
    Xip0 = 0,
    #[doc = "1: INDEX pulse initializes the position counter"]
    Xip1 = 1,
}
impl From<Xip> for bool {
    #[inline(always)]
    fn from(variant: Xip) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `XIP` reader - INDEX Triggered Initialization of Position Counters UPOS and LPOS"]
pub type XipR = crate::BitReader<Xip>;
impl XipR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Xip {
        match self.bits {
            false => Xip::Xip0,
            true => Xip::Xip1,
        }
    }
    #[doc = "INDEX pulse does not initialize the position counter"]
    #[inline(always)]
    pub fn is_xip0(&self) -> bool {
        *self == Xip::Xip0
    }
    #[doc = "INDEX pulse initializes the position counter"]
    #[inline(always)]
    pub fn is_xip1(&self) -> bool {
        *self == Xip::Xip1
    }
}
#[doc = "Field `XIP` writer - INDEX Triggered Initialization of Position Counters UPOS and LPOS"]
pub type XipW<'a, REG> = crate::BitWriter<'a, REG, Xip>;
impl<'a, REG> XipW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "INDEX pulse does not initialize the position counter"]
    #[inline(always)]
    pub fn xip0(self) -> &'a mut crate::W<REG> {
        self.variant(Xip::Xip0)
    }
    #[doc = "INDEX pulse initializes the position counter"]
    #[inline(always)]
    pub fn xip1(self) -> &'a mut crate::W<REG> {
        self.variant(Xip::Xip1)
    }
}
#[doc = "INDEX/PRESET Pulse Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Xie {
    #[doc = "0: Disabled"]
    Xie0 = 0,
    #[doc = "1: Enabled"]
    Xie1 = 1,
}
impl From<Xie> for bool {
    #[inline(always)]
    fn from(variant: Xie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `XIE` reader - INDEX/PRESET Pulse Interrupt Enable"]
pub type XieR = crate::BitReader<Xie>;
impl XieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Xie {
        match self.bits {
            false => Xie::Xie0,
            true => Xie::Xie1,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_xie0(&self) -> bool {
        *self == Xie::Xie0
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_xie1(&self) -> bool {
        *self == Xie::Xie1
    }
}
#[doc = "Field `XIE` writer - INDEX/PRESET Pulse Interrupt Enable"]
pub type XieW<'a, REG> = crate::BitWriter<'a, REG, Xie>;
impl<'a, REG> XieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn xie0(self) -> &'a mut crate::W<REG> {
        self.variant(Xie::Xie0)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn xie1(self) -> &'a mut crate::W<REG> {
        self.variant(Xie::Xie1)
    }
}
#[doc = "INDEX/PRESET Pulse Interrupt Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Xirq {
    #[doc = "0: INDEX/PRESET pulse has not occurred"]
    Xirq0 = 0,
    #[doc = "1: INDEX/PRESET pulse has occurred"]
    Xirq1 = 1,
}
impl From<Xirq> for bool {
    #[inline(always)]
    fn from(variant: Xirq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `XIRQ` reader - INDEX/PRESET Pulse Interrupt Request"]
pub type XirqR = crate::BitReader<Xirq>;
impl XirqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Xirq {
        match self.bits {
            false => Xirq::Xirq0,
            true => Xirq::Xirq1,
        }
    }
    #[doc = "INDEX/PRESET pulse has not occurred"]
    #[inline(always)]
    pub fn is_xirq0(&self) -> bool {
        *self == Xirq::Xirq0
    }
    #[doc = "INDEX/PRESET pulse has occurred"]
    #[inline(always)]
    pub fn is_xirq1(&self) -> bool {
        *self == Xirq::Xirq1
    }
}
#[doc = "Field `XIRQ` writer - INDEX/PRESET Pulse Interrupt Request"]
pub type XirqW<'a, REG> = crate::BitWriter1C<'a, REG, Xirq>;
impl<'a, REG> XirqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "INDEX/PRESET pulse has not occurred"]
    #[inline(always)]
    pub fn xirq0(self) -> &'a mut crate::W<REG> {
        self.variant(Xirq::Xirq0)
    }
    #[doc = "INDEX/PRESET pulse has occurred"]
    #[inline(always)]
    pub fn xirq1(self) -> &'a mut crate::W<REG> {
        self.variant(Xirq::Xirq1)
    }
}
#[doc = "Enable Single Phase Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ph1 {
    #[doc = "0: Standard quadrature decoder, where PHASEA and PHASEB represent a two-phase quadrature signal."]
    Ph10 = 0,
    #[doc = "1: Single phase mode, bypass the quadrature decoder, refer to CTRL2\\[CMODE\\] description"]
    Ph11 = 1,
}
impl From<Ph1> for bool {
    #[inline(always)]
    fn from(variant: Ph1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PH1` reader - Enable Single Phase Mode"]
pub type Ph1R = crate::BitReader<Ph1>;
impl Ph1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ph1 {
        match self.bits {
            false => Ph1::Ph10,
            true => Ph1::Ph11,
        }
    }
    #[doc = "Standard quadrature decoder, where PHASEA and PHASEB represent a two-phase quadrature signal."]
    #[inline(always)]
    pub fn is_ph10(&self) -> bool {
        *self == Ph1::Ph10
    }
    #[doc = "Single phase mode, bypass the quadrature decoder, refer to CTRL2\\[CMODE\\] description"]
    #[inline(always)]
    pub fn is_ph11(&self) -> bool {
        *self == Ph1::Ph11
    }
}
#[doc = "Field `PH1` writer - Enable Single Phase Mode"]
pub type Ph1W<'a, REG> = crate::BitWriter<'a, REG, Ph1>;
impl<'a, REG> Ph1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Standard quadrature decoder, where PHASEA and PHASEB represent a two-phase quadrature signal."]
    #[inline(always)]
    pub fn ph10(self) -> &'a mut crate::W<REG> {
        self.variant(Ph1::Ph10)
    }
    #[doc = "Single phase mode, bypass the quadrature decoder, refer to CTRL2\\[CMODE\\] description"]
    #[inline(always)]
    pub fn ph11(self) -> &'a mut crate::W<REG> {
        self.variant(Ph1::Ph11)
    }
}
#[doc = "Enable Reverse Direction Counting\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rev {
    #[doc = "0: Count normally and the position counter initialization uses upper/lower initialization register UINIT/LINIT"]
    Rev0 = 0,
    #[doc = "1: Count in the reverse direction and the position counter initialization uses upper/lower modulus register UMOD/LMOD"]
    Rev1 = 1,
}
impl From<Rev> for bool {
    #[inline(always)]
    fn from(variant: Rev) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `REV` reader - Enable Reverse Direction Counting"]
pub type RevR = crate::BitReader<Rev>;
impl RevR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rev {
        match self.bits {
            false => Rev::Rev0,
            true => Rev::Rev1,
        }
    }
    #[doc = "Count normally and the position counter initialization uses upper/lower initialization register UINIT/LINIT"]
    #[inline(always)]
    pub fn is_rev0(&self) -> bool {
        *self == Rev::Rev0
    }
    #[doc = "Count in the reverse direction and the position counter initialization uses upper/lower modulus register UMOD/LMOD"]
    #[inline(always)]
    pub fn is_rev1(&self) -> bool {
        *self == Rev::Rev1
    }
}
#[doc = "Field `REV` writer - Enable Reverse Direction Counting"]
pub type RevW<'a, REG> = crate::BitWriter<'a, REG, Rev>;
impl<'a, REG> RevW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Count normally and the position counter initialization uses upper/lower initialization register UINIT/LINIT"]
    #[inline(always)]
    pub fn rev0(self) -> &'a mut crate::W<REG> {
        self.variant(Rev::Rev0)
    }
    #[doc = "Count in the reverse direction and the position counter initialization uses upper/lower modulus register UMOD/LMOD"]
    #[inline(always)]
    pub fn rev1(self) -> &'a mut crate::W<REG> {
        self.variant(Rev::Rev1)
    }
}
#[doc = "Software-Triggered Initialization of Position Counters UPOS and LPOS\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swip {
    #[doc = "0: No action"]
    Swip0 = 0,
    #[doc = "1: Initialize position counter"]
    Swip1 = 1,
}
impl From<Swip> for bool {
    #[inline(always)]
    fn from(variant: Swip) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SWIP` reader - Software-Triggered Initialization of Position Counters UPOS and LPOS"]
pub type SwipR = crate::BitReader<Swip>;
impl SwipR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Swip {
        match self.bits {
            false => Swip::Swip0,
            true => Swip::Swip1,
        }
    }
    #[doc = "No action"]
    #[inline(always)]
    pub fn is_swip0(&self) -> bool {
        *self == Swip::Swip0
    }
    #[doc = "Initialize position counter"]
    #[inline(always)]
    pub fn is_swip1(&self) -> bool {
        *self == Swip::Swip1
    }
}
#[doc = "Field `SWIP` writer - Software-Triggered Initialization of Position Counters UPOS and LPOS"]
pub type SwipW<'a, REG> = crate::BitWriter<'a, REG, Swip>;
impl<'a, REG> SwipW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No action"]
    #[inline(always)]
    pub fn swip0(self) -> &'a mut crate::W<REG> {
        self.variant(Swip::Swip0)
    }
    #[doc = "Initialize position counter"]
    #[inline(always)]
    pub fn swip1(self) -> &'a mut crate::W<REG> {
        self.variant(Swip::Swip1)
    }
}
#[doc = "Use Negative Edge of HOME/ENABLE Input\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hne {
    #[doc = "0: When CTRL\\[OPMODE\\] = 0,use HOME positive edge to trigger initialization of position counters. When CTRL\\[OPMODE\\] = 1,use ENABLE high level to enable POS/POSD/WDG/REV counters"]
    Hne0 = 0,
    #[doc = "1: When CTRL\\[OPMODE\\] = 0,use HOME negative edge to trigger initialization of position counters. When CTRL\\[OPMODE\\] = 1,use ENABLE low level to enable POS/POSD/WDG/REV counters"]
    Hne1 = 1,
}
impl From<Hne> for bool {
    #[inline(always)]
    fn from(variant: Hne) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HNE` reader - Use Negative Edge of HOME/ENABLE Input"]
pub type HneR = crate::BitReader<Hne>;
impl HneR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hne {
        match self.bits {
            false => Hne::Hne0,
            true => Hne::Hne1,
        }
    }
    #[doc = "When CTRL\\[OPMODE\\] = 0,use HOME positive edge to trigger initialization of position counters. When CTRL\\[OPMODE\\] = 1,use ENABLE high level to enable POS/POSD/WDG/REV counters"]
    #[inline(always)]
    pub fn is_hne0(&self) -> bool {
        *self == Hne::Hne0
    }
    #[doc = "When CTRL\\[OPMODE\\] = 0,use HOME negative edge to trigger initialization of position counters. When CTRL\\[OPMODE\\] = 1,use ENABLE low level to enable POS/POSD/WDG/REV counters"]
    #[inline(always)]
    pub fn is_hne1(&self) -> bool {
        *self == Hne::Hne1
    }
}
#[doc = "Field `HNE` writer - Use Negative Edge of HOME/ENABLE Input"]
pub type HneW<'a, REG> = crate::BitWriter<'a, REG, Hne>;
impl<'a, REG> HneW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "When CTRL\\[OPMODE\\] = 0,use HOME positive edge to trigger initialization of position counters. When CTRL\\[OPMODE\\] = 1,use ENABLE high level to enable POS/POSD/WDG/REV counters"]
    #[inline(always)]
    pub fn hne0(self) -> &'a mut crate::W<REG> {
        self.variant(Hne::Hne0)
    }
    #[doc = "When CTRL\\[OPMODE\\] = 0,use HOME negative edge to trigger initialization of position counters. When CTRL\\[OPMODE\\] = 1,use ENABLE low level to enable POS/POSD/WDG/REV counters"]
    #[inline(always)]
    pub fn hne1(self) -> &'a mut crate::W<REG> {
        self.variant(Hne::Hne1)
    }
}
#[doc = "Enable HOME to Initialize Position Counter UPOS/LPOS\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hip {
    #[doc = "0: No action"]
    Hip0 = 0,
    #[doc = "1: HOME signal initializes the position counter"]
    Hip1 = 1,
}
impl From<Hip> for bool {
    #[inline(always)]
    fn from(variant: Hip) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HIP` reader - Enable HOME to Initialize Position Counter UPOS/LPOS"]
pub type HipR = crate::BitReader<Hip>;
impl HipR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hip {
        match self.bits {
            false => Hip::Hip0,
            true => Hip::Hip1,
        }
    }
    #[doc = "No action"]
    #[inline(always)]
    pub fn is_hip0(&self) -> bool {
        *self == Hip::Hip0
    }
    #[doc = "HOME signal initializes the position counter"]
    #[inline(always)]
    pub fn is_hip1(&self) -> bool {
        *self == Hip::Hip1
    }
}
#[doc = "Field `HIP` writer - Enable HOME to Initialize Position Counter UPOS/LPOS"]
pub type HipW<'a, REG> = crate::BitWriter<'a, REG, Hip>;
impl<'a, REG> HipW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No action"]
    #[inline(always)]
    pub fn hip0(self) -> &'a mut crate::W<REG> {
        self.variant(Hip::Hip0)
    }
    #[doc = "HOME signal initializes the position counter"]
    #[inline(always)]
    pub fn hip1(self) -> &'a mut crate::W<REG> {
        self.variant(Hip::Hip1)
    }
}
#[doc = "HOME/ENABLE Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hie {
    #[doc = "0: Disabled"]
    Hie0 = 0,
    #[doc = "1: Enabled"]
    Hie1 = 1,
}
impl From<Hie> for bool {
    #[inline(always)]
    fn from(variant: Hie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HIE` reader - HOME/ENABLE Interrupt Enable"]
pub type HieR = crate::BitReader<Hie>;
impl HieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hie {
        match self.bits {
            false => Hie::Hie0,
            true => Hie::Hie1,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_hie0(&self) -> bool {
        *self == Hie::Hie0
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_hie1(&self) -> bool {
        *self == Hie::Hie1
    }
}
#[doc = "Field `HIE` writer - HOME/ENABLE Interrupt Enable"]
pub type HieW<'a, REG> = crate::BitWriter<'a, REG, Hie>;
impl<'a, REG> HieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn hie0(self) -> &'a mut crate::W<REG> {
        self.variant(Hie::Hie0)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn hie1(self) -> &'a mut crate::W<REG> {
        self.variant(Hie::Hie1)
    }
}
#[doc = "HOME/ENABLE Signal Transition Interrupt Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hirq {
    #[doc = "0: No transition on the HOME/ENABLE signal has occurred"]
    Hirq0 = 0,
    #[doc = "1: A transition on the HOME/ENABLE signal has occurred"]
    Hirq1 = 1,
}
impl From<Hirq> for bool {
    #[inline(always)]
    fn from(variant: Hirq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HIRQ` reader - HOME/ENABLE Signal Transition Interrupt Request"]
pub type HirqR = crate::BitReader<Hirq>;
impl HirqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hirq {
        match self.bits {
            false => Hirq::Hirq0,
            true => Hirq::Hirq1,
        }
    }
    #[doc = "No transition on the HOME/ENABLE signal has occurred"]
    #[inline(always)]
    pub fn is_hirq0(&self) -> bool {
        *self == Hirq::Hirq0
    }
    #[doc = "A transition on the HOME/ENABLE signal has occurred"]
    #[inline(always)]
    pub fn is_hirq1(&self) -> bool {
        *self == Hirq::Hirq1
    }
}
#[doc = "Field `HIRQ` writer - HOME/ENABLE Signal Transition Interrupt Request"]
pub type HirqW<'a, REG> = crate::BitWriter1C<'a, REG, Hirq>;
impl<'a, REG> HirqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No transition on the HOME/ENABLE signal has occurred"]
    #[inline(always)]
    pub fn hirq0(self) -> &'a mut crate::W<REG> {
        self.variant(Hirq::Hirq0)
    }
    #[doc = "A transition on the HOME/ENABLE signal has occurred"]
    #[inline(always)]
    pub fn hirq1(self) -> &'a mut crate::W<REG> {
        self.variant(Hirq::Hirq1)
    }
}
impl R {
    #[doc = "Bit 0 - Load Okay"]
    #[inline(always)]
    pub fn ldok(&self) -> LdokR {
        LdokR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - DMA Enable"]
    #[inline(always)]
    pub fn dmaen(&self) -> DmaenR {
        DmaenR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Watchdog Enable"]
    #[inline(always)]
    pub fn wde(&self) -> WdeR {
        WdeR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Watchdog Timeout Interrupt Enable"]
    #[inline(always)]
    pub fn wdie(&self) -> WdieR {
        WdieR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Watchdog Timeout Interrupt Request"]
    #[inline(always)]
    pub fn wdirq(&self) -> WdirqR {
        WdirqR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Select Positive/Negative Edge of INDEX/PRESET Pulse"]
    #[inline(always)]
    pub fn xne(&self) -> XneR {
        XneR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - INDEX Triggered Initialization of Position Counters UPOS and LPOS"]
    #[inline(always)]
    pub fn xip(&self) -> XipR {
        XipR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - INDEX/PRESET Pulse Interrupt Enable"]
    #[inline(always)]
    pub fn xie(&self) -> XieR {
        XieR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - INDEX/PRESET Pulse Interrupt Request"]
    #[inline(always)]
    pub fn xirq(&self) -> XirqR {
        XirqR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Enable Single Phase Mode"]
    #[inline(always)]
    pub fn ph1(&self) -> Ph1R {
        Ph1R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Enable Reverse Direction Counting"]
    #[inline(always)]
    pub fn rev(&self) -> RevR {
        RevR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Software-Triggered Initialization of Position Counters UPOS and LPOS"]
    #[inline(always)]
    pub fn swip(&self) -> SwipR {
        SwipR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Use Negative Edge of HOME/ENABLE Input"]
    #[inline(always)]
    pub fn hne(&self) -> HneR {
        HneR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Enable HOME to Initialize Position Counter UPOS/LPOS"]
    #[inline(always)]
    pub fn hip(&self) -> HipR {
        HipR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - HOME/ENABLE Interrupt Enable"]
    #[inline(always)]
    pub fn hie(&self) -> HieR {
        HieR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - HOME/ENABLE Signal Transition Interrupt Request"]
    #[inline(always)]
    pub fn hirq(&self) -> HirqR {
        HirqR::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Load Okay"]
    #[inline(always)]
    pub fn ldok(&mut self) -> LdokW<CtrlSpec> {
        LdokW::new(self, 0)
    }
    #[doc = "Bit 1 - DMA Enable"]
    #[inline(always)]
    pub fn dmaen(&mut self) -> DmaenW<CtrlSpec> {
        DmaenW::new(self, 1)
    }
    #[doc = "Bit 2 - Watchdog Enable"]
    #[inline(always)]
    pub fn wde(&mut self) -> WdeW<CtrlSpec> {
        WdeW::new(self, 2)
    }
    #[doc = "Bit 3 - Watchdog Timeout Interrupt Enable"]
    #[inline(always)]
    pub fn wdie(&mut self) -> WdieW<CtrlSpec> {
        WdieW::new(self, 3)
    }
    #[doc = "Bit 4 - Watchdog Timeout Interrupt Request"]
    #[inline(always)]
    pub fn wdirq(&mut self) -> WdirqW<CtrlSpec> {
        WdirqW::new(self, 4)
    }
    #[doc = "Bit 5 - Select Positive/Negative Edge of INDEX/PRESET Pulse"]
    #[inline(always)]
    pub fn xne(&mut self) -> XneW<CtrlSpec> {
        XneW::new(self, 5)
    }
    #[doc = "Bit 6 - INDEX Triggered Initialization of Position Counters UPOS and LPOS"]
    #[inline(always)]
    pub fn xip(&mut self) -> XipW<CtrlSpec> {
        XipW::new(self, 6)
    }
    #[doc = "Bit 7 - INDEX/PRESET Pulse Interrupt Enable"]
    #[inline(always)]
    pub fn xie(&mut self) -> XieW<CtrlSpec> {
        XieW::new(self, 7)
    }
    #[doc = "Bit 8 - INDEX/PRESET Pulse Interrupt Request"]
    #[inline(always)]
    pub fn xirq(&mut self) -> XirqW<CtrlSpec> {
        XirqW::new(self, 8)
    }
    #[doc = "Bit 9 - Enable Single Phase Mode"]
    #[inline(always)]
    pub fn ph1(&mut self) -> Ph1W<CtrlSpec> {
        Ph1W::new(self, 9)
    }
    #[doc = "Bit 10 - Enable Reverse Direction Counting"]
    #[inline(always)]
    pub fn rev(&mut self) -> RevW<CtrlSpec> {
        RevW::new(self, 10)
    }
    #[doc = "Bit 11 - Software-Triggered Initialization of Position Counters UPOS and LPOS"]
    #[inline(always)]
    pub fn swip(&mut self) -> SwipW<CtrlSpec> {
        SwipW::new(self, 11)
    }
    #[doc = "Bit 12 - Use Negative Edge of HOME/ENABLE Input"]
    #[inline(always)]
    pub fn hne(&mut self) -> HneW<CtrlSpec> {
        HneW::new(self, 12)
    }
    #[doc = "Bit 13 - Enable HOME to Initialize Position Counter UPOS/LPOS"]
    #[inline(always)]
    pub fn hip(&mut self) -> HipW<CtrlSpec> {
        HipW::new(self, 13)
    }
    #[doc = "Bit 14 - HOME/ENABLE Interrupt Enable"]
    #[inline(always)]
    pub fn hie(&mut self) -> HieW<CtrlSpec> {
        HieW::new(self, 14)
    }
    #[doc = "Bit 15 - HOME/ENABLE Signal Transition Interrupt Request"]
    #[inline(always)]
    pub fn hirq(&mut self) -> HirqW<CtrlSpec> {
        HirqW::new(self, 15)
    }
}
#[doc = "Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CtrlSpec;
impl crate::RegisterSpec for CtrlSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`ctrl::R`](R) reader structure"]
impl crate::Readable for CtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`ctrl::W`](W) writer structure"]
impl crate::Writable for CtrlSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u16 = 0x8110;
}
#[doc = "`reset()` method sets CTRL to value 0"]
impl crate::Resettable for CtrlSpec {}
