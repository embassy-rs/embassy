#[doc = "Register `GCR` reader"]
pub type R = crate::R<GcrSpec>;
#[doc = "Register `GCR` writer"]
pub type W = crate::W<GcrSpec>;
#[doc = "DAC Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dacen {
    #[doc = "0: Disables"]
    Disabled = 0,
    #[doc = "1: Enables"]
    Enabled = 1,
}
impl From<Dacen> for bool {
    #[inline(always)]
    fn from(variant: Dacen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DACEN` reader - DAC Enable"]
pub type DacenR = crate::BitReader<Dacen>;
impl DacenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dacen {
        match self.bits {
            false => Dacen::Disabled,
            true => Dacen::Enabled,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Dacen::Disabled
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Dacen::Enabled
    }
}
#[doc = "Field `DACEN` writer - DAC Enable"]
pub type DacenW<'a, REG> = crate::BitWriter<'a, REG, Dacen>;
impl<'a, REG> DacenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dacen::Disabled)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dacen::Enabled)
    }
}
#[doc = "DAC Reference Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Dacrfs {
    #[doc = "0: Selects VREFH0 as the reference voltage."]
    Vrefh0 = 0,
    #[doc = "1: Selects VREFH1 as the reference voltage."]
    Vrefh1 = 1,
    #[doc = "2: Selects VREFH2 as the reference voltage."]
    Vrefh2 = 2,
}
impl From<Dacrfs> for u8 {
    #[inline(always)]
    fn from(variant: Dacrfs) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Dacrfs {
    type Ux = u8;
}
impl crate::IsEnum for Dacrfs {}
#[doc = "Field `DACRFS` reader - DAC Reference Select"]
pub type DacrfsR = crate::FieldReader<Dacrfs>;
impl DacrfsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Dacrfs> {
        match self.bits {
            0 => Some(Dacrfs::Vrefh0),
            1 => Some(Dacrfs::Vrefh1),
            2 => Some(Dacrfs::Vrefh2),
            _ => None,
        }
    }
    #[doc = "Selects VREFH0 as the reference voltage."]
    #[inline(always)]
    pub fn is_vrefh0(&self) -> bool {
        *self == Dacrfs::Vrefh0
    }
    #[doc = "Selects VREFH1 as the reference voltage."]
    #[inline(always)]
    pub fn is_vrefh1(&self) -> bool {
        *self == Dacrfs::Vrefh1
    }
    #[doc = "Selects VREFH2 as the reference voltage."]
    #[inline(always)]
    pub fn is_vrefh2(&self) -> bool {
        *self == Dacrfs::Vrefh2
    }
}
#[doc = "Field `DACRFS` writer - DAC Reference Select"]
pub type DacrfsW<'a, REG> = crate::FieldWriter<'a, REG, 2, Dacrfs>;
impl<'a, REG> DacrfsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Selects VREFH0 as the reference voltage."]
    #[inline(always)]
    pub fn vrefh0(self) -> &'a mut crate::W<REG> {
        self.variant(Dacrfs::Vrefh0)
    }
    #[doc = "Selects VREFH1 as the reference voltage."]
    #[inline(always)]
    pub fn vrefh1(self) -> &'a mut crate::W<REG> {
        self.variant(Dacrfs::Vrefh1)
    }
    #[doc = "Selects VREFH2 as the reference voltage."]
    #[inline(always)]
    pub fn vrefh2(self) -> &'a mut crate::W<REG> {
        self.variant(Dacrfs::Vrefh2)
    }
}
#[doc = "FIFO Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fifoen {
    #[doc = "0: Disables FIFO mode and enables Buffer mode. Any data written to DATA\\[DATA\\] goes to buffer then goes to conversion."]
    BufferMode = 0,
    #[doc = "1: Enables FIFO mode. Data will be first read from FIFO to buffer and then goes to conversion."]
    FifoMode = 1,
}
impl From<Fifoen> for bool {
    #[inline(always)]
    fn from(variant: Fifoen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FIFOEN` reader - FIFO Enable"]
pub type FifoenR = crate::BitReader<Fifoen>;
impl FifoenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fifoen {
        match self.bits {
            false => Fifoen::BufferMode,
            true => Fifoen::FifoMode,
        }
    }
    #[doc = "Disables FIFO mode and enables Buffer mode. Any data written to DATA\\[DATA\\] goes to buffer then goes to conversion."]
    #[inline(always)]
    pub fn is_buffer_mode(&self) -> bool {
        *self == Fifoen::BufferMode
    }
    #[doc = "Enables FIFO mode. Data will be first read from FIFO to buffer and then goes to conversion."]
    #[inline(always)]
    pub fn is_fifo_mode(&self) -> bool {
        *self == Fifoen::FifoMode
    }
}
#[doc = "Field `FIFOEN` writer - FIFO Enable"]
pub type FifoenW<'a, REG> = crate::BitWriter<'a, REG, Fifoen>;
impl<'a, REG> FifoenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables FIFO mode and enables Buffer mode. Any data written to DATA\\[DATA\\] goes to buffer then goes to conversion."]
    #[inline(always)]
    pub fn buffer_mode(self) -> &'a mut crate::W<REG> {
        self.variant(Fifoen::BufferMode)
    }
    #[doc = "Enables FIFO mode. Data will be first read from FIFO to buffer and then goes to conversion."]
    #[inline(always)]
    pub fn fifo_mode(self) -> &'a mut crate::W<REG> {
        self.variant(Fifoen::FifoMode)
    }
}
#[doc = "Swing Back Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swmd {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Swmd> for bool {
    #[inline(always)]
    fn from(variant: Swmd) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SWMD` reader - Swing Back Mode"]
pub type SwmdR = crate::BitReader<Swmd>;
impl SwmdR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Swmd {
        match self.bits {
            false => Swmd::Disable,
            true => Swmd::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Swmd::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Swmd::Enable
    }
}
#[doc = "Field `SWMD` writer - Swing Back Mode"]
pub type SwmdW<'a, REG> = crate::BitWriter<'a, REG, Swmd>;
impl<'a, REG> SwmdW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Swmd::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Swmd::Enable)
    }
}
#[doc = "DAC Trigger Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trgsel {
    #[doc = "0: Hardware trigger"]
    Hardware = 0,
    #[doc = "1: Software trigger"]
    Software = 1,
}
impl From<Trgsel> for bool {
    #[inline(always)]
    fn from(variant: Trgsel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRGSEL` reader - DAC Trigger Select"]
