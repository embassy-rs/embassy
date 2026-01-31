//! FMC NOR/PSRAM/SRAM driver.

use crate::fmc::FmcSramBank;
// Shadow the metapac values to make them more convenient to access.
pub use crate::pac::fmc::vals;

/// Specifies the type of external memory device.
///
/// Register: BCR1/2/3/4[MTYP]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NorSramMemoryType {
    /// SRAM memory
    Sram,
    /// PSRAM (CRAM) memory
    Psram,
    /// NOR Flash/OneNAND Flash
    Flash,
}

/// Specifies the external memory device width.
///
/// Register: BCR1/2/3/4[MWID]
pub enum NorSramMemoryDataWidth {
    /// 8 bit data.
    Bits8,
    /// 16 bit data.
    Bits16,
    /// 32 bit data.
    Bits32,
}

/// Specifies the wait signal polarity, valid only when accessing
/// the Flash memory in burst mode.
///
/// Register: BCR1/2/3/4[WAITPOL]
pub enum NorSramWaitSignalPolarity {
    /// Active low wait signal polarity.
    ActiveLow,
    /// Active high wait signal polarity.
    ActiveHigh,
}

/// Specifies if the wait signal is asserted by the memory one
/// clock cycle before the wait state or during the wait state,
/// valid only when accessing memories in burst mode.
///
/// Register: BCR1/2/3/4[WAITCFG]
pub enum NorSramWaitSignalActive {
    /// Wait signal active before wait state.
    BeforeWaitState,
    /// Wait signal active during wait state.
    DuringWaitState,
}

/// Specifies the memory page size.
///
/// Register: BCR1/2/3/4[CPSIZE]
pub enum NorSramPageSize {
    /// No split pages.
    NoBurstSplit,
    /// 128-byte page size.
    Bytes128,
    /// 256-byte page size.
    Bytes256,
    /// 512-byte page size.
    Bytes512,
    /// 1024-byte page size.
    Bytes1024,
}

/// See RM0433 Rev 8 P.g. 813 for timing diagrams based on these settings.
pub struct NorSramConfig {
    /// Specifies whether the address and data values are
    /// multiplexed on the data bus or not.
    ///
    /// Register: BCR1/2/3/4[MUXEN]
    data_address_mux_enabled: bool, // MUXEN

    /// Specifies the type of external memory attached to
    /// the corresponding memory device.
    memory_type: NorSramMemoryType, // MTYP

    /// Specifies the external memory device width.
    memory_data_width: NorSramMemoryDataWidth, // MWID

    /// Enables or disables the burst access mode for Flash memory,
    /// valid only with synchronous burst Flash memories.
    ///
    /// Register: BCR1/2/3/4[BURSTEN]
    burst_access_mode_enable: bool, // BURSTEN

    /// Specifies the wait signal polarity, valid only when accessing
    /// the Flash memory in burst mode.
    wait_signal_enable_polarity: NorSramWaitSignalPolarity, // WAITPOL

    /// Specifies if the wait signal is asserted by the memory one
    /// clock cycle before the wait state or during the wait state,
    /// valid only when accessing memories in burst mode.
    wait_signal_enable_active: NorSramWaitSignalActive, // WAITCFG

    /// Enables or disables the write operation in the selected device by the FMC.
    ///
    /// Register: BCR1/2/3/4[WREN]
    write_enable: bool, // WREN

    /// Enables or disables the wait state insertion via wait
    /// signal, valid for Flash memory access in burst mode.
    ///
    /// Register: BCR1/2/3/4[WAITEN]
    wait_signal_enable: bool, // WAITEN

    /// Enables or disables the extended mode.
    ///
    /// Register: BCR1/2/3/4[EXTMOD]
    extended_mode: bool, // EXTMOD

    /// Enables or disables wait signal during asynchronous transfers,
    /// valid only with asynchronous Flash memories.
    ///
    /// Register: BCR1/2/3/4[ASYNCWAIT]
    asynchronous_wait: bool, // ASYNCWAIT

    /// Enables or disables the write burst operation for PSRAM.
    ///
    /// Register: BCR1/2/3/4[CBURSTRW]
    write_burst_enable: bool, // CBURSTRW

