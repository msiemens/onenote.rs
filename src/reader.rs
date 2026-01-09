use crate::errors::{ErrorKind, Result};
use bytes::Buf;

pub(crate) struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Reader<'a> {
        Reader(data)
    }

    pub(crate) fn read(&mut self, cnt: usize) -> Result<&[u8]> {
        if self.remaining() < cnt {
            return Err(ErrorKind::UnexpectedEof.into());
        }

        let data = &self.0[0..cnt];
        self.0.advance(cnt);

        Ok(data)
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        self.0.chunk()
    }

    pub(crate) fn remaining(&self) -> usize {
        self.0.remaining()
    }

    pub(crate) fn advance(&mut self, cnt: usize) -> Result<()> {
        if self.remaining() < cnt {
            return Err(ErrorKind::UnexpectedEof.into());
        }

        self.0.advance(cnt);

        Ok(())
    }

    pub(crate) fn get_u8(&mut self) -> Result<u8> {
        self.0
            .try_get_u8()
            .map_err(|_| ErrorKind::UnexpectedEof.into())
    }

    pub(crate) fn get_u16(&mut self) -> Result<u16> {
        self.0
            .try_get_u16_le()
            .map_err(|_| ErrorKind::UnexpectedEof.into())
    }

    pub(crate) fn get_u32(&mut self) -> Result<u32> {
        self.0
            .try_get_u32_le()
            .map_err(|_| ErrorKind::UnexpectedEof.into())
    }

    pub(crate) fn get_u64(&mut self) -> Result<u64> {
        self.0
            .try_get_u64_le()
            .map_err(|_| ErrorKind::UnexpectedEof.into())
    }

    pub(crate) fn get_u128(&mut self) -> Result<u128> {
        self.0
            .try_get_u128_le()
            .map_err(|_| ErrorKind::UnexpectedEof.into())
    }

    pub(crate) fn get_f32(&mut self) -> Result<f32> {
        self.0
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
}
