MEMORY
{
    /* RM "Table 16. System memory map": 0x0000_0000/0x2000_0000 are the
     * Non-Secure flash/RAM aliases. SYSCON0, SCG0 and other peripherals
     * default to Secure-only access (AHB Secure Controller), so Non-Secure
     * code faults touching them. The examples run entirely as Secure code,
     * through the Secure aliases at +0x1000_0000.
     */
    FLASH : ORIGIN = 0x10000000, LENGTH = 2M
    RAM   : ORIGIN = 0x30000000, LENGTH = 256K
}
