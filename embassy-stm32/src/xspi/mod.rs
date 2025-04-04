//! XSPI Serial Peripheral Interface
//!

#![macro_use]

pub mod enums;

use core::marker::PhantomData;

use embassy_embedded_hal::{GetConfig, SetConfig};
use embassy_hal_internal::PeripheralType;
pub use enums::*;

use crate::dma::{word, ChannelAndRequest};
use crate::gpio::{AfType, AnyPin, OutputType, Pull, SealedPin as _, Speed};
use crate::mode::{Async, Blocking, Mode as PeriMode};
use crate::pac::xspi::vals::*;
use crate::pac::xspi::Xspi as Regs;
#[cfg(xspim_v1)]
use crate::pac::xspim::Xspim;
use crate::rcc::{self, RccPeripheral};
use crate::{peripherals, Peri};

/// XPSI driver config.
#[derive(Clone, Copy)]
pub struct Config {
    /// Fifo threshold used by the peripheral to generate the interrupt indicating data
    /// or space is available in the FIFO
    pub fifo_threshold: FIFOThresholdLevel,
    /// Indicates the type of external device connected
    pub memory_type: MemoryType, // Need to add an additional enum to provide this public interface
    /// Defines the size of the external device connected to the XSPI corresponding
    /// to the number of address bits required to access the device
    pub device_size: MemorySize,
    /// Sets the minimum number of clock cycles that the chip select signal must be held high
    /// between commands
    pub chip_select_high_time: ChipSelectHighTime,
    /// Enables the free running clock
    pub free_running_clock: bool,
    /// Sets the clock level when the device is not selected
    pub clock_mode: bool,
    /// Indicates the wrap size corresponding to the external device configuration
    pub wrap_size: WrapSize,
    /// Specified the prescaler factor used for generating the external clock based
    /// on the AHB clock. 0 = Fkernel, 1 = Fkernel/2, 2 = Fkernel/3 etc.
    pub clock_prescaler: u8,
    /// Allows the delay of 1/2 cycle the data sampling to account for external
    /// signal delays
    pub sample_shifting: bool,
    /// Allows hold to 1/4 cycle the data
    pub delay_hold_quarter_cycle: bool,
    /// Enables the transaction boundary feature and defines the boundary to release
    /// the chip select
    pub chip_select_boundary: u8,
    /// Enables communication regulation feature. Chip select is released when the other
    /// XSpi requests access to the bus
    pub max_transfer: u8,
    /// Enables the refresh feature, chip select is released every refresh + 1 clock cycles
    pub refresh: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fifo_threshold: FIFOThresholdLevel::_16Bytes, // 32 bytes FIFO, half capacity
            memory_type: MemoryType::Micron,
            device_size: MemorySize::Other(0),
            chip_select_high_time: ChipSelectHighTime::_5Cycle,
            free_running_clock: false,
            clock_mode: false,
            wrap_size: WrapSize::None,
            clock_prescaler: 0,
            sample_shifting: false,
            delay_hold_quarter_cycle: false,
            chip_select_boundary: 0, // Acceptable range 0 to 31
            max_transfer: 0,
            refresh: 0,
        }
    }
}

/// XSPI transfer configuration.
pub struct TransferConfig {
    /// Instruction width (IMODE)
    pub iwidth: XspiWidth,
    /// Instruction Id
    pub instruction: Option<u32>,
    /// Number of Instruction Bytes
    pub isize: AddressSize,
    /// Instruction Double Transfer rate enable
    pub idtr: bool,

    /// Address width (ADMODE)
    pub adwidth: XspiWidth,
    /// Device memory address
    pub address: Option<u32>,
    /// Number of Address Bytes
    pub adsize: AddressSize,
    /// Address Double Transfer rate enable
    pub addtr: bool,

    /// Alternate bytes width (ABMODE)
    pub abwidth: XspiWidth,
    /// Alternate Bytes
    pub alternate_bytes: Option<u32>,
    /// Number of Alternate Bytes
    pub absize: AddressSize,
    /// Alternate Bytes Double Transfer rate enable
    pub abdtr: bool,

    /// Data width (DMODE)
    pub dwidth: XspiWidth,
    /// Data buffer
    pub ddtr: bool,

    /// Number of dummy cycles (DCYC)
    pub dummy: DummyCycles,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            iwidth: XspiWidth::NONE,
            instruction: None,
            isize: AddressSize::_8bit,
            idtr: false,

            adwidth: XspiWidth::NONE,
            address: None,
            adsize: AddressSize::_8bit,
            addtr: false,

            abwidth: XspiWidth::NONE,
            alternate_bytes: None,
            absize: AddressSize::_8bit,
            abdtr: false,

            dwidth: XspiWidth::NONE,
            ddtr: false,

            dummy: DummyCycles::_0,
        }
    }
}

/// Error used for Xspi implementation
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum XspiError {
    /// Peripheral configuration is invalid
    InvalidConfiguration,
    /// Operation configuration is invalid
    InvalidCommand,
    /// Size zero buffer passed to instruction
    EmptyBuffer,
}