    /// Enables or disables the FMC clock output to external memory devices.
    /// This parameter is only enabled through the FMC_BCR1 register,
    /// and don't care through FMC_BCR2..4 registers.
    ///
    /// Register: BCR1[CCLKEN]
    continuous_clock_enable: bool, // CCLKEN

    /// Enables or disables the write FIFO used by the FMC controller.
    /// This parameter is only enabled through the FMC_BCR1 register,
    /// and don't care through FMC_BCR2..4 registers.
    ///
    /// Register: BCR1[WFDIS]
    write_fifo_disable: bool, // WFDIS

    // Specifies the memory page size.
    page_size: NorSramPageSize, // CPSIZE
}

/// Specifies the access mode for the attached NOR/PSRAM/SRAM device.
pub enum NorSramAccessMode {
    /// Access mode A.
    AccessModeA,
    /// Access mode B.
    AccessModeB,
    /// Access mode C.
    AccessModeC,
    /// Access mode D.
    AccessModeD,
}

impl Into<vals::Accmod> for NorSramAccessMode {
    fn into(self) -> vals::Accmod {
        match self {
            NorSramAccessMode::AccessModeA => vals::Accmod::A,
            NorSramAccessMode::AccessModeB => vals::Accmod::B,
            NorSramAccessMode::AccessModeC => vals::Accmod::C,
            NorSramAccessMode::AccessModeD => vals::Accmod::D,
        }
    }
}

/// Timing parameters for communicating with the external NOR/PSRAM/SRAM device.
pub struct NorSramTiming {
    /// Defines the number of HCLK cycles to configure
    /// the duration of the address setup time.
    /// This parameter can be a value between Min_Data = 0 and Max_Data = 15.
    ///
    /// Duration of the first access phase (ADDSET fmc_ker_ck cycles).
    /// Minimum value for ADDSET is 0
    ///
    /// This parameter is not used with synchronous NOR Flash memories.
    address_setup_time: u8, // addset

    /// Defines the number of HCLK cycles to configure
    /// the duration of the address hold time.
    /// This parameter can be a value between Min_Data = 1 and Max_Data = 15.
    ///
    ///  This parameter is not used with synchronous NOR Flash memories.
    address_hold_time: u8, // addhld

    /// Defines the number of HCLK cycles to configure
    /// the duration of the data setup time.
    /// This parameter can be a value between Min_Data = 1 and Max_Data = 255.
    ///
    /// Duration of the second access phase (DATAST+1 fmc_ker_ck cycles
    /// for write accesses, DATAST fmc_ker_ck cycles for read accesses).
    ///
    /// This parameter is used for SRAMs, ROMs and asynchronous multiplexed NOR Flash memories.
    data_setup_time: u8, // datast

    /// Defines the number of HCLK cycles to configure
    /// the duration of the bus turnaround.
    /// This parameter can be a value between Min_Data = 0 and Max_Data = 15.
    ///
    /// Time between NEx high to NEx low (BUSTURN fmc_ker_ck)
    ///
    /// This parameter is only used for multiplexed NOR Flash memories.
    bus_turn_around_duration: u8, // busturn

    /// Defines the period of CLK clock output signal, expressed in number of
    /// HCLK cycles. This parameter can be a value between Min_Data = 2 and
    /// Max_Data = 16.
    ///
    /// This parameter is not used for asynchronous NOR Flash, SRAM or ROM accesses.
    clock_division: u8, // clkdiv

    /// Defines the number of memory clock cycles to issue
    /// to the memory before getting the first data.
    /// The parameter value depends on the memory type as shown below:
    /// - It must be set to 0 in case of a CRAM
    /// - It is don't care in asynchronous NOR, SRAM or ROM accesses
    /// - It may assume a value between Min_Data = 2 and Max_Data = 17
    /// in NOR Flash memories with synchronous burst mode enable.
    data_latency: u8, // datlat

    /// Specifies the asynchronous access mode.
    access_mode: NorSramAccessMode, // accmod
}

/// Describes why NOR/PSRAM/SRAM controller initialization failed.
#[derive(Debug)]
pub enum NorSramInitError {
    /// Indicates that the supplied NOR/PSRAM/SRAM config was invalid.
    InvalidConfig,
    /// Indicates that the supplied NOR/PSRAM/SRAM timing parameters was invalid.
    InvalidTiming(NorSramTimingError),
}

impl From<NorSramTimingError> for NorSramInitError {
    fn from(value: NorSramTimingError) -> Self {
        NorSramInitError::InvalidTiming(value)
    }
}

