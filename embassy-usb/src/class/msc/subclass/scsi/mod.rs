// pub mod bitfield;
pub mod block_device;
pub mod commands;
pub mod enums;
pub mod packet;
pub mod responses;

use self::block_device::BlockDevice;
use crate::class::msc::transport::{self, CommandSetHandler};

pub struct Scsi<B: BlockDevice> {
    device: B,
}

impl<B: BlockDevice> CommandSetHandler for Scsi<B> {
    async fn command_out(
        &mut self,
        lun: u8,
        cmd: &[u8],
        pipe: &mut impl transport::DataPipeOut,
    ) -> Result<(), transport::CommandError> {
        assert!(lun == 0, "LUNs are not supported");

        Ok(())
    }

    async fn command_in(
        &mut self,
        lun: u8,
        cmd: &[u8],
        pipe: &mut impl transport::DataPipeIn,
    ) -> Result<(), transport::CommandError> {
        assert!(lun == 0, "LUNs are not supported");

        Ok(())
    }
}