/// XSPI driver.
pub struct Xspi<'d, T: Instance, M: PeriMode> {
    _peri: Peri<'d, T>,
    clk: Option<Peri<'d, AnyPin>>,
    d0: Option<Peri<'d, AnyPin>>,
    d1: Option<Peri<'d, AnyPin>>,
    d2: Option<Peri<'d, AnyPin>>,
    d3: Option<Peri<'d, AnyPin>>,
    d4: Option<Peri<'d, AnyPin>>,
    d5: Option<Peri<'d, AnyPin>>,
    d6: Option<Peri<'d, AnyPin>>,
    d7: Option<Peri<'d, AnyPin>>,
    d8: Option<Peri<'d, AnyPin>>,
    d9: Option<Peri<'d, AnyPin>>,
    d10: Option<Peri<'d, AnyPin>>,
    d11: Option<Peri<'d, AnyPin>>,
    d12: Option<Peri<'d, AnyPin>>,
    d13: Option<Peri<'d, AnyPin>>,
    d14: Option<Peri<'d, AnyPin>>,
    d15: Option<Peri<'d, AnyPin>>,
    ncs: Option<Peri<'d, AnyPin>>,
    // TODO: allow switching between multiple chips
    ncs_alt: Option<Peri<'d, AnyPin>>,
    // false if ncs == NCS1, true if ncs == NCS2
    // (ncs_alt will be the opposite to ncs).
    _cssel_swap: bool,
    dqs0: Option<Peri<'d, AnyPin>>,
    dqs1: Option<Peri<'d, AnyPin>>,
    dma: Option<ChannelAndRequest<'d>>,
    _phantom: PhantomData<M>,
    config: Config,
    width: XspiWidth,
}

impl<'d, T: Instance, M: PeriMode> Xspi<'d, T, M> {
    /// Enter memory mode.
    /// The Input `read_config` is used to configure the read operation in memory mode
    pub fn enable_memory_mapped_mode(
        &mut self,
        read_config: TransferConfig,
        write_config: TransferConfig,
    ) -> Result<(), XspiError> {
        // Use configure command to set read config
        self.configure_command(&read_config, None)?;

        let reg = T::REGS;
        while reg.sr().read().busy() {}

        reg.ccr().modify(|r| {
            r.set_dqse(false);
        });

        // Set wrting configurations, there are separate registers for write configurations in memory mapped mode
        reg.wccr().modify(|w| {
            w.set_imode(WccrImode::from_bits(write_config.iwidth.into()));
            w.set_idtr(write_config.idtr);
            w.set_isize(WccrIsize::from_bits(write_config.isize.into()));

            w.set_admode(WccrAdmode::from_bits(write_config.adwidth.into()));
            w.set_addtr(write_config.addtr);
            w.set_adsize(WccrAdsize::from_bits(write_config.adsize.into()));

            w.set_dmode(WccrDmode::from_bits(write_config.dwidth.into()));
            w.set_ddtr(write_config.ddtr);

            w.set_abmode(WccrAbmode::from_bits(write_config.abwidth.into()));
            w.set_dqse(true);
        });

        reg.wtcr().modify(|w| w.set_dcyc(write_config.dummy.into()));

        // Enable memory mapped mode
        reg.cr().modify(|r| {
            r.set_fmode(Fmode::B_0X3);
            r.set_tcen(false);
        });
        Ok(())
    }

    /// Quit from memory mapped mode
    pub fn disable_memory_mapped_mode(&mut self) {
        let reg = T::REGS;

        reg.cr().modify(|r| {
            r.set_fmode(Fmode::B_0X0);
            r.set_abort(true);
            r.set_dmaen(false);
            r.set_en(false);
        });

        // Clear transfer complete flag
        reg.fcr().write(|w| w.set_ctcf(true));

        // Re-enable ospi
        reg.cr().modify(|r| {
            r.set_en(true);
        });
    }

