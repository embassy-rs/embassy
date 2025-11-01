#[doc = "Register `SCFGR1` reader"]
pub type R = crate::R<Scfgr1Spec>;
#[doc = "Register `SCFGR1` writer"]
pub type W = crate::W<Scfgr1Spec>;
#[doc = "Address SCL Stall\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Adrstall {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Adrstall> for bool {
    #[inline(always)]
    fn from(variant: Adrstall) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ADRSTALL` reader - Address SCL Stall"]
pub type AdrstallR = crate::BitReader<Adrstall>;
impl AdrstallR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Adrstall {
        match self.bits {
            false => Adrstall::Disabled,
            true => Adrstall::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Adrstall::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Adrstall::Enabled
    }
}
#[doc = "Field `ADRSTALL` writer - Address SCL Stall"]
pub type AdrstallW<'a, REG> = crate::BitWriter<'a, REG, Adrstall>;
impl<'a, REG> AdrstallW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Adrstall::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Adrstall::Enabled)
    }
}
#[doc = "RX SCL Stall\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxstall {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Rxstall> for bool {
    #[inline(always)]
    fn from(variant: Rxstall) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXSTALL` reader - RX SCL Stall"]
pub type RxstallR = crate::BitReader<Rxstall>;
impl RxstallR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxstall {
        match self.bits {
            false => Rxstall::Disabled,
            true => Rxstall::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rxstall::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rxstall::Enabled
    }
}
#[doc = "Field `RXSTALL` writer - RX SCL Stall"]
pub type RxstallW<'a, REG> = crate::BitWriter<'a, REG, Rxstall>;
impl<'a, REG> RxstallW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rxstall::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rxstall::Enabled)
    }
}
#[doc = "Transmit Data SCL Stall\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txdstall {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Txdstall> for bool {
    #[inline(always)]
    fn from(variant: Txdstall) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXDSTALL` reader - Transmit Data SCL Stall"]
pub type TxdstallR = crate::BitReader<Txdstall>;
impl TxdstallR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txdstall {
        match self.bits {
            false => Txdstall::Disabled,
            true => Txdstall::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Txdstall::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Txdstall::Enabled
    }
}
#[doc = "Field `TXDSTALL` writer - Transmit Data SCL Stall"]
pub type TxdstallW<'a, REG> = crate::BitWriter<'a, REG, Txdstall>;
impl<'a, REG> TxdstallW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Txdstall::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Txdstall::Enabled)
    }
}
#[doc = "ACK SCL Stall\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ackstall {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Ackstall> for bool {
    #[inline(always)]
    fn from(variant: Ackstall) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ACKSTALL` reader - ACK SCL Stall"]
