#[doc = "Register `MP_CSR` reader"]
pub type R = crate::R<MpCsrSpec>;
#[doc = "Register `MP_CSR` writer"]
pub type W = crate::W<MpCsrSpec>;
#[doc = "Enable Debug\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edbg {
    #[doc = "0: Debug mode disabled"]
    Disable = 0,
    #[doc = "1: Debug mode is enabled."]
    Enable = 1,
}
impl From<Edbg> for bool {
    #[inline(always)]
    fn from(variant: Edbg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EDBG` reader - Enable Debug"]
pub type EdbgR = crate::BitReader<Edbg>;
impl EdbgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Edbg {
        match self.bits {
            false => Edbg::Disable,
            true => Edbg::Enable,
        }
    }
    #[doc = "Debug mode disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Edbg::Disable
    }
    #[doc = "Debug mode is enabled."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Edbg::Enable
    }
}
#[doc = "Field `EDBG` writer - Enable Debug"]
pub type EdbgW<'a, REG> = crate::BitWriter<'a, REG, Edbg>;
impl<'a, REG> EdbgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Debug mode disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Edbg::Disable)
    }
    #[doc = "Debug mode is enabled."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Edbg::Enable)
    }
}
#[doc = "Enable Round Robin Channel Arbitration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erca {
    #[doc = "0: Round-robin channel arbitration disabled"]
    Disable = 0,
    #[doc = "1: Round-robin channel arbitration enabled"]
    Enable = 1,
}
impl From<Erca> for bool {
    #[inline(always)]
    fn from(variant: Erca) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERCA` reader - Enable Round Robin Channel Arbitration"]
pub type ErcaR = crate::BitReader<Erca>;
impl ErcaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Erca {
        match self.bits {
            false => Erca::Disable,
            true => Erca::Enable,
        }
    }
    #[doc = "Round-robin channel arbitration disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Erca::Disable
    }
    #[doc = "Round-robin channel arbitration enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Erca::Enable
    }
}
#[doc = "Field `ERCA` writer - Enable Round Robin Channel Arbitration"]
pub type ErcaW<'a, REG> = crate::BitWriter<'a, REG, Erca>;
impl<'a, REG> ErcaW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Round-robin channel arbitration disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Erca::Disable)
    }
    #[doc = "Round-robin channel arbitration enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Erca::Enable)
    }
}
#[doc = "Halt After Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hae {
    #[doc = "0: Normal operation"]
    NormalOperation = 0,
    #[doc = "1: Any error causes the HALT field to be set to 1"]
    Halt = 1,
}
impl From<Hae> for bool {
    #[inline(always)]
    fn from(variant: Hae) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HAE` reader - Halt After Error"]
pub type HaeR = crate::BitReader<Hae>;
impl HaeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hae {
        match self.bits {
            false => Hae::NormalOperation,
            true => Hae::Halt,
        }
    }
    #[doc = "Normal operation"]
    #[inline(always)]
    pub fn is_normal_operation(&self) -> bool {
        *self == Hae::NormalOperation
    }
    #[doc = "Any error causes the HALT field to be set to 1"]
    #[inline(always)]
    pub fn is_halt(&self) -> bool {
        *self == Hae::Halt
    }
}
#[doc = "Field `HAE` writer - Halt After Error"]
pub type HaeW<'a, REG> = crate::BitWriter<'a, REG, Hae>;
impl<'a, REG> HaeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Normal operation"]
    #[inline(always)]
    pub fn normal_operation(self) -> &'a mut crate::W<REG> {
        self.variant(Hae::NormalOperation)
    }
    #[doc = "Any error causes the HALT field to be set to 1"]
    #[inline(always)]
    pub fn halt(self) -> &'a mut crate::W<REG> {
        self.variant(Hae::Halt)
    }
}
#[doc = "Halt DMA Operations\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Halt {
    #[doc = "0: Normal operation"]
    NormalOperation = 0,
    #[doc = "1: Stall the start of any new channels"]
    Stall = 1,
}
impl From<Halt> for bool {
    #[inline(always)]
    fn from(variant: Halt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HALT` reader - Halt DMA Operations"]
pub type HaltR = crate::BitReader<Halt>;
impl HaltR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Halt {
        match self.bits {
            false => Halt::NormalOperation,
            true => Halt::Stall,
        }
    }
    #[doc = "Normal operation"]
    #[inline(always)]
    pub fn is_normal_operation(&self) -> bool {
        *self == Halt::NormalOperation
    }
    #[doc = "Stall the start of any new channels"]
    #[inline(always)]
    pub fn is_stall(&self) -> bool {
        *self == Halt::Stall
    }
}
#[doc = "Field `HALT` writer - Halt DMA Operations"]
pub type HaltW<'a, REG> = crate::BitWriter<'a, REG, Halt>;
impl<'a, REG> HaltW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Normal operation"]
    #[inline(always)]
    pub fn normal_operation(self) -> &'a mut crate::W<REG> {
        self.variant(Halt::NormalOperation)
    }
    #[doc = "Stall the start of any new channels"]
    #[inline(always)]
    pub fn stall(self) -> &'a mut crate::W<REG> {
        self.variant(Halt::Stall)
    }
}
#[doc = "Global Channel Linking Control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gclc {
    #[doc = "0: Channel linking disabled for all channels"]
    Disable = 0,
    #[doc = "1: Channel linking available and controlled by each channel's link settings"]
    Available = 1,
}
impl From<Gclc> for bool {
    #[inline(always)]
    fn from(variant: Gclc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GCLC` reader - Global Channel Linking Control"]
pub type GclcR = crate::BitReader<Gclc>;
impl GclcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gclc {
        match self.bits {
            false => Gclc::Disable,
            true => Gclc::Available,
        }
    }
    #[doc = "Channel linking disabled for all channels"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Gclc::Disable
    }
    #[doc = "Channel linking available and controlled by each channel's link settings"]
    #[inline(always)]
    pub fn is_available(&self) -> bool {
        *self == Gclc::Available
    }
}
#[doc = "Field `GCLC` writer - Global Channel Linking Control"]
pub type GclcW<'a, REG> = crate::BitWriter<'a, REG, Gclc>;
impl<'a, REG> GclcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Channel linking disabled for all channels"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Gclc::Disable)
    }
    #[doc = "Channel linking available and controlled by each channel's link settings"]
    #[inline(always)]
    pub fn available(self) -> &'a mut crate::W<REG> {
        self.variant(Gclc::Available)
    }
}
#[doc = "Global Master ID Replication Control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gmrc {
    #[doc = "0: Master ID replication disabled for all channels"]
    Disable = 0,
    #[doc = "1: Master ID replication available and controlled by each channel's CHn_SBR\\[EMI\\] setting"]
    Available = 1,
}
impl From<Gmrc> for bool {
    #[inline(always)]
    fn from(variant: Gmrc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GMRC` reader - Global Master ID Replication Control"]
pub type GmrcR = crate::BitReader<Gmrc>;
impl GmrcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gmrc {
        match self.bits {
            false => Gmrc::Disable,
            true => Gmrc::Available,
        }
    }
    #[doc = "Master ID replication disabled for all channels"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Gmrc::Disable
    }
    #[doc = "Master ID replication available and controlled by each channel's CHn_SBR\\[EMI\\] setting"]
    #[inline(always)]
    pub fn is_available(&self) -> bool {
        *self == Gmrc::Available
    }
}
#[doc = "Field `GMRC` writer - Global Master ID Replication Control"]
pub type GmrcW<'a, REG> = crate::BitWriter<'a, REG, Gmrc>;
impl<'a, REG> GmrcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Master ID replication disabled for all channels"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Gmrc::Disable)
    }
    #[doc = "Master ID replication available and controlled by each channel's CHn_SBR\\[EMI\\] setting"]
    #[inline(always)]
    pub fn available(self) -> &'a mut crate::W<REG> {
        self.variant(Gmrc::Available)
    }
}
#[doc = "Cancel Transfer With Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ecx {
    #[doc = "0: Normal operation"]
    NormalOperation = 0,
    #[doc = "1: Cancel the remaining data transfer"]
    Cancel = 1,
}
impl From<Ecx> for bool {
    #[inline(always)]
    fn from(variant: Ecx) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ECX` reader - Cancel Transfer With Error"]
pub type EcxR = crate::BitReader<Ecx>;
impl EcxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ecx {
        match self.bits {
            false => Ecx::NormalOperation,
            true => Ecx::Cancel,
        }
    }
    #[doc = "Normal operation"]
    #[inline(always)]
    pub fn is_normal_operation(&self) -> bool {
        *self == Ecx::NormalOperation
    }
    #[doc = "Cancel the remaining data transfer"]
    #[inline(always)]
    pub fn is_cancel(&self) -> bool {
        *self == Ecx::Cancel
    }
}
#[doc = "Field `ECX` writer - Cancel Transfer With Error"]
pub type EcxW<'a, REG> = crate::BitWriter<'a, REG, Ecx>;
impl<'a, REG> EcxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Normal operation"]
    #[inline(always)]
    pub fn normal_operation(self) -> &'a mut crate::W<REG> {
        self.variant(Ecx::NormalOperation)
    }
    #[doc = "Cancel the remaining data transfer"]
    #[inline(always)]
    pub fn cancel(self) -> &'a mut crate::W<REG> {
        self.variant(Ecx::Cancel)
    }
}
#[doc = "Cancel Transfer\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cx {
    #[doc = "0: Normal operation"]
    NormalOperation = 0,
    #[doc = "1: Cancel the remaining data transfer"]
    DataTransferCancel = 1,
}
impl From<Cx> for bool {
    #[inline(always)]
    fn from(variant: Cx) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CX` reader - Cancel Transfer"]
pub type CxR = crate::BitReader<Cx>;
impl CxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cx {
        match self.bits {
            false => Cx::NormalOperation,
            true => Cx::DataTransferCancel,
        }
    }
    #[doc = "Normal operation"]
    #[inline(always)]
    pub fn is_normal_operation(&self) -> bool {
        *self == Cx::NormalOperation
    }
    #[doc = "Cancel the remaining data transfer"]
    #[inline(always)]
    pub fn is_data_transfer_cancel(&self) -> bool {
        *self == Cx::DataTransferCancel
    }
}
#[doc = "Field `CX` writer - Cancel Transfer"]
pub type CxW<'a, REG> = crate::BitWriter<'a, REG, Cx>;
impl<'a, REG> CxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Normal operation"]
    #[inline(always)]
    pub fn normal_operation(self) -> &'a mut crate::W<REG> {
        self.variant(Cx::NormalOperation)
    }
    #[doc = "Cancel the remaining data transfer"]
    #[inline(always)]
    pub fn data_transfer_cancel(self) -> &'a mut crate::W<REG> {
        self.variant(Cx::DataTransferCancel)
    }
}
#[doc = "Field `ACTIVE_ID` reader - Active Channel ID"]
pub type ActiveIdR = crate::FieldReader;
#[doc = "DMA Active Status\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Active {
    #[doc = "0: eDMA is idle"]
    Idle = 0,
    #[doc = "1: eDMA is executing a channel"]
    Execution = 1,
}
impl From<Active> for bool {
    #[inline(always)]
    fn from(variant: Active) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ACTIVE` reader - DMA Active Status"]
pub type ActiveR = crate::BitReader<Active>;
impl ActiveR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Active {
        match self.bits {
            false => Active::Idle,
            true => Active::Execution,
        }
    }
    #[doc = "eDMA is idle"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == Active::Idle
    }
    #[doc = "eDMA is executing a channel"]
    #[inline(always)]
    pub fn is_execution(&self) -> bool {
        *self == Active::Execution
    }
}
impl R {
    #[doc = "Bit 1 - Enable Debug"]
    #[inline(always)]
    pub fn edbg(&self) -> EdbgR {
        EdbgR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Enable Round Robin Channel Arbitration"]
    #[inline(always)]
    pub fn erca(&self) -> ErcaR {
        ErcaR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 4 - Halt After Error"]
    #[inline(always)]
    pub fn hae(&self) -> HaeR {
        HaeR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Halt DMA Operations"]
    #[inline(always)]
    pub fn halt(&self) -> HaltR {
        HaltR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Global Channel Linking Control"]
    #[inline(always)]
    pub fn gclc(&self) -> GclcR {
        GclcR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Global Master ID Replication Control"]
    #[inline(always)]
    pub fn gmrc(&self) -> GmrcR {
        GmrcR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Cancel Transfer With Error"]
    #[inline(always)]
    pub fn ecx(&self) -> EcxR {
        EcxR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Cancel Transfer"]
    #[inline(always)]
    pub fn cx(&self) -> CxR {
        CxR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bits 24:26 - Active Channel ID"]
    #[inline(always)]
    pub fn active_id(&self) -> ActiveIdR {
        ActiveIdR::new(((self.bits >> 24) & 7) as u8)
    }
    #[doc = "Bit 31 - DMA Active Status"]
    #[inline(always)]
    pub fn active(&self) -> ActiveR {
        ActiveR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 1 - Enable Debug"]
    #[inline(always)]
    pub fn edbg(&mut self) -> EdbgW<MpCsrSpec> {
        EdbgW::new(self, 1)
    }
    #[doc = "Bit 2 - Enable Round Robin Channel Arbitration"]
    #[inline(always)]
    pub fn erca(&mut self) -> ErcaW<MpCsrSpec> {
        ErcaW::new(self, 2)
    }
    #[doc = "Bit 4 - Halt After Error"]
    #[inline(always)]
    pub fn hae(&mut self) -> HaeW<MpCsrSpec> {
        HaeW::new(self, 4)
    }
    #[doc = "Bit 5 - Halt DMA Operations"]
    #[inline(always)]
    pub fn halt(&mut self) -> HaltW<MpCsrSpec> {
        HaltW::new(self, 5)
    }
    #[doc = "Bit 6 - Global Channel Linking Control"]
    #[inline(always)]
    pub fn gclc(&mut self) -> GclcW<MpCsrSpec> {
        GclcW::new(self, 6)
    }
    #[doc = "Bit 7 - Global Master ID Replication Control"]
    #[inline(always)]
    pub fn gmrc(&mut self) -> GmrcW<MpCsrSpec> {
        GmrcW::new(self, 7)
    }
    #[doc = "Bit 8 - Cancel Transfer With Error"]
    #[inline(always)]
    pub fn ecx(&mut self) -> EcxW<MpCsrSpec> {
        EcxW::new(self, 8)
    }
    #[doc = "Bit 9 - Cancel Transfer"]
    #[inline(always)]
    pub fn cx(&mut self) -> CxW<MpCsrSpec> {
        CxW::new(self, 9)
    }
}
#[doc = "Management Page Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mp_csr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mp_csr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MpCsrSpec;
impl crate::RegisterSpec for MpCsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mp_csr::R`](R) reader structure"]
impl crate::Readable for MpCsrSpec {}
#[doc = "`write(|w| ..)` method takes [`mp_csr::W`](W) writer structure"]
impl crate::Writable for MpCsrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MP_CSR to value 0x0031_0000"]
impl crate::Resettable for MpCsrSpec {
    const RESET_VALUE: u32 = 0x0031_0000;
}