    fn new_inner(
        peri: Peri<'d, T>,
        d0: Option<Peri<'d, AnyPin>>,
        d1: Option<Peri<'d, AnyPin>>,
        d2: Option<Peri<'d, AnyPin>>,
        d3: Option<Peri<'d, AnyPin>>,
        d4: Option<Peri<'d, AnyPin>>,
        d5: Option<Peri<'d, AnyPin>>,
        d6: Option<Peri<'d, AnyPin>>,
        d7: Option<Peri<'d, AnyPin>>,
        d8: Option<Peri<'d, AnyPin>>,
        d9: Option<Peri<'d, AnyPin>>,
        d10: Option<Peri<'d, AnyPin>>,
        d11: Option<Peri<'d, AnyPin>>,
        d12: Option<Peri<'d, AnyPin>>,
        d13: Option<Peri<'d, AnyPin>>,
        d14: Option<Peri<'d, AnyPin>>,
        d15: Option<Peri<'d, AnyPin>>,
        clk: Option<Peri<'d, AnyPin>>,
        ncs_cssel: u8,
        ncs: Option<Peri<'d, AnyPin>>,
        ncs_alt: Option<Peri<'d, AnyPin>>,
        dqs0: Option<Peri<'d, AnyPin>>,
        dqs1: Option<Peri<'d, AnyPin>>,
        dma: Option<ChannelAndRequest<'d>>,
        config: Config,
        width: XspiWidth,
        dual_quad: bool,
    ) -> Self {
        // Enable the interface
        match T::SPI_IDX {
            1 => crate::pac::PWR.csr2().modify(|r| r.set_en_xspim1(true)),
            2 => crate::pac::PWR.csr2().modify(|r| r.set_en_xspim2(true)),
            _ => unreachable!(),
        };

        #[cfg(xspim_v1)]
        {
            // RCC for xspim should be enabled before writing register
            crate::pac::RCC.ahb5enr().modify(|w| w.set_iomngren(true));

            // Disable XSPI peripheral first
            T::REGS.cr().modify(|w| {
                w.set_en(false);
            });

            // XSPI IO Manager has been enabled before
            T::SPIM_REGS.cr().modify(|w| {
                w.set_muxen(false);
                w.set_req2ack_time(0xff);
            });
        }

        // System configuration
        rcc::enable_and_reset::<T>();
        while T::REGS.sr().read().busy() {}

        // Device configuration
        T::REGS.dcr1().modify(|w| {
            w.set_devsize(config.device_size.into());
            w.set_mtyp(Mtyp::from_bits(config.memory_type.into()));
            w.set_csht(Csht::from_bits(config.chip_select_high_time.into()));
            w.set_frck(false);
            w.set_ckmode(Ckmode::from_bits(config.clock_mode.into()));
        });

        T::REGS.dcr2().modify(|w| {
            w.set_wrapsize(Wrapsize::from_bits(config.wrap_size.into()));
        });

        T::REGS.dcr3().modify(|w| {
            w.set_csbound(Csbound::from_bits(config.chip_select_boundary.into()));
            #[cfg(xspi_v1)]
            {
                w.set_maxtran(Maxtran::from_bits(config.max_transfer.into()));
            }
        });

        T::REGS.dcr4().modify(|w| {
            w.set_refresh(Refresh::from_bits(config.refresh.into()));
        });

        T::REGS.cr().modify(|w| {
            w.set_fthres(Fthres::from_bits(config.fifo_threshold.into()));
        });

        // Wait for busy flag to clear
        while T::REGS.sr().read().busy() {}

        T::REGS.dcr2().modify(|w| {
            w.set_prescaler(config.clock_prescaler);
        });

        // Wait for busy flag to clear after changing prescaler, during calibration
        while T::REGS.sr().read().busy() {}

        T::REGS.cr().modify(|w| {
            w.set_dmm(dual_quad);

            assert!(ncs_alt.is_none(), "ncs_alt TODO");
            let cssel = if ncs_cssel == 0 { Cssel::B_0X0 } else { Cssel::B_0X1 };
            w.set_cssel(cssel);
        });

        T::REGS.tcr().modify(|w| {
            w.set_sshift(config.sample_shifting);
            w.set_dhqc(config.delay_hold_quarter_cycle);
        });

        // Enable peripheral
        T::REGS.cr().modify(|w| {
            w.set_en(true);
        });

        // Free running clock needs to be set after peripheral enable
        if config.free_running_clock {
            T::REGS.dcr1().modify(|w| {
                w.set_frck(config.free_running_clock);
            });
        }

        Self {
            _peri: peri,
            clk,
            d0,
            d1,
            d2,
            d3,
            d4,
            d5,
            d6,
            d7,
            d8,
            d9,
            d10,
            d11,
            d12,
            d13,
            d14,
            d15,
            ncs,
            ncs_alt,
            _cssel_swap: ncs_cssel == 1,
            dqs0,
            dqs1,
            dma,
            _phantom: PhantomData,
            config,
            width,
        }
    }

