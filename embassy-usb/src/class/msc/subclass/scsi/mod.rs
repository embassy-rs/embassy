// pub mod bitfield;
pub mod block_device;
pub mod commands;
pub mod enums;

use core::mem::MaybeUninit;

use self::block_device::{BlockDevice, BlockDeviceError};
use self::enums::AdditionalSenseCode;
use crate::class::msc::subclass::scsi::commands::{
    CachingModePage, InformationalExceptionsControlModePage, InquiryCommand, InquiryResponse, ModeParameter6Writer,
    ModeParameterHeader6, ModeSense6Command, PageCode, PreventAllowMediumRemoval, Read10Command, ReadCapacity10Command,
    ReadCapacity10Response, ReadFormatCapacitiesCommand, ReadFormatCapacitiesResponse, RequestSenseCommand,
    RequestSenseResponse, SupportedVitalProductDataPages, TestUnitReadyCommand, UnitSerialNumberPage,
    VitalProductDataPage, Write10Command,
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

pub struct Scsi<'d, B: BlockDevice> {
    /// Backing storage block device
    device: B,
    buffer: &'d mut [u8],
    /// Last operation sense data
    sense: Option<SenseData>,
    vendor_id: [u8; 8],
    product_id: [u8; 16],
}

impl<'d, B: BlockDevice> Scsi<'d, B> {
    pub fn new(device: B, buffer: &'d mut [u8], vendor: &str, product: &str) -> Self {
        let mut vendor_id = [b' '; 8];
        fill_from_slice(&mut vendor_id, vendor.as_bytes());

        let mut product_id = [b' '; 16];
        fill_from_slice(&mut product_id, product.as_bytes());

        Self {
            device,
            buffer,
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
                if req.prevent() {
                    Err(ERR_INVALID_FIELD_IN_CBD)
                } else {
                    Ok(())
                }
            }
            Write10Command::OPCODE => {
                let req = Write10Command::from_bytes(cmd).ok_or(InternalError::CommandParseError)?;
                debug!("{:?}", req);

                let start_lba = req.lba();
                let transfer_length = req.transfer_length() as u32;

                if start_lba + transfer_length > self.device.num_blocks()? {
                    return Err(InternalError::LbaOutOfRange);
                }

                let block_size = self.device.block_size()?;
                assert!(
                    block_size <= self.buffer.len(),
                    "SCSI buffer smaller than device block size"
                );

                for lba in start_lba..start_lba + transfer_length {
                    pipe.read(&mut self.buffer[..block_size]).await?;
                    self.device.write_block(lba, &self.buffer[..block_size]).await?;
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

                if req.enable_vital_product_data() {
                    match req.page_code() {
                        Ok(VitalProductDataPage::SupportedVitalProductDataPages) => {
                            const SUPPORTED_PAGES: [VitalProductDataPage; 2] = [
                                VitalProductDataPage::SupportedVitalProductDataPages,
                                VitalProductDataPage::UnitSerialNumber,
                            ];
                            let mut buf = [0u8; SupportedVitalProductDataPages::SIZE + SUPPORTED_PAGES.len()];
                            let mut svpdp = SupportedVitalProductDataPages::from_bytes(&mut buf).unwrap();
                            svpdp.set_page_code(VitalProductDataPage::SupportedVitalProductDataPages);
                            svpdp.set_page_length(SUPPORTED_PAGES.len() as u8);

                            for (i, page) in SUPPORTED_PAGES.iter().enumerate() {
                                buf[SupportedVitalProductDataPages::SIZE + i] = (*page).into();
                            }

                            pipe.write(&buf).await?;
                            Ok(())
                        }
                        Ok(VitalProductDataPage::UnitSerialNumber) => {
                            const SERIAL_NUMBER: &[u8; 8] = b"01020304";
                            let mut buf = [0u8; UnitSerialNumberPage::SIZE + SERIAL_NUMBER.len()];
                            let mut usnp = UnitSerialNumberPage::from_bytes(&mut buf).unwrap();
                            usnp.set_page_code(VitalProductDataPage::UnitSerialNumber);
                            usnp.set_page_length(SERIAL_NUMBER.len() as u8);

                            buf[UnitSerialNumberPage::SIZE..].copy_from_slice(SERIAL_NUMBER);

                            pipe.write(&buf).await?;
                            Ok(())
                        }
                        _ => Err(ERR_INVALID_FIELD_IN_CBD),
                    }
                } else {
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
                    resp.set_product_revision_level(b"1.00");

                    pipe.write(&resp.data).await?;
                    Ok(())
                }
            }
            ModeSense6Command::OPCODE => {
                let req = ModeSense6Command::from_bytes(cmd).ok_or(InternalError::CommandParseError)?;
                debug!("{:?} {:?}", req, cmd);

                // let mut buf = [0u8; ModeParameterHeader6::SIZE + CachingModePage::SIZE];
                // let mut header = ModeParameterHeader6::from_bytes(&mut buf[..ModeParameterHeader6::SIZE]).unwrap();
                // header.set_mode_data_length((ModeParameterHeader6::SIZE + CachingModePage::SIZE - 1) as u8);

                // let mut caching_mode_page =
                //     CachingModePage::from_bytes(&mut buf[ModeParameterHeader6::SIZE..]).unwrap();
                // caching_mode_page.set_page_code(PageCode::CachingModePage);
                // caching_mode_page.set_page_length(CachingModePage::SIZE as u8 - 2);
                // caching_mode_page.set_read_cache_disable(true);
                // caching_mode_page.set_write_cache_enable(false);

                // pipe.write(&buf).await?;

                let mut writer = ModeParameter6Writer::new(self.buffer).map_err(|_| ERR_INVALID_FIELD_IN_CBD)?;

                let all_pages = matches!(req.page_code(), Ok(PageCode::AllPages));

                if all_pages || matches!(req.page_code(), Ok(PageCode::Caching)) {
                    let mut caching_mode_page = CachingModePage::from_bytes(
                        writer
                            .write_page(PageCode::Caching, CachingModePage::SIZE)
                            .map_err(|_| ERR_INVALID_FIELD_IN_CBD)?,
                    )
                    .unwrap();

                    caching_mode_page.set_read_cache_disable(true);
                    caching_mode_page.set_write_cache_enable(false);
                }

                if all_pages || matches!(req.page_code(), Ok(PageCode::InformationalExceptionsControl)) {
                    let mut _iec = InformationalExceptionsControlModePage::from_bytes(
                        writer
                            .write_page(
                                PageCode::InformationalExceptionsControl,
                                InformationalExceptionsControlModePage::SIZE,
                            )
                            .map_err(|_| ERR_INVALID_FIELD_IN_CBD)?,
                    )
                    .unwrap();
                }

                // Should never return zero pages
                if writer.page_size() == 0 {
                    Err(ERR_INVALID_FIELD_IN_CBD)
                } else {
                    pipe.write(writer.finalize()).await?;
                    Ok(())
                }
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
                resp.set_max_lba(self.device.num_blocks()? - 1);
                resp.set_block_size(self.device.block_size()? as u32);

                pipe.write(&resp.data).await?;
                Ok(())
            }
            ReadFormatCapacitiesCommand::OPCODE => {
                let req = ReadFormatCapacitiesCommand::from_bytes(cmd).ok_or(InternalError::CommandParseError)?;
                debug!("{:?}", req);

                let mut resp = ReadFormatCapacitiesResponse::new();
                resp.set_capacity_list_length(8);
                resp.set_num_blocks(self.device.num_blocks()?);
                resp.set_block_size(self.device.block_size()? as u32);
                resp.set_descriptor_type(0x03);

                pipe.write(&resp.data).await?;
                return Ok(());
            }
            Read10Command::OPCODE => {
                let req = Read10Command::from_bytes(cmd).ok_or(InternalError::CommandParseError)?;
                debug!("{:?}", req);

                let start_lba = req.lba();
                let transfer_length = req.transfer_length() as u32;

                if start_lba + transfer_length > self.device.num_blocks()? {
                    return Err(InternalError::LbaOutOfRange);
                }

                let block_size = self.device.block_size()?;
                assert!(
                    block_size <= self.buffer.len(),
                    "SCSI buffer smaller than device block size"
                );

                for lba in start_lba..start_lba + transfer_length {
                    self.device.read_block(lba, &mut self.buffer[..block_size]).await?;

                    pipe.write(&self.buffer[..block_size]).await?;
                }

                Ok(())
            }
            _ => Err(InternalError::UnknownOpcode),
        }
    }
}

impl<'d, B: BlockDevice> CommandSetHandler for Scsi<'d, B> {
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

/// Commonly used error (IllegalRequest, InvalidFieldInCdb)
const ERR_INVALID_FIELD_IN_CBD: InternalError = InternalError::CustomSenseData(SenseData {
    key: SenseKey::IllegalRequest,
    asc: AdditionalSenseCode::InvalidFieldInCdb,
});

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
