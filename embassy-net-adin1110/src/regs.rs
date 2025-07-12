use core::fmt::{Debug, Display};

use bitfield::{bitfield, bitfield_bitrange, bitfield_fields};

#[allow(missing_docs)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
/// SPI REGISTER DETAILS
/// Table 38.
pub enum SpiRegisters {
    IDVER = 0x00,
    PHYID = 0x01,
    CAPABILITY = 0x02,
    RESET = 0x03,
    CONFIG0 = 0x04,
    CONFIG2 = 0x06,
    STATUS0 = 0x08,
    STATUS1 = 0x09,
    IMASK0 = 0x0C,
    IMASK1 = 0x0D,
    MDIO_ACC = 0x20,
    TX_FSIZE = 0x30,
    TX = 0x31,
    TX_SPACE = 0x32,
    FIFO_CLR = 0x36,
    ADDR_FILT_UPR0 = 0x50,
    ADDR_FILT_LWR0 = 0x51,
    ADDR_FILT_UPR1 = 0x52,
    ADDR_FILT_LWR1 = 0x53,
    ADDR_MSK_LWR0 = 0x70,
    ADDR_MSK_UPR0 = 0x71,
    ADDR_MSK_LWR1 = 0x72,
    ADDR_MSK_UPR1 = 0x73,
    RX_FSIZE = 0x90,
    RX = 0x91,
}

