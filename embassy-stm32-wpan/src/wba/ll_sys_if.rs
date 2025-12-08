#![cfg(feature = "wba")]
// /* USER CODE BEGIN Header */
// /**
//   ******************************************************************************
//   * @file    ll_sys_if.c
//   * @author  MCD Application Team
//   * @brief   Source file for initiating system
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
// /* USER CODE END Header */
//
// #include "main.h"
// #include "app_common.h"
// #include "app_conf.h"
// #include "log_module.h"
// #include "ll_intf_cmn.h"
// #include "ll_sys.h"
// #include "ll_sys_if.h"
// #include "stm32_rtos.h"
// #include "utilities_common.h"
// #if (USE_TEMPERATURE_BASED_RADIO_CALIBRATION == 1)
// #include "temp_measurement.h"
// #endif /* (USE_TEMPERATURE_BASED_RADIO_CALIBRATION == 1) */
// #if (CFG_LPM_STANDBY_SUPPORTED == 0)
// extern void profile_reset(void);
// #endif
// /* Private defines -----------------------------------------------------------*/
// /* Radio event scheduling method - must be set at 1 */
// #define USE_RADIO_LOW_ISR                   (1)
// #define NEXT_EVENT_SCHEDULING_FROM_ISR      (1)
//
// /* USER CODE BEGIN PD */
//
// /* USER CODE END PD */
//
// /* Private macros ------------------------------------------------------------*/
// /* USER CODE BEGIN PM */
//
// /* USER CODE END PM */
//
// /* Private constants ---------------------------------------------------------*/
// /* USER CODE BEGIN PC */
//
// /* USER CODE END PC */
//
// /* Private variables ---------------------------------------------------------*/
// /* USER CODE BEGIN PV */
//
// /* USER CODE END PV */
//
// /* Global variables ----------------------------------------------------------*/
//
// /* USER CODE BEGIN GV */
//
// /* USER CODE END GV */
//
// /* Private functions prototypes-----------------------------------------------*/
// #if (USE_TEMPERATURE_BASED_RADIO_CALIBRATION == 1)
// static void ll_sys_bg_temperature_measurement_init(void);
// #endif /* USE_TEMPERATURE_BASED_RADIO_CALIBRATION */
// static void ll_sys_sleep_clock_source_selection(void);
// static uint8_t ll_sys_BLE_sleep_clock_accuracy_selection(void);
// void ll_sys_reset(void);
//
// /* USER CODE BEGIN PFP */
//
// /* USER CODE END PFP */
//
// /* External variables --------------------------------------------------------*/
//
// /* USER CODE BEGIN EV */
//
// /* USER CODE END EV */
//
// /* Functions Definition ------------------------------------------------------*/
//
// /**
//   * @brief  Link Layer background process initialization
//   * @param  None
//   * @retval None
//   */
// void ll_sys_bg_process_init(void)
// {
//   /* Register Link Layer task */
//   UTIL_SEQ_RegTask(1U << CFG_TASK_LINK_LAYER, UTIL_SEQ_RFU, ll_sys_bg_process);
// }
//
// /**
//   * @brief  Link Layer background process next iteration scheduling
//   * @param  None
//   * @retval None
//   */
// void ll_sys_schedule_bg_process(void)
// {
//   UTIL_SEQ_SetTask(1U << CFG_TASK_LINK_LAYER, TASK_PRIO_LINK_LAYER);
// }
//
// /**
//   * @brief  Link Layer background process next iteration scheduling from ISR
//   * @param  None
//   * @retval None
//   */
// void ll_sys_schedule_bg_process_isr(void)
// {
//   UTIL_SEQ_SetTask(1U << CFG_TASK_LINK_LAYER, TASK_PRIO_LINK_LAYER);
// }
//
// /**
//   * @brief  Link Layer configuration phase before application startup.
//   * @param  None
//   * @retval None
//   */
// void ll_sys_config_params(void)
// {
// /* USER CODE BEGIN ll_sys_config_params_0 */
//
// /* USER CODE END ll_sys_config_params_0 */
//
//   /* Configure link layer behavior for low ISR use and next event scheduling method:
//    * - SW low ISR is used.
//    * - Next event is scheduled from ISR.
//    */
//   ll_intf_cmn_config_ll_ctx_params(USE_RADIO_LOW_ISR, NEXT_EVENT_SCHEDULING_FROM_ISR);
//   /* Apply the selected link layer sleep timer source */
//   ll_sys_sleep_clock_source_selection();
//
// /* USER CODE BEGIN ll_sys_config_params_1 */
//
// /* USER CODE END ll_sys_config_params_1 */
//
// #if (USE_TEMPERATURE_BASED_RADIO_CALIBRATION == 1)
//   /* Initialize link layer temperature measurement background task */
//   ll_sys_bg_temperature_measurement_init();
//
//   /* Link layer IP uses temperature based calibration instead of periodic one */
//   ll_intf_cmn_set_temperature_sensor_state();
// #endif /* USE_TEMPERATURE_BASED_RADIO_CALIBRATION */
//
//   /* Link Layer power table */
//   ll_intf_cmn_select_tx_power_table(CFG_RF_TX_POWER_TABLE_ID);
//
// #if (USE_CTE_DEGRADATION == 1u)
//   /* Apply CTE degradation */
//   ll_sys_apply_cte_settings ();
// #endif /* (USE_CTE_DEGRADATION == 1u) */
//
// /* USER CODE BEGIN ll_sys_config_params_2 */
//
// /* USER CODE END ll_sys_config_params_2 */
// }
//
// #if (USE_TEMPERATURE_BASED_RADIO_CALIBRATION == 1)
//
// /**
//   * @brief  Link Layer temperature request background process initialization
//   * @param  None
//   * @retval None
//   */
// void ll_sys_bg_temperature_measurement_init(void)
// {
//   /* Register Temperature Measurement task */
//   UTIL_SEQ_RegTask(1U << CFG_TASK_TEMP_MEAS, UTIL_SEQ_RFU, TEMPMEAS_RequestTemperatureMeasurement);
// }
//
// /**
//   * @brief  Request backroud task processing for temperature measurement
//   * @param  None
//   * @retval None
//   */
// void ll_sys_bg_temperature_measurement(void)
// {
//   static uint8_t initial_temperature_acquisition = 0;
//
//   if(initial_temperature_acquisition == 0)
//   {
//     TEMPMEAS_RequestTemperatureMeasurement();
//     initial_temperature_acquisition = 1;
//   }
//   else
//   {
//     UTIL_SEQ_SetTask(1U << CFG_TASK_TEMP_MEAS, CFG_SEQ_PRIO_0);
//   }
// }
//
// #endif /* USE_TEMPERATURE_BASED_RADIO_CALIBRATION */
//
// uint8_t ll_sys_BLE_sleep_clock_accuracy_selection(void)
// {
//   uint8_t BLE_sleep_clock_accuracy = 0;
// #if (CFG_RADIO_LSE_SLEEP_TIMER_CUSTOM_SCA_RANGE == 0)
//   uint32_t RevID = LL_DBGMCU_GetRevisionID();
// #endif
//   uint32_t linklayer_slp_clk_src = LL_RCC_RADIO_GetSleepTimerClockSource();
//
//   if(linklayer_slp_clk_src == LL_RCC_RADIOSLEEPSOURCE_LSE)
//   {
//     /* LSE selected as Link Layer sleep clock source.
//        Sleep clock accuracy is different regarding the WBA device ID and revision
//      */
// #if (CFG_RADIO_LSE_SLEEP_TIMER_CUSTOM_SCA_RANGE == 0)
// #if defined(STM32WBA52xx) || defined(STM32WBA54xx) || defined(STM32WBA55xx)
//     if(RevID == REV_ID_A)
//     {
//       BLE_sleep_clock_accuracy = STM32WBA5x_REV_ID_A_SCA_RANGE;
//     }
//     else if(RevID == REV_ID_B)
//     {
//       BLE_sleep_clock_accuracy = STM32WBA5x_REV_ID_B_SCA_RANGE;
//     }
//     else
//     {
//       /* Revision ID not supported, default value of 500ppm applied */
//       BLE_sleep_clock_accuracy = STM32WBA5x_DEFAULT_SCA_RANGE;
//     }
// #elif defined(STM32WBA65xx)
//     BLE_sleep_clock_accuracy = STM32WBA6x_SCA_RANGE;
//     UNUSED(RevID);
// #else
//     UNUSED(RevID);
// #endif /* defined(STM32WBA52xx) || defined(STM32WBA54xx) || defined(STM32WBA55xx) */
// #else /* CFG_RADIO_LSE_SLEEP_TIMER_CUSTOM_SCA_RANGE */
//     BLE_sleep_clock_accuracy = CFG_RADIO_LSE_SLEEP_TIMER_CUSTOM_SCA_RANGE;
// #endif /* CFG_RADIO_LSE_SLEEP_TIMER_CUSTOM_SCA_RANGE */
//   }
//   else
//   {
//     /* LSE is not the Link Layer sleep clock source, sleep clock accurcay default value is 500 ppm */
//     BLE_sleep_clock_accuracy = STM32WBA5x_DEFAULT_SCA_RANGE;
//   }
//
//   return BLE_sleep_clock_accuracy;
// }
//
// void ll_sys_sleep_clock_source_selection(void)
// {
//   uint16_t freq_value = 0;
//   uint32_t linklayer_slp_clk_src = LL_RCC_RADIOSLEEPSOURCE_NONE;
//
//   linklayer_slp_clk_src = LL_RCC_RADIO_GetSleepTimerClockSource();
//   switch(linklayer_slp_clk_src)
//   {
//     case LL_RCC_RADIOSLEEPSOURCE_LSE:
//       linklayer_slp_clk_src = RTC_SLPTMR;
//       break;
//
//     case LL_RCC_RADIOSLEEPSOURCE_LSI:
//       linklayer_slp_clk_src = RCO_SLPTMR;
//       break;
//
//     case LL_RCC_RADIOSLEEPSOURCE_HSE_DIV1000:
//       linklayer_slp_clk_src = CRYSTAL_OSCILLATOR_SLPTMR;
//       break;
//
//     case LL_RCC_RADIOSLEEPSOURCE_NONE:
//       /* No Link Layer sleep clock source selected */
//       assert_param(0);
//       break;
//   }
//   ll_intf_cmn_le_select_slp_clk_src((uint8_t)linklayer_slp_clk_src, &freq_value);
// }
//
// void ll_sys_reset(void)
// {
//   uint8_t bsca = 0;
//   /* Link layer timings */
//   uint8_t drift_time = DRIFT_TIME_DEFAULT;
//   uint8_t exec_time = EXEC_TIME_DEFAULT;
//
// /* USER CODE BEGIN ll_sys_reset_0 */
//
// /* USER CODE END ll_sys_reset_0 */
//
//   /* Apply the selected link layer sleep timer source */
//   ll_sys_sleep_clock_source_selection();
//
//   /* Configure the link layer sleep clock accuracy */
//   bsca = ll_sys_BLE_sleep_clock_accuracy_selection();
//   ll_intf_le_set_sleep_clock_accuracy(bsca);
//
//   /* Update link layer timings depending on selected configuration */
//   if(LL_RCC_RADIO_GetSleepTimerClockSource() == LL_RCC_RADIOSLEEPSOURCE_LSI)
//   {
//     drift_time += DRIFT_TIME_EXTRA_LSI2;
//     exec_time += EXEC_TIME_EXTRA_LSI2;
//   }
//   else
//   {
// #if defined(__GNUC__) && defined(DEBUG)
//     drift_time += DRIFT_TIME_EXTRA_GCC_DEBUG;
//     exec_time += EXEC_TIME_EXTRA_GCC_DEBUG;
// #endif
//   }
//
//   /* USER CODE BEGIN ll_sys_reset_1 */
//
//   /* USER CODE END ll_sys_reset_1 */
//
//   if((drift_time != DRIFT_TIME_DEFAULT) || (exec_time != EXEC_TIME_DEFAULT))
//   {
//     ll_sys_config_BLE_schldr_timings(drift_time, exec_time);
//   }
//   /* USER CODE BEGIN ll_sys_reset_2 */
//
//   /* USER CODE END ll_sys_reset_2 */
// }
// #if defined(STM32WBA52xx) || defined(STM32WBA54xx) || defined(STM32WBA55xx) || defined(STM32WBA65xx)
// void ll_sys_apply_cte_settings(void)
// {
//   ll_intf_apply_cte_degrad_change();
// }
// #endif /* defined(STM32WBA52xx) || defined(STM32WBA54xx) || defined(STM32WBA55xx) || defined(STM32WBA65xx) */
//
// #if (CFG_LPM_STANDBY_SUPPORTED == 0)
// void ll_sys_get_ble_profile_statistics(uint32_t* exec_time, uint32_t* drift_time, uint32_t* average_drift_time, uint8_t reset)
// {
//   if (reset != 0U)
//   {
//     profile_reset();
//   }
//   ll_intf_get_profile_statistics(exec_time, drift_time, average_drift_time);
// }
// #endif
//
use super::bindings::{link_layer, mac};
use super::util_seq;

