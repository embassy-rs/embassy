/// Alliance Memory AS4C16M32MSA SDRAM
#[allow(unused)]

pub mod as4c16m32msa_6 {
    use crate::fmc::sdram::{
        CasLatency, ColumnBits, InternalBanks, MemoryDataWidth, ReadPipeDelayCycles, RowBits, SdramChip,
        SdramConfiguration, SdramTiming,
    };
    use crate::time::Hertz;

    const BURST_LENGTH_1: u16 = 0x0000;
    const BURST_LENGTH_2: u16 = 0x0001;
    const BURST_LENGTH_4: u16 = 0x0002;
    const BURST_LENGTH_8: u16 = 0x0004;
    const BURST_TYPE_SEQUENTIAL: u16 = 0x0000;
    const BURST_TYPE_INTERLEAVED: u16 = 0x0008;
    const CAS_LATENCY_1: u16 = 0x0010;
    const CAS_LATENCY_2: u16 = 0x0020;
    const CAS_LATENCY_3: u16 = 0x0030;
    const OPERATING_MODE_STANDARD: u16 = 0x0000;
    const WRITEBURST_MODE_PROGRAMMED: u16 = 0x0000;
    const WRITEBURST_MODE_SINGLE: u16 = 0x0200;

    /// As4c16m32msa
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct As4c16m32msa {}

    impl SdramChip for As4c16m32msa {
        /// Value of the mode register
        const MODE_REGISTER: u16 =
            BURST_LENGTH_1 | BURST_TYPE_SEQUENTIAL | CAS_LATENCY_3 | OPERATING_MODE_STANDARD | WRITEBURST_MODE_SINGLE;

        // 166MHz = 6.024ns per clock cycle

        /// Timing Parameters
        const TIMING: SdramTiming = SdramTiming {
            startup_delay_ns: 200_000,           // 200 µs
            max_sd_clock_hz: Hertz(166_000_000), // 166 MHz
            refresh_period_ns: 7_813,            // 64ms / (8192 rows) = 7812.5ns
            mode_register_to_active_cycles: 2,   // tMRD = 2 cycles
            exit_self_refresh_cycles: 14,        // tXSR = 80ns
            active_to_precharge_cycles: 8,       // tRAS = 48ns
            row_cycle: 10,                       // tRC = 60ns
            row_precharge_cycles: 3,             // tRP = 18ns
            row_to_column_cycles: 3,             // tRCD = 18ns
        };

        /// SDRAM controller configuration
        const CONFIG: SdramConfiguration = SdramConfiguration {
            column_bits: ColumnBits::Bits9,             // A0-A8
            row_bits: RowBits::Bits13,                  // A0-A12
            memory_data_width: MemoryDataWidth::Bits32, // 32-bit
            internal_banks: InternalBanks::FourBanks,   // 4 internal banks
            cas_latency: CasLatency::Cycle3,            // CAS latency = 3
            write_protection: false,
            read_burst: true,
            read_pipe_delay_cycles: ReadPipeDelayCycles::NoDelay,
        };
    }
}
