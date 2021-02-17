//! Multi-byte encoding/decoding
//!
//! Following the multi-byte encoding described in [\[MS-ISF\]] (sections _Multi-byte Encoding of
//! Signed Numbers_ and _Sizes of Tags and Numbers_).
//!
//! [\[MS-ISF\]]: https://docs.microsoft.com/en-us/uwp/specifications/ink-serialized-format

pub(crate) fn decode(input: &[u8]) -> Vec<u64> {
    let mut output = vec![];
    let mut index = 0;

    loop {
        let (value, offset) = decode_uint(&input[index..]);

        output.push(value);
        index += offset;

        if index >= input.len() {
            break;
        }
    }

    output
}

pub(crate) fn decode_signed(input: &[u8]) -> Vec<i64> {
    decode(input)
        .into_iter()
        .map(|value| {
            if value & 0x1 == 0x1 {
                (value >> 1) as i64 * -1
            } else {
                (value >> 1) as i64
            }
        })
        .collect()
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
