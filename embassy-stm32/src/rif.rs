//! STM32N6 Resource Isolation Framework (RIF)

use crate::pac::RIFSC;

/// Attributes for a RIF master peripheral
pub struct RifMasterAttributes {
    /// Master Compartment Identifier
    pub mcid: u8,
    /// Master Secure
    pub secure: bool,
    /// Master Privileged
    pub privileged: bool,
}

impl RifMasterAttributes {
    /// Create a new structure defining the RIF master attributes
    pub const fn new(mcid: u8, secure: bool, privileged: bool) -> Self {
        Self {
            mcid,
            secure,
            privileged,
        }
    }
}

/// RIF master peripherals
pub enum RifMaster {
    /// Embedded Trace Router
    Etr,
    /// Neural Processing Unit
    Npu,
    /// Secure Digital Multi Media Card 1
    Sdmmc1,
    /// Secure Digital Multi Media Card 2
    Sdmmc2,
    /// Universal Serial Bus On-the-Go 1
    Otg1,
    /// Universal Serial Bus On-the-Go 2
    Otg2,
    /// Ethernet 1
    Eth1,
    /// 2D Graphical Processing Unit
    Gpu2d,
    /// 2D Direct Memory Access controller
    Dma2d,
    /// Digital Camera Interface Pixel Pipeline
    Dcmipp,
    /// LCD-TFT Display Controller Layer 1
    LtdcL1,
    /// LCD-TFT Display Controller Layer 2
    LtdcL2,
    /// Video Encoder
    Venc,
}

impl RifMaster {
    /// Set the RIF master attributes
    pub fn set_attributes(&self, attr: &RifMasterAttributes) {
        // Find the matching register
        let master_index = match self {
            Self::Etr => 0,
            Self::Npu => 1,
            Self::Sdmmc1 => 2,
            Self::Sdmmc2 => 3,
            Self::Otg1 => 4,
            Self::Otg2 => 5,
            Self::Eth1 => 6,
            Self::Gpu2d => 7,
            Self::Dma2d => 8,
            Self::Dcmipp => 9,
            Self::LtdcL1 => 10,
            Self::LtdcL2 => 11,
            Self::Venc => 12,
        };

        // Set the attributes
        RIFSC.rimc_attr(master_index).modify(|w| {
            w.set_mcid(attr.mcid);
            w.set_msec(attr.secure);
            w.set_mpriv(attr.privileged);
        });
    }
}

/// Attributes for a RIF slave peripheral
pub struct RifPeripheralAttributes {
    /// Secure
    pub secure: bool,
    /// Privileged
    pub privileged: bool,
}

impl RifPeripheralAttributes {
    /// Create a new structure defining the RIF slave attributes
    pub const fn new(secure: bool, privileged: bool) -> Self {
        Self { secure, privileged }
    }
}