/// Describes the type of NOR/PSRAM/SRAM timing error.
#[derive(Debug)]
pub enum NorSramTimingError {
    /// Indicates that the address setup time was invalid.
    InvalidAddressSetupTime,
    /// Indicates that the address hold time was invalid.
    InvalidAddressHoldTime,
    /// Indicates that the data setup time was invalid.
    InvalidDataSetupTime,
    /// Indicates that the data hold duration was invalid.
    InvalidDataHoldDuration,
    /// Indicates that the turnaround time was invalid.
    InvalidTurnaroundTime,
    /// Indicates that the clock divisor was invalid.
    InvalidClockDivisor,
}

/// Wraps the FMC NOR/PSRAM/SRAM controller functionality.
pub struct NorSram<'a, 'd, T: super::Instance> {
    fmc: &'a super::Fmc<'d, T>,
    bank: FmcSramBank,

    config: NorSramConfig,
    timing: NorSramTiming,

    memory: &'a mut [u16],
}

impl<'a, 'd, T: super::Instance> NorSram<'a, 'd, T> {
    /// Note that the total size of each NOR/SRAM bank is 64Mbytes. (RM0433 Rev 8 P.g. 808)
    /// The maximum capacity is 512 Mbits (26 address lines).  (RM0433 Rev 8 P.g. 809)
    pub fn new(fmc: &'a super::Fmc<'d, T>, bank: FmcSramBank, config: NorSramConfig, timing: NorSramTiming) -> Self {
        let memory: &mut [u16] = unsafe {
            // Initialise controller and SDRAM
            let ram_ptr: *mut u16 = fmc.nor_sram_addr(bank) as *mut _;

            // Convert raw pointer to slice.
            //
            // We reference the full SRAM window so we can use some
            // memory tricks to write batches of commands and data.
            core::slice::from_raw_parts_mut(ram_ptr, bank.size() as usize)
        };

        Self {
            fmc,
            bank,
            config,
            timing,
            memory,
        }
    }

    /// Initializes the NOR/PSRAM/SRAM bank.
    pub fn init(&mut self) -> Result<(), NorSramInitError> {
        // Ensure that the bank is disabled before being configured.
        self.disable_bank();

        self.configure_features()?;
        self.configure_timing()?;
        // TODO: extended mode timing

        // Enable the bank after configuring the features and timings.
        self.enable_bank();

        Ok(())
    }

    /// Returns the base address of the NOR/PSRAM/SRAM memory mapped region.
    pub fn addr(&self) -> u32 {
        self.fmc.nor_sram_addr(self.bank)
    }

    /// Returns the a pointer to the base address
    /// of the NOR/PSRAM/SRAM memory mapped region.
    pub fn ptr(&self) -> *mut u16 {
        (self.addr()) as *mut u16
    }

    /// Returns the memory address for the FMC bank that, when written
    /// to, will send it as a "command" signal to the LCD.
    ///
    /// Because the LCD is tied to A0 as the D/C line, setting bit 0 to
    /// 1/0 toggles whether a data or command byte is being sent.
    pub fn cmd_addr(&self) -> *mut u16 {
        self.ptr()
    }

    /// Returns the address that, when written to,
    /// will send it as a "data" signal to the LCD.
    ///
    /// Because the LCD is tied to A0 as the D/C line, setting bit 0 to
    /// 1/0 toggles whether a data or command byte is being sent.
    pub fn data_addr(&self) -> *mut u16 {
        // Add one byte to get the data address.
        //
        // Setting A0 to 1/0 selects data/command
        //
        // NOTE: The 0xF comes from a hacky fix. Using A0 as the data/command line caused a
        // problem where DMA forcing aligned writes pushed data into the next even address.
        // Taking advanage of the full SRAM address space available and using an even address
        // with a 1 in the lower bit solves this problem.
        (self.addr() + 0xF) as *mut u16
    }

    /// Write a command byte to the LCD controller, and then
    /// reads a number of response bytes from the controller.
    pub fn write_command(&mut self, cmd: u8, args: &[u8]) {
        // Write to the lower byte so A0=0 and triggers command mode.
        // ptr::write(self.cmd_addr(), cmd as u16);
        self.memory[0] = cmd as u16;

        for i in 0..args.len() {
            // ptr::write(self.data_addr(), args[i] as u16);
            self.memory[1] = args[i] as u16;
        }
    }

