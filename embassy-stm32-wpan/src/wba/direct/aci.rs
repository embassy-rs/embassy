//! ST vendor (ACI) commands of `stm32wb-hci`, called directly (`direct` and
//! `wb-hci` features).
//!
//! Each command calls the ST function it is declared with in `stm32wb-hci`.
//! The argument lists follow the STM32CubeWBA 1.10.0 layouts of
//! `stm32wb-catalog`, by which the ST functions take a command's parameters in
//! order, followed by its return parameters. `stm32wb-hci` declares a command
//! only for the stack variants that have it, so the commands are grouped by
//! the variants they exist in.
//!
//! The three Core commands `stm32wb-hci` declares itself, because bt-hci
//! encodes them differently, are implemented here too.

/// Commands of the full, basic plus, basic, peripheral only and link layer only stacks.
mod every_stack {
    use stm32wb_hci::aci::general::*;
    use stm32wb_hci::aci::hal::*;

    use super::super::*;

    // HAL
    direct!(HalWriteConfigData<'a> => |p| aci_hal_write_config_data(p.u8(), p.u8(), p.list(1)));
    direct!(HalReadConfigData => |p, o| aci_hal_read_config_data(p.u8(), o.u8(), o.items(1, 250)));
    direct!(HalSetTxPowerLevel => |p| aci_hal_set_tx_power_level(p.u8(), p.u8()));
    direct!(HalLeTxTestPacketNumber => |p, o| aci_hal_le_tx_test_packet_number(o.u32()));
    direct!(HalToneStart => |p| aci_hal_tone_start(p.u8(), p.u8()));
    direct!(HalToneStop => |p| aci_hal_tone_stop());
    direct!(HalSetRadioActivityMask => |p| aci_hal_set_radio_activity_mask(p.u16()));
    direct!(HalSetEventMask => |p| aci_hal_set_event_mask(p.u32()));
    direct!(HalGetLinkStatusV2 => |p, o| aci_hal_get_link_status_v2(o.bytes(22), o.array_of::<u16>(22)));
    direct!(HalSetPeripheralLatency => |p| aci_hal_set_peripheral_latency(p.u8()));
    direct!(HalReadRssi => |p, o| aci_hal_read_rssi(o.u8()));
    direct!(HalPtaEnable => |p| aci_hal_pta_enable(p.u8()));
    direct!(HalPtaSetPriority => |p| aci_hal_pta_set_priority(p.u8(), p.u16(), p.u32(), p.u32(), p.u8(), p.u8()));

    // General
    direct!(AciReset => |p| aci_reset(p.u8(), p.u32()));
    direct!(AciGetInformation => |p, o| aci_get_information(o.array_of::<u32>(2), o.u32(), o.array_of::<u32>(3)));
    direct!(AciWriteConfigData<'a> => |p| aci_write_config_data(p.u8(), p.u8(), p.list(1)));
    direct!(AciReadConfigData => |p, o| aci_read_config_data(p.u8(), o.u8(), o.items(1, 250)));
}

/// Commands of the full, basic plus, basic and peripheral only stacks.
#[cfg(any(
    feature = "ble-stack-full",
    feature = "ble-stack-basic-plus",
    feature = "ble-stack-basic",
    feature = "ble-stack-po",
))]
mod host_stacks {
    use stm32wb_hci::aci::gap::*;
    use stm32wb_hci::aci::gatt::*;
    use stm32wb_hci::aci::l2cap::*;

    use super::super::*;

