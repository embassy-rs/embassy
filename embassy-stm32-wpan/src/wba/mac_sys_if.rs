use crate::bindings::mac::mac_baremetal_run;
//
// /* USER CODE BEGIN Header */
// /**
//   ******************************************************************************
//   * @file    mac_sys_if.c
//   * @author  MCD Application Team
//   * @brief   Source file for using MAC Layer with a RTOS
//   ******************************************************************************
//   * @attention
//   *
//   * Copyright (c) 2025 STMicroelectronics.
//   * All rights reserved.
//   *
//   * This software is licensed under terms that can be found in the LICENSE file
//   * in the root directory of this software component.
//   * If no LICENSE file comes with this software, it is provided AS-IS.
//   *
//   ******************************************************************************
//   */
// /* USER CODE END Header */
//
// #include "main.h"
// #include "app_common.h"
// #include "app_conf.h"
// #include "log_module.h"
// #include "stm32_rtos.h"
// #include "st_mac_802_15_4_sys.h"
//
// extern void mac_baremetal_run(void);
//
// /* Private defines -----------------------------------------------------------*/
// /* USER CODE BEGIN PD */
//
// /* USER CODE END PD */
//
// /* Private macros ------------------------------------------------------------*/
// /* USER CODE BEGIN PM */
//
// /* USER CODE END PM */
//
// /* Private variables ---------------------------------------------------------*/
// /* USER CODE BEGIN PV */
//
// /* USER CODE END PV */
//
// /* Global variables ----------------------------------------------------------*/
// /* USER CODE BEGIN GV */
//
// /* USER CODE END GV */
//
// /* Functions Definition ------------------------------------------------------*/
//
// /**
//   * @brief  Mac Layer Initialisation
//   * @param  None
//   * @retval None
//   */
// void MacSys_Init(void)
// {
//   /* Register tasks */
//   UTIL_SEQ_RegTask( TASK_MAC_LAYER, UTIL_SEQ_RFU, mac_baremetal_run);
// }
//
// /**
//   * @brief  Mac Layer Resume
//   * @param  None
//   * @retval None
//   */
// void MacSys_Resume(void)
// {
//   UTIL_SEQ_ResumeTask( TASK_MAC_LAYER );
// }
//
// /**
//   * @brief  MAC Layer set Task.
//   * @param  None
//   * @retval None
//   */
// void MacSys_SemaphoreSet(void)
// {
//   UTIL_SEQ_SetTask( TASK_MAC_LAYER, TASK_PRIO_MAC_LAYER );
// }
//
// /**
//   * @brief  MAC Layer Task wait.
//   * @param  None
//   * @retval None
//   */
// void MacSys_SemaphoreWait( void )
// {
//   /* Not used */
// }
//
// /**
//   * @brief  MAC Layer set Event.
//   * @param  None
//   * @retval None
//   */
// void MacSys_EventSet( void )
// {
//   UTIL_SEQ_SetEvt( EVENT_MAC_LAYER );
// }
//
// /**
//   * @brief  MAC Layer wait Event.
//   * @param  None
//   * @retval None
//   */
// void MacSys_EventWait( void )
// {
//   UTIL_SEQ_WaitEvt( EVENT_MAC_LAYER );
// }
//

/**
 * @brief  Mac Layer Initialisation
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
pub extern "C" fn MacSys_Init() {
    unsafe {
        mac_baremetal_run();
    }
}

/**
 * @brief  Mac Layer Resume
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
pub extern "C" fn MacSys_Resume() {
    unsafe {
        mac_baremetal_run();
    }
}

/**
 * @brief  MAC Layer set Task.
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
pub extern "C" fn MacSys_SemaphoreSet() {
    unsafe {
        mac_baremetal_run();
    }
}

/**
 * @brief  MAC Layer Task wait.
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
pub extern "C" fn MacSys_SemaphoreWait() {
    unsafe {
        mac_baremetal_run();
    }
}

/**
 * @brief  MAC Layer set Event.
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
pub extern "C" fn MacSys_EventSet() {
    unsafe {
        mac_baremetal_run();
    }
}

/**
 * @brief  MAC Layer wait Event.
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
pub extern "C" fn MacSys_EventWait() {
    unsafe {
        mac_baremetal_run();
    }
}