    /// Writes a half-word to the device.
    pub fn write_data(&mut self, data: u16) {
        // ptr::write(self.data_addr(), data);
        self.memory[1] = data;
    }

    /// Reads the next half-word from the device.
    pub fn read_data(&mut self) -> u16 {
        let result = self.memory[1]; // ptr::read(self.data_addr());

        trace!("read: 0x{:x}", result);

        result
    }

    /// Disables the configured NOR/PSRAM/SRAM bank.
    pub fn disable_bank(&mut self) {
        #[cfg(any(fmc_v1x3, fmc_v2x1, fmc_v3x1))]
        let regs = T::regs();
        #[cfg(fmc_v4)]
        let regs = T::regs().nor_psram();

        // Modify the SRAM/NOR-Flash chip-select control
        // register to disable the relevant bank.
        if self.bank == FmcSramBank::Bank1 {
            regs.bcr1().modify(|bcr1| {
                bcr1.set_mbken(false);
            });
        } else {
            regs.bcr(match self.bank {
                FmcSramBank::Bank1 => panic!("invalid"),
                FmcSramBank::Bank2 => 0,
                FmcSramBank::Bank3 => 1,
                FmcSramBank::Bank4 => 2,
            })
            .modify(|bcr| {
                bcr.set_mbken(false);
            });
        }
    }

    /// Enables the configured NOR/PSRAM/SRAM bank.
    pub fn enable_bank(&mut self) {
        #[cfg(any(fmc_v1x3, fmc_v2x1, fmc_v3x1))]
        let regs = T::regs();
        #[cfg(fmc_v4)]
        let regs = T::regs().nor_psram();

        // Modify the SRAM/NOR-Flash chip-select control
        // register to enable the relevant bank.
        if self.bank == FmcSramBank::Bank1 {
            regs.bcr1().modify(|bcr| {
                bcr.set_mbken(true);
            });
        } else {
            regs.bcr(match self.bank {
                FmcSramBank::Bank1 => panic!("invalid"),
                FmcSramBank::Bank2 => 0,
                FmcSramBank::Bank3 => 1,
                FmcSramBank::Bank4 => 2,
            })
            .modify(|bcr| {
                bcr.set_mbken(true);
            });
        }
    }

