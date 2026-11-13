/* EFM32GG390F1024 on the Bosch XDK110, for its USB bootloader.
 *
 * The bootloader uses the first 64 KiB of flash. An application starts at
 * 0x00010000 (XDK110 General Information Guide, revision 2.0, table 12,
 * command `u`). RAM is the full 128 KiB of the chip.
 */
MEMORY
{
  FLASH : ORIGIN = 0x00010000, LENGTH = 960K
  RAM : ORIGIN = 0x20000000, LENGTH = 128K
}
