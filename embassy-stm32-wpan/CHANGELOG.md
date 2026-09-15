# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

- WBA BLE: apply the full `AdvData` payload when starting connectable advertising
  (manufacturer data, 128-bit UUIDs, service data and explicit flags/TX power were
  previously dropped by `aci_gap_set_discoverable`).
- WBA BLE: add directed advertising via `AdvParams::peer_addr` +
  `AdvType::ConnectableDirected{HIGH,LOW}Duty` (`aci_gap_set_direct_connectable`).
- WBA BLE: add `HCI::read_rssi` and `HCI::set_radio_activity_mask`
  (`RadioActivityMask`).
- WBA BLE: add `SecurityManager::passkey_input` + `PasskeyInputType`.
- WBA BLE: add `GattServer::add_descriptor` / `update_descriptor_value`
  (`DescriptorHandle`, `AttributeAccess`) and `GattServer::store_db`.
- refactor into wb55 crate and add feature for wba
- wpan: restructure hil and test wpan mac
- restructure to allow embassy net driver to work.
- First release with changelog.