    // GAP
    direct!(GapSetNonDiscoverable => |p| aci_gap_set_non_discoverable());
    direct!(async GapSetLimitedDiscoverable<'a> => |p| aci_gap_set_limited_discoverable(
        p.u8(), p.u16(), p.u16(), p.u8(), p.u8(), p.u8(), p.list(1), p.u8(), p.list(1), p.u16(), p.u16()
    ));
    direct!(GapSetDiscoverable<'a> => |p| aci_gap_set_discoverable(
        p.u8(), p.u16(), p.u16(), p.u8(), p.u8(), p.u8(), p.list(1), p.u8(), p.list(1), p.u16(), p.u16()
    ));
    direct!(GapSetDirectConnectable => |p| aci_gap_set_direct_connectable(
        p.u8(), p.u8(), p.u8(), p.ptr(6), p.u16(), p.u16()
    ));
    direct!(GapSetIoCapability => |p| aci_gap_set_io_capability(p.u8()));
    direct!(GapSetAuthenticationRequirement => |p| aci_gap_set_authentication_requirement(
        p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u32(), p.u8()
    ));
    direct!(GapSetAuthorizationRequirement => |p| aci_gap_set_authorization_requirement(p.u16(), p.u8()));
    direct!(GapPassKeyResp => |p| aci_gap_pass_key_resp(p.u16(), p.u32()));
    direct!(GapAuthorizationResp => |p| aci_gap_authorization_resp(p.u16(), p.u8()));
    direct!(GapInit => |p, o| aci_gap_init(p.u8(), p.u8(), p.u8(), o.u16(), o.u16(), o.u16()));
    direct!(GapSetNonConnectable => |p| aci_gap_set_non_connectable(p.u8(), p.u8()));
    direct!(GapSetUndirectedConnectable => |p| aci_gap_set_undirected_connectable(p.u16(), p.u16(), p.u8(), p.u8()));
    direct!(async GapPeripheralSecurityReq => |p| aci_gap_peripheral_security_req(p.u16()));
    direct!(GapUpdateAdvData<'a> => |p| aci_gap_update_adv_data(p.u8(), p.list(1)));
    direct!(GapDeleteAdType => |p| aci_gap_delete_ad_type(p.u8()));
    direct!(GapGetSecurityLevel => |p, o| aci_gap_get_security_level(p.u16(), o.u8(), o.u8()));
    direct!(GapSetEventMask => |p| aci_gap_set_event_mask(p.u16()));
    direct!(async GapTerminate => |p| aci_gap_terminate(p.u16(), p.u8()));
    direct!(GapClearSecurityDb => |p| aci_gap_clear_security_db());
    direct!(GapAllowRebond => |p| aci_gap_allow_rebond(p.u16()));
    direct!(async GapStartConnectionUpdate => |p| aci_gap_start_connection_update(
        p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u16()
    ));
    direct!(GapGetBondedDevices => |p, o| aci_gap_get_bonded_devices(o.u8(), o.items_of::<Bonded_Device_Entry_t>(35)));
    direct!(GapCheckBondedDevice => |p, o| aci_gap_check_bonded_device(p.u8(), p.ptr(6), o.u8(), o.bytes(6)));
    direct!(GapNumericComparisonValueConfirmYesno => |p| aci_gap_numeric_comparison_value_confirm_yesno(
        p.u16(), p.u8()
    ));
    direct!(GapPasskeyInput => |p| aci_gap_passkey_input(p.u16(), p.u8()));
    direct!(GapGetOobData => |p, o| aci_gap_get_oob_data(
        p.u8(), o.u8(), o.bytes(6), { o.echo(p.bytes_at(0, 1)); o.u8() }, o.bytes(16)
    ));
    direct!(GapSetOobData => |p| aci_gap_set_oob_data(p.u8(), p.u8(), p.ptr(6), p.u8(), p.u8(), p.ptr(16)));
    direct!(GapRemoveBondedDevice => |p| aci_gap_remove_bonded_device(p.u8(), p.ptr(6)));
    direct!(GapAddDevicesToList<'a> => |p| aci_gap_add_devices_to_list(p.u8(), p.list_of::<List_Entry_t>(), p.u8()));
    direct!(GapPairingRequestReply => |p| aci_gap_pairing_request_reply(p.u16(), p.u8()));

    // GATT
    direct!(GattInit => |p| aci_gatt_init());
    direct!(GattAddService => |p, o| aci_gatt_add_service(
        p.u8(), p.uuid().cast::<Service_UUID_t>(), p.u8(), p.u8(), o.u16()
    ));
    direct!(GattIncludeService => |p, o| aci_gatt_include_service(
        p.u16(), p.u16(), p.u16(), p.u8(), p.uuid().cast::<Include_UUID_t>(), o.u16()
    ));
    direct!(GattAddChar => |p, o| aci_gatt_add_char(
        p.u16(), p.u8(), p.uuid().cast::<Char_UUID_t>(), p.u16(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), o.u16()
    ));
    direct!(GattAddCharDesc<'a> => |p, o| aci_gatt_add_char_desc(
        p.u16(), p.u16(), p.u8(), p.uuid().cast::<Char_Desc_Uuid_t>(), p.u8(), p.u8(), p.list(1), p.u8(), p.u8(),
        p.u8(), p.u8(), p.u8(), o.u16()
    ));
    direct!(GattUpdateCharValue<'a> => |p| aci_gatt_update_char_value(p.u16(), p.u16(), p.u8(), p.u8(), p.list(1)));
    direct!(GattDelChar => |p| aci_gatt_del_char(p.u16(), p.u16()));
    direct!(GattDelService => |p| aci_gatt_del_service(p.u16()));
    direct!(GattDelIncludeService => |p| aci_gatt_del_include_service(p.u16(), p.u16()));
    direct!(GattSetEventMask => |p| aci_gatt_set_event_mask(p.u32()));
    direct!(GattPermitWrite<'a> => |p| aci_gatt_permit_write(p.u16(), p.u16(), p.u8(), p.u8(), p.u8(), p.list(1)));
    direct!(GattPermitRead => |p| aci_gatt_permit_read(p.u16(), p.u8(), p.u8(), p.u16()));
    direct!(GattSetSecurityPermission => |p| aci_gatt_set_security_permission(p.u16(), p.u16(), p.u8()));
    direct!(GattSetDescValue<'a> => |p| aci_gatt_set_desc_value(p.u16(), p.u16(), p.u16(), p.u16(), p.u8(), p.list(1)));
    direct!(GattReadHandleValue => |p, o| aci_gatt_read_handle_value(
        p.u16(), p.u16(), p.u16(), o.u16(), o.u16(), o.items(1, 247)
    ));
    direct!(GattGetAttributeValue => |p, o| aci_gatt_get_attribute_value(
        p.u16(), p.u16(), p.u16(), p.u16(), o.u16(), o.u16(), o.items(1, 247)
    ));
    direct!(GattUpdateCharValueExt<'a> => |p| aci_gatt_update_char_value_ext(
        p.u16(), p.u16(), p.u16(), p.u8(), p.u16(), p.u16(), p.u8(), p.list(1)
    ));
    direct!(GattSetAccessPermission => |p| aci_gatt_set_access_permission(p.u16(), p.u16(), p.u8()));
    direct!(GattStoreDb => |p| aci_gatt_store_db());

    // L2CAP
    direct!(async L2capConnectionParameterUpdateReq => |p| aci_l2cap_connection_parameter_update_req(
        p.u16(), p.u16(), p.u16(), p.u16(), p.u16()
    ));
}

/// Commands of the full, basic plus and basic stacks.
#[cfg(any(
    feature = "ble-stack-full",
    feature = "ble-stack-basic-plus",
    feature = "ble-stack-basic"
))]
mod basic {
    use stm32wb_hci::aci::att::*;
    use stm32wb_hci::aci::gap::*;
    use stm32wb_hci::aci::gatt::*;
    use stm32wb_hci::aci::l2cap::*;

