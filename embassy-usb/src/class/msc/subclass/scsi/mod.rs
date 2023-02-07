// pub mod bitfield;
pub mod block_device;
pub mod commands;
pub mod enums;

use core::mem::MaybeUninit;

use self::block_device::{BlockDevice, BlockDeviceError};
use self::enums::AdditionalSenseCode;
use crate::class::msc::subclass::scsi::commands::{
    InquiryCommand, InquiryResponse, ModeParameterHeader6, ModeSense6Command, PreventAllowMediumRemoval, Read10Command,
    ReadCapacity10Command, ReadCapacity10Response, ReadFormatCapacitiesCommand, ReadFormatCapacitiesResponse,
    RequestSenseCommand, RequestSenseResponse, TestUnitReadyCommand, Write10Command,
};
use crate::class::msc::subclass::scsi::enums::{
    PeripheralDeviceType, PeripheralQualifier, ResponseCode, ResponseDataFormat, SenseKey, SpcVersion,
    TargetPortGroupSupport,
};
use crate::class::msc::transport::{self, CommandSetHandler};
use crate::class::msc::MscSubclass;

/// Stores information (errors) about last operation.
///
/// Sent on `RequestSenseCommand` as `RequestSenseResponse`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SenseData {
    /// Sense key
    key: SenseKey,
    /// Additional Sense Code
    asc: AdditionalSenseCode,
}

pub struct Scsi<B: BlockDevice> {
    /// Backing storage block device
    device: B,
    /// Last operation sense data
    sense: Option<SenseData>,
    vendor_id: [u8; 8],
    product_id: [u8; 16],
}

impl<B: BlockDevice> Scsi<B> {
    pub fn new(device: B, vendor: &str, product: &str) -> Self {
        let mut vendor_id = [b' '; 8];
        fill_from_slice(&mut vendor_id, vendor.as_bytes());

        let mut product_id = [b' '; 16];
        fill_from_slice(&mut product_id, product.as_bytes());

        Self {
            device,
            sense: None,
            vendor_id,
            product_id,
        }
    }

    async fn handle_command_out(
        &mut self,
        lun: u8,
        cmd: &[u8],
        pipe: &mut impl transport::DataPipeOut,
    ) -> Result<(), InternalError> {
        if lun != 0 {
            return Err(InternalError::LunsNotSupported);
        }

        let op_code = cmd.get(0).ok_or(InternalError::CommandParseError)?;

        match *op_code {
            TestUnitReadyCommand::OPCODE => {
                let req = TestUnitReadyCommand::from_bytes(cmd).ok_or(InternalError::CommandParseError)?;
                debug!("{:?}", req);
                self.device.status().map_err(|e| e.into())
            }
            PreventAllowMediumRemoval::OPCODE => {
                let req = PreventAllowMediumRemoval::from_bytes(cmd).ok_or(InternalError::CommandParseError)?;
                debug!("{:?}", req);

                // From spec:
                // If the device server does not support the medium changer prevent state, it shall terminate the
                // PREVENT ALLOW MEDIUM REMOVAL command with CHECK CONDITION status with the sense
                // key set to ILLEGAL REQUEST and the additional sense code set to INVALID FIELD IN CDB.
                Err(InternalError::CustomSenseData(SenseData {
                    key: SenseKey::IllegalRequest,
                    asc: AdditionalSenseCode::InvalidFieldInCdb,
                }))
            }
            Write10Command::OPCODE => {
                let req = Write10Command::from_bytes(cmd).ok_or(InternalError::CommandParseError)?;
                debug!("{:?}", req);

                let mut data = MaybeUninit::<[u8; 512]>::uninit();

                let start_lba = req.lba();
                let transfer_length = req.transfer_length() as u32;

                if start_lba + transfer_length > self.device.max_lba()? + 1 {
                    return Err(InternalError::LbaOutOfRange);
                }

                for lba in start_lba..start_lba + transfer_length {
                    pipe.read(unsafe { data.assume_init_mut() }).await?;
                    self.device.write_block(lba, unsafe { data.assume_init_ref() }).await?;
                }

                Ok(())
            }
            _ => Err(InternalError::UnknownOpcode),
        }
    }

