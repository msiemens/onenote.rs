use crate::Reader;

#[derive(Debug)]
pub(crate) struct ObjectStreamHeader {
    pub(crate) count: u32,
    pub(crate) extended_streams_present: bool,
    pub(crate) osid_stream_not_present: bool,
}

impl ObjectStreamHeader {
    pub(crate) fn parse(reader: Reader) -> ObjectStreamHeader {
        let data = reader.get_u32_le();

        let count = data & 0xFFFFFF;
        let extended_streams_present = (data >> 30) & 0x1 != 0;
        let osid_stream_not_present = (data >> 31) != 0;

        ObjectStreamHeader {
            count,
            extended_streams_present,
            osid_stream_not_present,
        }
    }
}