    // Function to configure the peripheral for the requested command
    fn configure_command(&mut self, command: &TransferConfig, data_len: Option<usize>) -> Result<(), XspiError> {
        // Check that transaction doesn't use more than hardware initialized pins
        if <enums::XspiWidth as Into<u8>>::into(command.iwidth) > <enums::XspiWidth as Into<u8>>::into(self.width)
            || <enums::XspiWidth as Into<u8>>::into(command.adwidth) > <enums::XspiWidth as Into<u8>>::into(self.width)
            || <enums::XspiWidth as Into<u8>>::into(command.abwidth) > <enums::XspiWidth as Into<u8>>::into(self.width)
            || <enums::XspiWidth as Into<u8>>::into(command.dwidth) > <enums::XspiWidth as Into<u8>>::into(self.width)
        {
            return Err(XspiError::InvalidCommand);
        }

        T::REGS.cr().modify(|w| {
            w.set_fmode(0.into());
        });

        // Configure alternate bytes
        if let Some(ab) = command.alternate_bytes {
            T::REGS.abr().write(|v| v.set_alternate(ab));
            T::REGS.ccr().modify(|w| {
                w.set_abmode(CcrAbmode::from_bits(command.abwidth.into()));
                w.set_abdtr(command.abdtr);
                w.set_absize(CcrAbsize::from_bits(command.absize.into()));
            })
        }

        // Configure dummy cycles
        T::REGS.tcr().modify(|w| {
            w.set_dcyc(command.dummy.into());
        });

        // Configure data
        if let Some(data_length) = data_len {
            T::REGS.dlr().write(|v| {
                v.set_dl((data_length - 1) as u32);
            })
        } else {
            T::REGS.dlr().write(|v| {
                v.set_dl((0) as u32);
            })
        }

        // Configure instruction/address/data modes
        T::REGS.ccr().modify(|w| {
            w.set_imode(CcrImode::from_bits(command.iwidth.into()));
            w.set_idtr(command.idtr);
            w.set_isize(CcrIsize::from_bits(command.isize.into()));

            w.set_admode(CcrAdmode::from_bits(command.adwidth.into()));
            w.set_addtr(command.addtr);
            w.set_adsize(CcrAdsize::from_bits(command.adsize.into()));

            w.set_dmode(CcrDmode::from_bits(command.dwidth.into()));
            w.set_ddtr(command.ddtr);
        });

        // Set information required to initiate transaction
        if let Some(instruction) = command.instruction {
            if let Some(address) = command.address {
                T::REGS.ir().write(|v| {
                    v.set_instruction(instruction);
                });

                T::REGS.ar().write(|v| {
                    v.set_address(address);
                });
            } else {
                // Double check requirements for delay hold and sample shifting
                // if let None = command.data_len {
                //     if self.config.delay_hold_quarter_cycle && command.idtr {
                //         T::REGS.ccr().modify(|w| {
                //             w.set_ddtr(true);
                //         });
                //     }
                // }

                // warn!("instruction: {:#x}", instruction);
                T::REGS.ir().write(|v| {
                    v.set_instruction(instruction);
                });
            }
        } else {
            if let Some(address) = command.address {
                T::REGS.ar().write(|v| {
                    v.set_address(address);
                });
            } else {
                // The only single phase transaction supported is instruction only
                return Err(XspiError::InvalidCommand);
            }
        }

        Ok(())
    }

    /// Function used to control or configure the target device without data transfer
    pub fn blocking_command(&mut self, command: &TransferConfig) -> Result<(), XspiError> {
        // Wait for peripheral to be free
        while T::REGS.sr().read().busy() {}

        // Need additional validation that command configuration doesn't have data set
        self.configure_command(command, None)?;

        // Transaction initiated by setting final configuration, i.e the instruction register
        while !T::REGS.sr().read().tcf() {}
        T::REGS.fcr().write(|w| {
            w.set_ctcf(true);
        });

        Ok(())
    }

    /// Blocking read with byte by byte data transfer
    pub fn blocking_read<W: Word>(&mut self, buf: &mut [W], transaction: TransferConfig) -> Result<(), XspiError> {
        if buf.is_empty() {
            return Err(XspiError::EmptyBuffer);
        }

        // Wait for peripheral to be free
        while T::REGS.sr().read().busy() {}

        // Ensure DMA is not enabled for this transaction
        T::REGS.cr().modify(|w| {
            w.set_dmaen(false);
        });

        // self.configure_command(&transaction, Some(buf.len()))?;
        self.configure_command(&transaction, Some(buf.len())).unwrap();

        let current_address = T::REGS.ar().read().address();
        let current_instruction = T::REGS.ir().read().instruction();

        // For a indirect read transaction, the transaction begins when the instruction/address is set
        T::REGS
            .cr()
            .modify(|v| v.set_fmode(Fmode::from_bits(XspiMode::IndirectRead.into())));
        if T::REGS.ccr().read().admode() == CcrAdmode::B_0X0 {
            T::REGS.ir().write(|v| v.set_instruction(current_instruction));
        } else {
            T::REGS.ar().write(|v| v.set_address(current_address));
        }

        for idx in 0..buf.len() {
            while !T::REGS.sr().read().tcf() && !T::REGS.sr().read().ftf() {}
            buf[idx] = unsafe { (T::REGS.dr().as_ptr() as *mut W).read_volatile() };
        }

        while !T::REGS.sr().read().tcf() {}
        T::REGS.fcr().write(|v| v.set_ctcf(true));

        Ok(())
    }

