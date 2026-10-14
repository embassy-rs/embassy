MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
    /* blinky_ram executes from the main SRAM banks after boot2 copies it. */
    FLASH : ORIGIN = DEFINED(__run_from_ram) ? 0x20000000 : 0x10000100,
            LENGTH = DEFINED(__run_from_ram) ? 256K : 2048K - 0x100

    /* Pick one of the two options for RAM layout     */

    /* OPTION A: Use all RAM banks as one big block   */
    /* Reasonable, unless you are doing something     */
    /* really particular with DMA or other concurrent */
    /* access that would benefit from striping        */
    /* Its writable data and stack use the 8 KiB scratch banks. */
    RAM   : ORIGIN = DEFINED(__run_from_ram) ? 0x20040000 : 0x20000000,
            LENGTH = DEFINED(__run_from_ram) ? 8K : 264K

    /* OPTION B: Keep the unstriped sections separate */
    /* RAM: ORIGIN = 0x20000000, LENGTH = 256K        */
    /* SCRATCH_A: ORIGIN = 0x20040000, LENGTH = 4K    */
    /* SCRATCH_B: ORIGIN = 0x20041000, LENGTH = 4K    */
}
