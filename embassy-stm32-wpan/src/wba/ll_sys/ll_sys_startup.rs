use crate::bindings::link_layer::{
    LL_SYS_STATUS_T_LL_SYS_OK, ll_sys_assert, ll_sys_bg_process_init, ll_sys_config_params, ll_sys_dp_slp_init,
    ll_sys_status_t,
};
#[cfg(feature = "wba_ble")]
use crate::bindings::link_layer::{
    ble_buff_hdr_p, hci_dispatch_tbl, hci_get_dis_tbl, hst_cbk, ll_intf_init, ll_intf_rgstr_hst_cbk,
    ll_intf_rgstr_hst_cbk_ll_queue_full,
};
#[cfg(feature = "wba_mac")]
use crate::bindings::mac::ST_MAC_preInit;
// /**
//   ******************************************************************************
//   * @file    ll_sys_startup.c
//   * @author  MCD Application Team
//   * @brief   Link Layer IP system interface startup module
//   ******************************************************************************
//   * @attention
//   *
//   * Copyright (c) 2022 STMicroelectronics.
//   * All rights reserved.
//   *
//   * This software is licensed under terms that can be found in the LICENSE file
//   * in the root directory of this software component.
//   * If no LICENSE file comes with this software, it is provided AS-IS.
//   *
//   ******************************************************************************
//   */
//
// #include "ll_fw_config.h"
// #include "ll_sys.h"
// #include "ll_intf.h"
// #include "ll_sys_startup.h"
// #include "common_types.h"
// #if defined(MAC)
// #ifndef OPENTHREAD_CONFIG_FILE
// /* Projects with MAC Layer (i.e. 15.4 except Thread) */
// #include "st_mac_802_15_4_sap.h"
// #endif /* OPENTHREAD_CONFIG_FILE */
// #endif /* MAC */
//

#[allow(dead_code)]
/**
 * @brief  Missed HCI event flag
 */
static mut MISSED_HCI_EVENT_FLAG: u8 = 0;

// static void ll_sys_dependencies_init(void);
// #if SUPPORT_BLE

#[cfg(feature = "wba_ble")]
#[allow(dead_code)]
unsafe extern "C" fn ll_sys_event_missed_cb(_ptr_evnt_hdr: ble_buff_hdr_p) {
    MISSED_HCI_EVENT_FLAG = 1;
}

#[cfg(feature = "wba_ble")]
/**
 * @brief  Initialize the Link Layer IP BLE controller
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_ble_cntrl_init(host_callback: hst_cbk) {
    let p_hci_dis_tbl: *const hci_dispatch_tbl = core::ptr::null();

    hci_get_dis_tbl(&p_hci_dis_tbl as *const *const _ as *mut *const _);

    ll_intf_init(p_hci_dis_tbl);

    ll_intf_rgstr_hst_cbk(host_callback);

    ll_intf_rgstr_hst_cbk_ll_queue_full(Some(ll_sys_event_missed_cb));

    ll_sys_dependencies_init();
}
// #endif /* SUPPORT_BLE */
// #if defined(MAC)
// #ifndef OPENTHREAD_CONFIG_FILE
#[cfg(feature = "wba_mac")]
/**
 * @brief  Initialize the Link Layer IP 802.15.4 MAC controller
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_mac_cntrl_init() {
    ST_MAC_preInit();
    ll_sys_dependencies_init();
}
// #endif /* OPENTHREAD_CONFIG_FILE */
// #endif /* MAC */
/**
 * @brief  Start the Link Layer IP in OpenThread configuration
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_thread_init() {
    ll_sys_dependencies_init();
}

/**
 * @brief  Initialize the Link Layer resources for startup.
 *         This includes: - Deep Sleep feature resources
 *                        - Link Layer background task
 * @param  None
 * @retval None
 */
unsafe fn ll_sys_dependencies_init() {
    static mut IS_LL_INITIALIZED: u8 = 0;
    let dp_slp_status: ll_sys_status_t;

    /* Ensure Link Layer resources are created only once */
    if IS_LL_INITIALIZED == 1 {
        return;
    }
    IS_LL_INITIALIZED = 1;

    /* Deep sleep feature initialization */
    dp_slp_status = ll_sys_dp_slp_init();
    ll_sys_assert((dp_slp_status == LL_SYS_STATUS_T_LL_SYS_OK) as u8);

    /* Background task initialization */
    ll_sys_bg_process_init();

    /* Link Layer user parameters application */
    ll_sys_config_params();
}