pub type AckstallR = crate::BitReader<Ackstall>;
impl AckstallR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ackstall {
        match self.bits {
            false => Ackstall::Disabled,
            true => Ackstall::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ackstall::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ackstall::Enabled
    }
}
#[doc = "Field `ACKSTALL` writer - ACK SCL Stall"]
pub type AckstallW<'a, REG> = crate::BitWriter<'a, REG, Ackstall>;
impl<'a, REG> AckstallW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ackstall::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ackstall::Enabled)
    }
}
#[doc = "Receive NACK\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxnack {
    #[doc = "0: ACK or NACK always determined by STAR\\[TXNACK\\]"]
    SetByTxnack = 0,
    #[doc = "1: NACK always generated on address overrun or receive data overrun, otherwise ACK or NACK is determined by STAR\\[TXNACK\\]"]
    AlwaysGeneratedOnAddressOrReceiveDataOverrun = 1,
}
impl From<Rxnack> for bool {
    #[inline(always)]
    fn from(variant: Rxnack) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXNACK` reader - Receive NACK"]
pub type RxnackR = crate::BitReader<Rxnack>;
impl RxnackR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxnack {
        match self.bits {
            false => Rxnack::SetByTxnack,
            true => Rxnack::AlwaysGeneratedOnAddressOrReceiveDataOverrun,
        }
    }
    #[doc = "ACK or NACK always determined by STAR\\[TXNACK\\]"]
    #[inline(always)]
    pub fn is_set_by_txnack(&self) -> bool {
        *self == Rxnack::SetByTxnack
    }
    #[doc = "NACK always generated on address overrun or receive data overrun, otherwise ACK or NACK is determined by STAR\\[TXNACK\\]"]
    #[inline(always)]
    pub fn is_always_generated_on_address_or_receive_data_overrun(&self) -> bool {
        *self == Rxnack::AlwaysGeneratedOnAddressOrReceiveDataOverrun
    }
}
#[doc = "Field `RXNACK` writer - Receive NACK"]
pub type RxnackW<'a, REG> = crate::BitWriter<'a, REG, Rxnack>;
impl<'a, REG> RxnackW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "ACK or NACK always determined by STAR\\[TXNACK\\]"]
    #[inline(always)]
    pub fn set_by_txnack(self) -> &'a mut crate::W<REG> {
        self.variant(Rxnack::SetByTxnack)
    }
    #[doc = "NACK always generated on address overrun or receive data overrun, otherwise ACK or NACK is determined by STAR\\[TXNACK\\]"]
    #[inline(always)]
    pub fn always_generated_on_address_or_receive_data_overrun(self) -> &'a mut crate::W<REG> {
        self.variant(Rxnack::AlwaysGeneratedOnAddressOrReceiveDataOverrun)
    }
}
#[doc = "General Call Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gcen {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Gcen> for bool {
    #[inline(always)]
    fn from(variant: Gcen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GCEN` reader - General Call Enable"]
pub type GcenR = crate::BitReader<Gcen>;
impl GcenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gcen {
        match self.bits {
            false => Gcen::Disabled,
            true => Gcen::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Gcen::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Gcen::Enabled
    }
}
#[doc = "Field `GCEN` writer - General Call Enable"]
pub type GcenW<'a, REG> = crate::BitWriter<'a, REG, Gcen>;
impl<'a, REG> GcenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gcen::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gcen::Enabled)
    }
}
#[doc = "SMBus Alert Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Saen {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Saen> for bool {
    #[inline(always)]
    fn from(variant: Saen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SAEN` reader - SMBus Alert Enable"]
pub type SaenR = crate::BitReader<Saen>;
impl SaenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Saen {
        match self.bits {
            false => Saen::Disable,
            true => Saen::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Saen::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Saen::Enable
    }
}
#[doc = "Field `SAEN` writer - SMBus Alert Enable"]
pub type SaenW<'a, REG> = crate::BitWriter<'a, REG, Saen>;
impl<'a, REG> SaenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Saen::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Saen::Enable)
    }
}
#[doc = "Transmit Flag Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txcfg {
    #[doc = "0: MSR\\[TDF\\] is set only during a target-transmit transfer when STDR is empty"]
    AssertsDuringSlaveTransmitTransferWhenTxDataEmpty = 0,
    #[doc = "1: MSR\\[TDF\\] is set whenever STDR is empty"]
    AssertsWhenTxDataEmpty = 1,
}
impl From<Txcfg> for bool {
    #[inline(always)]
    fn from(variant: Txcfg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXCFG` reader - Transmit Flag Configuration"]
pub type TxcfgR = crate::BitReader<Txcfg>;
impl TxcfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txcfg {
        match self.bits {
            false => Txcfg::AssertsDuringSlaveTransmitTransferWhenTxDataEmpty,
            true => Txcfg::AssertsWhenTxDataEmpty,
        }
    }
    #[doc = "MSR\\[TDF\\] is set only during a target-transmit transfer when STDR is empty"]
    #[inline(always)]
    pub fn is_asserts_during_slave_transmit_transfer_when_tx_data_empty(&self) -> bool {
        *self == Txcfg::AssertsDuringSlaveTransmitTransferWhenTxDataEmpty
    }
    #[doc = "MSR\\[TDF\\] is set whenever STDR is empty"]
    #[inline(always)]
    pub fn is_asserts_when_tx_data_empty(&self) -> bool {
        *self == Txcfg::AssertsWhenTxDataEmpty
    }
}
#[doc = "Field `TXCFG` writer - Transmit Flag Configuration"]
pub type TxcfgW<'a, REG> = crate::BitWriter<'a, REG, Txcfg>;
impl<'a, REG> TxcfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "MSR\\[TDF\\] is set only during a target-transmit transfer when STDR is empty"]
    #[inline(always)]
    pub fn asserts_during_slave_transmit_transfer_when_tx_data_empty(
        self,
    ) -> &'a mut crate::W<REG> {
        self.variant(Txcfg::AssertsDuringSlaveTransmitTransferWhenTxDataEmpty)
    }
    #[doc = "MSR\\[TDF\\] is set whenever STDR is empty"]
    #[inline(always)]
    pub fn asserts_when_tx_data_empty(self) -> &'a mut crate::W<REG> {
        self.variant(Txcfg::AssertsWhenTxDataEmpty)
    }
}
#[doc = "Receive Data Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxcfg {
    #[doc = "0: Return received data, clear MSR\\[RDF\\]"]
    ReturnsReceivedDataAndClearsRxDataFlag = 0,
    #[doc = "1: Return SASR and clear SSR\\[AVF\\] when SSR\\[AVF\\] is set, return received data and clear MSR\\[RDF\\] when SSR\\[AFV\\] is not set"]
    WhenAddressValidFlagSetReturnsAddressStatusAndClearsAddressValidFlag = 1,
}
impl From<Rxcfg> for bool {
    #[inline(always)]
    fn from(variant: Rxcfg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXCFG` reader - Receive Data Configuration"]
pub type RxcfgR = crate::BitReader<Rxcfg>;
impl RxcfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxcfg {
        match self.bits {
            false => Rxcfg::ReturnsReceivedDataAndClearsRxDataFlag,
            true => Rxcfg::WhenAddressValidFlagSetReturnsAddressStatusAndClearsAddressValidFlag,
        }
    }
    #[doc = "Return received data, clear MSR\\[RDF\\]"]
    #[inline(always)]
    pub fn is_returns_received_data_and_clears_rx_data_flag(&self) -> bool {
        *self == Rxcfg::ReturnsReceivedDataAndClearsRxDataFlag
    }
    #[doc = "Return SASR and clear SSR\\[AVF\\] when SSR\\[AVF\\] is set, return received data and clear MSR\\[RDF\\] when SSR\\[AFV\\] is not set"]
    #[inline(always)]
    pub fn is_when_address_valid_flag_set_returns_address_status_and_clears_address_valid_flag(
        &self,
    ) -> bool {
        *self == Rxcfg::WhenAddressValidFlagSetReturnsAddressStatusAndClearsAddressValidFlag
    }
}
#[doc = "Field `RXCFG` writer - Receive Data Configuration"]
pub type RxcfgW<'a, REG> = crate::BitWriter<'a, REG, Rxcfg>;
impl<'a, REG> RxcfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Return received data, clear MSR\\[RDF\\]"]
    #[inline(always)]
    pub fn returns_received_data_and_clears_rx_data_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Rxcfg::ReturnsReceivedDataAndClearsRxDataFlag)
    }
    #[doc = "Return SASR and clear SSR\\[AVF\\] when SSR\\[AVF\\] is set, return received data and clear MSR\\[RDF\\] when SSR\\[AFV\\] is not set"]
    #[inline(always)]
    pub fn when_address_valid_flag_set_returns_address_status_and_clears_address_valid_flag(
        self,
    ) -> &'a mut crate::W<REG> {
        self.variant(Rxcfg::WhenAddressValidFlagSetReturnsAddressStatusAndClearsAddressValidFlag)
    }
}
#[doc = "Ignore NACK\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ignack {
    #[doc = "0: End transfer on NACK"]
    EndsTransferOnNack = 0,
    #[doc = "1: Do not end transfer on NACK"]
    DoesNotEndTransferOnNack = 1,
}
impl From<Ignack> for bool {
    #[inline(always)]
    fn from(variant: Ignack) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IGNACK` reader - Ignore NACK"]
pub type IgnackR = crate::BitReader<Ignack>;
impl IgnackR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ignack {
        match self.bits {
            false => Ignack::EndsTransferOnNack,
            true => Ignack::DoesNotEndTransferOnNack,
        }
    }
    #[doc = "End transfer on NACK"]
    #[inline(always)]
    pub fn is_ends_transfer_on_nack(&self) -> bool {
        *self == Ignack::EndsTransferOnNack
    }
    #[doc = "Do not end transfer on NACK"]
    #[inline(always)]
    pub fn is_does_not_end_transfer_on_nack(&self) -> bool {
        *self == Ignack::DoesNotEndTransferOnNack
    }
}
#[doc = "Field `IGNACK` writer - Ignore NACK"]
pub type IgnackW<'a, REG> = crate::BitWriter<'a, REG, Ignack>;
impl<'a, REG> IgnackW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "End transfer on NACK"]
    #[inline(always)]
    pub fn ends_transfer_on_nack(self) -> &'a mut crate::W<REG> {
        self.variant(Ignack::EndsTransferOnNack)
    }
    #[doc = "Do not end transfer on NACK"]
    #[inline(always)]
    pub fn does_not_end_transfer_on_nack(self) -> &'a mut crate::W<REG> {
        self.variant(Ignack::DoesNotEndTransferOnNack)
    }
}
#[doc = "HS Mode Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hsmen {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Hsmen> for bool {
    #[inline(always)]
    fn from(variant: Hsmen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HSMEN` reader - HS Mode Enable"]
pub type HsmenR = crate::BitReader<Hsmen>;
impl HsmenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hsmen {
        match self.bits {
            false => Hsmen::Disabled,
            true => Hsmen::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Hsmen::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Hsmen::Enabled
    }
}
#[doc = "Field `HSMEN` writer - HS Mode Enable"]
pub type HsmenW<'a, REG> = crate::BitWriter<'a, REG, Hsmen>;
impl<'a, REG> HsmenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Hsmen::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Hsmen::Enabled)
    }
}
#[doc = "Address Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Addrcfg {
    #[doc = "0: Address match 0 (7-bit)"]
    AddressMatch0_7Bit = 0,
    #[doc = "1: Address match 0 (10-bit)"]
    AddressMatch0_10Bit = 1,
    #[doc = "2: Address match 0 (7-bit) or address match 1 (7-bit)"]
    AddressMatch0_7BitOrAddressMatch1_7Bit = 2,
    #[doc = "3: Address match 0 (10-bit) or address match 1 (10-bit)"]
    AddressMatch0_10BitOrAddressMatch1_10Bit = 3,
    #[doc = "4: Address match 0 (7-bit) or address match 1 (10-bit)"]
    AddressMatch0_7BitOrAddressMatch1_10Bit = 4,
    #[doc = "5: Address match 0 (10-bit) or address match 1 (7-bit)"]
    AddressMatch0_10BitOrAddressMatch1_7Bit = 5,
    #[doc = "6: From address match 0 (7-bit) to address match 1 (7-bit)"]
    FromAddressMatch0_7BitToAddressMatch1_7Bit = 6,
    #[doc = "7: From address match 0 (10-bit) to address match 1 (10-bit)"]
    FromAddressMatch0_10BitToAddressMatch1_10Bit = 7,
}
impl From<Addrcfg> for u8 {
    #[inline(always)]
    fn from(variant: Addrcfg) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Addrcfg {
    type Ux = u8;
}
impl crate::IsEnum for Addrcfg {}
#[doc = "Field `ADDRCFG` reader - Address Configuration"]
pub type AddrcfgR = crate::FieldReader<Addrcfg>;
impl AddrcfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Addrcfg {
        match self.bits {
            0 => Addrcfg::AddressMatch0_7Bit,
            1 => Addrcfg::AddressMatch0_10Bit,
            2 => Addrcfg::AddressMatch0_7BitOrAddressMatch1_7Bit,
            3 => Addrcfg::AddressMatch0_10BitOrAddressMatch1_10Bit,
            4 => Addrcfg::AddressMatch0_7BitOrAddressMatch1_10Bit,
            5 => Addrcfg::AddressMatch0_10BitOrAddressMatch1_7Bit,
            6 => Addrcfg::FromAddressMatch0_7BitToAddressMatch1_7Bit,
            7 => Addrcfg::FromAddressMatch0_10BitToAddressMatch1_10Bit,
            _ => unreachable!(),
        }
    }
    #[doc = "Address match 0 (7-bit)"]
    #[inline(always)]
    pub fn is_address_match0_7_bit(&self) -> bool {
        *self == Addrcfg::AddressMatch0_7Bit
    }
    #[doc = "Address match 0 (10-bit)"]
    #[inline(always)]
    pub fn is_address_match0_10_bit(&self) -> bool {
        *self == Addrcfg::AddressMatch0_10Bit
    }
    #[doc = "Address match 0 (7-bit) or address match 1 (7-bit)"]
    #[inline(always)]
    pub fn is_address_match0_7_bit_or_address_match1_7_bit(&self) -> bool {
        *self == Addrcfg::AddressMatch0_7BitOrAddressMatch1_7Bit
    }
    #[doc = "Address match 0 (10-bit) or address match 1 (10-bit)"]
    #[inline(always)]
    pub fn is_address_match0_10_bit_or_address_match1_10_bit(&self) -> bool {
        *self == Addrcfg::AddressMatch0_10BitOrAddressMatch1_10Bit
    }
    #[doc = "Address match 0 (7-bit) or address match 1 (10-bit)"]
    #[inline(always)]
    pub fn is_address_match0_7_bit_or_address_match1_10_bit(&self) -> bool {
        *self == Addrcfg::AddressMatch0_7BitOrAddressMatch1_10Bit
    }
    #[doc = "Address match 0 (10-bit) or address match 1 (7-bit)"]
    #[inline(always)]
    pub fn is_address_match0_10_bit_or_address_match1_7_bit(&self) -> bool {
        *self == Addrcfg::AddressMatch0_10BitOrAddressMatch1_7Bit
    }
    #[doc = "From address match 0 (7-bit) to address match 1 (7-bit)"]
    #[inline(always)]
    pub fn is_from_address_match0_7_bit_to_address_match1_7_bit(&self) -> bool {
        *self == Addrcfg::FromAddressMatch0_7BitToAddressMatch1_7Bit
    }
    #[doc = "From address match 0 (10-bit) to address match 1 (10-bit)"]
    #[inline(always)]
    pub fn is_from_address_match0_10_bit_to_address_match1_10_bit(&self) -> bool {
        *self == Addrcfg::FromAddressMatch0_10BitToAddressMatch1_10Bit
    }
}
#[doc = "Field `ADDRCFG` writer - Address Configuration"]
pub type AddrcfgW<'a, REG> = crate::FieldWriter<'a, REG, 3, Addrcfg, crate::Safe>;
impl<'a, REG> AddrcfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Address match 0 (7-bit)"]
    #[inline(always)]
    pub fn address_match0_7_bit(self) -> &'a mut crate::W<REG> {
        self.variant(Addrcfg::AddressMatch0_7Bit)
    }
    #[doc = "Address match 0 (10-bit)"]
    #[inline(always)]
    pub fn address_match0_10_bit(self) -> &'a mut crate::W<REG> {
        self.variant(Addrcfg::AddressMatch0_10Bit)
    }
    #[doc = "Address match 0 (7-bit) or address match 1 (7-bit)"]
    #[inline(always)]
    pub fn address_match0_7_bit_or_address_match1_7_bit(self) -> &'a mut crate::W<REG> {
        self.variant(Addrcfg::AddressMatch0_7BitOrAddressMatch1_7Bit)
    }
    #[doc = "Address match 0 (10-bit) or address match 1 (10-bit)"]
    #[inline(always)]
    pub fn address_match0_10_bit_or_address_match1_10_bit(self) -> &'a mut crate::W<REG> {
        self.variant(Addrcfg::AddressMatch0_10BitOrAddressMatch1_10Bit)
    }
    #[doc = "Address match 0 (7-bit) or address match 1 (10-bit)"]
    #[inline(always)]
    pub fn address_match0_7_bit_or_address_match1_10_bit(self) -> &'a mut crate::W<REG> {
        self.variant(Addrcfg::AddressMatch0_7BitOrAddressMatch1_10Bit)
    }
    #[doc = "Address match 0 (10-bit) or address match 1 (7-bit)"]
    #[inline(always)]
    pub fn address_match0_10_bit_or_address_match1_7_bit(self) -> &'a mut crate::W<REG> {
        self.variant(Addrcfg::AddressMatch0_10BitOrAddressMatch1_7Bit)
    }
    #[doc = "From address match 0 (7-bit) to address match 1 (7-bit)"]
    #[inline(always)]
    pub fn from_address_match0_7_bit_to_address_match1_7_bit(self) -> &'a mut crate::W<REG> {
        self.variant(Addrcfg::FromAddressMatch0_7BitToAddressMatch1_7Bit)
    }
    #[doc = "From address match 0 (10-bit) to address match 1 (10-bit)"]
    #[inline(always)]
    pub fn from_address_match0_10_bit_to_address_match1_10_bit(self) -> &'a mut crate::W<REG> {
        self.variant(Addrcfg::FromAddressMatch0_10BitToAddressMatch1_10Bit)
    }
}
#[doc = "Receive All\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxall {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Rxall> for bool {
    #[inline(always)]
    fn from(variant: Rxall) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXALL` reader - Receive All"]
pub type RxallR = crate::BitReader<Rxall>;
impl RxallR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxall {
        match self.bits {
            false => Rxall::Disabled,
            true => Rxall::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rxall::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rxall::Enabled
    }
}
#[doc = "Field `RXALL` writer - Receive All"]
pub type RxallW<'a, REG> = crate::BitWriter<'a, REG, Rxall>;
impl<'a, REG> RxallW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rxall::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rxall::Enabled)
    }
}
#[doc = "Repeated Start Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rscfg {
    #[doc = "0: Any repeated Start condition following an address match"]
    AnyRepeatedStartAfterAddressMatch = 0,
    #[doc = "1: Any repeated Start condition"]
    AnyRepeatedStart = 1,
}
impl From<Rscfg> for bool {
    #[inline(always)]
    fn from(variant: Rscfg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RSCFG` reader - Repeated Start Configuration"]
pub type RscfgR = crate::BitReader<Rscfg>;
impl RscfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rscfg {
        match self.bits {
            false => Rscfg::AnyRepeatedStartAfterAddressMatch,
            true => Rscfg::AnyRepeatedStart,
        }
    }
    #[doc = "Any repeated Start condition following an address match"]
    #[inline(always)]
    pub fn is_any_repeated_start_after_address_match(&self) -> bool {
        *self == Rscfg::AnyRepeatedStartAfterAddressMatch
    }
    #[doc = "Any repeated Start condition"]
    #[inline(always)]
    pub fn is_any_repeated_start(&self) -> bool {
        *self == Rscfg::AnyRepeatedStart
    }
}
#[doc = "Field `RSCFG` writer - Repeated Start Configuration"]
pub type RscfgW<'a, REG> = crate::BitWriter<'a, REG, Rscfg>;
impl<'a, REG> RscfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Any repeated Start condition following an address match"]
    #[inline(always)]
    pub fn any_repeated_start_after_address_match(self) -> &'a mut crate::W<REG> {
        self.variant(Rscfg::AnyRepeatedStartAfterAddressMatch)
    }
    #[doc = "Any repeated Start condition"]
    #[inline(always)]
    pub fn any_repeated_start(self) -> &'a mut crate::W<REG> {
        self.variant(Rscfg::AnyRepeatedStart)
    }
}
#[doc = "Stop Detect Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sdcfg {
    #[doc = "0: Any Stop condition following an address match"]
    AnyStopAfterAddressMatch = 0,
    #[doc = "1: Any Stop condition"]
    AnyStop = 1,
}
impl From<Sdcfg> for bool {
    #[inline(always)]
    fn from(variant: Sdcfg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SDCFG` reader - Stop Detect Configuration"]
pub type SdcfgR = crate::BitReader<Sdcfg>;
impl SdcfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sdcfg {
        match self.bits {
            false => Sdcfg::AnyStopAfterAddressMatch,
            true => Sdcfg::AnyStop,
        }
    }
    #[doc = "Any Stop condition following an address match"]
    #[inline(always)]
    pub fn is_any_stop_after_address_match(&self) -> bool {
        *self == Sdcfg::AnyStopAfterAddressMatch
    }
    #[doc = "Any Stop condition"]
    #[inline(always)]
    pub fn is_any_stop(&self) -> bool {
        *self == Sdcfg::AnyStop
    }
}
#[doc = "Field `SDCFG` writer - Stop Detect Configuration"]
pub type SdcfgW<'a, REG> = crate::BitWriter<'a, REG, Sdcfg>;
impl<'a, REG> SdcfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Any Stop condition following an address match"]
    #[inline(always)]
    pub fn any_stop_after_address_match(self) -> &'a mut crate::W<REG> {
        self.variant(Sdcfg::AnyStopAfterAddressMatch)
    }
    #[doc = "Any Stop condition"]
    #[inline(always)]
    pub fn any_stop(self) -> &'a mut crate::W<REG> {
        self.variant(Sdcfg::AnyStop)
    }
}
impl R {
    #[doc = "Bit 0 - Address SCL Stall"]
    #[inline(always)]
    pub fn adrstall(&self) -> AdrstallR {
        AdrstallR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - RX SCL Stall"]
    #[inline(always)]
    pub fn rxstall(&self) -> RxstallR {
        RxstallR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Transmit Data SCL Stall"]
    #[inline(always)]
    pub fn txdstall(&self) -> TxdstallR {
        TxdstallR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - ACK SCL Stall"]
    #[inline(always)]
    pub fn ackstall(&self) -> AckstallR {
        AckstallR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Receive NACK"]
    #[inline(always)]
    pub fn rxnack(&self) -> RxnackR {
        RxnackR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 8 - General Call Enable"]
    #[inline(always)]
    pub fn gcen(&self) -> GcenR {
        GcenR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - SMBus Alert Enable"]
    #[inline(always)]
    pub fn saen(&self) -> SaenR {
        SaenR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Transmit Flag Configuration"]
    #[inline(always)]
    pub fn txcfg(&self) -> TxcfgR {
        TxcfgR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Receive Data Configuration"]
    #[inline(always)]
    pub fn rxcfg(&self) -> RxcfgR {
        RxcfgR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Ignore NACK"]
    #[inline(always)]
    pub fn ignack(&self) -> IgnackR {
        IgnackR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - HS Mode Enable"]
    #[inline(always)]
    pub fn hsmen(&self) -> HsmenR {
        HsmenR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bits 16:18 - Address Configuration"]
    #[inline(always)]
    pub fn addrcfg(&self) -> AddrcfgR {
        AddrcfgR::new(((self.bits >> 16) & 7) as u8)
    }
    #[doc = "Bit 24 - Receive All"]
    #[inline(always)]
    pub fn rxall(&self) -> RxallR {
        RxallR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Repeated Start Configuration"]
    #[inline(always)]
    pub fn rscfg(&self) -> RscfgR {
        RscfgR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Stop Detect Configuration"]
    #[inline(always)]
    pub fn sdcfg(&self) -> SdcfgR {
        SdcfgR::new(((self.bits >> 26) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Address SCL Stall"]
    #[inline(always)]
    pub fn adrstall(&mut self) -> AdrstallW<Scfgr1Spec> {
        AdrstallW::new(self, 0)
    }
    #[doc = "Bit 1 - RX SCL Stall"]
    #[inline(always)]
    pub fn rxstall(&mut self) -> RxstallW<Scfgr1Spec> {
        RxstallW::new(self, 1)
    }
    #[doc = "Bit 2 - Transmit Data SCL Stall"]
    #[inline(always)]
    pub fn txdstall(&mut self) -> TxdstallW<Scfgr1Spec> {
        TxdstallW::new(self, 2)
    }
    #[doc = "Bit 3 - ACK SCL Stall"]
    #[inline(always)]
    pub fn ackstall(&mut self) -> AckstallW<Scfgr1Spec> {
        AckstallW::new(self, 3)
    }
    #[doc = "Bit 4 - Receive NACK"]
    #[inline(always)]
    pub fn rxnack(&mut self) -> RxnackW<Scfgr1Spec> {
        RxnackW::new(self, 4)
    }
    #[doc = "Bit 8 - General Call Enable"]
    #[inline(always)]
    pub fn gcen(&mut self) -> GcenW<Scfgr1Spec> {
        GcenW::new(self, 8)
    }
    #[doc = "Bit 9 - SMBus Alert Enable"]
    #[inline(always)]
    pub fn saen(&mut self) -> SaenW<Scfgr1Spec> {
        SaenW::new(self, 9)
    }
    #[doc = "Bit 10 - Transmit Flag Configuration"]
    #[inline(always)]
    pub fn txcfg(&mut self) -> TxcfgW<Scfgr1Spec> {
        TxcfgW::new(self, 10)
    }
    #[doc = "Bit 11 - Receive Data Configuration"]
    #[inline(always)]
    pub fn rxcfg(&mut self) -> RxcfgW<Scfgr1Spec> {
        RxcfgW::new(self, 11)
    }
    #[doc = "Bit 12 - Ignore NACK"]
    #[inline(always)]
    pub fn ignack(&mut self) -> IgnackW<Scfgr1Spec> {
        IgnackW::new(self, 12)
    }
    #[doc = "Bit 13 - HS Mode Enable"]
    #[inline(always)]
    pub fn hsmen(&mut self) -> HsmenW<Scfgr1Spec> {
        HsmenW::new(self, 13)
    }
    #[doc = "Bits 16:18 - Address Configuration"]
    #[inline(always)]
    pub fn addrcfg(&mut self) -> AddrcfgW<Scfgr1Spec> {
        AddrcfgW::new(self, 16)
    }
    #[doc = "Bit 24 - Receive All"]
    #[inline(always)]
    pub fn rxall(&mut self) -> RxallW<Scfgr1Spec> {
        RxallW::new(self, 24)
    }
    #[doc = "Bit 25 - Repeated Start Configuration"]
    #[inline(always)]
    pub fn rscfg(&mut self) -> RscfgW<Scfgr1Spec> {
        RscfgW::new(self, 25)
    }
    #[doc = "Bit 26 - Stop Detect Configuration"]
    #[inline(always)]
    pub fn sdcfg(&mut self) -> SdcfgW<Scfgr1Spec> {
        SdcfgW::new(self, 26)
    }
}
#[doc = "Target Configuration 1\n\nYou can [`read`](crate::Reg::read) this register and get [`scfgr1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`scfgr1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Scfgr1Spec;
impl crate::RegisterSpec for Scfgr1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`scfgr1::R`](R) reader structure"]
impl crate::Readable for Scfgr1Spec {}
#[doc = "`write(|w| ..)` method takes [`scfgr1::W`](W) writer structure"]
impl crate::Writable for Scfgr1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SCFGR1 to value 0"]
impl crate::Resettable for Scfgr1Spec {}