/// RIF slave peripherals
pub enum RifPeripheral {
    /// Serial Peripheral Interface 1
    Spi1,
    /// Serial Peripheral Interface 2
    Spi2,
    /// Serial Peripheral Interface 3
    Spi3,
    /// Serial Peripheral Interface 4
    Spi4,
    /// Serial Peripheral Interface 5
    Spi5,
    /// Serial Peripheral Interface 6
    Spi6,
    /// Serial Audio Interface 1
    Sai1,
    /// Serial Audio Interface 2
    Sai2,
    /// Inter-Integrated Circuit 1
    I2c1,
    /// Inter-Integrated Circuit 2
    I2c2,
    /// Inter-Integrated Circuit 3
    I2c3,
    /// Inter-Integrated Circuit 4
    I2c4,
    /// Improved Inter-Integrated Circuit 1
    I3c1,
    /// Improved Inter-Integrated Circuit 2
    I3c2,
    /// Universal Synchronous Asynchronous Receiver Transmitter 1
    Usart1,
    /// Universal Synchronous Asynchronous Receiver Transmitter 2
    Usart2,
    /// Universal Synchronous Asynchronous Receiver Transmitter 3
    Usart3,
    /// Universal Asynchronous Receiver Transmitter 4
    Uart4,
    /// Universal Asynchronous Receiver Transmitter 5
    Uart5,
    /// Universal Synchronous Asynchronous Receiver Transmitter 6
    Usart6,
    /// Universal Asynchronous Receiver Transmitter 7
    Uart7,
    /// Universal Asynchronous Receiver Transmitter 8
    Uart8,
    /// Universal Asynchronous Receiver Transmitter 9
    Uart9,
    /// Universal Synchronous Asynchronous Receiver Transmitter 10
    Usart10,
    /// Low-Power Universal Asynchronous Receiver Transmitter 1
    Lpuart1,
    /// Controller Area Network 1
    Fdcan1,
    /// Controller Area Network 2
    Fdcan2,
    /// Controller Area Network 3
    Fdcan3,
    /// Timer 1
    Tim1,
    /// Timer 2
    Tim2,
    /// Timer 3
    Tim3,
    /// Timer 4
    Tim4,
    /// Timer 5
    Tim5,
    /// Timer 6
    Tim6,
    /// Timer 7
    Tim7,
    /// Timer 8
    Tim8,
    /// Timer 9
    Tim9,
    /// Timer 10
    Tim10,
    /// Timer 11
    Tim11,
    /// Timer 12
    Tim12,
    /// Timer 13
    Tim13,
    /// Timer 14
    Tim14,
    /// Timer 15
    Tim15,
    /// Timer 16
    Tim16,
    /// Timer 17
    Tim17,
    /// Timer 18
    Tim18,
    /// Graphic Timer
    Gfxtim,
    /// Low-Power Timer 1
    Lptim1,
    /// Low-Power Timer 2
    Lptim2,
    /// Low-Power Timer 3
    Lptim3,
    /// Low-Power Timer 4
    Lptim4,
    /// Low-Power Timer 5
    Lptim5,
    /// Audio Digital Filter 1
    Adf1,
    /// Multi-function Digital Filter 1
    Mdf1,
    /// Secure Digital Multi Media Card 1
    Sdmmc1,
    /// Secure Digital Multi Media Card 2
    Sdmmc2,
    /// Management Data Input / Output
    Mdios,
    /// Universal Serial Bus On-the-Go High-Speed 1
    Otg1hs,
    /// Universal Serial Bus On-the-Go High-Speed 2
    Otg2hs,
    /// Univeral Serial Bus Type-C Power Delivery interface 1
    Ucpd1,
    /// Ethernet 1
    Eth1,
    /// Sony / Philips Digital InterFace Receiver
    Spdifrx,
    /// System Configuration controller
    Syscfg,
    /// Analog to Digital Converter 12
    Adc12,
    /// Voltage Reference Buffer
    Vrefbuf,
    /// Cyclic Redundancy Check
    Crc,
    /// Independent Watchdog
    Iwdg,
    /// System Window Watchdog
    Wwdg,
    /// Random Number Generator
    Rng,
    /// Public Key Accelerator
    Pka,
    /// Secure Advanced Encryption Standard coprocessor
    Saes,
    /// Hash processor
    Hash,
    /// Cryptographic processor
    Cryp,
    /// Memory Cipher Engine 1
    Mce1,
    /// Memory Cipher Engine 2
    Mce2,
    /// Memory Cipher Engine 3
    Mce3,
    /// Memory Cipher Engine 4
    Mce4,
    /// Extended Serial Peripheral Interface 1
    Xspi1,
    /// Extended Serial Peripheral Interface 2
    Xspi2,
    /// Extended Serial Peripheral Interface 3
    Xspi3,
    /// Extended Serial Peripheral Interface I/O Manager
    Xspim,
    /// Flexible Memory Controller
    Fmc,
    /// Camera Serial Interface
    Csi,
    /// Digital Camera Interface Pixel Pipeline
    Dcmipp,
    /// Digital Camera Interface
    Dcmi,
    /// Joint Photographic Experts Group encoder
    Jpeg,
    /// Video Encoder
    Venc,
    /// Instruction Cache
    Icache,
    /// 2D Graphical Processing Unit
    Gpu2d,
    /// Chrom-GRC
    Gfxmmu,
    /// 2D Direct Memory Access controller
    Dma2d,
    /// LCD-TFT Display Controller
    Ltdc,
    /// LCD-TFT Display Controller Layer 1
    LtdcL1,
    /// LCD-TFT Display Controller Layer 2
    LtdcL2,
    /// Neural Processing Unit
    Npu,
}

