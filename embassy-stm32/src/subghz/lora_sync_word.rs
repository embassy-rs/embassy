/// LoRa synchronization word.
///
/// Argument of [`set_lora_sync_word`][crate::subghz::SubGhz::set_lora_sync_word].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LoRaSyncWord {
    /// LoRa private network.
    Private,
    /// LoRa public network.
    Public,
}

impl LoRaSyncWord {
    pub(crate) const fn bytes(self) -> [u8; 2] {
        match self {
            LoRaSyncWord::Private => [0x14, 0x24],
            LoRaSyncWord::Public => [0x34, 0x44],
        }
    }
}