    /// Blocking write with byte by byte data transfer
    pub fn blocking_write<W: Word>(&mut self, buf: &[W], transaction: TransferConfig) -> Result<(), XspiError> {
        if buf.is_empty() {
            return Err(XspiError::EmptyBuffer);
        }

        // Wait for peripheral to be free
        while T::REGS.sr().read().busy() {}

        T::REGS.cr().modify(|w| {
            w.set_dmaen(false);
        });

        self.configure_command(&transaction, Some(buf.len()))?;

        T::REGS
            .cr()
            .modify(|v| v.set_fmode(Fmode::from_bits(XspiMode::IndirectWrite.into())));

        for idx in 0..buf.len() {
            while !T::REGS.sr().read().ftf() {}
            unsafe { (T::REGS.dr().as_ptr() as *mut W).write_volatile(buf[idx]) };
        }

        while !T::REGS.sr().read().tcf() {}
        T::REGS.fcr().write(|v| v.set_ctcf(true));

        Ok(())
    }

    /// Set new bus configuration
    pub fn set_config(&mut self, config: &Config) {
        // Wait for busy flag to clear
        while T::REGS.sr().read().busy() {}

        // Disable DMA channel while configuring the peripheral
        T::REGS.cr().modify(|w| {
            w.set_dmaen(false);
        });

        // Device configuration
        T::REGS.dcr1().modify(|w| {
            w.set_devsize(config.device_size.into());
            w.set_mtyp(Mtyp::from_bits(config.memory_type.into()));
            w.set_csht(Csht::from_bits(config.chip_select_high_time.into()));
            w.set_frck(false);
            w.set_ckmode(match config.clock_mode {
                true => Ckmode::B_0X1,
                false => Ckmode::B_0X0,
            });
        });

        T::REGS.dcr2().modify(|w| {
            w.set_wrapsize(Wrapsize::from_bits(config.wrap_size.into()));
        });

        T::REGS.dcr3().modify(|w| {
            w.set_csbound(Csbound::from_bits(config.chip_select_boundary.into()));
            #[cfg(xspi_v1)]
            {
                w.set_maxtran(Maxtran::from_bits(config.max_transfer.into()));
            }
        });

        T::REGS.dcr4().modify(|w| {
            w.set_refresh(Refresh::from_bits(config.refresh.into()));
        });

        T::REGS.cr().modify(|w| {
            w.set_fthres(Fthres::from_bits(config.fifo_threshold.into()));
        });

        // Wait for busy flag to clear
        while T::REGS.sr().read().busy() {}

        T::REGS.dcr2().modify(|w| {
            w.set_prescaler(config.clock_prescaler);
        });

        T::REGS.tcr().modify(|w| {
            w.set_sshift(config.sample_shifting);
            w.set_dhqc(config.delay_hold_quarter_cycle);
        });

        // Enable peripheral
        T::REGS.cr().modify(|w| {
            w.set_en(true);
        });

        // Free running clock needs to be set after peripheral enable
        if config.free_running_clock {
            T::REGS.dcr1().modify(|w| {
                w.set_frck(config.free_running_clock);
            });
        }

        self.config = *config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> Config {
        self.config
    }
}

impl<'d, T: Instance> Xspi<'d, T, Blocking> {
    /// Create new blocking XSPI driver for a single spi external chip
    pub fn new_blocking_singlespi(
        peri: Peri<'d, T>,
        clk: Peri<'d, impl CLKPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        ncs: Peri<'d, impl NCSEither<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d1, AfType::input(Pull::None)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            new_pin!(clk, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            ncs.sel(),
            new_pin!(
                ncs,
                AfType::output_pull(OutputType::PushPull, Speed::VeryHigh, Pull::Up)
            ),
            None,
            None,
            None,
            None,
            config,
            XspiWidth::SING,
            false,
        )
    }

    /// Create new blocking XSPI driver for a dualspi external chip
    pub fn new_blocking_dualspi(
        peri: Peri<'d, T>,
        clk: Peri<'d, impl CLKPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        ncs: Peri<'d, impl NCSEither<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d1, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            new_pin!(clk, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            ncs.sel(),
            new_pin!(
                ncs,
                AfType::output_pull(OutputType::PushPull, Speed::VeryHigh, Pull::Up)
            ),
            None,
            None,
            None,
            None,
            config,
            XspiWidth::DUAL,
            false,
        )
    }

    /// Create new blocking XSPI driver for a quadspi external chip
    pub fn new_blocking_quadspi(
        peri: Peri<'d, T>,
        clk: Peri<'d, impl CLKPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        ncs: Peri<'d, impl NCSEither<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d1, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d2, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d3, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            new_pin!(clk, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            ncs.sel(),
            new_pin!(
                ncs,
                AfType::output_pull(OutputType::PushPull, Speed::VeryHigh, Pull::Up)
            ),
            None,
            None,
            None,
            None,
            config,
            XspiWidth::QUAD,
            false,
        )
    }

