use core::sync::atomic::{Ordering, compiler_fence};

use embassy_net_driver_channel::StateRunner;
use embassy_net_driver_channel::driver::{HardwareAddress, LinkState};

use crate::net::ZeroCopyPubSub;
use crate::net::commands::*;
use crate::net::iface::{Controller, HostToControllerPacket};
use crate::net::typedefs::*;
use crate::net::util::RefBox;

pub struct Control<'a, C: Controller> {
    controller: &'a C,

    state_ch: StateRunner<'a>,
    events: &'a ZeroCopyPubSub<'a, C::Packet<'a>>,
}

impl<'a, C: Controller> Control<'a, C> {
    pub(crate) const fn new(
        controller: &'a C,
        state_ch: StateRunner<'a>,
        events: &'a ZeroCopyPubSub<'a, C::Packet<'a>>,
    ) -> Self {
        Self {
            controller,
            state_ch,
            events,
        }
    }

    pub async fn send_command_and_get_response(
        &self,
        cmd: &impl HostToControllerPacket,
    ) -> Result<RefBox<'a, C::Packet<'a>>, MacError> {
        let subscriber = self.events.subscribe();

        compiler_fence(Ordering::Release);

        self.controller.write(cmd).await.map_err(|_e| MacError::Error)?;

        let response = subscriber.wait().await;

        Ok(response)
    }

    pub async fn scan(&mut self) -> Result<(), ()> {
        todo!()
    }

    pub async fn join(&mut self) -> Result<(), ()> {
        todo!()
    }

    pub async fn leave(&mut self) -> Result<(), ()> {
        todo!()
    }

    pub async fn start_ap(&mut self, pan_id: [u8; 2], short_addr: [u8; 2]) -> Result<(), ()> {
        {
            debug!("resetting");

            let resp = self
                .send_command_and_get_response(&ResetRequest {
                    set_default_pib: true,
                    ..Default::default()
                })
                .await
                .unwrap();

            debug!("{:#x}", resp.packet());
        }

        {
            debug!("setting extended address");
            let extended_addr = match self.state_ch.get_hardware_address() {
                HardwareAddress::Ieee802154(addr) => addr,
                _ => unreachable!(),
            };

            let resp = self
                .send_command_and_get_response(&SetRequest {
                    pib_attribute_ptr: &extended_addr as *const _ as *const u8,
                    pib_attribute: PibId::ExtendedAddress,
                })
                .await
                .unwrap();

            debug!("{:#x}", resp.packet());
        }

        {
            debug!("setting short address");
            let resp = self
                .send_command_and_get_response(&SetRequest {
                    pib_attribute_ptr: &short_addr as *const _ as *const u8,
                    pib_attribute: PibId::ExtendedAddress,
                })
                .await
                .unwrap();

            debug!("{:#x}", resp.packet());
        }

        {
            debug!("setting association permit");
            let association_permit: bool = true;

            let resp = self
                .send_command_and_get_response(&SetRequest {
                    pib_attribute_ptr: &association_permit as *const _ as *const u8,
                    pib_attribute: PibId::AssociationPermit,
                })
                .await
                .unwrap();

            debug!("{:#x}", resp.packet());
        }

        {
            debug!("setting TX power");
            let transmit_power: i8 = 2;

            let resp = self
                .send_command_and_get_response(&SetRequest {
                    pib_attribute_ptr: &transmit_power as *const _ as *const u8,
                    pib_attribute: PibId::TransmitPower,
                })
                .await
                .unwrap();

            debug!("{:#x}", resp.packet());
        }

        {
            debug!("starting FFD device");

            let resp = self
                .send_command_and_get_response(&StartRequest {
                    pan_id: PanId(pan_id),
                    channel_number: MacChannel::Channel16,
                    beacon_order: 0x0F,
                    superframe_order: 0x0F,
                    pan_coordinator: true,
                    battery_life_extension: false,
                    ..Default::default()
                })
                .await
                .unwrap();

            debug!("{:#x}", resp.packet());
        }

        {
            debug!("setting RX on when idle");
            let rx_on_while_idle: bool = true;

            let resp = self
                .send_command_and_get_response(&SetRequest {
                    pib_attribute_ptr: &rx_on_while_idle as *const _ as *const u8,
                    pib_attribute: PibId::RxOnWhenIdle,
                })
                .await
                .unwrap();

            debug!("{:#x}", resp.packet());
        }

        self.state_ch.set_link_state(LinkState::Up);
        Ok(())
    }

    pub async fn close_ap(&mut self) -> Result<(), ()> {
        todo!()
    }
}
