const FLASH_BASE: u32 = 0x0800_0000;
pub(crate) const SMALL_SECTOR_SIZE: u32 = 32 * 1024;
pub(crate) const MEDIUM_SECTOR_SIZE: u32 = 128 * 1024;
pub(crate) const LARGE_SECTOR_SIZE: u32 = 256 * 1024;

#[derive(Debug, PartialEq)]
pub struct FlashSector {
    pub index: u8,
    pub size: u32,
}

pub fn get_sector(address: u32) -> FlashSector {
    // First 4 sectors are 32KB, then one 128KB, and rest are 256KB
    let offset = address - FLASH_BASE;
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
    fn can_get_sector() {
        let assert_sector = |index: u8, size: u32, addr: u32| assert_eq!(FlashSector { index, size }, get_sector(addr));

        assert_sector(0, SMALL_SECTOR_SIZE, 0x0800_0000);
        assert_sector(0, SMALL_SECTOR_SIZE, 0x0800_7FFF);
        assert_sector(3, SMALL_SECTOR_SIZE, 0x0801_8000);
        assert_sector(3, SMALL_SECTOR_SIZE, 0x0801_FFFF);

        assert_sector(4, MEDIUM_SECTOR_SIZE, 0x0802_0000);
        assert_sector(4, MEDIUM_SECTOR_SIZE, 0x0803_FFFF);

        assert_sector(5, LARGE_SECTOR_SIZE, 0x0804_0000);
        assert_sector(5, LARGE_SECTOR_SIZE, 0x0807_FFFF);
        assert_sector(7, LARGE_SECTOR_SIZE, 0x080C_0000);
        assert_sector(7, LARGE_SECTOR_SIZE, 0x080F_FFFF);
    }
}