impl RifPeripheral {
    /// Set the RIF slave attributes
    pub fn set_attributes(&self, attr: &RifPeripheralAttributes) {
        // Find the matching register
        let (risc_index, bit_pos) = match self {
            Self::Spi1 => (0, 0),
            Self::Spi2 => (0, 1),
            Self::Spi3 => (0, 2),
            Self::Spi4 => (0, 3),
            Self::Spi5 => (0, 4),
            Self::Spi6 => (0, 5),
            Self::Sai1 => (0, 6),
            Self::Sai2 => (0, 8),
            Self::I2c1 => (0, 9),
            Self::I2c2 => (0, 10),
            Self::I2c3 => (0, 11),
            Self::I2c4 => (0, 12),
            Self::I3c1 => (0, 13),
            Self::I3c2 => (0, 14),
            Self::Usart1 => (0, 15),
            Self::Usart2 => (0, 16),
            Self::Usart3 => (0, 17),
            Self::Uart4 => (0, 18),
            Self::Uart5 => (0, 19),
            Self::Usart6 => (0, 20),
            Self::Uart7 => (0, 21),
            Self::Uart8 => (0, 22),
            Self::Uart9 => (0, 23),
            Self::Usart10 => (0, 24),
            Self::Lpuart1 => (0, 25),
            Self::Fdcan1 => (0, 26),
            Self::Fdcan2 => (0, 26),
            Self::Fdcan3 => (0, 26),
            Self::Tim1 => (0, 27),
            Self::Tim2 => (0, 28),
            Self::Tim3 => (0, 29),
            Self::Tim4 => (0, 30),
            Self::Tim5 => (0, 31),
            Self::Tim6 => (1, 0),
            Self::Tim7 => (1, 1),
            Self::Tim8 => (1, 2),
            Self::Tim9 => (1, 3),
            Self::Tim10 => (1, 4),
            Self::Tim11 => (1, 5),
            Self::Tim12 => (1, 6),
            Self::Tim13 => (1, 7),
            Self::Tim14 => (1, 8),
            Self::Tim15 => (1, 9),
            Self::Tim16 => (1, 10),
            Self::Tim17 => (1, 11),
            Self::Tim18 => (1, 12),
            Self::Gfxtim => (1, 13),
            Self::Lptim1 => (1, 14),
            Self::Lptim2 => (1, 15),
            Self::Lptim3 => (1, 16),
            Self::Lptim4 => (1, 17),
            Self::Lptim5 => (1, 18),
            Self::Adf1 => (1, 19),
            Self::Mdf1 => (1, 20),
            Self::Sdmmc1 => (1, 21),
            Self::Sdmmc2 => (1, 22),
            Self::Mdios => (1, 23),
            Self::Otg1hs => (1, 24),
            Self::Otg2hs => (1, 25),
            Self::Ucpd1 => (1, 26),
            Self::Eth1 => (1, 28),
            Self::Spdifrx => (1, 29),
            Self::Syscfg => (1, 30),
            Self::Adc12 => (2, 0),
            Self::Vrefbuf => (2, 1),
            Self::Crc => (2, 3),
            Self::Iwdg => (2, 4),
            Self::Wwdg => (2, 5),
            Self::Rng => (2, 12),
            Self::Pka => (2, 13),
            Self::Saes => (2, 14),
            Self::Hash => (2, 15),
            Self::Cryp => (2, 16),
            Self::Mce1 => (2, 17),
            Self::Mce2 => (2, 18),
            Self::Mce3 => (2, 19),
            Self::Mce4 => (2, 20),
            Self::Xspi1 => (2, 22),
            Self::Xspi2 => (2, 23),
            Self::Xspi3 => (2, 24),
            Self::Xspim => (2, 25),
            Self::Fmc => (2, 26),
            Self::Csi => (2, 28),
            Self::Dcmipp => (2, 29),
            Self::Dcmi => (2, 30),
            Self::Jpeg => (3, 0),
            Self::Venc => (3, 1),
            Self::Icache => (3, 2),
            Self::Gpu2d => (3, 3),
            Self::Gfxmmu => (3, 4),
            Self::Dma2d => (3, 5),
            Self::Ltdc => (3, 6),
            Self::LtdcL1 => (3, 7),
            Self::LtdcL2 => (3, 8),
            Self::Npu => (3, 10),
        };

        // Set the peripheral secure bit
        RIFSC
            .risc_seccfgr(risc_index)
            .modify(|w| w.set_cfg(bit_pos, attr.secure));

        // Set the peripheral privilege bit
        RIFSC
            .risc_privcfgr(risc_index)
            .modify(|w| w.set_cfg(bit_pos, attr.privileged));
    }
}
