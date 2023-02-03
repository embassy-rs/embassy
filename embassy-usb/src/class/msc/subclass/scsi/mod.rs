// pub mod bitfield;
pub mod block_device;
pub mod commands;
pub mod enums;

use core::mem::MaybeUninit;

use self::block_device::BlockDevice;
use crate::class::msc::subclass::scsi::commands::{
    CachingModePage, InquiryCommand, InquiryResponse, ModeParameterHeader6, ModeSense6Command, PageCode,
    PreventAllowMediumRemoval, Read10Command, ReadCapacity10Command, ReadCapacity10Response,
    ReadFormatCapacitiesCommand, ReadFormatCapacitiesResponse, RequestSenseCommand, RequestSenseResponse,
    TestUnitReadyCommand, Write10Command,
};
use crate::class::msc::subclass::scsi::enums::{
    PeripheralDeviceType, PeripheralQualifier, ResponseCode, ResponseDataFormat, SenseKey, SpcVersion,
    TargetPortGroupSupport,
};
use crate::class::msc::transport::{self, CommandSetHandler};

pub struct Scsi<B: BlockDevice> {
    device: B,
}

impl<B: BlockDevice> Scsi<B> {
    pub fn new(device: B) -> Self {
        Self { device }
    }
}

impl<B: BlockDevice> CommandSetHandler for Scsi<B> {
    async fn command_out(
        &mut self,
        lun: u8,
        cmd: &[u8],
        pipe: &mut impl transport::DataPipeOut,
    ) -> Result<(), transport::CommandError> {
        assert!(lun == 0, "LUNs are not supported");

        let op_code = cmd[0];
        match op_code {
            TestUnitReadyCommand::OPCODE => {
                info!("TestUnitReadyCommand: {:#?}", TestUnitReadyCommand::from_bytes(cmd));
                return Ok(());
            }
            PreventAllowMediumRemoval::OPCODE => match PreventAllowMediumRemoval::from_bytes(cmd) {
                Some(req) => {
                    info!("PreventAllowMediumRemoval: {:?}", req);
                    return Ok(());
                }
                None => error!("Error parsing PreventAllowMediumRemoval"),
            },
            Write10Command::OPCODE => match Write10Command::from_bytes(cmd) {
                Some(req) => {
                    info!("Write10Command: {:?}", req);

                    let mut data = MaybeUninit::<[u8; 512]>::uninit();

                    let start_lba = req.lba();
                    let transfer_length = req.transfer_length() as u32;

                    if start_lba + transfer_length - 1 > self.device.num_blocks() {
                        return Err(transport::CommandError::CommandError);
                    }

                    for lba in start_lba..start_lba + transfer_length {
                        pipe.read(unsafe { data.assume_init_mut() }).await?;
                        self.device.write_block(lba, unsafe { data.assume_init_ref() }).unwrap();
                    }

                    return Ok(());
                }
                None => error!("Error parsing Write10Command"),
            },
            _ => warn!("Unknown OUT opcode: {}", op_code),
        }

        Err(transport::CommandError::CommandError)
    }

