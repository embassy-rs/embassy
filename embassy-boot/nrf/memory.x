MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  FLASH                             : ORIGIN = 0x000f9000, LENGTH = 20K
  BOOTLOADER_STATE                  : ORIGIN = 0x000ff000, LENGTH = 4K
  RAM                         (rwx) : ORIGIN = 0x20000008, LENGTH = 0x2fff8
  uicr_bootloader_start_address (r) : ORIGIN = 0x10001014, LENGTH = 0x4
  uicr_mbr_params_page          (r) : ORIGIN = 0x10001018, LENGTH = 0x4
}

SECTIONS
{
  .bootloader_state :
  {
    KEEP(*(SORT(.bootloader_state*)))
  } > BOOTSTATE

  .uicr_bootloader_start_address :
  {
    KEEP(*(SORT(.uicr_bootloader_start_address*)))
  } > uicr_bootloader_start_address

  .uicr_mbr_params_page :
  {
    KEEP(*(SORT(.uicr_mbr_params_page*)))
  } > uicr_mbr_params_page
}
