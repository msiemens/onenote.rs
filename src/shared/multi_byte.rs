//! Multi-byte encoding/decoding
//!
//! Following the multi-byte encoding described in [\[MS-ISF\]] (sections _Multi-byte Encoding of
//! Signed Numbers_ and _Sizes of Tags and Numbers_).
//!
//! [\[MS-ISF\]]: https://docs.microsoft.com/en-us/uwp/specifications/ink-serialized-format

pub(crate) fn decode_signed(input: &[u8]) -> Vec<i64> {
    decode(input)
        .into_iter()
        .map(|value| {
            let shifted = (value >> 1) as i64;

            if value & 0x1 == 0x1 {
                -shifted
            } else {
                shifted
            }
        })
        .collect()
}

fn decode(input: &[u8]) -> Vec<u64> {
    let mut output = vec![];

    // Decode the multi-byte data length
    let (length, offset) = decode_uint(input);

    // The length is actually a signed value so we need to remove the sign bit
    // (see also `decode_signed`). This may not be the case for unsigned multi-byte blobs
    // but for ink data it's always signed so this should be safe for now.
    let length = length >> 1;

    // Decode the remaining data
    let mut index = offset;
    for _ in 0..length {
        let (value, offset) = decode_uint(&input[index..]);

        output.push(value);
        index += offset;
    }

    output
}

fn decode_uint(data: &[u8]) -> (u64, usize) {
    let mut value: u64 = 0;
    let mut count = 0;

    for byte in data {
        let flag = byte & 0x80 == 0x80;
        value |= (*byte as u64 & 0x7F) << (count * 7);

        count += 1;

        if !flag {
            break;
        }
    }

    (value, count)
}

#[cfg(test)]
mod tests {
    use super::{decode, decode_signed, decode_uint};

    #[test]
    fn test_decode_uint_single_byte() {
        assert_eq!(decode_uint(&[0x00]), (0, 1));
        assert_eq!(decode_uint(&[0x7F]), (127, 1));
    }

    #[test]
    fn test_decode_uint_multi_byte() {
        assert_eq!(decode_uint(&[0x81, 0x01]), (129, 2));
        assert_eq!(decode_uint(&[0xFF, 0x01]), (255, 2));
    }

    #[test]
    fn test_decode_unsigned_values() {
        let input = [0x04, 0x01, 0x02];
        assert_eq!(decode(&input), vec![1, 2]);
    }

    #[test]
    fn test_decode_signed_values() {
        let input = [0x04, 0x06, 0x05];
        assert_eq!(decode_signed(&input), vec![3, -2]);
    }
}