impl Display for SpiRegisters {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<SpiRegisters> for u16 {
    fn from(val: SpiRegisters) -> Self {
        val as u16
    }
}

impl From<u16> for SpiRegisters {
    fn from(value: u16) -> Self {
        match value {
            0x00 => Self::IDVER,
            0x01 => Self::PHYID,
            0x02 => Self::CAPABILITY,
            0x03 => Self::RESET,
            0x04 => Self::CONFIG0,
            0x06 => Self::CONFIG2,
            0x08 => Self::STATUS0,
            0x09 => Self::STATUS1,
            0x0C => Self::IMASK0,
            0x0D => Self::IMASK1,
            0x20 => Self::MDIO_ACC,
            0x30 => Self::TX_FSIZE,
            0x31 => Self::TX,
            0x32 => Self::TX_SPACE,
            0x36 => Self::FIFO_CLR,
            0x50 => Self::ADDR_FILT_UPR0,
            0x51 => Self::ADDR_FILT_LWR0,
            0x52 => Self::ADDR_FILT_UPR1,
            0x53 => Self::ADDR_FILT_LWR1,
            0x70 => Self::ADDR_MSK_LWR0,
            0x71 => Self::ADDR_MSK_UPR0,
            0x72 => Self::ADDR_MSK_LWR1,
            0x73 => Self::ADDR_MSK_UPR1,
            0x90 => Self::RX_FSIZE,
            0x91 => Self::RX,
            e => panic!("Unknown value {}", e),
        }
    }
}

// Register definitions
bitfield! {
    /// Status0 Register bits
    pub struct Status0(u32);
    impl Debug;
    u32;
    /// Control Data Protection Error
    pub cdpe, _ : 12;
    /// Transmit Frame Check Squence Error
    pub txfcse, _: 11;
    /// Transmit Time Stamp Capture Available C
    pub ttscac, _ : 10;
    /// Transmit Time Stamp Capture Available B
    pub ttscab, _ : 9;
    /// Transmit Time Stamp Capture Available A
    pub ttscaa, _ : 8;
    /// PHY Interrupt for Port 1
    pub phyint, _ : 7;
    /// Reset Complete
    pub resetc, _ : 6;
    /// Header error
    pub hdre, _ : 5;
    /// Loss of Frame Error
    pub lofe, _ : 4;
    /// Receiver Buffer Overflow Error
    pub rxboe, _ : 3;
    /// Host Tx FIFO Under Run Error
    pub txbue, _ : 2;
    /// Host Tx FIFO Overflow
    pub txboe, _ : 1;
    /// Transmit Protocol Error
    pub txpe, _ : 0;
}

bitfield! {
    /// Status1 Register bits
    pub struct Status1(u32);
    impl Debug;
    u32;
    /// ECC Error on Reading the Frame Size from a Tx FIFO
    pub tx_ecc_err, set_tx_ecc_err: 12;
    /// ECC Error on Reading the Frame Size from an Rx FIFO
    pub rx_ecc_err, set_rx_ecc_err : 11;
    /// Detected an Error on an SPI Transaction
    pub spi_err, set_spi_err: 10;
    /// Rx MAC Interframe Gap Error
    pub p1_rx_ifg_err, set_p1_rx_ifg_err : 8;
    /// Port1 Rx Ready High Priority
    pub p1_rx_rdy_hi, set_p1_rx_rdy_hi : 5;
    /// Port 1 Rx FIFO Contains Data
    pub p1_rx_rdy, set_p1_rx_rdy : 4;
    /// Tx Ready
    pub tx_rdy, set_tx_rdy : 3;
    /// Link Status Changed
    pub link_change, set_link_change : 1;
    /// Port 1 Link Status
    pub p1_link_status, _ : 0;
}

bitfield! {
    /// Config0 Register bits
    pub struct Config0(u32);
    impl Debug;
    u32;
    /// Configuration Synchronization
    pub sync, set_sync : 15;
    /// Transmit Frame Check Sequence Validation Enable
    pub txfcsve, set_txfcsve : 14;
    /// !CS Align Receive Frame Enable
    pub csarfe, set_csarfe : 13;
    /// Zero Align Receive Frame Enable
    pub zarfe, set_zarfe : 12;
    /// Transmit Credit Threshold
    pub tcxthresh, set_tcxthresh : 11, 10;
    /// Transmit Cut Through Enable
    pub txcte, set_txcte : 9;
    /// Receive Cut Through Enable
    pub rxcte, set_rxcte : 8;
    /// Frame Time Stamp Enable
    pub ftse, set_ftse : 7;
    /// Receive Frame Time Stamp Select
    pub ftss, set_ftss : 6;
    /// Enable Control Data Read Write Protection
    pub prote, set_prote : 5;
    /// Enable TX Data Chunk Sequence and Retry
    pub seqe, set_seqe : 4;
    /// Chunk Payload Selector (N).
    pub cps, set_cps : 2, 0;
}

bitfield! {
    /// Config2 Register bits
    pub struct Config2(u32);
    impl Debug;
    u32;
    /// Assert TX_RDY When the Tx FIFO is Empty
    pub tx_rdy_on_empty, set_tx_rdy_on_empty : 8;
    /// Determines If the SFD is Detected in the PHY or MAC
    pub sdf_detect_src, set_sdf_detect_src : 7;
    /// Statistics Clear on Reading
    pub stats_clr_on_rd, set_stats_clr_on_rd : 6;
    /// Enable SPI CRC
    pub crc_append, set_crc_append : 5;
    /// Admit Frames with IFG Errors on Port 1 (P1)
    pub p1_rcv_ifg_err_frm, set_p1_rcv_ifg_err_frm : 4;
    /// Forward Frames Not Matching Any MAC Address to the Host
    pub p1_fwd_unk2host, set_p1_fwd_unk2host : 2;
    /// SPI to MDIO Bridge MDC Clock Speed
    pub mspeed, set_mspeed : 0;
}

bitfield! {
    /// IMASK0 Register bits
    pub struct IMask0(u32);
    impl Debug;
    u32;
    /// Control Data Protection Error Mask
    pub cppem, set_cppem : 12;
    /// Transmit Frame Check Sequence Error Mask
    pub txfcsem, set_txfcsem : 11;
    /// Transmit Time Stamp Capture Available C Mask
    pub ttscacm, set_ttscacm : 10;
    /// Transmit Time Stamp Capture Available B Mask
    pub ttscabm, set_ttscabm : 9;
    /// Transmit Time Stamp Capture Available A Mask
    pub ttscaam, set_ttscaam : 8;
    /// Physical Layer Interrupt Mask
    pub phyintm, set_phyintm : 7;
    /// RESET Complete Mask
    pub resetcm, set_resetcm : 6;
    /// Header Error Mask
    pub hdrem, set_hdrem : 5;
    /// Loss of Frame Error Mask
    pub lofem, set_lofem : 4;
    /// Receive Buffer Overflow Error Mask
    pub rxboem, set_rxboem : 3;
    /// Transmit Buffer Underflow Error Mask
    pub txbuem, set_txbuem : 2;
    /// Transmit Buffer Overflow Error Mask
    pub txboem, set_txboem : 1;
    /// Transmit Protocol Error Mask
    pub txpem, set_txpem : 0;
}

bitfield! {
    /// IMASK1 Register bits
    pub struct IMask1(u32);
    impl Debug;
    u32;
    /// Mask Bit for TXF_ECC_ERR
    pub tx_ecc_err_mask, set_tx_ecc_err_mask : 12;
    /// Mask Bit for RXF_ECC_ERR
    pub rx_ecc_err_mask, set_rx_ecc_err_mask : 11;
    /// Mask Bit for SPI_ERR
    /// This field is only used with the generic SPI protocol
    pub spi_err_mask, set_spi_err_mask : 10;
    /// Mask Bit for RX_IFG_ERR
    pub p1_rx_ifg_err_mask, set_p1_rx_ifg_err_mask : 8;
    /// Mask Bit for P1_RX_RDY
    /// This field is only used with the generic SPI protocol
    pub p1_rx_rdy_mask, set_p1_rx_rdy_mask : 4;
    /// Mask Bit for TX_FRM_DONE
    /// This field is only used with the generic SPI protocol
    pub tx_rdy_mask, set_tx_rdy_mask : 3;
    /// Mask Bit for LINK_CHANGE
    pub link_change_mask, set_link_change_mask : 1;
}

/// LED Functions
#[repr(u8)]
pub enum LedFunc {
    LinkupTxRxActicity = 0,
    LinkupTxActicity,
    LinkupRxActicity,
    LinkupOnly,
    TxRxActivity,
    TxActivity,
    RxActivity,
    LinkupRxEr,
    LinkupRxTxEr,
    RxEr,
    RxTxEr,
    TxSop,
    RxSop,
    On,
    Off,
    Blink,
    TxLevel2P4,
    TxLevel1P0,
    Master,
    Slave,
    IncompatiableLinkCfg,
    AnLinkGood,
    AnComplete,
    TsTimer,
    LocRcvrStatus,
    RemRcvrStatus,
    Clk25Ref,
    TxTCLK,
    Clk120MHz,
}

impl From<LedFunc> for u8 {
    fn from(val: LedFunc) -> Self {
        val as u8
    }
}

impl From<u8> for LedFunc {
    fn from(value: u8) -> Self {
        match value {
            0 => LedFunc::LinkupTxRxActicity,
            1 => LedFunc::LinkupTxActicity,
            2 => LedFunc::LinkupRxActicity,
            3 => LedFunc::LinkupOnly,
            4 => LedFunc::TxRxActivity,
            5 => LedFunc::TxActivity,
            6 => LedFunc::RxActivity,
            7 => LedFunc::LinkupRxEr,
            8 => LedFunc::LinkupRxTxEr,
            9 => LedFunc::RxEr,
            10 => LedFunc::RxTxEr,
            11 => LedFunc::TxSop,
            12 => LedFunc::RxSop,
            13 => LedFunc::On,
            14 => LedFunc::Off,
            15 => LedFunc::Blink,
            16 => LedFunc::TxLevel2P4,
            17 => LedFunc::TxLevel1P0,
            18 => LedFunc::Master,
            19 => LedFunc::Slave,
            20 => LedFunc::IncompatiableLinkCfg,
            21 => LedFunc::AnLinkGood,
            22 => LedFunc::AnComplete,
            23 => LedFunc::TsTimer,
            24 => LedFunc::LocRcvrStatus,
            25 => LedFunc::RemRcvrStatus,
            26 => LedFunc::Clk25Ref,
            27 => LedFunc::TxTCLK,
            28 => LedFunc::Clk120MHz,
            e => panic!("Invalid value {}", e),
        }
    }
}

/// LED Control Register
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct LedCntrl(pub u16);
bitfield_bitrange! {struct LedCntrl(u16)}

impl LedCntrl {
    bitfield_fields! {
        u8;
        /// LED 0 Pin Function
        pub from into LedFunc, led0_function, set_led0_function: 4, 0;
        /// LED 0 Mode Selection
        pub led0_mode, set_led0_mode: 5;
        /// Qualify Certain LED 0 Options with Link Status.
        pub led0_link_st_qualify, set_led0_link_st_qualify: 6;
        /// LED 0 Enable
        pub led0_en, set_led0_en: 7;
        /// LED 1 Pin Function
        pub from into LedFunc, led1_function, set_led1_function: 12, 8;
        /// /// LED 1 Mode Selection
        pub led1_mode, set_led1_mode: 13;
        /// Qualify Certain LED 1 Options with Link Status.
        pub led1_link_st_qualify, set_led1_link_st_qualify: 14;
        /// LED 1 Enable
        pub led1_en, set_led1_en: 15;
    }