const UTIL_SEQ_RFU: u32 = 0;
const TASK_LINK_LAYER_MASK: u32 = 1 << mac::CFG_TASK_ID_T_CFG_TASK_LINK_LAYER;
const TASK_PRIO_LINK_LAYER: u32 = mac::CFG_SEQ_PRIO_ID_T_CFG_SEQ_PRIO_0 as u32;

/**
 * @brief  Link Layer background process initialization
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ll_sys_bg_process_init() {
    util_seq::UTIL_SEQ_RegTask(TASK_LINK_LAYER_MASK, UTIL_SEQ_RFU, Some(link_layer::ll_sys_bg_process));
}

/**
 * @brief  Link Layer background process next iteration scheduling
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ll_sys_schedule_bg_process() {
    util_seq::UTIL_SEQ_SetTask(TASK_LINK_LAYER_MASK, TASK_PRIO_LINK_LAYER);
}

/**
 * @brief  Link Layer background process next iteration scheduling from ISR
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ll_sys_schedule_bg_process_isr() {
    util_seq::UTIL_SEQ_SetTask(TASK_LINK_LAYER_MASK, TASK_PRIO_LINK_LAYER);
}

/**
 * @brief  Link Layer configuration phase before application startup.
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ll_sys_config_params() {
    let allow_low_isr = mac::USE_RADIO_LOW_ISR as u8;
    let run_from_isr = mac::NEXT_EVENT_SCHEDULING_FROM_ISR as u8;
    let _ = link_layer::ll_intf_cmn_config_ll_ctx_params(allow_low_isr, run_from_isr);

    ll_sys_sleep_clock_source_selection();
    let _ = link_layer::ll_intf_cmn_select_tx_power_table(mac::CFG_RF_TX_POWER_TABLE_ID as u8);
}

/**
 * @brief  Reset Link Layer timing parameters to their default configuration.
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ll_sys_reset() {
    ll_sys_sleep_clock_source_selection();

    let sleep_accuracy = ll_sys_BLE_sleep_clock_accuracy_selection();
    let _ = link_layer::ll_intf_le_set_sleep_clock_accuracy(sleep_accuracy);
}

/// Select the sleep-clock source used by the Link Layer.
/// Defaults to the crystal oscillator when no explicit configuration is available.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ll_sys_sleep_clock_source_selection() {
    let mut frequency: u16 = 0;
    let _ = link_layer::ll_intf_cmn_le_select_slp_clk_src(
        link_layer::_SLPTMR_SRC_TYPE_E_CRYSTAL_OSCILLATOR_SLPTMR as u8,
        &mut frequency as *mut u16,
    );
}

/// Determine the BLE sleep-clock accuracy used by the stack.
/// Returns zero when board-specific calibration data is unavailable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ll_sys_BLE_sleep_clock_accuracy_selection() -> u8 {
    // TODO: derive the board-specific sleep clock accuracy once calibration data is available.
    0
}
