use crate::bindings::link_layer::{
    _NULL as NULL, DPSLP_STATE_DEEP_SLEEP_DISABLE, DPSLP_STATE_DEEP_SLEEP_ENABLE, LINKLAYER_PLAT_DisableRadioIT,
    LINKLAYER_PLAT_EnableRadioIT, LL_SYS_DP_SLP_STATE_T_LL_SYS_DP_SLP_DISABLED,
    LL_SYS_DP_SLP_STATE_T_LL_SYS_DP_SLP_ENABLED, LL_SYS_STATUS_T_LL_SYS_ERROR, LL_SYS_STATUS_T_LL_SYS_OK,
    OS_TIMER_PRIO_HG_PRIO_TMR, OS_TIMER_STATE_OSTIMERSTOPPED, OS_TIMER_TYPE_OS_TIMER_ONCE, SUCCESS, ble_stat_t,
    ll_intf_cmn_le_set_dp_slp_mode, ll_sys_dp_slp_state_t, ll_sys_status_t, os_get_tmr_state, os_timer_create,
    os_timer_id, os_timer_set_prio, os_timer_start, os_timer_stop,
};

macro_rules! LL_DP_SLP_NO_WAKEUP {
    () => {
        !0u32
    };
}

macro_rules! LL_INTERNAL_TMR_US_TO_STEPS {
    ($us:expr) => {
        ((($us) * 4) / 125)
    };
}

// /**
//   ******************************************************************************
//   * @file    ll_sys_dp_slp.c
//   * @author  MCD Application Team
//   * @brief   Link Layer IP system interface deep sleep management
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
// #include "linklayer_plat.h"
// #include "ll_sys.h"
// #include "ll_intf_cmn.h"
//
// /* Link Layer deep sleep timer */
static mut RADIO_DP_SLP_TMR_ID: os_timer_id = NULL as *mut _;
//
// /* Link Layer deep sleep state */
static mut LINKLAYER_DP_SLP_STATE: ll_sys_dp_slp_state_t = LL_SYS_DP_SLP_STATE_T_LL_SYS_DP_SLP_DISABLED;
//
// /**
//   * @brief  Initialize resources to handle deep sleep entry/exit
//   * @param  None
//   * @retval LL_SYS status
//   */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_dp_slp_init() -> ll_sys_status_t {
    let mut return_status: ll_sys_status_t = LL_SYS_STATUS_T_LL_SYS_ERROR;

    /* Create link layer timer for handling IP DEEP SLEEP mode */
    RADIO_DP_SLP_TMR_ID = os_timer_create(
        Some(ll_sys_dp_slp_wakeup_evt_clbk),
        OS_TIMER_TYPE_OS_TIMER_ONCE,
        NULL as *mut _,
    );

    /* Set priority of deep sleep timer */
    os_timer_set_prio(RADIO_DP_SLP_TMR_ID, OS_TIMER_PRIO_HG_PRIO_TMR);

    if RADIO_DP_SLP_TMR_ID != NULL as *mut _ {
        return_status = LL_SYS_STATUS_T_LL_SYS_OK;
    }

    return return_status;
}
//
// /**
//   * @brief  Link Layer deep sleep status getter
//   * @param  None
//   * @retval Link Layer deep sleep state
//   */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_dp_slp_get_state() -> ll_sys_dp_slp_state_t {
    return LINKLAYER_DP_SLP_STATE;
}
//
// /**
//   * @brief  The Link Layer IP enters deep sleep mode
//   * @param  dp_slp_duration    deep sleep duration in us
//   * @retval LL_SYS status
//   */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_dp_slp_enter(dp_slp_duration: u32) -> ll_sys_status_t {
    let cmd_status: ble_stat_t;
    let os_status: i32;
    let mut return_status: ll_sys_status_t = LL_SYS_STATUS_T_LL_SYS_ERROR;

    /* Check if deep sleep timer has to be started */
    if dp_slp_duration < LL_DP_SLP_NO_WAKEUP!() {
        /* Start deep sleep timer */
        os_status = os_timer_start(RADIO_DP_SLP_TMR_ID, LL_INTERNAL_TMR_US_TO_STEPS!(dp_slp_duration));
    } else {
        /* No timer started */
        os_status = SUCCESS as i32;
    }

    if os_status == SUCCESS as i32 {
        /* Switch Link Layer IP to DEEP SLEEP mode */
        cmd_status = ll_intf_cmn_le_set_dp_slp_mode(DPSLP_STATE_DEEP_SLEEP_ENABLE as u8);
        if cmd_status == SUCCESS {
            LINKLAYER_DP_SLP_STATE = LL_SYS_DP_SLP_STATE_T_LL_SYS_DP_SLP_ENABLED;
            return_status = LL_SYS_STATUS_T_LL_SYS_OK;
        }
    }

    return return_status;
}
//
// /**
//   * @brief  The Link Layer IP exits deep sleep mode
//   * @param  None
//   * @retval LL_SYS status
//   */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_dp_slp_exit() -> ll_sys_status_t {
    let cmd_status: ble_stat_t;
    let mut return_status: ll_sys_status_t = LL_SYS_STATUS_T_LL_SYS_ERROR;

    /* Disable radio interrupt */
    LINKLAYER_PLAT_DisableRadioIT();

    if LINKLAYER_DP_SLP_STATE == LL_SYS_DP_SLP_STATE_T_LL_SYS_DP_SLP_DISABLED {
        /* Radio not in sleep mode */
        return_status = LL_SYS_STATUS_T_LL_SYS_OK;
    } else {
        /* Switch Link Layer IP to SLEEP mode (by deactivate DEEP SLEEP mode) */
        cmd_status = ll_intf_cmn_le_set_dp_slp_mode(DPSLP_STATE_DEEP_SLEEP_DISABLE as u8);
        if cmd_status == SUCCESS {
            LINKLAYER_DP_SLP_STATE = LL_SYS_DP_SLP_STATE_T_LL_SYS_DP_SLP_DISABLED;
            return_status = LL_SYS_STATUS_T_LL_SYS_OK;
        }

        /* Stop the deep sleep wake-up timer if running */
        if os_get_tmr_state(RADIO_DP_SLP_TMR_ID) != OS_TIMER_STATE_OSTIMERSTOPPED {
            os_timer_stop(RADIO_DP_SLP_TMR_ID);
        }
    }

    /* Re-enable radio interrupt */
    LINKLAYER_PLAT_EnableRadioIT();

    return return_status;
}

/**
 * @brief  Link Layer deep sleep wake-up timer callback
 * @param  ptr_arg    pointer passed through the callback
 * @retval LL_SYS status
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_dp_slp_wakeup_evt_clbk(_ptr_arg: *const ::core::ffi::c_void) {
    /* Link Layer IP exits from DEEP SLEEP mode */
    ll_sys_dp_slp_exit();
}