    /// Create new blocking XSPI driver for two quadspi external chips
    pub fn new_blocking_dualquadspi(
        peri: Peri<'d, T>,
        clk: Peri<'d, impl CLKPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        ncs: Peri<'d, impl NCSEither<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d1, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d2, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d3, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d4, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d5, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d6, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d7, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            new_pin!(clk, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            ncs.sel(),
            new_pin!(
                ncs,
                AfType::output_pull(OutputType::PushPull, Speed::VeryHigh, Pull::Up)
            ),
            None,
            None,
            None,
            None,
            config,
            XspiWidth::QUAD,
            true,
        )
    }

    /// Create new blocking XSPI driver for xspi external chips
    pub fn new_blocking_xspi(
        peri: Peri<'d, T>,
        clk: Peri<'d, impl CLKPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        ncs: Peri<'d, impl NCSEither<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d1, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d2, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d3, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d4, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d5, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d6, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d7, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            new_pin!(clk, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            ncs.sel(),
            new_pin!(
                ncs,
                AfType::output_pull(OutputType::PushPull, Speed::VeryHigh, Pull::Up)
            ),
            None,
            None,
            None,
            None,
            config,
            XspiWidth::OCTO,
            false,
        )
    }
}

impl<'d, T: Instance> Xspi<'d, T, Async> {
    /// Create new blocking XSPI driver for a single spi external chip
    pub fn new_singlespi(
        peri: Peri<'d, T>,
        clk: Peri<'d, impl CLKPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        ncs: Peri<'d, impl NCSEither<T>>,
        dma: Peri<'d, impl XDma<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d1, AfType::input(Pull::None)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            new_pin!(clk, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            ncs.sel(),
            new_pin!(
                ncs,
                AfType::output_pull(OutputType::PushPull, Speed::VeryHigh, Pull::Up)
            ),
            None,
            None,
            None,
            new_dma!(dma),
            config,
            XspiWidth::SING,
            false,
        )
    }

