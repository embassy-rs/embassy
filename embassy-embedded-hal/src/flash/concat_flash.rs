use embedded_storage::nor_flash::{ErrorType, NorFlash, NorFlashError, ReadNorFlash};
#[cfg(feature = "nightly")]
use embedded_storage_async::nor_flash::{NorFlash as AsyncNorFlash, ReadNorFlash as AsyncReadNorFlash};

/// Convenience helper for concatenating two consecutive flashes into one.
/// This is especially useful if used with "flash regions", where one may
/// want to concatenate multiple regions into one larger region.
pub struct ConcatFlash<First, Second>(First, Second);

impl<First, Second> ConcatFlash<First, Second> {
    /// Create a new flash that concatenates two consecutive flashes.
    pub fn new(first: First, second: Second) -> Self {
        Self(first, second)
    }
}

const fn get_read_size(first_read_size: usize, second_read_size: usize) -> usize {
    if first_read_size != second_read_size {
        panic!("The read size for the concatenated flashes must be the same");
    }
    first_read_size
}

const fn get_write_size(first_write_size: usize, second_write_size: usize) -> usize {
    if first_write_size != second_write_size {
        panic!("The write size for the concatenated flashes must be the same");
    }
    first_write_size
}

const fn get_max_erase_size(first_erase_size: usize, second_erase_size: usize) -> usize {
    let max_erase_size = if first_erase_size > second_erase_size {
        first_erase_size
    } else {
        second_erase_size
    };
    if max_erase_size % first_erase_size != 0 || max_erase_size % second_erase_size != 0 {
        panic!("The erase sizes for the concatenated flashes must have have a gcd equal to the max erase size");
    }
    max_erase_size
}

impl<First, Second, E> ErrorType for ConcatFlash<First, Second>
where
    First: ErrorType<Error = E>,
    Second: ErrorType<Error = E>,
    E: NorFlashError,
{
    type Error = E;
}

impl<First, Second, E> ReadNorFlash for ConcatFlash<First, Second>
where
    First: ReadNorFlash<Error = E>,
    Second: ReadNorFlash<Error = E>,
    E: NorFlashError,
{
    const READ_SIZE: usize = get_read_size(First::READ_SIZE, Second::READ_SIZE);

    fn read(&mut self, mut offset: u32, mut bytes: &mut [u8]) -> Result<(), E> {
        if offset < self.0.capacity() as u32 {
            let len = core::cmp::min(self.0.capacity() - offset as usize, bytes.len());
            self.0.read(offset, &mut bytes[..len])?;
            offset += len as u32;
            bytes = &mut bytes[len..];
        }

        if !bytes.is_empty() {
            self.1.read(offset - self.0.capacity() as u32, bytes)?;
        }

        Ok(())
    }

    fn capacity(&self) -> usize {
        self.0.capacity() + self.1.capacity()
    }
}

impl<First, Second, E> NorFlash for ConcatFlash<First, Second>
where
    First: NorFlash<Error = E>,
    Second: NorFlash<Error = E>,
    E: NorFlashError,
{
    const WRITE_SIZE: usize = get_write_size(First::WRITE_SIZE, Second::WRITE_SIZE);
    const ERASE_SIZE: usize = get_max_erase_size(First::ERASE_SIZE, Second::ERASE_SIZE);

    fn write(&mut self, mut offset: u32, mut bytes: &[u8]) -> Result<(), E> {
        if offset < self.0.capacity() as u32 {
            let len = core::cmp::min(self.0.capacity() - offset as usize, bytes.len());
            self.0.write(offset, &bytes[..len])?;
            offset += len as u32;
            bytes = &bytes[len..];
        }

        if !bytes.is_empty() {
            self.1.write(offset - self.0.capacity() as u32, bytes)?;
        }

        Ok(())
    }

    fn erase(&mut self, mut from: u32, to: u32) -> Result<(), E> {
        if from < self.0.capacity() as u32 {
            let to = core::cmp::min(self.0.capacity() as u32, to);
            self.0.erase(from, to)?;
            from = self.0.capacity() as u32;
        }

        if from < to {
            self.1
                .erase(from - self.0.capacity() as u32, to - self.0.capacity() as u32)?;
        }

        Ok(())
    }
}