    async fn command_in(
        &mut self,
        lun: u8,
        cmd: &[u8],
        pipe: &mut impl transport::DataPipeIn,
    ) -> Result<(), transport::CommandError> {
        assert!(lun == 0, "LUNs are not supported");

        let op_code = cmd[0];
        info!("op_code: {}", op_code);
        match op_code {
            InquiryCommand::OPCODE => match InquiryCommand::from_bytes(cmd) {
                Some(req) => {
                    info!("inquiry: {:#?}", req);

                    let vendor_ident = b"FAKE    ";
                    let product_ident = b"PRODUCT         ";

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
                    resp.set_vendor_identification(vendor_ident);
                    resp.set_product_identification(product_ident);
                    resp.set_product_revision_level(&[b' '; 4]);

                    pipe.write(&resp.data).await?;
                    return Ok(());
                }
                None => error!("Error parsing InquiryCommand"),
            },
            ModeSense6Command::OPCODE => match ModeSense6Command::from_bytes(cmd) {
                Some(req) => {
                    info!("ModeSense6Command: {:?}", req);

                    // let mut buf = [0u8; ModeParameterHeader6::SIZE + CachingModePage::SIZE];

                    // let mut header = ModeParameterHeader6::from_bytes(&mut buf[0..ModeSense6Command::SIZE]).unwrap();
                    // header.set_mode_data_length((ModeParameterHeader6::SIZE + CachingModePage::SIZE - 1) as u8);

                    // let mut caching_mode_page =
                    //     CachingModePage::from_bytes(&mut buf[ModeParameterHeader6::SIZE..]).unwrap();
                    // caching_mode_page.set_page_code(PageCode::CachingModePage);
                    // caching_mode_page.set_page_length(CachingModePage::SIZE as u8);
                    // caching_mode_page.set_read_cache_disable(true);
                    // caching_mode_page.set_write_cache_enable(false);

                    // pipe.write(&buf).await?;

                    let mut header = ModeParameterHeader6::new();
                    header.set_mode_data_length(ModeParameterHeader6::SIZE as u8 - 1);
                    pipe.write(&header.data).await?;

                    return Ok(());
                }
                None => error!("Error parsing ModeSense6Command"),
            },
            RequestSenseCommand::OPCODE => match RequestSenseCommand::from_bytes(cmd) {
                Some(req) => {
                    info!("RequestSenseCommand: {:?}", req);

                    let mut resp = RequestSenseResponse::new();
                    resp.set_response_code(ResponseCode::CurrentFixedSenseData);
                    resp.set_sense_key(SenseKey::NoSense);

                    pipe.write(&resp.data).await?;
                    return Ok(());
                }
                None => error!("Error parsing RequestSenseCommand"),
            },
            ReadCapacity10Command::OPCODE => match ReadCapacity10Command::from_bytes(cmd) {
                Some(req) => {
                    info!("ReadCapacity10Command: {:?}", req);

                    let mut resp = ReadCapacity10Response::new();
                    resp.set_max_lba(self.device.num_blocks());
                    resp.set_block_size(self.device.block_size() as u32);

                    pipe.write(&resp.data).await?;
                    return Ok(());
                }
                None => error!("Error parsing ReadCapacity10Command"),
            },
            ReadFormatCapacitiesCommand::OPCODE => match ReadFormatCapacitiesCommand::from_bytes(cmd) {
                Some(req) => {
                    info!("ReadFormatCapacitiesCommand: {:?}", req);

                    let mut resp = ReadFormatCapacitiesResponse::new();
                    resp.set_capacity_list_length(8);
                    resp.set_max_lba(self.device.num_blocks());
                    resp.set_block_size(self.device.block_size() as u32);

                    pipe.write(&resp.data).await?;
                    return Ok(());
                }
                None => error!("Error parsing ReadFormatCapacitiesCommand"),
            },
            Read10Command::OPCODE => match Read10Command::from_bytes(cmd) {
                Some(req) => {
                    info!("Read10: {:?} {:?}", req, cmd);

                    let mut data = MaybeUninit::<[u8; 512]>::uninit();

                    let start_lba = req.lba();
                    let transfer_length = req.transfer_length() as u32;

                    if start_lba + transfer_length - 1 > self.device.num_blocks() {
                        return Err(transport::CommandError::CommandError);
                    }

                    for lba in start_lba..start_lba + transfer_length {
                        self.device.read_block(lba, unsafe { data.assume_init_mut() }).unwrap();

                        pipe.write(unsafe { data.assume_init_ref() }).await?;
                    }

                    return Ok(());
                }
                None => error!("Error parsing Read10Command"),
            },
            _ => warn!("Unknown IN opcode: {}", op_code),
        }

        Err(transport::CommandError::CommandError)
    }
}
