//! OCTOSPI Serial Peripheral Interface
//!

#![macro_use]

pub mod enums;

use core::marker::PhantomData;

use embassy_embedded_hal::{GetConfig, SetConfig};
use embassy_hal_internal::{into_ref, PeripheralRef};
pub use enums::*;
use stm32_metapac::octospi::vals::{PhaseMode, SizeInBits};

use crate::dma::{word, ChannelAndRequest};
use crate::gpio::{AFType, AnyPin, Pull, SealedPin as _, Speed};
use crate::mode::{Async, Blocking, Mode as PeriMode};
use crate::pac::octospi::{vals, Octospi as Regs};
use crate::rcc::{self, RccPeripheral};
use crate::{peripherals, Peripheral};

/// OPSI driver config.
#[derive(Clone, Copy)]
pub struct Config {
    /// Fifo threshold used by the peripheral to generate the interrupt indicating data
    /// or space is available in the FIFO
    pub fifo_threshold: FIFOThresholdLevel,
    /// Indicates the type of external device connected
    pub memory_type: MemoryType, // Need to add an additional enum to provide this public interface
    /// Defines the size of the external device connected to the OSPI corresponding
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
    /// on the AHB clock
    pub clock_prescaler: u8,
    /// Allows the delay of 1/2 cycle the data sampling to account for external
    /// signal delays
    pub sample_shifting: bool,
    /// Allows hold to 1/4 cycle the data
    pub delay_hold_quarter_cycle: bool,
    /// Enables the transaction boundary feature and defines the boundary to release
    /// the chip select
    pub chip_select_boundary: u8,
    /// Enbales the delay block bypass so the sampling is not affected by the delay block
    pub delay_block_bypass: bool,
    /// Enables communication regulation feature. Chip select is released when the other
    /// OctoSpi requests access to the bus
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
            delay_block_bypass: true,
            max_transfer: 0,
            refresh: 0,
        }
    }
}

/// OSPI transfer configuration.
pub struct TransferConfig {
    /// Instruction width (IMODE)
    pub iwidth: OspiWidth,
    /// Instruction Id
    pub instruction: Option<u32>,
    /// Number of Instruction Bytes
    pub isize: AddressSize,
    /// Instruction Double Transfer rate enable
    pub idtr: bool,

    /// Address width (ADMODE)
    pub adwidth: OspiWidth,
    /// Device memory address
    pub address: Option<u32>,
    /// Number of Address Bytes
    pub adsize: AddressSize,
    /// Address Double Transfer rate enable
    pub addtr: bool,

    /// Alternate bytes width (ABMODE)
    pub abwidth: OspiWidth,
    /// Alternate Bytes
    pub alternate_bytes: Option<u32>,
    /// Number of Alternate Bytes
    pub absize: AddressSize,
    /// Alternate Bytes Double Transfer rate enable
    pub abdtr: bool,

    /// Data width (DMODE)
    pub dwidth: OspiWidth,
    /// Data buffer
    pub ddtr: bool,

    /// Number of dummy cycles (DCYC)
    pub dummy: DummyCycles,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            iwidth: OspiWidth::NONE,
            instruction: None,
            isize: AddressSize::_8Bit,
            idtr: false,

            adwidth: OspiWidth::NONE,
            address: None,
            adsize: AddressSize::_8Bit,
            addtr: false,

            abwidth: OspiWidth::NONE,
            alternate_bytes: None,
            absize: AddressSize::_8Bit,
            abdtr: false,

            dwidth: OspiWidth::NONE,
            ddtr: false,

            dummy: DummyCycles::_0,
        }
    }
}

/// Error used for Octospi implementation
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OspiError {
    /// Peripheral configuration is invalid
    InvalidConfiguration,
    /// Operation configuration is invalid
    InvalidCommand,
    /// Size zero buffer passed to instruction
    EmptyBuffer,
}