pub type TrgselR = crate::BitReader<Trgsel>;
impl TrgselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Trgsel {
        match self.bits {
            false => Trgsel::Hardware,
            true => Trgsel::Software,
        }
    }
    #[doc = "Hardware trigger"]
    #[inline(always)]
    pub fn is_hardware(&self) -> bool {
        *self == Trgsel::Hardware
    }
    #[doc = "Software trigger"]
    #[inline(always)]
    pub fn is_software(&self) -> bool {
        *self == Trgsel::Software
    }
}
#[doc = "Field `TRGSEL` writer - DAC Trigger Select"]
pub type TrgselW<'a, REG> = crate::BitWriter<'a, REG, Trgsel>;
impl<'a, REG> TrgselW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Hardware trigger"]
    #[inline(always)]
    pub fn hardware(self) -> &'a mut crate::W<REG> {
        self.variant(Trgsel::Hardware)
    }
    #[doc = "Software trigger"]
    #[inline(always)]
    pub fn software(self) -> &'a mut crate::W<REG> {
        self.variant(Trgsel::Software)
    }
}
#[doc = "DAC Periodic Trigger Mode Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ptgen {
    #[doc = "0: Disables"]
    Disabled = 0,
    #[doc = "1: Enables"]
    Enabled = 1,
}
impl From<Ptgen> for bool {
    #[inline(always)]
    fn from(variant: Ptgen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PTGEN` reader - DAC Periodic Trigger Mode Enable"]
pub type PtgenR = crate::BitReader<Ptgen>;
impl PtgenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ptgen {
        match self.bits {
            false => Ptgen::Disabled,
            true => Ptgen::Enabled,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ptgen::Disabled
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ptgen::Enabled
    }
}
#[doc = "Field `PTGEN` writer - DAC Periodic Trigger Mode Enable"]
pub type PtgenW<'a, REG> = crate::BitWriter<'a, REG, Ptgen>;
impl<'a, REG> PtgenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ptgen::Disabled)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ptgen::Enabled)
    }
}
#[doc = "Field `LATCH_CYC` reader - RCLK Cycles Before Data Latch"]
pub type LatchCycR = crate::FieldReader;
#[doc = "Field `LATCH_CYC` writer - RCLK Cycles Before Data Latch"]
pub type LatchCycW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Buffer Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufEn {
    #[doc = "0: Not used"]
    UseBuf = 0,
    #[doc = "1: Used"]
    NoUseBuf = 1,
}
impl From<BufEn> for bool {
    #[inline(always)]
    fn from(variant: BufEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BUF_EN` reader - Buffer Enable"]
pub type BufEnR = crate::BitReader<BufEn>;
impl BufEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> BufEn {
        match self.bits {
            false => BufEn::UseBuf,
            true => BufEn::NoUseBuf,
        }
    }
    #[doc = "Not used"]
    #[inline(always)]
    pub fn is_use_buf(&self) -> bool {
        *self == BufEn::UseBuf
    }
    #[doc = "Used"]
    #[inline(always)]
    pub fn is_no_use_buf(&self) -> bool {
        *self == BufEn::NoUseBuf
    }
}
#[doc = "Field `BUF_EN` writer - Buffer Enable"]
pub type BufEnW<'a, REG> = crate::BitWriter<'a, REG, BufEn>;
impl<'a, REG> BufEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not used"]
    #[inline(always)]
    pub fn use_buf(self) -> &'a mut crate::W<REG> {
        self.variant(BufEn::UseBuf)
    }
    #[doc = "Used"]
    #[inline(always)]
    pub fn no_use_buf(self) -> &'a mut crate::W<REG> {
        self.variant(BufEn::NoUseBuf)
    }
}
#[doc = "External On-Chip PTAT Current Reference Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IrefPtatExtSel {
    #[doc = "0: Not selected"]
    NotSelected = 0,
    #[doc = "1: Selected"]
    Selected = 1,
}
impl From<IrefPtatExtSel> for bool {
    #[inline(always)]
    fn from(variant: IrefPtatExtSel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IREF_PTAT_EXT_SEL` reader - External On-Chip PTAT Current Reference Select"]
pub type IrefPtatExtSelR = crate::BitReader<IrefPtatExtSel>;
impl IrefPtatExtSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> IrefPtatExtSel {
        match self.bits {
            false => IrefPtatExtSel::NotSelected,
            true => IrefPtatExtSel::Selected,
        }
    }
    #[doc = "Not selected"]
    #[inline(always)]
    pub fn is_not_selected(&self) -> bool {
        *self == IrefPtatExtSel::NotSelected
    }
    #[doc = "Selected"]
    #[inline(always)]
    pub fn is_selected(&self) -> bool {
        *self == IrefPtatExtSel::Selected
    }
}
#[doc = "Field `IREF_PTAT_EXT_SEL` writer - External On-Chip PTAT Current Reference Select"]
pub type IrefPtatExtSelW<'a, REG> = crate::BitWriter<'a, REG, IrefPtatExtSel>;
impl<'a, REG> IrefPtatExtSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not selected"]
    #[inline(always)]
    pub fn not_selected(self) -> &'a mut crate::W<REG> {
        self.variant(IrefPtatExtSel::NotSelected)
    }
    #[doc = "Selected"]
    #[inline(always)]
    pub fn selected(self) -> &'a mut crate::W<REG> {
        self.variant(IrefPtatExtSel::Selected)
    }
}
#[doc = "External On-Chip ZTC Current Reference Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IrefZtcExtSel {
    #[doc = "0: Not selected"]
    NotSelected = 0,
    #[doc = "1: Selected"]
    Selected = 1,
}
impl From<IrefZtcExtSel> for bool {
    #[inline(always)]
    fn from(variant: IrefZtcExtSel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IREF_ZTC_EXT_SEL` reader - External On-Chip ZTC Current Reference Select"]
pub type IrefZtcExtSelR = crate::BitReader<IrefZtcExtSel>;
impl IrefZtcExtSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> IrefZtcExtSel {
        match self.bits {
            false => IrefZtcExtSel::NotSelected,
            true => IrefZtcExtSel::Selected,
        }
    }
    #[doc = "Not selected"]
    #[inline(always)]
    pub fn is_not_selected(&self) -> bool {
        *self == IrefZtcExtSel::NotSelected
    }
    #[doc = "Selected"]
    #[inline(always)]
    pub fn is_selected(&self) -> bool {
        *self == IrefZtcExtSel::Selected
    }
}
#[doc = "Field `IREF_ZTC_EXT_SEL` writer - External On-Chip ZTC Current Reference Select"]
pub type IrefZtcExtSelW<'a, REG> = crate::BitWriter<'a, REG, IrefZtcExtSel>;
impl<'a, REG> IrefZtcExtSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not selected"]
    #[inline(always)]
    pub fn not_selected(self) -> &'a mut crate::W<REG> {
        self.variant(IrefZtcExtSel::NotSelected)
    }
    #[doc = "Selected"]
    #[inline(always)]
    pub fn selected(self) -> &'a mut crate::W<REG> {
        self.variant(IrefZtcExtSel::Selected)
    }
}
#[doc = "OPAMP as Buffer, Speed Control Signal\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufSpdCtrl {
    #[doc = "0: Lower Low-Power mode"]
    LlpMode = 0,
    #[doc = "1: Low-Power mode"]
    LpMode = 1,
}
impl From<BufSpdCtrl> for bool {
    #[inline(always)]
    fn from(variant: BufSpdCtrl) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BUF_SPD_CTRL` reader - OPAMP as Buffer, Speed Control Signal"]
pub type BufSpdCtrlR = crate::BitReader<BufSpdCtrl>;
impl BufSpdCtrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> BufSpdCtrl {
        match self.bits {
            false => BufSpdCtrl::LlpMode,
            true => BufSpdCtrl::LpMode,
        }
    }
    #[doc = "Lower Low-Power mode"]
    #[inline(always)]
    pub fn is_llp_mode(&self) -> bool {
        *self == BufSpdCtrl::LlpMode
    }
    #[doc = "Low-Power mode"]
    #[inline(always)]
    pub fn is_lp_mode(&self) -> bool {
        *self == BufSpdCtrl::LpMode
    }
}
#[doc = "Field `BUF_SPD_CTRL` writer - OPAMP as Buffer, Speed Control Signal"]
pub type BufSpdCtrlW<'a, REG> = crate::BitWriter<'a, REG, BufSpdCtrl>;
impl<'a, REG> BufSpdCtrlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Lower Low-Power mode"]
    #[inline(always)]
    pub fn llp_mode(self) -> &'a mut crate::W<REG> {
        self.variant(BufSpdCtrl::LlpMode)
    }
    #[doc = "Low-Power mode"]
    #[inline(always)]
    pub fn lp_mode(self) -> &'a mut crate::W<REG> {
        self.variant(BufSpdCtrl::LpMode)
    }
}
impl R {
    #[doc = "Bit 0 - DAC Enable"]
    #[inline(always)]
    pub fn dacen(&self) -> DacenR {
        DacenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bits 1:2 - DAC Reference Select"]
    #[inline(always)]
    pub fn dacrfs(&self) -> DacrfsR {
        DacrfsR::new(((self.bits >> 1) & 3) as u8)
    }
    #[doc = "Bit 3 - FIFO Enable"]
    #[inline(always)]
    pub fn fifoen(&self) -> FifoenR {
        FifoenR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Swing Back Mode"]
    #[inline(always)]
    pub fn swmd(&self) -> SwmdR {
        SwmdR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - DAC Trigger Select"]
    #[inline(always)]
    pub fn trgsel(&self) -> TrgselR {
        TrgselR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - DAC Periodic Trigger Mode Enable"]
    #[inline(always)]
    pub fn ptgen(&self) -> PtgenR {
        PtgenR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bits 8:11 - RCLK Cycles Before Data Latch"]
    #[inline(always)]
    pub fn latch_cyc(&self) -> LatchCycR {
        LatchCycR::new(((self.bits >> 8) & 0x0f) as u8)
    }
    #[doc = "Bit 17 - Buffer Enable"]
    #[inline(always)]
    pub fn buf_en(&self) -> BufEnR {
        BufEnR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 20 - External On-Chip PTAT Current Reference Select"]
    #[inline(always)]
    pub fn iref_ptat_ext_sel(&self) -> IrefPtatExtSelR {
        IrefPtatExtSelR::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - External On-Chip ZTC Current Reference Select"]
    #[inline(always)]
    pub fn iref_ztc_ext_sel(&self) -> IrefZtcExtSelR {
        IrefZtcExtSelR::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 23 - OPAMP as Buffer, Speed Control Signal"]
    #[inline(always)]
    pub fn buf_spd_ctrl(&self) -> BufSpdCtrlR {
        BufSpdCtrlR::new(((self.bits >> 23) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - DAC Enable"]
    #[inline(always)]
    pub fn dacen(&mut self) -> DacenW<GcrSpec> {
        DacenW::new(self, 0)
    }
    #[doc = "Bits 1:2 - DAC Reference Select"]
    #[inline(always)]
    pub fn dacrfs(&mut self) -> DacrfsW<GcrSpec> {
        DacrfsW::new(self, 1)
    }
    #[doc = "Bit 3 - FIFO Enable"]
    #[inline(always)]
    pub fn fifoen(&mut self) -> FifoenW<GcrSpec> {
        FifoenW::new(self, 3)
    }
    #[doc = "Bit 4 - Swing Back Mode"]
    #[inline(always)]
    pub fn swmd(&mut self) -> SwmdW<GcrSpec> {
        SwmdW::new(self, 4)
    }
    #[doc = "Bit 5 - DAC Trigger Select"]
    #[inline(always)]
    pub fn trgsel(&mut self) -> TrgselW<GcrSpec> {
        TrgselW::new(self, 5)
    }
    #[doc = "Bit 6 - DAC Periodic Trigger Mode Enable"]
    #[inline(always)]
    pub fn ptgen(&mut self) -> PtgenW<GcrSpec> {
        PtgenW::new(self, 6)
    }
    #[doc = "Bits 8:11 - RCLK Cycles Before Data Latch"]
    #[inline(always)]
    pub fn latch_cyc(&mut self) -> LatchCycW<GcrSpec> {
        LatchCycW::new(self, 8)
    }
    #[doc = "Bit 17 - Buffer Enable"]
    #[inline(always)]
    pub fn buf_en(&mut self) -> BufEnW<GcrSpec> {
        BufEnW::new(self, 17)
    }
    #[doc = "Bit 20 - External On-Chip PTAT Current Reference Select"]
    #[inline(always)]
    pub fn iref_ptat_ext_sel(&mut self) -> IrefPtatExtSelW<GcrSpec> {
        IrefPtatExtSelW::new(self, 20)
    }
    #[doc = "Bit 21 - External On-Chip ZTC Current Reference Select"]
    #[inline(always)]
    pub fn iref_ztc_ext_sel(&mut self) -> IrefZtcExtSelW<GcrSpec> {
        IrefZtcExtSelW::new(self, 21)
    }
    #[doc = "Bit 23 - OPAMP as Buffer, Speed Control Signal"]
    #[inline(always)]
    pub fn buf_spd_ctrl(&mut self) -> BufSpdCtrlW<GcrSpec> {
        BufSpdCtrlW::new(self, 23)
    }
}
#[doc = "Global Control\n\nYou can [`read`](crate::Reg::read) this register and get [`gcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct GcrSpec;
impl crate::RegisterSpec for GcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`gcr::R`](R) reader structure"]
impl crate::Readable for GcrSpec {}
#[doc = "`write(|w| ..)` method takes [`gcr::W`](W) writer structure"]
impl crate::Writable for GcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets GCR to value 0x0100"]
impl crate::Resettable for GcrSpec {
    const RESET_VALUE: u32 = 0x0100;
}