    use super::super::*;

    // GAP
    direct!(async GapStartAutoConnectionEstablishProc<'a> => |p| aci_gap_start_auto_connection_establish_proc(
        p.u16(), p.u16(), p.u8(), p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u8(),
        p.list_of::<Peer_Entry_t>()
    ));
    direct!(async GapCreateConnection => |p| aci_gap_create_connection(
        p.u16(), p.u16(), p.u8(), p.ptr(6), p.u8(), p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u16()
    ));
    direct!(GapTerminateGapProc => |p| aci_gap_terminate_gap_proc(p.u8()));
    direct!(async GapSendPairingReq => |p| aci_gap_send_pairing_req(p.u16(), p.u8()));
    direct!(GapSetBroadcastMode<'a> => |p| aci_gap_set_broadcast_mode(
        p.u16(), p.u16(), p.u8(), p.u8(), p.u8(), p.list(1), p.u8(), p.list_of::<Peer_Entry_t>()
    ));
    direct!(async GapExtStartScan => |p| aci_gap_start_scan(
        p.u8(), p.u8(), p.u8(), p.u8(), p.u16(), p.u16(), p.u8(), p.u8(), p.array_of::<Scan_Param_Phy_t>(2)
    ));

    // GATT
    direct!(async GattExchangeConfig => |p| aci_gatt_exchange_config(p.u16()));
    direct!(async GattDiscAllPrimaryServices => |p| aci_gatt_disc_all_primary_services(p.u16()));
    direct!(async GattDiscPrimaryServiceByUuid => |p| aci_gatt_disc_primary_service_by_uuid(
        p.u16(), p.u8(), p.uuid().cast::<UUID_t>()
    ));
    direct!(async GattFindIncludedServices => |p| aci_gatt_find_included_services(p.u16(), p.u16(), p.u16()));
    direct!(async GattDiscAllCharOfService => |p| aci_gatt_disc_all_char_of_service(p.u16(), p.u16(), p.u16()));
    direct!(async GattDiscCharByUuid => |p| aci_gatt_disc_char_by_uuid(
        p.u16(), p.u16(), p.u16(), p.u8(), p.uuid().cast::<UUID_t>()
    ));
    direct!(async GattDiscAllCharDesc => |p| aci_gatt_disc_all_char_desc(p.u16(), p.u16(), p.u16()));
    direct!(async GattReadCharValue => |p| aci_gatt_read_char_value(p.u16(), p.u16()));
    direct!(async GattReadUsingCharUuid => |p| aci_gatt_read_using_char_uuid(
        p.u16(), p.u16(), p.u16(), p.u8(), p.uuid().cast::<UUID_t>()
    ));
    direct!(async GattReadLongCharValue => |p| aci_gatt_read_long_char_value(p.u16(), p.u16(), p.u16()));
    direct!(async GattReadMultipleCharValue<'a> => |p| aci_gatt_read_multiple_char_value(
        p.u16(), p.u8(), p.list_of::<Handle_Entry_t>()
    ));
    direct!(async GattWriteCharValue<'a> => |p| aci_gatt_write_char_value(p.u16(), p.u16(), p.u8(), p.list(1)));
    direct!(async GattWriteLongCharValue<'a> => |p| aci_gatt_write_long_char_value(
        p.u16(), p.u16(), p.u16(), p.u8(), p.list(1)
    ));
    direct!(async GattWriteCharReliable<'a> => |p| aci_gatt_write_char_reliable(
        p.u16(), p.u16(), p.u16(), p.u8(), p.list(1)
    ));
    direct!(GattWriteWithoutResp<'a> => |p| aci_gatt_write_without_resp(p.u16(), p.u16(), p.u8(), p.list(1)));
    direct!(GattConfirmIndication => |p| aci_gatt_confirm_indication(p.u16()));
    direct!(async GattReadMultipleVarCharValue<'a> => |p| aci_gatt_read_multiple_var_char_value(
        p.u16(), p.u8(), p.list_of::<Handle_Entry_t>()
    ));
    direct!(GattWriteWithoutRespExt => |p| aci_gatt_write_without_resp_ext(p.u16(), p.u16(), p.u8(), p.u16(), p.u32()));
    direct!(async GattWriteWithRespExt => |p| aci_gatt_write_with_resp_ext(
        p.u16(), p.u16(), p.u8(), p.u16(), p.u16(), p.u32()
    ));

    // ATT
    direct!(async AttFindInfoReq => |p| aci_att_find_info_req(p.u16(), p.u16(), p.u16()));
    direct!(async AttFindByTypeValueReq<'a> => |p| aci_att_find_by_type_value_req(
        p.u16(), p.u16(), p.u16(), p.u16(), p.u8(), p.list(1)
    ));
    direct!(async AttReadByTypeReq => |p| aci_att_read_by_type_req(
        p.u16(), p.u16(), p.u16(), p.u8(), p.uuid().cast::<UUID_t>()
    ));
    direct!(async AttReadByGroupTypeReq => |p| aci_att_read_by_group_type_req(
        p.u16(), p.u16(), p.u16(), p.u8(), p.uuid().cast::<UUID_t>()
    ));
    direct!(async AttPrepareWriteReq<'a> => |p| aci_att_prepare_write_req(
        p.u16(), p.u16(), p.u16(), p.u8(), p.list(1)
    ));
    direct!(async AttExecuteWriteReq => |p| aci_att_execute_write_req(p.u16(), p.u8()));

    // L2CAP
    direct!(L2capConnectionParameterUpdateResp => |p| aci_l2cap_connection_parameter_update_resp(
        p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u8(), p.u8()
    ));
}