/// OSPI driver.
pub struct Ospi<'d, T: Instance, M: PeriMode> {
    _peri: PeripheralRef<'d, T>,
    sck: Option<PeripheralRef<'d, AnyPin>>,
    d0: Option<PeripheralRef<'d, AnyPin>>,
    d1: Option<PeripheralRef<'d, AnyPin>>,
    d2: Option<PeripheralRef<'d, AnyPin>>,
    d3: Option<PeripheralRef<'d, AnyPin>>,
    d4: Option<PeripheralRef<'d, AnyPin>>,
    d5: Option<PeripheralRef<'d, AnyPin>>,
    d6: Option<PeripheralRef<'d, AnyPin>>,
    d7: Option<PeripheralRef<'d, AnyPin>>,
    nss: Option<PeripheralRef<'d, AnyPin>>,
    dqs: Option<PeripheralRef<'d, AnyPin>>,
    dma: Option<ChannelAndRequest<'d>>,
    _phantom: PhantomData<M>,
    config: Config,
    width: OspiWidth,
}

impl<'d, T: Instance, M: PeriMode> Ospi<'d, T, M> {
    fn new_inner(
        peri: impl Peripheral<P = T> + 'd,
        d0: Option<PeripheralRef<'d, AnyPin>>,
        d1: Option<PeripheralRef<'d, AnyPin>>,
        d2: Option<PeripheralRef<'d, AnyPin>>,
        d3: Option<PeripheralRef<'d, AnyPin>>,
        d4: Option<PeripheralRef<'d, AnyPin>>,
        d5: Option<PeripheralRef<'d, AnyPin>>,
        d6: Option<PeripheralRef<'d, AnyPin>>,
        d7: Option<PeripheralRef<'d, AnyPin>>,
        sck: Option<PeripheralRef<'d, AnyPin>>,
        nss: Option<PeripheralRef<'d, AnyPin>>,
        dqs: Option<PeripheralRef<'d, AnyPin>>,
        dma: Option<ChannelAndRequest<'d>>,
        config: Config,
        width: OspiWidth,
        dual_quad: bool,
    ) -> Self {
        into_ref!(peri);

        // System configuration
        rcc::enable_and_reset::<T>();
        while T::REGS.sr().read().busy() {}

        // Device configuration
        T::REGS.dcr1().modify(|w| {
            w.set_devsize(config.device_size.into());
            w.set_mtyp(vals::MemType::from_bits(config.memory_type.into()));
            w.set_csht(config.chip_select_high_time.into());
            w.set_dlybyp(config.delay_block_bypass);
            w.set_frck(false);
            w.set_ckmode(config.clock_mode);
        });

        T::REGS.dcr2().modify(|w| {
            w.set_wrapsize(config.wrap_size.into());
        });

        T::REGS.dcr3().modify(|w| {
            w.set_csbound(config.chip_select_boundary);
            #[cfg(octospi_v1)]
            {
                w.set_maxtran(config.max_transfer);
            }
        });

        T::REGS.dcr4().modify(|w| {
            w.set_refresh(config.refresh);
        });

        T::REGS.cr().modify(|w| {
            w.set_fthres(vals::Threshold(config.fifo_threshold.into()));
        });

        // Wait for busy flag to clear
        while T::REGS.sr().read().busy() {}

        T::REGS.dcr2().modify(|w| {
            w.set_prescaler(config.clock_prescaler);
        });

        T::REGS.cr().modify(|w| {
            w.set_dmm(dual_quad);
        });

        T::REGS.tcr().modify(|w| {
            w.set_sshift(match config.sample_shifting {
                true => vals::SampleShift::HALFCYCLE,
                false => vals::SampleShift::NONE,
            });
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
            sck,
            d0,
            d1,
            d2,
            d3,
            d4,
            d5,
            d6,
            d7,
            nss,
            dqs,
            dma,
            _phantom: PhantomData,
            config,
            width,
        }
    }

    // Function to configure the peripheral for the requested command
    fn configure_command(&mut self, command: &TransferConfig, data_len: Option<usize>) -> Result<(), OspiError> {
        // Check that transaction doesn't use more than hardware initialized pins
        if <enums::OspiWidth as Into<u8>>::into(command.iwidth) > <enums::OspiWidth as Into<u8>>::into(self.width)
            || <enums::OspiWidth as Into<u8>>::into(command.adwidth) > <enums::OspiWidth as Into<u8>>::into(self.width)
            || <enums::OspiWidth as Into<u8>>::into(command.abwidth) > <enums::OspiWidth as Into<u8>>::into(self.width)
            || <enums::OspiWidth as Into<u8>>::into(command.dwidth) > <enums::OspiWidth as Into<u8>>::into(self.width)
        {
            return Err(OspiError::InvalidCommand);
        }

        T::REGS.cr().modify(|w| {
            w.set_fmode(0.into());
        });

        // Configure alternate bytes
        if let Some(ab) = command.alternate_bytes {
            T::REGS.abr().write(|v| v.set_alternate(ab));
            T::REGS.ccr().modify(|w| {
                w.set_abmode(PhaseMode::from_bits(command.abwidth.into()));
                w.set_abdtr(command.abdtr);
                w.set_absize(SizeInBits::from_bits(command.absize.into()));
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
            w.set_imode(PhaseMode::from_bits(command.iwidth.into()));
            w.set_idtr(command.idtr);
            w.set_isize(SizeInBits::from_bits(command.isize.into()));

            w.set_admode(PhaseMode::from_bits(command.adwidth.into()));
            w.set_addtr(command.idtr);
            w.set_adsize(SizeInBits::from_bits(command.adsize.into()));

            w.set_dmode(PhaseMode::from_bits(command.dwidth.into()));
            w.set_ddtr(command.ddtr);
        });

        // Set informationrequired to initiate transaction
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
                return Err(OspiError::InvalidCommand);
            }
        }

        Ok(())
    }

