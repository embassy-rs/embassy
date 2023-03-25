const FLASH_BASE: u32 = 0x0800_0000;
pub(crate) const SMALL_SECTOR_SIZE: u32 = 32 * 1024;
pub(crate) const MEDIUM_SECTOR_SIZE: u32 = 128 * 1024;
pub(crate) const LARGE_SECTOR_SIZE: u32 = 256 * 1024;

#[derive(Debug, PartialEq)]
pub struct FlashSector {
    pub index: u8,
    pub start: u32,
    pub size: u32,
}

pub fn get_sector(address: u32) -> FlashSector {
    // First 4 sectors are 32KB, then one 128KB, and rest are 256KB
    let offset = address - FLASH_BASE;
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
    fn can_get_sector() {
        let assert_sector = |index: u8, start: u32, size: u32, addr: u32| {
            assert_eq!(FlashSector { index, start, size }, get_sector(addr))
        };

        assert_sector(0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_0000);
        assert_sector(0, 0x0800_0000, SMALL_SECTOR_SIZE, 0x0800_7FFF);
        assert_sector(3, 0x0801_8000, SMALL_SECTOR_SIZE, 0x0801_8000);
        assert_sector(3, 0x0801_8000, SMALL_SECTOR_SIZE, 0x0801_FFFF);

        assert_sector(4, 0x0802_0000, MEDIUM_SECTOR_SIZE, 0x0802_0000);
        assert_sector(4, 0x0802_0000, MEDIUM_SECTOR_SIZE, 0x0803_FFFF);

        assert_sector(5, 0x0804_0000, LARGE_SECTOR_SIZE, 0x0804_0000);
        assert_sector(5, 0x0804_0000, LARGE_SECTOR_SIZE, 0x0807_FFFF);
        assert_sector(7, 0x080C_0000, LARGE_SECTOR_SIZE, 0x080C_0000);
        assert_sector(7, 0x080C_0000, LARGE_SECTOR_SIZE, 0x080F_FFFF);
    }
}
