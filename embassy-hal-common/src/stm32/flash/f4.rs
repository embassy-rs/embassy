const FLASH_BASE: u32 = 0x0800_0000;
pub(crate) const SMALL_SECTOR_SIZE: u32 = 16 * 1024;
pub(crate) const MEDIUM_SECTOR_SIZE: u32 = 64 * 1024;
pub(crate) const LARGE_SECTOR_SIZE: u32 = 128 * 1024;
pub const SECOND_BANK_SECTOR_OFFSET: u8 = 12;

#[derive(Debug, PartialEq)]
pub struct FlashSector {
    pub index: u8,
    pub start: u32,
    pub size: u32,
}

pub fn get_sector(address: u32, dual_bank: bool, flash_size: u32) -> FlashSector {
    let offset = address - FLASH_BASE;
    if !dual_bank {
        get_single_bank_sector(offset)
    } else {
        let bank_size = flash_size / 2;
        if offset < bank_size {
            get_single_bank_sector(offset)
        } else {
            let sector = get_single_bank_sector(offset - bank_size);
            FlashSector {
                index: SECOND_BANK_SECTOR_OFFSET + sector.index,
                start: sector.start + bank_size,
                size: sector.size,
            }
        }
    }
}

fn get_single_bank_sector(offset: u32) -> FlashSector {
    // First 4 sectors are 16KB, then one 64KB, and rest are 128KB
    match offset / LARGE_SECTOR_SIZE {
        0 => {
            if offset < 4 * SMALL_SECTOR_SIZE {
                let small_sector_index = offset / SMALL_SECTOR_SIZE;
                FlashSector {
                    index: small_sector_index as u8,
                    start: FLASH_BASE + small_sector_index * SMALL_SECTOR_SIZE,
                    size: SMALL_SECTOR_SIZE,
                }
            } else {
                FlashSector {
                    index: 4,
                    start: FLASH_BASE + 4 * SMALL_SECTOR_SIZE,
                    size: MEDIUM_SECTOR_SIZE,
                }
            }
        }
        i => {
            let large_sector_index = i - 1;
            FlashSector {
                index: (5 + large_sector_index) as u8,
                start: FLASH_BASE + 4 * SMALL_SECTOR_SIZE + MEDIUM_SECTOR_SIZE + large_sector_index * LARGE_SECTOR_SIZE,
                size: LARGE_SECTOR_SIZE,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_sector_single_bank() {
        let assert_sector = |index: u8, start: u32, size: u32, addr: u32| {
            assert_eq!(FlashSector { index, start, size }, get_sector(addr, false, 1024 * 1024))
        };

        assert_sector(0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_0000);
        assert_sector(0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_3FFF);
        assert_sector(3, 0x0800_C000, SMALL_SECTOR_SIZE, 0x0800_C000);
        assert_sector(3, 0x0800_C000, SMALL_SECTOR_SIZE, 0x0800_FFFF);

        assert_sector(4, 0x0801_0000, MEDIUM_SECTOR_SIZE, 0x0801_0000);
        assert_sector(4, 0x0801_0000, MEDIUM_SECTOR_SIZE, 0x0801_FFFF);

        assert_sector(5, 0x0802_0000, LARGE_SECTOR_SIZE, 0x0802_0000);
        assert_sector(5, 0x0802_0000, LARGE_SECTOR_SIZE, 0x0803_FFFF);
        assert_sector(11, 0x080E_0000, LARGE_SECTOR_SIZE, 0x080E_0000);
        assert_sector(11, 0x080E_0000, LARGE_SECTOR_SIZE, 0x080F_FFFF);
    }

    #[test]
    fn can_get_sector_dual_bank() {
        let assert_sector = |index: u8, start: u32, size: u32, addr: u32| {
            assert_eq!(FlashSector { index, start, size }, get_sector(addr, true, 1024 * 1024))
        };

        assert_sector(0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_0000);
        assert_sector(0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_3FFF);
        assert_sector(3, 0x0800_C000, SMALL_SECTOR_SIZE, 0x0800_C000);
        assert_sector(3, 0x0800_C000, SMALL_SECTOR_SIZE, 0x0800_FFFF);

        assert_sector(4, 0x0801_0000, MEDIUM_SECTOR_SIZE, 0x0801_0000);
        assert_sector(4, 0x0801_0000, MEDIUM_SECTOR_SIZE, 0x0801_FFFF);

        assert_sector(5, 0x0802_0000, LARGE_SECTOR_SIZE, 0x0802_0000);
        assert_sector(5, 0x0802_0000, LARGE_SECTOR_SIZE, 0x0803_FFFF);
        assert_sector(7, 0x0806_0000, LARGE_SECTOR_SIZE, 0x0806_0000);
        assert_sector(7, 0x0806_0000, LARGE_SECTOR_SIZE, 0x0807_FFFF);

        assert_sector(12, 0x0808_0000, SMALL_SECTOR_SIZE, 0x0808_0000);
        assert_sector(12, 0x0808_0000, SMALL_SECTOR_SIZE, 0x0808_3FFF);
        assert_sector(15, 0x0808_C000, SMALL_SECTOR_SIZE, 0x0808_C000);
        assert_sector(15, 0x0808_C000, SMALL_SECTOR_SIZE, 0x0808_FFFF);

        assert_sector(16, 0x0809_0000, MEDIUM_SECTOR_SIZE, 0x0809_0000);
        assert_sector(16, 0x0809_0000, MEDIUM_SECTOR_SIZE, 0x0809_FFFF);

        assert_sector(17, 0x080A_0000, LARGE_SECTOR_SIZE, 0x080A_0000);
        assert_sector(17, 0x080A_0000, LARGE_SECTOR_SIZE, 0x080B_FFFF);
        assert_sector(19, 0x080E_0000, LARGE_SECTOR_SIZE, 0x080E_0000);
        assert_sector(19, 0x080E_0000, LARGE_SECTOR_SIZE, 0x080F_FFFF);
    }
}