    /// Function used to control or configure the target device without data transfer
    pub async fn command(&mut self, command: &TransferConfig) -> Result<(), OspiError> {
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
    pub fn blocking_read<W: Word>(&mut self, buf: &mut [W], transaction: TransferConfig) -> Result<(), OspiError> {
        if buf.is_empty() {
            return Err(OspiError::EmptyBuffer);
        }

        // Wait for peripheral to be free
        while T::REGS.sr().read().busy() {}

        // Ensure DMA is not enabled for this transaction
        T::REGS.cr().modify(|w| {
            w.set_dmaen(false);
        });

        self.configure_command(&transaction, Some(buf.len()))?;

        let current_address = T::REGS.ar().read().address();
        let current_instruction = T::REGS.ir().read().instruction();

        // For a indirect read transaction, the transaction begins when the instruction/address is set
        T::REGS.cr().modify(|v| v.set_fmode(vals::FunctionalMode::INDIRECTREAD));
        if T::REGS.ccr().read().admode() == vals::PhaseMode::NONE {
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
    pub fn blocking_write<W: Word>(&mut self, buf: &[W], transaction: TransferConfig) -> Result<(), OspiError> {
        if buf.is_empty() {
            return Err(OspiError::EmptyBuffer);
        }

        // Wait for peripheral to be free
        while T::REGS.sr().read().busy() {}

        T::REGS.cr().modify(|w| {
            w.set_dmaen(false);
        });

        self.configure_command(&transaction, Some(buf.len()))?;

        T::REGS
            .cr()
            .modify(|v| v.set_fmode(vals::FunctionalMode::INDIRECTWRITE));

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
            w.set_mtyp(vals::MemType::from_bits(config.memory_type.into()));
            w.set_csht(config.chip_select_high_time.into());
            w.set_dlybyp(config.delay_block_bypass);
            w.set_frck(false);
            w.set_ckmode(config.clock_mode);
        });

        T::REGS.dcr2().modify(|w| {
            w.set_wrapsize(config.wrap_size.into());
        });

        T::REGS.dcr3().modify(|w| {
            w.set_csbound(config.chip_select_boundary);
            #[cfg(octospi_v1)]
            {
                w.set_maxtran(config.max_transfer);
            }
        });

        T::REGS.dcr4().modify(|w| {
            w.set_refresh(config.refresh);
        });

        T::REGS.cr().modify(|w| {
            w.set_fthres(vals::Threshold(config.fifo_threshold.into()));
        });

        // Wait for busy flag to clear
        while T::REGS.sr().read().busy() {}

        T::REGS.dcr2().modify(|w| {
            w.set_prescaler(config.clock_prescaler);
        });

        T::REGS.tcr().modify(|w| {
            w.set_sshift(match config.sample_shifting {
                true => vals::SampleShift::HALFCYCLE,
                false => vals::SampleShift::NONE,
            });
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

impl<'d, T: Instance> Ospi<'d, T, Blocking> {
    /// Create new blocking OSPI driver for a single spi external chip
    pub fn new_blocking_singlespi(
        peri: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        nss: impl Peripheral<P = impl NSSPin<T>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d1, AFType::Input, Speed::VeryHigh),
            None,
            None,
            None,
            None,
            None,
            None,
            new_pin!(sck, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(nss, AFType::OutputPushPull, Speed::VeryHigh, Pull::Up),
            None,
            None,
            config,
            OspiWidth::SING,
            false,
        )
    }

    /// Create new blocking OSPI driver for a dualspi external chip
    pub fn new_blocking_dualspi(
        peri: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        nss: impl Peripheral<P = impl NSSPin<T>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d1, AFType::OutputPushPull, Speed::VeryHigh),
            None,
            None,
            None,
            None,
            None,
            None,
            new_pin!(sck, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(nss, AFType::OutputPushPull, Speed::VeryHigh, Pull::Up),
            None,
            None,
            config,
            OspiWidth::DUAL,
            false,
        )
    }

    /// Create new blocking OSPI driver for a quadspi external chip
    pub fn new_blocking_quadspi(
        peri: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        nss: impl Peripheral<P = impl NSSPin<T>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d1, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d2, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d3, AFType::OutputPushPull, Speed::VeryHigh),
            None,
            None,
            None,
            None,
            new_pin!(sck, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(nss, AFType::OutputPushPull, Speed::VeryHigh, Pull::Up),
            None,
            None,
            config,
            OspiWidth::QUAD,
            false,
        )
    }

    /// Create new blocking OSPI driver for two quadspi external chips
    pub fn new_blocking_dualquadspi(
        peri: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        d4: impl Peripheral<P = impl D4Pin<T>> + 'd,
        d5: impl Peripheral<P = impl D5Pin<T>> + 'd,
        d6: impl Peripheral<P = impl D6Pin<T>> + 'd,
        d7: impl Peripheral<P = impl D7Pin<T>> + 'd,
        nss: impl Peripheral<P = impl NSSPin<T>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d1, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d2, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d3, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d4, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d5, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d6, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d7, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(sck, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(nss, AFType::OutputPushPull, Speed::VeryHigh, Pull::Up),
            None,
            None,
            config,
            OspiWidth::QUAD,
            true,
        )
    }

    /// Create new blocking OSPI driver for octospi external chips
    pub fn new_blocking_octospi(
        peri: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        d4: impl Peripheral<P = impl D4Pin<T>> + 'd,
        d5: impl Peripheral<P = impl D5Pin<T>> + 'd,
        d6: impl Peripheral<P = impl D6Pin<T>> + 'd,
        d7: impl Peripheral<P = impl D7Pin<T>> + 'd,
        nss: impl Peripheral<P = impl NSSPin<T>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d1, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d2, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d3, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d4, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d5, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d6, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d7, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(sck, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(nss, AFType::OutputPushPull, Speed::VeryHigh, Pull::Up),
            None,
            None,
            config,
            OspiWidth::OCTO,
            false,
        )
    }
}

impl<'d, T: Instance> Ospi<'d, T, Async> {
    /// Create new blocking OSPI driver for a single spi external chip
    pub fn new_singlespi(
        peri: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        nss: impl Peripheral<P = impl NSSPin<T>> + 'd,
        dma: impl Peripheral<P = impl OctoDma<T>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d1, AFType::Input, Speed::VeryHigh),
            None,
            None,
            None,
            None,
            None,
            None,
            new_pin!(sck, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(nss, AFType::OutputPushPull, Speed::VeryHigh, Pull::Up),
            None,
            new_dma!(dma),
            config,
            OspiWidth::SING,
            false,
        )
    }

    /// Create new blocking OSPI driver for a dualspi external chip
    pub fn new_dualspi(
        peri: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        nss: impl Peripheral<P = impl NSSPin<T>> + 'd,
        dma: impl Peripheral<P = impl OctoDma<T>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d1, AFType::OutputPushPull, Speed::VeryHigh),
            None,
            None,
            None,
            None,
            None,
            None,
            new_pin!(sck, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(nss, AFType::OutputPushPull, Speed::VeryHigh, Pull::Up),
            None,
            new_dma!(dma),
            config,
            OspiWidth::DUAL,
            false,
        )
    }

