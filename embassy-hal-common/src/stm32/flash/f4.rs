const FLASH_BASE: u32 = 0x0800_0000;
pub(crate) const SMALL_SECTOR_SIZE: u32 = 16 * 1024;
pub(crate) const MEDIUM_SECTOR_SIZE: u32 = 64 * 1024;
pub(crate) const LARGE_SECTOR_SIZE: u32 = 128 * 1024;
pub const SECOND_BANK_SECTOR_OFFSET: u8 = 12;

#[derive(Debug, PartialEq)]
pub struct FlashSector {
    pub index: u8,
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
                ..sector
            }
        }
    }
}

fn get_single_bank_sector(offset: u32) -> FlashSector {
    // First 4 sectors are 16KB, then one 64KB, and rest are 128KB

    match offset / LARGE_SECTOR_SIZE {
        0 => {
            if offset < 4 * SMALL_SECTOR_SIZE {
                FlashSector {
                    index: (offset / SMALL_SECTOR_SIZE) as u8,
                    size: SMALL_SECTOR_SIZE,
                }
            } else {
                FlashSector {
                    index: 4,
                    size: MEDIUM_SECTOR_SIZE,
                }
            }
        }
        i => FlashSector {
            index: 4 + i as u8,
            size: LARGE_SECTOR_SIZE,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_sector_single_bank() {
        let assert_sector = |index: u8, size: u32, addr: u32| {
            assert_eq!(FlashSector { index, size }, get_sector(addr, false, 1024 * 1024))
        };

        assert_sector(0, SMALL_SECTOR_SIZE, 0x0800_0000);
        assert_sector(0, SMALL_SECTOR_SIZE, 0x0800_3FFF);
        assert_sector(3, SMALL_SECTOR_SIZE, 0x0800_C000);
        assert_sector(3, SMALL_SECTOR_SIZE, 0x0800_FFFF);

        assert_sector(4, MEDIUM_SECTOR_SIZE, 0x0801_0000);
        assert_sector(4, MEDIUM_SECTOR_SIZE, 0x0801_FFFF);

        assert_sector(5, LARGE_SECTOR_SIZE, 0x0802_0000);
        assert_sector(5, LARGE_SECTOR_SIZE, 0x0803_FFFF);
        assert_sector(11, LARGE_SECTOR_SIZE, 0x080E_0000);
        assert_sector(11, LARGE_SECTOR_SIZE, 0x080F_FFFF);
    }

    #[test]
    fn can_get_sector_dual_bank() {
        let assert_sector = |index: u8, size: u32, addr: u32| {
            assert_eq!(FlashSector { index, size }, get_sector(addr, true, 1024 * 1024))
        };

        assert_sector(0, SMALL_SECTOR_SIZE, 0x0800_0000);
        assert_sector(0, SMALL_SECTOR_SIZE, 0x0800_3FFF);
        assert_sector(3, SMALL_SECTOR_SIZE, 0x0800_C000);
        assert_sector(3, SMALL_SECTOR_SIZE, 0x0800_FFFF);

        assert_sector(4, MEDIUM_SECTOR_SIZE, 0x0801_0000);
        assert_sector(4, MEDIUM_SECTOR_SIZE, 0x0801_FFFF);

        assert_sector(5, LARGE_SECTOR_SIZE, 0x0802_0000);
        assert_sector(5, LARGE_SECTOR_SIZE, 0x0803_FFFF);
        assert_sector(7, LARGE_SECTOR_SIZE, 0x0806_0000);
        assert_sector(7, LARGE_SECTOR_SIZE, 0x0807_FFFF);

        assert_sector(12, SMALL_SECTOR_SIZE, 0x0808_0000);
        assert_sector(12, SMALL_SECTOR_SIZE, 0x0808_3FFF);
        assert_sector(15, SMALL_SECTOR_SIZE, 0x0808_C000);
        assert_sector(15, SMALL_SECTOR_SIZE, 0x0808_FFFF);

        assert_sector(16, MEDIUM_SECTOR_SIZE, 0x0809_0000);
        assert_sector(16, MEDIUM_SECTOR_SIZE, 0x0809_FFFF);

        assert_sector(17, LARGE_SECTOR_SIZE, 0x080A_0000);
        assert_sector(17, LARGE_SECTOR_SIZE, 0x080B_FFFF);
        assert_sector(19, LARGE_SECTOR_SIZE, 0x080E_0000);
        assert_sector(19, LARGE_SECTOR_SIZE, 0x080F_FFFF);
    }
}