    pub fn new() -> Self {
        LedCntrl(0)
    }
}

// LED Polarity
#[repr(u8)]
pub enum LedPol {
    AutoSense = 0,
    ActiveHigh,
    ActiveLow,
}

impl From<LedPol> for u8 {
    fn from(val: LedPol) -> Self {
        val as u8
    }
}

impl From<u8> for LedPol {
    fn from(value: u8) -> Self {
        match value {
            0 => LedPol::AutoSense,
            1 => LedPol::ActiveHigh,
            2 => LedPol::ActiveLow,
            e => panic!("Invalid value {}", e),
        }
    }
}

/// LED Control Register
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct LedPolarity(pub u16);
bitfield_bitrange! {struct LedPolarity(u16)}

impl LedPolarity {
    bitfield_fields! {
        u8;
        /// LED 1 Polarity
        pub from into LedPol, led1_polarity, set_led1_polarity: 3, 2;
        /// LED 0 Polarity
        pub from into LedPol, led0_polarity, set_led0_polarity: 1, 0;
    }
}

/// SPI Header
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct SpiHeader(pub u16);
bitfield_bitrange! {struct SpiHeader(u16)}

impl SpiHeader {
    bitfield_fields! {
        u16;
        /// Mask Bit for TXF_ECC_ERR
        pub control, set_control : 15;
        pub full_duplex, set_full_duplex : 14;
        /// Read or Write to register
        pub write, set_write : 13;
        /// Registers ID/addr
        pub from into SpiRegisters, addr, set_addr: 11, 0;
    }
}
