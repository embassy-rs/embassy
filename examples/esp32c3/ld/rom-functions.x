ets_printf = 0x40000040;
PROVIDE(esp_rom_printf = ets_printf);
PROVIDE(cache_invalidate_icache_all = 0x400004d8);
PROVIDE(cache_suspend_icache = 0x40000524);
PROVIDE(cache_resume_icache = 0x40000528);
PROVIDE(cache_ibus_mmu_set = 0x40000560);
PROVIDE(cache_dbus_mmu_set = 0x40000564);
PROVIDE(ets_delay_us = 0x40000050);
PROVIDE(ets_update_cpu_frequency_rom = 0x40000588);
PROVIDE(rom_i2c_writeReg = 0x4000195c);
PROVIDE(rom_i2c_writeReg_Mask = 0x40001960);
PROVIDE(rtc_get_reset_reason = 0x40000018);
PROVIDE(software_reset = 0x40000090);
PROVIDE(software_reset_cpu = 0x40000094);

PROVIDE(esp_rom_crc32_be = 0x4000062c);
PROVIDE(esp_rom_crc16_be = 0x40000634);
PROVIDE(esp_rom_crc8_be = 0x4000063c);
PROVIDE(esp_rom_crc32_le = 0x40000628);
PROVIDE(esp_rom_crc16_le = 0x40000630);
PROVIDE(esp_rom_crc8_le = 0x40000638);

PROVIDE(esp_rom_md5_init = 0x40000614);
PROVIDE(esp_rom_md5_update = 0x40000618);
PROVIDE(esp_rom_md5_final = 0x4000061c);