    /// Initializes the FMC configuration register corresponding to the configured bank.
    pub fn configure_features(&mut self) -> Result<(), NorSramInitError> {
        if self.config.continuous_clock_enable == true && self.bank != FmcSramBank::Bank1 {
            // See STM32 HAL for implementation detail.
            warn!(
                "Bank 1 continous clocks needs to be in synchronous mode when continous clock is enabled for banks 2, 3, and 4."
            );
        }

        if self.bank != FmcSramBank::Bank1 {
            // See STM32 HAL for implementation detail.
            warn!("Write FIFO should be configured on bank 1 when enabled on banks 2, 3, and 4.")
        }

        // RM0433 Rev 8 (P.g. 838)
        // if self.bank == FMCRegion::Bank2Remapped
        //     || self.bank == FMCRegion::Bank3Remapped
        //     || self.bank == FMCRegion::Bank4Remapped
        // {
        //     warn!(
        //         "Ensure that FMC BCR bank 1 is remapped to allow remapped addresses of 2, 3, and 4."
        //     )
        // }

        let mut norsram_flash_access_enable = false;
        if self.config.memory_type == NorSramMemoryType::Flash {
            norsram_flash_access_enable = true;
        }

        #[cfg(any(fmc_v1x3, fmc_v2x1, fmc_v3x1))]
        let regs = T::regs();
        #[cfg(fmc_v4)]
        let regs = T::regs().nor_psram();

        if self.bank == FmcSramBank::Bank1 {
            // SRAM/NOR-Flash chip-select control register
            regs.bcr1().modify::<Result<(), NorSramInitError>>(|bcr| {
                // Remap the address space for the entire SRAM bank.
                // if self.bank == FmcSramBank::Bank1Remapped {
                //     bcr.set_bmap(0b01); // NOR/PSRAM bank and SDRAM bank 1/bank2 are swapped
                // }

                bcr.set_cclken(self.config.continuous_clock_enable);

                // Burst enable for PSRAM
                #[cfg(any(fmc_v1x3, fmc_v2x1))]
                bcr.set_cburstrw(self.config.write_burst_enable);
                #[cfg(not(any(fmc_v1x3, fmc_v2x1)))]
                bcr.set_cburstrw(match self.config.write_burst_enable {
                    true => vals::Cburstrw::ASYNCHRONOUS,
                    false => vals::Cburstrw::SYNCHRONOUS,
                });

                // tODO: is true enabled?
                bcr.set_asyncwait(self.config.asynchronous_wait); // Asyncronous wait
                bcr.set_extmod(self.config.extended_mode); // Extended mode
                bcr.set_waiten(self.config.wait_signal_enable); // Wait signal
                bcr.set_wren(self.config.write_enable); // Write operation
                bcr.set_waitcfg(match self.config.wait_signal_enable_active {
                    NorSramWaitSignalActive::BeforeWaitState => vals::Waitcfg::BEFORE_WAIT_STATE,
                    NorSramWaitSignalActive::DuringWaitState => vals::Waitcfg::DURING_WAIT_STATE,
                }); // Wait signal active

                bcr.set_waitpol(match self.config.wait_signal_enable_polarity {
                    NorSramWaitSignalPolarity::ActiveLow => vals::Waitpol::ACTIVE_LOW,
                    NorSramWaitSignalPolarity::ActiveHigh => vals::Waitpol::ACTIVE_HIGH,
                }); // Wait signal polarity

                bcr.set_bursten(self.config.burst_access_mode_enable); // Burst access mode

                bcr.set_faccen(norsram_flash_access_enable); // NOR Flash memory access 

                bcr.set_mwid(match self.config.memory_data_width {
                    NorSramMemoryDataWidth::Bits8 => vals::Mwid::BITS8,
                    NorSramMemoryDataWidth::Bits16 => vals::Mwid::BITS16,
                    NorSramMemoryDataWidth::Bits32 => vals::Mwid::BITS32,
                }); // Memory data width

                bcr.set_mtyp(match self.config.memory_type {
                    NorSramMemoryType::Sram => vals::Mtyp::SRAM,
                    NorSramMemoryType::Psram => vals::Mtyp::PSRAM,
                    NorSramMemoryType::Flash => vals::Mtyp::FLASH,
                }); // Memory types

                bcr.set_muxen(self.config.data_address_mux_enabled); // Data address mux

                #[cfg(not(fmc_v1x3))]
                bcr.set_wfdis(self.config.write_fifo_disable);

                bcr.set_cpsize(match self.config.page_size {
                    NorSramPageSize::NoBurstSplit => vals::Cpsize::NO_BURST_SPLIT,
                    NorSramPageSize::Bytes128 => vals::Cpsize::BYTES128,
                    NorSramPageSize::Bytes256 => vals::Cpsize::BYTES256,
                    NorSramPageSize::Bytes512 => vals::Cpsize::BYTES512,
                    NorSramPageSize::Bytes1024 => vals::Cpsize::BYTES1024,
                });

                trace!("fmc bcr: {}", bcr);

                Ok(())
            })?;
        } else {
            regs.bcr(match self.bank {
                FmcSramBank::Bank1 => panic!("invalid"),
                FmcSramBank::Bank2 => 0,
                FmcSramBank::Bank3 => 1,
                FmcSramBank::Bank4 => 2,
            })
            .modify::<Result<(), NorSramInitError>>(|bcr| {
                // Burst enable for PSRAM
                #[cfg(any(fmc_v1x3, fmc_v2x1))]
                bcr.set_cburstrw(self.config.write_burst_enable);
                #[cfg(not(any(fmc_v1x3, fmc_v2x1)))]
                bcr.set_cburstrw(match self.config.write_burst_enable {
                    true => vals::Cburstrw::ASYNCHRONOUS,
                    false => vals::Cburstrw::SYNCHRONOUS,
                });

                bcr.set_asyncwait(self.config.asynchronous_wait); // Asyncronous wait
                bcr.set_extmod(self.config.extended_mode); // Extended mode
                bcr.set_waiten(self.config.wait_signal_enable); // Wait signal
                bcr.set_wren(self.config.write_enable); // Write operation
                bcr.set_waitcfg(match self.config.wait_signal_enable_active {
                    NorSramWaitSignalActive::BeforeWaitState => vals::Waitcfg::BEFORE_WAIT_STATE,
                    NorSramWaitSignalActive::DuringWaitState => vals::Waitcfg::DURING_WAIT_STATE,
                }); // Wait signal active

                bcr.set_waitpol(match self.config.wait_signal_enable_polarity {
                    NorSramWaitSignalPolarity::ActiveLow => vals::Waitpol::ACTIVE_LOW,
                    NorSramWaitSignalPolarity::ActiveHigh => vals::Waitpol::ACTIVE_HIGH,
                }); // Wait signal polarity

                bcr.set_bursten(self.config.burst_access_mode_enable); // Burst access mode

                bcr.set_faccen(norsram_flash_access_enable); // NOR Flash memory access 

                bcr.set_mwid(match self.config.memory_data_width {
                    NorSramMemoryDataWidth::Bits8 => vals::Mwid::BITS8,
                    NorSramMemoryDataWidth::Bits16 => vals::Mwid::BITS16,
                    NorSramMemoryDataWidth::Bits32 => vals::Mwid::BITS32,
                }); // Memory data width

                bcr.set_mtyp(match self.config.memory_type {
                    NorSramMemoryType::Sram => vals::Mtyp::SRAM,
                    NorSramMemoryType::Psram => vals::Mtyp::PSRAM,
                    NorSramMemoryType::Flash => vals::Mtyp::FLASH,
                }); // Memory types

                bcr.set_muxen(self.config.data_address_mux_enabled); // Data address mux

                bcr.set_cpsize(match self.config.page_size {
                    NorSramPageSize::NoBurstSplit => vals::Cpsize::NO_BURST_SPLIT,
                    NorSramPageSize::Bytes128 => vals::Cpsize::BYTES128,
                    NorSramPageSize::Bytes256 => vals::Cpsize::BYTES256,
                    NorSramPageSize::Bytes512 => vals::Cpsize::BYTES512,
                    NorSramPageSize::Bytes1024 => vals::Cpsize::BYTES1024,
                });

                trace!("fmc bcr: {}", bcr);

                Ok(())
            })?;
        }

        Ok(())
    }