/// Commands of the full and basic plus stacks.
#[cfg(any(feature = "ble-stack-full", feature = "ble-stack-basic-plus"))]
mod basic_plus {
    use stm32wb_hci::aci::gap::*;
    use stm32wb_hci::aci::gatt::*;
    use stm32wb_hci::aci::l2cap::*;

    use super::super::*;

    // GAP
    direct!(GapAdvSetConfiguration => |p| aci_gap_adv_set_configuration(
        p.u8(), p.u8(), p.u16(), p.u32(), p.u32(), p.u8(), p.u8(), p.u8(), p.ptr(6), p.u8(), p.u8(), p.u8(), p.u8(),
        p.u8(), p.u8()
    ));
    direct!(GapAdvSetEnable<'a> => |p| aci_gap_adv_set_enable(p.u8(), p.u8(), p.list_of::<Adv_Set_t>()));
    direct!(GapAdvSetAdvData<'a> => |p| aci_gap_adv_set_adv_data(p.u8(), p.u8(), p.u8(), p.u8(), p.list(1)));
    direct!(GapAdvSetScanRespData<'a> => |p| aci_gap_adv_set_scan_resp_data(p.u8(), p.u8(), p.u8(), p.u8(), p.list(1)));
    direct!(GapAdvRemoveSet => |p| aci_gap_adv_remove_set(p.u8()));
    direct!(GapAdvClearSets => |p| aci_gap_adv_clear_sets());
    direct!(GapAdvSetRandomAddress => |p| aci_gap_adv_set_random_address(p.u8(), p.ptr(6)));
    direct!(GapAdvSetConfigurationV2 => |p| aci_gap_adv_set_configuration_v2(
        p.u8(), p.u8(), p.u16(), p.u32(), p.u32(), p.u8(), p.u8(), p.u8(), p.ptr(6), p.u8(), p.u8(), p.u8(), p.u8(),
        p.u8(), p.u8(), p.u8(), p.u8(), p.u8()
    ));
    direct!(async GapExtCreateConnection => |p| aci_gap_ext_create_connection(
        p.u8(), p.u8(), p.u8(), p.u8(), p.ptr(6), p.u8(), p.u8(), p.u8(), p.u8(), p.array_of::<Init_Param_Phy_t>(3)
    ));

    // GATT
    direct!(GattSendMultNotification<'a> => |p| aci_gatt_send_mult_notification(
        p.u16(), p.u8(), p.list_of::<Handle_Entry_t>()
    ));

    // L2CAP
    direct!(L2capCocConnect => |p| aci_l2cap_coc_connect(p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u8()));
    direct!(L2capCocConnectConfirm => |p, o| aci_l2cap_coc_connect_confirm(
        p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u8(), o.u8(), o.items(1, 250)
    ));
    direct!(L2capCocReconf<'a> => |p| aci_l2cap_coc_reconf(p.u16(), p.u16(), p.u16(), p.u8(), p.list(1)));
    direct!(L2capCocReconfConfirm => |p| aci_l2cap_coc_reconf_confirm(p.u16(), p.u16()));
    direct!(L2capCocDisconnect => |p| aci_l2cap_coc_disconnect(p.u8()));
    direct!(L2capCocFlowControl => |p| aci_l2cap_coc_flow_control(p.u8(), p.u16()));
    direct!(L2capCocTxData<'a> => |p| aci_l2cap_coc_tx_data(p.u8(), p.u16(), p.list(1)));
}

/// Commands of the full and link layer only stacks.
#[cfg(any(feature = "ble-stack-full", feature = "ble-stack-llo"))]
mod full_and_link_layer_only {
    use stm32wb_hci::aci::hal::*;
    use stm32wb_hci::standard::*;

    use super::super::*;

    // HAL
    direct!(HalSetSyncEventConfig => |p| aci_hal_set_sync_event_config(p.u8(), p.u8(), p.u8(), p.u8()));
    direct!(HalEadEncryptDecrypt<'a> => |p, o| aci_hal_ead_encrypt_decrypt(
        p.u8(), p.ptr(16), p.ptr(8), p.u16(), p.list(1), o.u16(), o.items(1, 249)
    ));

    // Core commands stm32wb-hci encodes differently from bt-hci
    direct!(LeSetExtScanParams => |p| hci_le_set_extended_scan_parameters(
        p.u8(), p.u8(), p.u8(), p.array_of::<Scan_Param_Phy_t>(2)
    ));
    direct!(async LeExtCreateConn => |p| hci_le_extended_create_connection(
        p.u8(), p.u8(), p.u8(), p.ptr(6), p.u8(), p.array_of::<Init_Param_Phy_t>(3)
    ));
    direct!(async LeExtCreateConnV2 => |p| hci_le_extended_create_connection_v2(
        p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.ptr(6), p.u8(), p.array_of::<Init_Param_Phy_t>(3)
    ));
}

/// Commands of the full stack.
#[cfg(feature = "ble-stack-full")]
mod full {
    use stm32wb_hci::aci::gap::*;
    use stm32wb_hci::aci::hal::*;

    use super::super::*;

    // GAP
    direct!(GapAdvSetPeriodicParameters => |p| aci_gap_adv_set_periodic_parameters(
        p.u8(), p.u16(), p.u16(), p.u16(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8()
    ));
    direct!(GapAdvSetPeriodicData<'a> => |p| aci_gap_adv_set_periodic_data(p.u8(), p.u8(), p.u8(), p.list(1)));
    direct!(GapAdvSetPeriodicEnable => |p| aci_gap_adv_set_periodic_enable(p.u8(), p.u8()));

    // HAL
    direct!(HalContinuousTxStart => |p| aci_hal_continuous_tx_start(p.u8(), p.u8(), p.u8()));
}