    async fn handle_command_in(
        &mut self,
        lun: u8,
        cmd: &[u8],
        pipe: &mut impl transport::DataPipeIn,
    ) -> Result<(), InternalError> {
        if lun != 0 {
            return Err(InternalError::LunsNotSupported);
        }

        let op_code = cmd.get(0).ok_or(InternalError::CommandParseError)?;

        match *op_code {
            InquiryCommand::OPCODE => {
                let req = InquiryCommand::from_bytes(cmd).ok_or(InternalError::CommandParseError)?;
                debug!("{:#?}", req);

                let mut resp = InquiryResponse::new();
                resp.set_peripheral_device_type(PeripheralDeviceType::DirectAccessBlock);
                resp.set_peripheral_qualifier(PeripheralQualifier::Connected);
                resp.set_removable_medium(true);
                resp.set_version(SpcVersion::Spc3);
                resp.set_response_data_format(ResponseDataFormat::Standard);
                resp.set_hierarchical_support(false);
                resp.set_normal_aca(false);
                resp.set_additional_length((InquiryResponse::SIZE - 4) as u8);
                resp.set_protect(false);
                resp.set_third_party_copy(false);
                resp.set_target_port_group_support(TargetPortGroupSupport::Unsupported);
                resp.set_access_controls_coordinator(false);
                resp.set_scc_supported(false);
                resp.set_multi_port(false);
                resp.set_enclosure_services(false);
                resp.set_vendor_identification(&self.vendor_id);
                resp.set_product_identification(&self.product_id);
                resp.set_product_revision_level(&[b' '; 4]);

                pipe.write(&resp.data).await?;
                Ok(())
            }
            ModeSense6Command::OPCODE => {
                let req = ModeSense6Command::from_bytes(cmd).ok_or(InternalError::CommandParseError)?;
                debug!("{:?}", req);

                let mut header = ModeParameterHeader6::new();
                header.set_mode_data_length(ModeParameterHeader6::SIZE as u8 - 1);
                pipe.write(&header.data).await?;

                Ok(())
            }
            RequestSenseCommand::OPCODE => {
                let req = RequestSenseCommand::from_bytes(cmd).ok_or(InternalError::CommandParseError)?;
                debug!("{:?}", req);

                let mut resp = RequestSenseResponse::new();
                resp.set_response_code(ResponseCode::CurrentFixedSenseData);

                let len = RequestSenseResponse::SIZE.min(req.allocation_length() as usize);
                resp.set_additional_sense_length((len - 7) as u8);

                match &self.sense {
                    Some(sense) => {
                        resp.set_sense_key(sense.key);
                        resp.set_additional_sense_code(sense.asc.asc());
                        resp.set_additional_sense_code_qualifier(sense.asc.ascq());
                    }
                    None => {
                        resp.set_sense_key(SenseKey::NoSense);
                    }
                }

                pipe.write(&resp.data[..len]).await?;
                Ok(())
            }
            ReadCapacity10Command::OPCODE => {
                let req = ReadCapacity10Command::from_bytes(cmd).ok_or(InternalError::CommandParseError)?;
                debug!("{:?}", req);

                let mut resp = ReadCapacity10Response::new();
                resp.set_max_lba(self.device.max_lba()?);
                resp.set_block_size(self.device.block_size()? as u32);

                pipe.write(&resp.data).await?;
                Ok(())
            }
            ReadFormatCapacitiesCommand::OPCODE => {
                let req = ReadFormatCapacitiesCommand::from_bytes(cmd).ok_or(InternalError::CommandParseError)?;
                debug!("{:?}", req);

                let mut resp = ReadFormatCapacitiesResponse::new();
                resp.set_capacity_list_length(8);
                resp.set_max_lba(self.device.max_lba()?);
                resp.set_block_size(self.device.block_size()? as u32);

                pipe.write(&resp.data).await?;
                return Ok(());
            }
            Read10Command::OPCODE => {
                let req = Read10Command::from_bytes(cmd).ok_or(InternalError::CommandParseError)?;
                debug!("{:?}", req);

                let mut data = MaybeUninit::<[u8; 512]>::uninit();

                let start_lba = req.lba();
                let transfer_length = req.transfer_length() as u32;

                if start_lba + transfer_length > self.device.max_lba()? + 1 {
                    return Err(InternalError::LbaOutOfRange);
                }

                for lba in start_lba..start_lba + transfer_length {
                    self.device.read_block(lba, unsafe { data.assume_init_mut() }).await?;

                    pipe.write(unsafe { data.assume_init_ref() }).await?;
                }

                Ok(())
            }
            _ => Err(InternalError::UnknownOpcode),
        }
    }
}

impl<B: BlockDevice> CommandSetHandler for Scsi<B> {
    const MSC_SUBCLASS: MscSubclass = MscSubclass::ScsiTransparentCommandSet;
    const MAX_LUN: u8 = 0;