    /// Initializes the FMC timing register corresponding to the enabled bank.
    pub fn configure_timing(&mut self) -> Result<(), NorSramTimingError> {
        if !(self.timing.address_setup_time <= 15) {
            return Err(NorSramTimingError::InvalidAddressSetupTime);
        }

        if !(self.timing.address_hold_time > 0 && self.timing.address_hold_time <= 15) {
            return Err(NorSramTimingError::InvalidAddressHoldTime);
        }

        if !(self.timing.data_setup_time > 0) {
            return Err(NorSramTimingError::InvalidDataSetupTime);
        }

        if !(self.timing.bus_turn_around_duration <= 15) {
            return Err(NorSramTimingError::InvalidTurnaroundTime);
        }

        if !(self.timing.clock_division > 1 && self.timing.clock_division <= 16) {
            return Err(NorSramTimingError::InvalidClockDivisor);
        }

        #[cfg(any(fmc_v1x3, fmc_v2x1, fmc_v3x1))]
        let regs = T::regs();
        #[cfg(fmc_v4)]
        let regs = T::regs().nor_psram();

        // BTR1/2/3/4 SRAM/NOR-Flash write timing registers
        regs.btr(match self.bank {
            FmcSramBank::Bank1 => 0,
            FmcSramBank::Bank2 => 1,
            FmcSramBank::Bank3 => 2,
            FmcSramBank::Bank4 => 3,
        })
        .modify(|btr| {
            btr.set_addset(self.timing.address_setup_time);
            btr.set_addhld(self.timing.address_hold_time);
            btr.set_datast(self.timing.data_setup_time);
            btr.set_busturn(self.timing.bus_turn_around_duration);
            btr.set_clkdiv(self.timing.clock_division);
            btr.set_datlat(self.timing.data_latency);
            btr.set_accmod(match self.timing.access_mode {
                NorSramAccessMode::AccessModeA => vals::Accmod::A,
                NorSramAccessMode::AccessModeB => vals::Accmod::B,
                NorSramAccessMode::AccessModeC => vals::Accmod::C,
                NorSramAccessMode::AccessModeD => vals::Accmod::D,
            });
            trace!("fmc btr: {}", btr);
        });

        Ok(())
    }
}
