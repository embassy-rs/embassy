/// Alliance Memory AS4C4M16SA SDRAM
/// <https://www.alliancememory.com/wp-content/uploads/pdf/dram/Alliance_Memory_64M-AS4C4M16SA-CI_v5.0_October_2018.pdf>
#[allow(unused)]

pub mod as4c4m16sa_6 {
    use crate::fmc::sdram::{
        CasLatency, ColumnBits, InternalBanks, MemoryDataWidth, ReadPipeDelayCycles, RowBits, SdramChip,
        SdramConfiguration, SdramTiming,
    };
    use crate::time::Hertz;

    // Burst length
    const BURST_LENGTH_1: u16 = 0b0000_0000_0000_0000; // A2 = 0, A1 = 0, A0 = 0
    const BURST_LENGTH_2: u16 = 0b0000_0000_0000_0001; // A2 = 0, A1 = 0, A0 = 1
    const BURST_LENGTH_4: u16 = 0b0000_0000_0000_0010; // A2 = 0, A1 = 1, A0 = 0
    const BURST_LENGTH_8: u16 = 0b0000_0000_0000_0011; // A2 = 0, A1 = 1, A0 = 1
    const BURST_LENGTH_FULL_PAGE_SEQUENTIAL: u16 = 0b0000_0000_0000_0111; // A2 = 1, A1 = 1, A0 = 1

    // Burst type
    const BURST_TYPE_SEQUENTIAL: u16 = 0b0000_0000_0000_0000; // A3 = 0
    const BURST_TYPE_INTERLEAVED: u16 = 0b0000_0000_0000_1000; // A3 = 1

    // CAS Latency
    const CAS_LATENCY_2: u16 = 0b0000_0000_0010_0000; // A6 = 0, A5 = 1, A4 = 0
    const CAS_LATENCY_3: u16 = 0b0000_0000_0011_0000; // A6 = 0, A5 = 1, A4 = 1

    // Test mode
    const TEST_MODE_NORMAL: u16 = 0b0000_0000_0000_0000; // A8 = 0, A7 = 0
    const TEST_MODE_VENDOR_USE_ONLY_10: u16 = 0b0000_0001_0000_0000; // A8 = 1, A7 = 0
    const TEST_MODE_VENDOR_USE_ONLY_01: u16 = 0b0000_0000_1000_0000; // A8 = 0, A7 = 1

    // Write burst length
    const WRITE_BURST_LENGTH_BURST: u16 = 0b0000_0000_0000_0000; // A9 = 0
    const WRITE_BURST_LENGTH_SINGLE_BIT: u16 = 0b0000_0010_0000_0000; // A9 = 1

    // RFU* = 0

    /// As4c4m16sa
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct As4c4m16sa {}

    impl SdramChip for As4c4m16sa {
        /// Value of the mode register
        const MODE_REGISTER: u16 =
            BURST_LENGTH_1 | BURST_TYPE_SEQUENTIAL | CAS_LATENCY_3 | TEST_MODE_NORMAL | WRITE_BURST_LENGTH_SINGLE_BIT;

        // 166MHz = 6.024ns per clock cycle

        /// Timing Parameters
        const TIMING: SdramTiming = SdramTiming {
            startup_delay_ns: 200_000,           // 200 µs
            max_sd_clock_hz: Hertz(166_000_000), // 166 MHz
            refresh_period_ns: 15_625,           // 64ms / (4096 rows) = 15625ns
            mode_register_to_active_cycles: 2,   // tMRD = 2 cycles
            exit_self_refresh_cycles: 11,        // tXSR = 62ns, cycles = ceil(166000000*(62*10^(-9)))
            active_to_precharge_cycles: 7,       // tRAS = 42ns cycles = ceil(166000000*(42*10^(-9)))
            row_cycle: 10,                       // tRC = 60ns cycles = ceil(166000000*(60*10^(-9)))
            row_precharge_cycles: 3,             // tRP = 18ns cycles = ceil(166000000*(18*10^(-9)))
            row_to_column_cycles: 3,             // tRCD = 18ns cycles = ceil(166000000*(18*10^(-9)))
        };

        /// SDRAM controller configuration
        const CONFIG: SdramConfiguration = SdramConfiguration {
            column_bits: ColumnBits::Bits8,             // A0-A7
            row_bits: RowBits::Bits13,                  // A0-A12
            memory_data_width: MemoryDataWidth::Bits16, // 16-bit
            internal_banks: InternalBanks::FourBanks,   // 4 internal banks
            cas_latency: CasLatency::Cycle3,            // CAS latency = 3
            write_protection: false,
            read_burst: true,
            read_pipe_delay_cycles: ReadPipeDelayCycles::NoDelay,
        };
    }
}
