/// SkyHigh S34ML08G3 SLC NAND Flash
#[allow(unused)]

/// SkyHigh S34ML08G3 SLC NAND Flash with 4kB pages
pub mod s34ml08g3_4kb {
    use crate::fmc::nand::{NandChip, NandConfiguration, NandTiming, NandDataWidth};

    /// S32ML08G3
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct S34ml08g3 {}

    impl NandChip for S34ml08g3 {
        /// Timing Parameters
        const TIMING: NandTiming = NandTiming {
            nce_setup_time: 15,       // tCS = 15ns min
            data_setup_time: 7,       // tDS = 7ns min
            ale_hold_time: 5,         // tALH = 5ns min
            cle_hold_time: 5,         // tCLH = 5ns min
            ale_to_nre_delay: 10,     // tAR = 10ns min
            cle_to_nre_delay: 10,     // tCLR = 10ns min
            nre_pulse_width_ns: 10,   // tRP = 10ns min
            nwe_pulse_width_ns: 10,   // tWP = 10ns min
            read_cycle_time_ns: 20,   // tRC = 20ns min
            write_cycle_time_ns: 20,  // tWC = 20ns min
            nwe_high_to_busy_ns: 100, // tWB = 100ns max
        };

        /// Nand controller configuration
        const CONFIG: NandConfiguration = NandConfiguration {
            data_width: NandDataWidth::Bits8, // 8-bit
            column_bits: 12,                  // 4096 byte pages
        };
    }
}