    /// Create new blocking XSPI driver for a dualspi external chip
    pub fn new_dualspi(
        peri: Peri<'d, T>,
        clk: Peri<'d, impl CLKPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        ncs: Peri<'d, impl NCSEither<T>>,
        dma: Peri<'d, impl XDma<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d1, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            new_pin!(clk, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            ncs.sel(),
            new_pin!(
                ncs,
                AfType::output_pull(OutputType::PushPull, Speed::VeryHigh, Pull::Up)
            ),
            None,
            None,
            None,
            new_dma!(dma),
            config,
            XspiWidth::DUAL,
            false,
        )
    }

    /// Create new blocking XSPI driver for a quadspi external chip
    pub fn new_quadspi(
        peri: Peri<'d, T>,
        clk: Peri<'d, impl CLKPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        ncs: Peri<'d, impl NCSEither<T>>,
        dma: Peri<'d, impl XDma<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d1, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d2, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d3, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            new_pin!(clk, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            ncs.sel(),
            new_pin!(
                ncs,
                AfType::output_pull(OutputType::PushPull, Speed::VeryHigh, Pull::Up)
            ),
            None,
            None,
            None,
            new_dma!(dma),
            config,
            XspiWidth::QUAD,
            false,
        )
    }

    /// Create new blocking XSPI driver for two quadspi external chips
    pub fn new_dualquadspi(
        peri: Peri<'d, T>,
        clk: Peri<'d, impl CLKPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        ncs: Peri<'d, impl NCSEither<T>>,
        dma: Peri<'d, impl XDma<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d1, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d2, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d3, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d4, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d5, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d6, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d7, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            new_pin!(clk, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            ncs.sel(),
            new_pin!(
                ncs,
                AfType::output_pull(OutputType::PushPull, Speed::VeryHigh, Pull::Up)
            ),
            None,
            None,
            None,
            new_dma!(dma),
            config,
            XspiWidth::QUAD,
            true,
        )
    }

    /// Create new blocking XSPI driver for xspi external chips
    pub fn new_xspi(
        peri: Peri<'d, T>,
        clk: Peri<'d, impl CLKPin<T>>,
        d0: Peri<'d, impl D0Pin<T>>,
        d1: Peri<'d, impl D1Pin<T>>,
        d2: Peri<'d, impl D2Pin<T>>,
        d3: Peri<'d, impl D3Pin<T>>,
        d4: Peri<'d, impl D4Pin<T>>,
        d5: Peri<'d, impl D5Pin<T>>,
        d6: Peri<'d, impl D6Pin<T>>,
        d7: Peri<'d, impl D7Pin<T>>,
        ncs: Peri<'d, impl NCSEither<T>>,
        dma: Peri<'d, impl XDma<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d1, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d2, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d3, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d4, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d5, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d6, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            new_pin!(d7, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            new_pin!(clk, AfType::output(OutputType::PushPull, Speed::VeryHigh)),
            ncs.sel(),
            new_pin!(
                ncs,
                AfType::output_pull(OutputType::PushPull, Speed::VeryHigh, Pull::Up)
            ),
            None,
            None,
            None,
            new_dma!(dma),
            config,
            XspiWidth::OCTO,
            false,
        )
    }

    /// Blocking read with DMA transfer
    pub fn blocking_read_dma<W: Word>(&mut self, buf: &mut [W], transaction: TransferConfig) -> Result<(), XspiError> {
        if buf.is_empty() {
            return Err(XspiError::EmptyBuffer);
        }

        // Wait for peripheral to be free
        while T::REGS.sr().read().busy() {}

        self.configure_command(&transaction, Some(buf.len()))?;

        let current_address = T::REGS.ar().read().address();
        let current_instruction = T::REGS.ir().read().instruction();

        // For a indirect read transaction, the transaction begins when the instruction/address is set
        T::REGS
            .cr()
            .modify(|v| v.set_fmode(Fmode::from_bits(XspiMode::IndirectRead.into())));
        if T::REGS.ccr().read().admode() == CcrAdmode::B_0X0 {
            T::REGS.ir().write(|v| v.set_instruction(current_instruction));
        } else {
            T::REGS.ar().write(|v| v.set_address(current_address));
        }

        let transfer = unsafe {
            self.dma
                .as_mut()
                .unwrap()
                .read(T::REGS.dr().as_ptr() as *mut W, buf, Default::default())
        };

        T::REGS.cr().modify(|w| w.set_dmaen(true));

        transfer.blocking_wait();

        finish_dma(T::REGS);

        Ok(())
    }

    /// Blocking write with DMA transfer
    pub fn blocking_write_dma<W: Word>(&mut self, buf: &[W], transaction: TransferConfig) -> Result<(), XspiError> {
        if buf.is_empty() {
            return Err(XspiError::EmptyBuffer);
        }

        // Wait for peripheral to be free
        while T::REGS.sr().read().busy() {}

        self.configure_command(&transaction, Some(buf.len()))?;
        T::REGS
            .cr()
            .modify(|v| v.set_fmode(Fmode::from_bits(XspiMode::IndirectWrite.into())));

        let transfer = unsafe {
            self.dma
                .as_mut()
                .unwrap()
                .write(buf, T::REGS.dr().as_ptr() as *mut W, Default::default())
        };

        T::REGS.cr().modify(|w| w.set_dmaen(true));

        transfer.blocking_wait();

        finish_dma(T::REGS);

        Ok(())
    }

    /// Asynchronous read from external device
    pub async fn read<W: Word>(&mut self, buf: &mut [W], transaction: TransferConfig) -> Result<(), XspiError> {
        if buf.is_empty() {
            return Err(XspiError::EmptyBuffer);
        }

        // Wait for peripheral to be free
        while T::REGS.sr().read().busy() {}

        self.configure_command(&transaction, Some(buf.len()))?;

        let current_address = T::REGS.ar().read().address();
        let current_instruction = T::REGS.ir().read().instruction();

        // For a indirect read transaction, the transaction begins when the instruction/address is set
        T::REGS
            .cr()
            .modify(|v| v.set_fmode(Fmode::from_bits(XspiMode::IndirectRead.into())));
        if T::REGS.ccr().read().admode() == CcrAdmode::B_0X0 {
            T::REGS.ir().write(|v| v.set_instruction(current_instruction));
        } else {
            T::REGS.ar().write(|v| v.set_address(current_address));
        }

        let transfer = unsafe {
            self.dma
                .as_mut()
                .unwrap()
                .read(T::REGS.dr().as_ptr() as *mut W, buf, Default::default())
        };

        T::REGS.cr().modify(|w| w.set_dmaen(true));

        transfer.await;

        finish_dma(T::REGS);

        Ok(())
    }

    /// Asynchronous write to external device
    pub async fn write<W: Word>(&mut self, buf: &[W], transaction: TransferConfig) -> Result<(), XspiError> {
        if buf.is_empty() {
            return Err(XspiError::EmptyBuffer);
        }

        // Wait for peripheral to be free
        while T::REGS.sr().read().busy() {}

        self.configure_command(&transaction, Some(buf.len()))?;
        T::REGS
            .cr()
            .modify(|v| v.set_fmode(Fmode::from_bits(XspiMode::IndirectWrite.into())));

        let transfer = unsafe {
            self.dma
                .as_mut()
                .unwrap()
                .write(buf, T::REGS.dr().as_ptr() as *mut W, Default::default())
        };

        T::REGS.cr().modify(|w| w.set_dmaen(true));

        transfer.await;

        finish_dma(T::REGS);

        Ok(())
    }
}

