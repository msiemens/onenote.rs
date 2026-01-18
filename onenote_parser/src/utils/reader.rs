use crate::utils::errors::{ErrorKind, Result};
use bytes::Buf;

pub struct Reader<'a> {
    buff: &'a [u8],
    original: &'a [u8],
}

impl<'a> Clone for Reader<'a> {
    fn clone(&self) -> Self {
        let mut result = Self::new(self.original);
        result
            .advance(self.absolute_offset())
            .expect("should re-advance to the original's position");
        result
    }
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Reader<'a> {
        Reader {
            buff: data,
            original: data,
        }
    }

    pub(crate) fn read(&mut self, cnt: usize) -> Result<&[u8]> {
        if self.remaining() < cnt {
            return Err(ErrorKind::UnexpectedEof.into());
        }

        let data = &self.buff[0..cnt];
        self.buff.advance(cnt);

        Ok(data)
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        self.buff.chunk()
    }

    pub(crate) fn remaining(&self) -> usize {
        self.buff.remaining()
    }

    pub fn advance(&mut self, count: usize) -> Result<()> {
        if self.remaining() < count {
            return Err(ErrorKind::UnexpectedEof.into());
        }

        self.buff.advance(count);

        Ok(())
    }

    pub fn absolute_offset(&self) -> usize {
        // Use pointer arithmetic (in a way similar to the [subslice offset](https://docs.rs/crate/subslice-offset/latest/source/src/lib.rs)
        // crate and [this StackOverflow post](https://stackoverflow.com/questions/50781561/how-to-find-the-starting-offset-of-a-string-slice-of-another-string/50781657))
        // to calculate the offset.
        let offset = (self.buff.as_ptr() as usize) - (self.original.as_ptr() as usize);
        if offset > self.original.len() {
            panic!("self.buff must be a subslice of self.original!");
        }

        offset
    }

    pub(crate) fn with_updated_bounds(&self, start: usize, end: usize) -> Result<Reader<'a>> {
        if start > self.original.len() {
            return Err(ErrorKind::UnexpectedEof.into());
        }
        if end > self.original.len() {
            return Err(ErrorKind::UnexpectedEof.into());
        }

        Ok(Reader {
            buff: &self.original[start..end],
            original: self.original,
        })
    }

    pub(crate) fn get_u8(&mut self) -> Result<u8> {
        self.buff
            .try_get_u8()
            .map_err(|_| ErrorKind::UnexpectedEof.into())
    }

    pub(crate) fn get_u16(&mut self) -> Result<u16> {
        self.buff
            .try_get_u16_le()
            .map_err(|_| ErrorKind::UnexpectedEof.into())
    }

    pub(crate) fn get_u32(&mut self) -> Result<u32> {
        self.buff
            .try_get_u32_le()
            .map_err(|_| ErrorKind::UnexpectedEof.into())
    }

    pub(crate) fn get_u64(&mut self) -> Result<u64> {
        self.buff
            .try_get_u64_le()
            .map_err(|_| ErrorKind::UnexpectedEof.into())
    }

    pub(crate) fn get_u128(&mut self) -> Result<u128> {
        self.buff
            .try_get_u128_le()
            .map_err(|_| ErrorKind::UnexpectedEof.into())
    }

    pub(crate) fn get_f32(&mut self) -> Result<f32> {
        self.buff
            .try_get_f32_le()
            .map_err(|_| ErrorKind::UnexpectedEof.into())
    }
}

#[cfg(test)]
mod tests {
    use super::Reader;

    #[test]
    fn test_read_and_advance() {
        let data = [1u8, 2, 3, 4];
        let mut reader = Reader::new(&data);

        assert_eq!(reader.remaining(), 4);
        assert_eq!(reader.read(2).unwrap(), &[1, 2]);
        assert_eq!(reader.remaining(), 2);

        reader.advance(1).unwrap();
        assert_eq!(reader.remaining(), 1);
        assert_eq!(reader.get_u8().unwrap(), 4);
        assert!(reader.get_u8().is_err());
    }

    #[test]
    fn test_get_numeric_types() {
        let data = [
            0x34, 0x12, // u16 = 0x1234
            0x78, 0x56, 0x34, 0x12, // u32 = 0x12345678
            0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01, // u64
        ];
        let mut reader = Reader::new(&data);

        assert_eq!(reader.get_u16().unwrap(), 0x1234);
        assert_eq!(reader.get_u32().unwrap(), 0x1234_5678);
        assert_eq!(reader.get_u64().unwrap(), 0x0123_4567_89AB_CDEF);
    }

    #[test]
    fn test_get_f32() {
        let data = [0x00, 0x00, 0x80, 0x3F]; // 1.0 in LE
        let mut reader = Reader::new(&data);

        assert_eq!(reader.get_f32().unwrap(), 1.0);
        assert!(reader.get_u8().is_err());
    }

    #[test]
    fn with_start_index_should_seek() {
        let data: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let mut reader = Reader::new(&data);
        assert_eq!(reader.get_u8().unwrap(), 1);
        assert_eq!(reader.get_u8().unwrap(), 2);
        assert_eq!(reader.get_u8().unwrap(), 3);
        {
            let mut reader = reader.with_updated_bounds(0, 8).unwrap();
            assert_eq!(reader.get_u8().unwrap(), 1);
            assert_eq!(reader.get_u8().unwrap(), 2);
            assert_eq!(reader.get_u8().unwrap(), 3);
            assert_eq!(reader.get_u8().unwrap(), 4);
            let mut reader = reader.with_updated_bounds(1, 7).unwrap();
            assert_eq!(reader.get_u8().unwrap(), 2);
            assert_eq!(reader.get_u8().unwrap(), 3);
            let mut reader = reader.with_updated_bounds(1, 7).unwrap();
            assert_eq!(reader.get_u8().unwrap(), 2);
            assert_eq!(reader.get_u8().unwrap(), 3);
            let reader = reader.with_updated_bounds(5, 7).unwrap();
            assert_eq!(reader.remaining(), 2);
            let reader = reader.with_updated_bounds(6, 6).unwrap();
            assert_eq!(reader.remaining(), 0);
        }
        assert_eq!(reader.get_u8().unwrap(), 4);
    }
}