    /// Create new blocking OSPI driver for a quadspi external chip
    pub fn new_quadspi(
        peri: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        nss: impl Peripheral<P = impl NSSPin<T>> + 'd,
        dma: impl Peripheral<P = impl OctoDma<T>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d1, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d2, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d3, AFType::OutputPushPull, Speed::VeryHigh),
            None,
            None,
            None,
            None,
            new_pin!(sck, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(nss, AFType::OutputPushPull, Speed::VeryHigh, Pull::Up),
            None,
            new_dma!(dma),
            config,
            OspiWidth::QUAD,
            false,
        )
    }

    /// Create new blocking OSPI driver for two quadspi external chips
    pub fn new_dualquadspi(
        peri: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        d4: impl Peripheral<P = impl D4Pin<T>> + 'd,
        d5: impl Peripheral<P = impl D5Pin<T>> + 'd,
        d6: impl Peripheral<P = impl D6Pin<T>> + 'd,
        d7: impl Peripheral<P = impl D7Pin<T>> + 'd,
        nss: impl Peripheral<P = impl NSSPin<T>> + 'd,
        dma: impl Peripheral<P = impl OctoDma<T>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d1, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d2, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d3, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d4, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d5, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d6, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d7, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(sck, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(nss, AFType::OutputPushPull, Speed::VeryHigh, Pull::Up),
            None,
            new_dma!(dma),
            config,
            OspiWidth::QUAD,
            true,
        )
    }

    /// Create new blocking OSPI driver for octospi external chips
    pub fn new_octospi(
        peri: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        d0: impl Peripheral<P = impl D0Pin<T>> + 'd,
        d1: impl Peripheral<P = impl D1Pin<T>> + 'd,
        d2: impl Peripheral<P = impl D2Pin<T>> + 'd,
        d3: impl Peripheral<P = impl D3Pin<T>> + 'd,
        d4: impl Peripheral<P = impl D4Pin<T>> + 'd,
        d5: impl Peripheral<P = impl D5Pin<T>> + 'd,
        d6: impl Peripheral<P = impl D6Pin<T>> + 'd,
        d7: impl Peripheral<P = impl D7Pin<T>> + 'd,
        nss: impl Peripheral<P = impl NSSPin<T>> + 'd,
        dma: impl Peripheral<P = impl OctoDma<T>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(d0, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d1, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d2, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d3, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d4, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d5, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d6, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(d7, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(sck, AFType::OutputPushPull, Speed::VeryHigh),
            new_pin!(nss, AFType::OutputPushPull, Speed::VeryHigh, Pull::Up),
            None,
            new_dma!(dma),
            config,
            OspiWidth::OCTO,
            false,
        )
    }

    /// Blocking read with DMA transfer
    pub fn blocking_read_dma<W: Word>(&mut self, buf: &mut [W], transaction: TransferConfig) -> Result<(), OspiError> {
        if buf.is_empty() {
            return Err(OspiError::EmptyBuffer);
        }

        // Wait for peripheral to be free
        while T::REGS.sr().read().busy() {}

        self.configure_command(&transaction, Some(buf.len()))?;

        let current_address = T::REGS.ar().read().address();
        let current_instruction = T::REGS.ir().read().instruction();

        // For a indirect read transaction, the transaction begins when the instruction/address is set
        T::REGS.cr().modify(|v| v.set_fmode(vals::FunctionalMode::INDIRECTREAD));
        if T::REGS.ccr().read().admode() == vals::PhaseMode::NONE {
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
    pub fn blocking_write_dma<W: Word>(&mut self, buf: &[W], transaction: TransferConfig) -> Result<(), OspiError> {
        if buf.is_empty() {
            return Err(OspiError::EmptyBuffer);
        }

        // Wait for peripheral to be free
        while T::REGS.sr().read().busy() {}

        self.configure_command(&transaction, Some(buf.len()))?;
        T::REGS
            .cr()
            .modify(|v| v.set_fmode(vals::FunctionalMode::INDIRECTWRITE));

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
    pub async fn read<W: Word>(&mut self, buf: &mut [W], transaction: TransferConfig) -> Result<(), OspiError> {
        if buf.is_empty() {
            return Err(OspiError::EmptyBuffer);
        }

        // Wait for peripheral to be free
        while T::REGS.sr().read().busy() {}

        self.configure_command(&transaction, Some(buf.len()))?;

        let current_address = T::REGS.ar().read().address();
        let current_instruction = T::REGS.ir().read().instruction();

        // For a indirect read transaction, the transaction begins when the instruction/address is set
        T::REGS.cr().modify(|v| v.set_fmode(vals::FunctionalMode::INDIRECTREAD));
        if T::REGS.ccr().read().admode() == vals::PhaseMode::NONE {
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
    pub async fn write<W: Word>(&mut self, buf: &[W], transaction: TransferConfig) -> Result<(), OspiError> {
        if buf.is_empty() {
            return Err(OspiError::EmptyBuffer);
        }

        // Wait for peripheral to be free
        while T::REGS.sr().read().busy() {}

        self.configure_command(&transaction, Some(buf.len()))?;
        T::REGS
            .cr()
            .modify(|v| v.set_fmode(vals::FunctionalMode::INDIRECTWRITE));

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

impl<'d, T: Instance, M: PeriMode> Drop for Ospi<'d, T, M> {
    fn drop(&mut self) {
        self.sck.as_ref().map(|x| x.set_as_disconnected());
        self.d0.as_ref().map(|x| x.set_as_disconnected());
        self.d1.as_ref().map(|x| x.set_as_disconnected());
        self.d2.as_ref().map(|x| x.set_as_disconnected());
        self.d3.as_ref().map(|x| x.set_as_disconnected());
        self.d4.as_ref().map(|x| x.set_as_disconnected());
        self.d5.as_ref().map(|x| x.set_as_disconnected());
        self.d6.as_ref().map(|x| x.set_as_disconnected());
        self.d7.as_ref().map(|x| x.set_as_disconnected());
        self.nss.as_ref().map(|x| x.set_as_disconnected());
        self.dqs.as_ref().map(|x| x.set_as_disconnected());

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

pub(crate) trait SealedInstance {
    const REGS: Regs;
}

trait SealedWord {
    const CONFIG: u8;
}

/// OSPI instance trait.
#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + SealedInstance + RccPeripheral {}

pin_trait!(SckPin, Instance);
pin_trait!(NckPin, Instance);
pin_trait!(D0Pin, Instance);
pin_trait!(D1Pin, Instance);
pin_trait!(D2Pin, Instance);
pin_trait!(D3Pin, Instance);
pin_trait!(D4Pin, Instance);
pin_trait!(D5Pin, Instance);
pin_trait!(D6Pin, Instance);
pin_trait!(D7Pin, Instance);
pin_trait!(DQSPin, Instance);
pin_trait!(NSSPin, Instance);
dma_trait!(OctoDma, Instance);

foreach_peripheral!(
    (octospi, $inst:ident) => {
        impl SealedInstance for peripherals::$inst {
            const REGS: Regs = crate::pac::$inst;
        }

        impl Instance for peripherals::$inst {}
    };
);

impl<'d, T: Instance, M: PeriMode> SetConfig for Ospi<'d, T, M> {
    type Config = Config;
    type ConfigError = ();
    fn set_config(&mut self, config: &Self::Config) -> Result<(), ()> {
        self.set_config(config);
        Ok(())
    }
}

impl<'d, T: Instance, M: PeriMode> GetConfig for Ospi<'d, T, M> {
    type Config = Config;
    fn get_config(&self) -> Self::Config {
        self.get_config()
    }
}

/// Word sizes usable for OSPI.
#[allow(private_bounds)]
pub trait Word: word::Word + SealedWord {}

macro_rules! impl_word {
    ($T:ty, $config:expr) => {
        impl SealedWord for $T {
            const CONFIG: u8 = $config;
        }
        impl Word for $T {}
    };
}

impl_word!(u8, 8);
impl_word!(u16, 16);
impl_word!(u32, 32);