impl<'d, T: Instance, M: PeriMode> Drop for Xspi<'d, T, M> {
    fn drop(&mut self) {
        self.clk.as_ref().map(|x| x.set_as_disconnected());
        self.d0.as_ref().map(|x| x.set_as_disconnected());
        self.d1.as_ref().map(|x| x.set_as_disconnected());
        self.d2.as_ref().map(|x| x.set_as_disconnected());
        self.d3.as_ref().map(|x| x.set_as_disconnected());
        self.d4.as_ref().map(|x| x.set_as_disconnected());
        self.d5.as_ref().map(|x| x.set_as_disconnected());
        self.d6.as_ref().map(|x| x.set_as_disconnected());
        self.d7.as_ref().map(|x| x.set_as_disconnected());
        self.d8.as_ref().map(|x| x.set_as_disconnected());
        self.d9.as_ref().map(|x| x.set_as_disconnected());
        self.d10.as_ref().map(|x| x.set_as_disconnected());
        self.d11.as_ref().map(|x| x.set_as_disconnected());
        self.d12.as_ref().map(|x| x.set_as_disconnected());
        self.d13.as_ref().map(|x| x.set_as_disconnected());
        self.d14.as_ref().map(|x| x.set_as_disconnected());
        self.d15.as_ref().map(|x| x.set_as_disconnected());
        self.ncs.as_ref().map(|x| x.set_as_disconnected());
        self.ncs_alt.as_ref().map(|x| x.set_as_disconnected());
        self.dqs0.as_ref().map(|x| x.set_as_disconnected());
        self.dqs1.as_ref().map(|x| x.set_as_disconnected());

        rcc::disable::<T>();
    }
}

fn finish_dma(regs: Regs) {
    while !regs.sr().read().tcf() {}
    regs.fcr().write(|v| v.set_ctcf(true));

    regs.cr().modify(|w| {
        w.set_dmaen(false);
    });
}

/// XSPI I/O manager instance trait.
#[cfg(xspim_v1)]
pub(crate) trait SealedXspimInstance {
    const SPIM_REGS: Xspim;
    const SPI_IDX: u8;
}

/// XSPI instance trait.
pub(crate) trait SealedInstance {
    const REGS: Regs;
}

/// XSPI instance trait.
#[cfg(xspim_v1)]
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral + SealedXspimInstance {}

/// XSPI instance trait.
#[cfg(not(xspim_v1))]
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral {}

pin_trait!(D0Pin, Instance);
pin_trait!(D1Pin, Instance);
pin_trait!(D2Pin, Instance);
pin_trait!(D3Pin, Instance);
pin_trait!(D4Pin, Instance);
pin_trait!(D5Pin, Instance);
pin_trait!(D6Pin, Instance);
pin_trait!(D7Pin, Instance);
pin_trait!(D8Pin, Instance);
pin_trait!(D9Pin, Instance);
pin_trait!(D10Pin, Instance);
pin_trait!(D11Pin, Instance);
pin_trait!(D12Pin, Instance);
pin_trait!(D13Pin, Instance);
pin_trait!(D14Pin, Instance);
pin_trait!(D15Pin, Instance);
pin_trait!(DQS0Pin, Instance);
pin_trait!(DQS1Pin, Instance);
pin_trait!(NCSPin, Instance);
pin_trait!(CLKPin, Instance);
pin_trait!(NCLKPin, Instance);
dma_trait!(XDma, Instance);

/// Trait for either NCS1 or NCS2 pins
pub trait NCSEither<T: Instance>: NCSPin<T> {
    /// Get the CSSEL for this NCS pin
    fn sel(&self) -> u8;
}

// Hard-coded the xspi index, for SPIM
#[cfg(xspim_v1)]
impl SealedXspimInstance for peripherals::XSPI1 {
    const SPIM_REGS: Xspim = crate::pac::XSPIM;
    const SPI_IDX: u8 = 1;
}

// Some cubedb files are missing XSPI2, for example STM32H7R3Z8
#[cfg(all(xspim_v1, peri_xspi2))]
impl SealedXspimInstance for peripherals::XSPI2 {
    const SPIM_REGS: Xspim = crate::pac::XSPIM;
    const SPI_IDX: u8 = 2;
}

#[cfg(xspim_v1)]
foreach_peripheral!(
    (xspi, $inst:ident) => {
        impl SealedInstance for peripherals::$inst {
            const REGS: Regs = crate::pac::$inst;
        }

        impl Instance for peripherals::$inst {}
    };
);

#[cfg(not(xspim_v1))]
foreach_peripheral!(
    (xspi, $inst:ident) => {
        impl SealedInstance for peripherals::$inst {
            const REGS: Regs = crate::pac::$inst;
        }

        impl Instance for peripherals::$inst {}
    };
);

impl<'d, T: Instance, M: PeriMode> SetConfig for Xspi<'d, T, M> {
    type Config = Config;
    type ConfigError = ();
    fn set_config(&mut self, config: &Self::Config) -> Result<(), ()> {
        self.set_config(config);
        Ok(())
    }
}

impl<'d, T: Instance, M: PeriMode> GetConfig for Xspi<'d, T, M> {
    type Config = Config;
    fn get_config(&self) -> Self::Config {
        self.get_config()
    }
}

/// Word sizes usable for XSPI.
#[allow(private_bounds)]
pub trait Word: word::Word {}

macro_rules! impl_word {
    ($T:ty) => {
        impl Word for $T {}
    };
}

impl_word!(u8);
impl_word!(u16);
impl_word!(u32);