    async fn command_out(
        &mut self,
        lun: u8,
        cmd: &[u8],
        pipe: &mut impl transport::DataPipeOut,
    ) -> Result<(), transport::CommandError> {
        match self.handle_command_out(lun, cmd, pipe).await {
            Ok(_) => {
                self.sense = None;
                Ok(())
            }
            Err(e) => {
                error!("command_out error op={}, err={:?}", cmd.get(0), e);
                self.sense = Some(e.into_sense_data());
                Err(match e {
                    InternalError::DataPipeError(e) => e.into(),
                    _ => transport::CommandError::CommandError,
                })
            }
        }
    }

    async fn command_in(
        &mut self,
        lun: u8,
        cmd: &[u8],
        pipe: &mut impl transport::DataPipeIn,
    ) -> Result<(), transport::CommandError> {
        match self.handle_command_in(lun, cmd, pipe).await {
            Ok(_) => {
                self.sense = None;
                Ok(())
            }
            Err(e) => {
                error!("command_in error op={}, err={:?}", cmd.get(0), e);
                self.sense = Some(e.into_sense_data());
                Err(match e {
                    InternalError::DataPipeError(e) => e.into(),
                    _ => transport::CommandError::CommandError,
                })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum InternalError {
    /// Unknown opcode
    UnknownOpcode,
    /// Command could not be parsed
    CommandParseError,
    /// Logical block address exceeded max_lba
    LbaOutOfRange,
    /// LUNs not supported
    LunsNotSupported,
    /// Block device error
    BlockDeviceError(BlockDeviceError),
    /// Data pipe error
    DataPipeError(transport::DataPipeError),
    /// Custom sense data
    CustomSenseData(SenseData),
}

impl InternalError {
    fn into_sense_data(self) -> SenseData {
        match self {
            InternalError::UnknownOpcode => SenseData {
                key: SenseKey::IllegalRequest,
                asc: AdditionalSenseCode::InvalidCommandOperationCode,
            },
            InternalError::CommandParseError => SenseData {
                key: SenseKey::IllegalRequest,
                asc: AdditionalSenseCode::InvalidFieldInCdb,
            },
            InternalError::LbaOutOfRange => SenseData {
                key: SenseKey::IllegalRequest,
                asc: AdditionalSenseCode::LogicalBlockAddressOutOfRange,
            },
            InternalError::LunsNotSupported => SenseData {
                key: SenseKey::IllegalRequest,
                asc: AdditionalSenseCode::LogicalUnitNotSupported,
            },
            InternalError::BlockDeviceError(e) => match e {
                BlockDeviceError::MediumNotPresent => SenseData {
                    key: SenseKey::NotReady,
                    asc: AdditionalSenseCode::MediumNotPresent,
                },
                BlockDeviceError::LbaOutOfRange => SenseData {
                    key: SenseKey::IllegalRequest,
                    asc: AdditionalSenseCode::LogicalBlockAddressOutOfRange,
                },
                BlockDeviceError::HardwareError => SenseData {
                    key: SenseKey::HardwareError,
                    asc: AdditionalSenseCode::NoAdditionalSenseInformation,
                },
                BlockDeviceError::ReadError => SenseData {
                    key: SenseKey::MediumError,
                    asc: AdditionalSenseCode::UnrecoveredReadError,
                },
                BlockDeviceError::WriteError => SenseData {
                    key: SenseKey::MediumError,
                    asc: AdditionalSenseCode::WriteError,
                },
                BlockDeviceError::EraseError => SenseData {
                    key: SenseKey::MediumError,
                    asc: AdditionalSenseCode::EraseFailure,
                },
            },
            InternalError::DataPipeError(_) => SenseData {
                // Not sure if this is correct.
                // It's hard to find information on what happens when USB transport fails.
                key: SenseKey::AbortedCommand,
                asc: AdditionalSenseCode::NoAdditionalSenseInformation,
            },
            InternalError::CustomSenseData(sense) => sense,
        }
    }
}

impl From<BlockDeviceError> for InternalError {
    fn from(value: BlockDeviceError) -> Self {
        Self::BlockDeviceError(value)
    }
}

impl From<transport::DataPipeError> for InternalError {
    fn from(value: transport::DataPipeError) -> Self {
        Self::DataPipeError(value)
    }
}

// Why is this not in the standard library?
fn fill_from_slice(dst: &mut [u8], src: &[u8]) {
    let limit = dst.len().min(src.len());
    dst[..limit].copy_from_slice(&src[..limit]);
}
