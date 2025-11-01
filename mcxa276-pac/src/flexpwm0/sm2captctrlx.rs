#[doc = "Register `SM2CAPTCTRLX` reader"]
pub type R = crate::R<Sm2captctrlxSpec>;
#[doc = "Register `SM2CAPTCTRLX` writer"]
pub type W = crate::W<Sm2captctrlxSpec>;
#[doc = "Arm X\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Armx {
    #[doc = "0: Input capture operation is disabled."]
    Disabled = 0,
    #[doc = "1: Input capture operation as specified by CAPTCTRLX\\[EDGXx\\] is enabled."]
    Enabled = 1,
}
impl From<Armx> for bool {
    #[inline(always)]
    fn from(variant: Armx) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ARMX` reader - Arm X"]
pub type ArmxR = crate::BitReader<Armx>;
impl ArmxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Armx {
        match self.bits {
            false => Armx::Disabled,
            true => Armx::Enabled,
        }
    }
    #[doc = "Input capture operation is disabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Armx::Disabled
    }
    #[doc = "Input capture operation as specified by CAPTCTRLX\\[EDGXx\\] is enabled."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Armx::Enabled
    }
}
#[doc = "Field `ARMX` writer - Arm X"]
pub type ArmxW<'a, REG> = crate::BitWriter<'a, REG, Armx>;
impl<'a, REG> ArmxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Input capture operation is disabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Armx::Disabled)
    }
    #[doc = "Input capture operation as specified by CAPTCTRLX\\[EDGXx\\] is enabled."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Armx::Enabled)
    }
}
#[doc = "One Shot Mode Aux\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Oneshotx {
    #[doc = "0: Free Running"]
    FreeRunning = 0,
    #[doc = "1: One Shot"]
    OneShot = 1,
}
impl From<Oneshotx> for bool {
    #[inline(always)]
    fn from(variant: Oneshotx) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ONESHOTX` reader - One Shot Mode Aux"]
pub type OneshotxR = crate::BitReader<Oneshotx>;
impl OneshotxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Oneshotx {
        match self.bits {
            false => Oneshotx::FreeRunning,
            true => Oneshotx::OneShot,
        }
    }
    #[doc = "Free Running"]
    #[inline(always)]
    pub fn is_free_running(&self) -> bool {
        *self == Oneshotx::FreeRunning
    }
    #[doc = "One Shot"]
    #[inline(always)]
    pub fn is_one_shot(&self) -> bool {
        *self == Oneshotx::OneShot
    }
}
#[doc = "Field `ONESHOTX` writer - One Shot Mode Aux"]
pub type OneshotxW<'a, REG> = crate::BitWriter<'a, REG, Oneshotx>;
impl<'a, REG> OneshotxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Free Running"]
    #[inline(always)]
    pub fn free_running(self) -> &'a mut crate::W<REG> {
        self.variant(Oneshotx::FreeRunning)
    }
    #[doc = "One Shot"]
    #[inline(always)]
    pub fn one_shot(self) -> &'a mut crate::W<REG> {
        self.variant(Oneshotx::OneShot)
    }
}
#[doc = "Edge X 0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Edgx0 {
    #[doc = "0: Disabled"]
    Disabled = 0,
    #[doc = "1: Capture falling edges"]
    FallingEdge = 1,
    #[doc = "2: Capture rising edges"]
    RisingEdge = 2,
    #[doc = "3: Capture any edge"]
    AnyEdge = 3,
}
impl From<Edgx0> for u8 {
    #[inline(always)]
    fn from(variant: Edgx0) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Edgx0 {
    type Ux = u8;
}
impl crate::IsEnum for Edgx0 {}
#[doc = "Field `EDGX0` reader - Edge X 0"]
pub type Edgx0R = crate::FieldReader<Edgx0>;
impl Edgx0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Edgx0 {
        match self.bits {
            0 => Edgx0::Disabled,
            1 => Edgx0::FallingEdge,
            2 => Edgx0::RisingEdge,
            3 => Edgx0::AnyEdge,
            _ => unreachable!(),
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Edgx0::Disabled
    }
    #[doc = "Capture falling edges"]
    #[inline(always)]
    pub fn is_falling_edge(&self) -> bool {
        *self == Edgx0::FallingEdge
    }
    #[doc = "Capture rising edges"]
    #[inline(always)]
    pub fn is_rising_edge(&self) -> bool {
        *self == Edgx0::RisingEdge
    }
    #[doc = "Capture any edge"]
    #[inline(always)]
    pub fn is_any_edge(&self) -> bool {
        *self == Edgx0::AnyEdge
    }
}
#[doc = "Field `EDGX0` writer - Edge X 0"]
pub type Edgx0W<'a, REG> = crate::FieldWriter<'a, REG, 2, Edgx0, crate::Safe>;
impl<'a, REG> Edgx0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Edgx0::Disabled)
    }
    #[doc = "Capture falling edges"]
    #[inline(always)]
    pub fn falling_edge(self) -> &'a mut crate::W<REG> {
        self.variant(Edgx0::FallingEdge)
    }
    #[doc = "Capture rising edges"]
    #[inline(always)]
    pub fn rising_edge(self) -> &'a mut crate::W<REG> {
        self.variant(Edgx0::RisingEdge)
    }
    #[doc = "Capture any edge"]
    #[inline(always)]
    pub fn any_edge(self) -> &'a mut crate::W<REG> {
        self.variant(Edgx0::AnyEdge)
    }
}
#[doc = "Edge X 1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Edgx1 {
    #[doc = "0: Disabled"]
    Disabled = 0,
    #[doc = "1: Capture falling edges"]
    FallingEdge = 1,
    #[doc = "2: Capture rising edges"]
    RisingEdge = 2,
    #[doc = "3: Capture any edge"]
    AnyEdge = 3,
}
impl From<Edgx1> for u8 {
    #[inline(always)]
    fn from(variant: Edgx1) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Edgx1 {
    type Ux = u8;
}
impl crate::IsEnum for Edgx1 {}
#[doc = "Field `EDGX1` reader - Edge X 1"]
pub type Edgx1R = crate::FieldReader<Edgx1>;
impl Edgx1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Edgx1 {
        match self.bits {
            0 => Edgx1::Disabled,
            1 => Edgx1::FallingEdge,
            2 => Edgx1::RisingEdge,
            3 => Edgx1::AnyEdge,
            _ => unreachable!(),
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Edgx1::Disabled
    }
    #[doc = "Capture falling edges"]
    #[inline(always)]
    pub fn is_falling_edge(&self) -> bool {
        *self == Edgx1::FallingEdge
    }
    #[doc = "Capture rising edges"]
    #[inline(always)]
    pub fn is_rising_edge(&self) -> bool {
        *self == Edgx1::RisingEdge
    }
    #[doc = "Capture any edge"]
    #[inline(always)]
    pub fn is_any_edge(&self) -> bool {
        *self == Edgx1::AnyEdge
    }
}
#[doc = "Field `EDGX1` writer - Edge X 1"]
pub type Edgx1W<'a, REG> = crate::FieldWriter<'a, REG, 2, Edgx1, crate::Safe>;
impl<'a, REG> Edgx1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Edgx1::Disabled)
    }
    #[doc = "Capture falling edges"]
    #[inline(always)]
    pub fn falling_edge(self) -> &'a mut crate::W<REG> {
        self.variant(Edgx1::FallingEdge)
    }
    #[doc = "Capture rising edges"]
    #[inline(always)]
    pub fn rising_edge(self) -> &'a mut crate::W<REG> {
        self.variant(Edgx1::RisingEdge)
    }
    #[doc = "Capture any edge"]
    #[inline(always)]
    pub fn any_edge(self) -> &'a mut crate::W<REG> {
        self.variant(Edgx1::AnyEdge)
    }
}
#[doc = "Input Select X\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InpSelx {
    #[doc = "0: Raw PWM_X input signal selected as source."]
    PwmX = 0,
    #[doc = "1: Edge Counter"]
    EdgeCounter = 1,
}
impl From<InpSelx> for bool {
    #[inline(always)]
    fn from(variant: InpSelx) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INP_SELX` reader - Input Select X"]
pub type InpSelxR = crate::BitReader<InpSelx>;
impl InpSelxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> InpSelx {
        match self.bits {
            false => InpSelx::PwmX,
            true => InpSelx::EdgeCounter,
        }
    }
    #[doc = "Raw PWM_X input signal selected as source."]
    #[inline(always)]
    pub fn is_pwm_x(&self) -> bool {
        *self == InpSelx::PwmX
    }
    #[doc = "Edge Counter"]
    #[inline(always)]
    pub fn is_edge_counter(&self) -> bool {
        *self == InpSelx::EdgeCounter
    }
}
#[doc = "Field `INP_SELX` writer - Input Select X"]
pub type InpSelxW<'a, REG> = crate::BitWriter<'a, REG, InpSelx>;
impl<'a, REG> InpSelxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Raw PWM_X input signal selected as source."]
    #[inline(always)]
    pub fn pwm_x(self) -> &'a mut crate::W<REG> {
        self.variant(InpSelx::PwmX)
    }
    #[doc = "Edge Counter"]
    #[inline(always)]
    pub fn edge_counter(self) -> &'a mut crate::W<REG> {
        self.variant(InpSelx::EdgeCounter)
    }
}
#[doc = "Edge Counter X Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgcntxEn {
    #[doc = "0: Edge counter disabled and held in reset"]
    Disabled = 0,
    #[doc = "1: Edge counter enabled"]
    Enabled = 1,
}
impl From<EdgcntxEn> for bool {
    #[inline(always)]
    fn from(variant: EdgcntxEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EDGCNTX_EN` reader - Edge Counter X Enable"]
pub type EdgcntxEnR = crate::BitReader<EdgcntxEn>;
impl EdgcntxEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> EdgcntxEn {
        match self.bits {
            false => EdgcntxEn::Disabled,
            true => EdgcntxEn::Enabled,
        }
    }
    #[doc = "Edge counter disabled and held in reset"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == EdgcntxEn::Disabled
    }
    #[doc = "Edge counter enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == EdgcntxEn::Enabled
    }
}
#[doc = "Field `EDGCNTX_EN` writer - Edge Counter X Enable"]
pub type EdgcntxEnW<'a, REG> = crate::BitWriter<'a, REG, EdgcntxEn>;
impl<'a, REG> EdgcntxEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Edge counter disabled and held in reset"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(EdgcntxEn::Disabled)
    }
    #[doc = "Edge counter enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(EdgcntxEn::Enabled)
    }
}
#[doc = "Field `CFXWM` reader - Capture X FIFOs Water Mark"]
pub type CfxwmR = crate::FieldReader;
#[doc = "Field `CFXWM` writer - Capture X FIFOs Water Mark"]
pub type CfxwmW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `CX0CNT` reader - Capture X0 FIFO Word Count"]
pub type Cx0cntR = crate::FieldReader;
#[doc = "Field `CX1CNT` reader - Capture X1 FIFO Word Count"]
pub type Cx1cntR = crate::FieldReader;
impl R {
    #[doc = "Bit 0 - Arm X"]
    #[inline(always)]
    pub fn armx(&self) -> ArmxR {
        ArmxR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - One Shot Mode Aux"]
    #[inline(always)]
    pub fn oneshotx(&self) -> OneshotxR {
        OneshotxR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bits 2:3 - Edge X 0"]
    #[inline(always)]
    pub fn edgx0(&self) -> Edgx0R {
        Edgx0R::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:5 - Edge X 1"]
    #[inline(always)]
    pub fn edgx1(&self) -> Edgx1R {
        Edgx1R::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bit 6 - Input Select X"]
    #[inline(always)]
    pub fn inp_selx(&self) -> InpSelxR {
        InpSelxR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Edge Counter X Enable"]
    #[inline(always)]
    pub fn edgcntx_en(&self) -> EdgcntxEnR {
        EdgcntxEnR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bits 8:9 - Capture X FIFOs Water Mark"]
    #[inline(always)]
    pub fn cfxwm(&self) -> CfxwmR {
        CfxwmR::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bits 10:12 - Capture X0 FIFO Word Count"]
    #[inline(always)]
    pub fn cx0cnt(&self) -> Cx0cntR {
        Cx0cntR::new(((self.bits >> 10) & 7) as u8)
    }
    #[doc = "Bits 13:15 - Capture X1 FIFO Word Count"]
    #[inline(always)]
    pub fn cx1cnt(&self) -> Cx1cntR {
        Cx1cntR::new(((self.bits >> 13) & 7) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Arm X"]
    #[inline(always)]
    pub fn armx(&mut self) -> ArmxW<Sm2captctrlxSpec> {
        ArmxW::new(self, 0)
    }
    #[doc = "Bit 1 - One Shot Mode Aux"]
    #[inline(always)]
    pub fn oneshotx(&mut self) -> OneshotxW<Sm2captctrlxSpec> {
        OneshotxW::new(self, 1)
    }
    #[doc = "Bits 2:3 - Edge X 0"]
    #[inline(always)]
    pub fn edgx0(&mut self) -> Edgx0W<Sm2captctrlxSpec> {
        Edgx0W::new(self, 2)
    }
    #[doc = "Bits 4:5 - Edge X 1"]
    #[inline(always)]
    pub fn edgx1(&mut self) -> Edgx1W<Sm2captctrlxSpec> {
        Edgx1W::new(self, 4)
    }
    #[doc = "Bit 6 - Input Select X"]
    #[inline(always)]
    pub fn inp_selx(&mut self) -> InpSelxW<Sm2captctrlxSpec> {
        InpSelxW::new(self, 6)
    }
    #[doc = "Bit 7 - Edge Counter X Enable"]
    #[inline(always)]
    pub fn edgcntx_en(&mut self) -> EdgcntxEnW<Sm2captctrlxSpec> {
        EdgcntxEnW::new(self, 7)
    }
    #[doc = "Bits 8:9 - Capture X FIFOs Water Mark"]
    #[inline(always)]
    pub fn cfxwm(&mut self) -> CfxwmW<Sm2captctrlxSpec> {
        CfxwmW::new(self, 8)
    }
}
#[doc = "Capture Control X Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2captctrlx::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2captctrlx::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm2captctrlxSpec;
impl crate::RegisterSpec for Sm2captctrlxSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm2captctrlx::R`](R) reader structure"]
impl crate::Readable for Sm2captctrlxSpec {}
#[doc = "`write(|w| ..)` method takes [`sm2captctrlx::W`](W) writer structure"]
impl crate::Writable for Sm2captctrlxSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM2CAPTCTRLX to value 0"]
impl crate::Resettable for Sm2captctrlxSpec {}