#[cfg(feature = "nightly")]
impl<First, Second, E> AsyncReadNorFlash for ConcatFlash<First, Second>
where
    First: AsyncReadNorFlash<Error = E>,
    Second: AsyncReadNorFlash<Error = E>,
    E: NorFlashError,
{
    const READ_SIZE: usize = get_read_size(First::READ_SIZE, Second::READ_SIZE);

    async fn read(&mut self, mut offset: u32, mut bytes: &mut [u8]) -> Result<(), E> {
        if offset < self.0.capacity() as u32 {
            let len = core::cmp::min(self.0.capacity() - offset as usize, bytes.len());
            self.0.read(offset, &mut bytes[..len]).await?;
            offset += len as u32;
            bytes = &mut bytes[len..];
        }

        if !bytes.is_empty() {
            self.1.read(offset - self.0.capacity() as u32, bytes).await?;
        }

        Ok(())
    }

    fn capacity(&self) -> usize {
        self.0.capacity() + self.1.capacity()
    }
}

#[cfg(feature = "nightly")]
impl<First, Second, E> AsyncNorFlash for ConcatFlash<First, Second>
where
    First: AsyncNorFlash<Error = E>,
    Second: AsyncNorFlash<Error = E>,
    E: NorFlashError,
{
    const WRITE_SIZE: usize = get_write_size(First::WRITE_SIZE, Second::WRITE_SIZE);
    const ERASE_SIZE: usize = get_max_erase_size(First::ERASE_SIZE, Second::ERASE_SIZE);

    async fn write(&mut self, mut offset: u32, mut bytes: &[u8]) -> Result<(), E> {
        if offset < self.0.capacity() as u32 {
            let len = core::cmp::min(self.0.capacity() - offset as usize, bytes.len());
            self.0.write(offset, &bytes[..len]).await?;
            offset += len as u32;
            bytes = &bytes[len..];
        }

        if !bytes.is_empty() {
            self.1.write(offset - self.0.capacity() as u32, bytes).await?;
        }

        Ok(())
    }

    async fn erase(&mut self, mut from: u32, to: u32) -> Result<(), E> {
        if from < self.0.capacity() as u32 {
            let to = core::cmp::min(self.0.capacity() as u32, to);
            self.0.erase(from, to).await?;
            from = self.0.capacity() as u32;
        }

        if from < to {
            self.1
                .erase(from - self.0.capacity() as u32, to - self.0.capacity() as u32)
                .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};

    use super::ConcatFlash;
    use crate::flash::mem_flash::MemFlash;

    #[test]
    fn can_write_and_read_across_flashes() {
        let first = MemFlash::<64, 16, 4>::default();
        let second = MemFlash::<64, 64, 4>::default();
        let mut f = ConcatFlash::new(first, second);

        f.write(60, &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]).unwrap();

        assert_eq!(&[0x11, 0x22, 0x33, 0x44], &f.0.mem[60..]);
        assert_eq!(&[0x55, 0x66, 0x77, 0x88], &f.1.mem[0..4]);

        let mut read_buf = [0; 8];
        f.read(60, &mut read_buf).unwrap();

        assert_eq!(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88], &read_buf);
    }

    #[test]
    fn can_erase_across_flashes() {
        let first = MemFlash::<128, 16, 4>::new(0x00);
        let second = MemFlash::<128, 64, 4>::new(0x00);
        let mut f = ConcatFlash::new(first, second);

        f.erase(64, 192).unwrap();

        assert_eq!(&[0x00; 64], &f.0.mem[0..64]);
        assert_eq!(&[0xff; 64], &f.0.mem[64..128]);
        assert_eq!(&[0xff; 64], &f.1.mem[0..64]);
        assert_eq!(&[0x00; 64], &f.1.mem[64..128]);
    }
}
