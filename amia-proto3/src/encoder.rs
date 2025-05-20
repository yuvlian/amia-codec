use crate::{Protobuf, WireType};
use std::io::{self, Write};

#[inline]
pub const fn make_tag(field_number: u32, wire_type: WireType) -> u32 {
    (field_number << 3) | (wire_type as u32)
}

#[inline]
pub fn encode_varint<W: Write>(value: u64, writer: &mut W) -> io::Result<()> {
    let mut value = value;
    while value >= 0x80 {
        writer.write_all(&[(value as u8) | 0x80])?;
        value >>= 7;
    }
    writer.write_all(&[value as u8])?;
    Ok(())
}

#[inline]
pub fn encode_zigzag<W: Write>(value: i64, writer: &mut W) -> io::Result<()> {
    let zigzag = ((value << 1) ^ (value >> 63)) as u64;
    encode_varint(zigzag, writer)
}

#[inline]
pub fn encode_tag<W: Write>(
    field_number: u32,
    wire_type: WireType,
    writer: &mut W,
) -> io::Result<()> {
    encode_varint(make_tag(field_number, wire_type) as u64, writer)
}

#[inline]
pub fn encode_uint32<W: Write>(field_number: u32, value: u32, writer: &mut W) -> io::Result<()> {
    if value == 0 {
        return Ok(());
    }
    encode_tag(field_number, WireType::Varint, writer)?;
    encode_varint(value as u64, writer)
}

#[inline]
pub fn encode_int32<W: Write>(field_number: u32, value: i32, writer: &mut W) -> io::Result<()> {
    if value == 0 {
        return Ok(());
    }
    encode_tag(field_number, WireType::Varint, writer)?;
    encode_varint(value as u64, writer)
}

#[inline]
pub fn encode_int64<W: Write>(field_number: u32, value: i64, writer: &mut W) -> io::Result<()> {
    if value == 0 {
        return Ok(());
    }
    encode_tag(field_number, WireType::Varint, writer)?;
    encode_varint(value as u64, writer)
}

#[inline]
pub fn encode_uint64<W: Write>(field_number: u32, value: u64, writer: &mut W) -> io::Result<()> {
    if value == 0 {
        return Ok(());
    }
    encode_tag(field_number, WireType::Varint, writer)?;
    encode_varint(value, writer)
}

#[inline]
pub fn encode_sint32<W: Write>(field_number: u32, value: i32, writer: &mut W) -> io::Result<()> {
    if value == 0 {
        return Ok(());
    }
    encode_tag(field_number, WireType::Varint, writer)?;
    encode_zigzag(value as i64, writer)
}

#[inline]
pub fn encode_sint64<W: Write>(field_number: u32, value: i64, writer: &mut W) -> io::Result<()> {
    if value == 0 {
        return Ok(());
    }
    encode_tag(field_number, WireType::Varint, writer)?;
    encode_zigzag(value, writer)
}

#[inline]
pub fn encode_bool<W: Write>(field_number: u32, value: bool, writer: &mut W) -> io::Result<()> {
    if !value {
        return Ok(());
    }
    encode_tag(field_number, WireType::Varint, writer)?;
    writer.write_all(&[value as u8])
}

#[inline]
pub fn encode_string<W: Write>(field_number: u32, value: &str, writer: &mut W) -> io::Result<()> {
    if value.is_empty() {
        return Ok(());
    }
    encode_tag(field_number, WireType::LengthDelimited, writer)?;
    encode_varint(value.len() as u64, writer)?;
    writer.write_all(value.as_bytes())
}

#[inline]
pub fn encode_bytes<W: Write>(field_number: u32, value: &[u8], writer: &mut W) -> io::Result<()> {
    if value.is_empty() {
        return Ok(());
    }
    encode_tag(field_number, WireType::LengthDelimited, writer)?;
    encode_varint(value.len() as u64, writer)?;
    writer.write_all(value)
}

#[inline]
pub fn encode_float<W: Write>(field_number: u32, value: f32, writer: &mut W) -> io::Result<()> {
    if value == 0.0 {
        return Ok(());
    }
    encode_tag(field_number, WireType::Fixed32, writer)?;
    writer.write_all(&value.to_le_bytes())
}

#[inline]
pub fn encode_double<W: Write>(field_number: u32, value: f64, writer: &mut W) -> io::Result<()> {
    if value == 0.0 {
        return Ok(());
    }
    encode_tag(field_number, WireType::Fixed64, writer)?;
    writer.write_all(&value.to_le_bytes())
}

#[inline]
pub fn encode_fixed32<W: Write>(field_number: u32, value: u32, writer: &mut W) -> io::Result<()> {
    if value == 0 {
        return Ok(());
    }
    encode_tag(field_number, WireType::Fixed32, writer)?;
    writer.write_all(&value.to_le_bytes())
}

#[inline]
pub fn encode_fixed64<W: Write>(field_number: u32, value: u64, writer: &mut W) -> io::Result<()> {
    if value == 0 {
        return Ok(());
    }
    encode_tag(field_number, WireType::Fixed64, writer)?;
    writer.write_all(&value.to_le_bytes())
}

#[inline]
pub fn encode_sfixed32<W: Write>(field_number: u32, value: i32, writer: &mut W) -> io::Result<()> {
    if value == 0 {
        return Ok(());
    }
    encode_tag(field_number, WireType::Fixed32, writer)?;
    writer.write_all(&value.to_le_bytes())
}

#[inline]
pub fn encode_sfixed64<W: Write>(field_number: u32, value: i64, writer: &mut W) -> io::Result<()> {
    if value == 0 {
        return Ok(());
    }
    encode_tag(field_number, WireType::Fixed64, writer)?;
    writer.write_all(&value.to_le_bytes())
}

#[inline]
pub fn encode_enum<W: Write, E: Into<i32>>(
    field_number: u32,
    value: E,
    writer: &mut W,
) -> io::Result<()> {
    encode_int32(field_number, value.into(), writer)
}

#[inline]
pub fn encode_repeated<T, F, W>(
    field_number: u32,
    values: &[T],
    writer: &mut W,
    encoder: F,
) -> io::Result<()>
where
    F: Fn(u32, &T, &mut W) -> io::Result<()>,
    W: Write,
{
    for value in values {
        encoder(field_number, value, writer)?;
    }
    Ok(())
}

// #[inline]
// pub fn encode_packed<T, F, W>(
//     field_number: u32,
//     values: &[T],
//     writer: &mut W,
//     value_size_fn: F,
//     value_writer_fn: impl Fn(&T, &mut Vec<u8>) -> io::Result<()>,
// ) -> io::Result<()>
// where
//     F: Fn(&T) -> usize,
//     W: Write,
// {
//     if values.is_empty() {
//         return Ok(());
//     }

//     let mut total_size = 0;
//     for value in values {
//         total_size += value_size_fn(value);
//     }

//     encode_tag(field_number, WireType::LengthDelimited, writer)?;
//     encode_varint(total_size as u64, writer)?;

//     let mut buffer = Vec::with_capacity(total_size);
//     for value in values {
//         value_writer_fn(value, &mut buffer)?;
//     }

//     writer.write_all(&buffer)
// }

#[inline]
pub fn encode_packed<T, W>(
    field_number: u32,
    values: &[T],
    writer: &mut W,
    value_writer_fn: impl Fn(&T, &mut Vec<u8>) -> io::Result<()>,
) -> io::Result<()>
where
    W: Write,
{
    if values.is_empty() {
        return Ok(());
    }

    let mut buffer = Vec::new();
    for value in values {
        value_writer_fn(value, &mut buffer)?;
    }

    encode_tag(field_number, WireType::LengthDelimited, writer)?;
    encode_varint(buffer.len() as u64, writer)?;
    writer.write_all(&buffer)
}

#[inline]
pub fn encode_message<W: Write, M: Protobuf>(
    field_number: u32,
    message: &M,
    writer: &mut W,
) -> io::Result<()> {
    encode_tag(field_number, WireType::LengthDelimited, writer)?;
    let encoded = message.encode_to_vec();
    encode_varint(encoded.len() as u64, writer)?;
    writer.write_all(&encoded)
}

// #[inline]
// pub fn encode_map<K, V, W, IK, IV, SK, SV>(
//     field_number: u32,
//     map: impl IntoIterator<Item = (K, V)>,
//     writer: &mut W,
//     mut key_encoder: IK,
//     mut value_encoder: IV,
//     key_size_fn: SK,
//     value_size_fn: SV,
// ) -> io::Result<()>
// where
//     W: Write,
//     IK: FnMut(u32, &K, &mut Vec<u8>) -> io::Result<()>,
//     IV: FnMut(u32, &V, &mut Vec<u8>) -> io::Result<()>,
//     SK: Fn(u32, &K) -> usize,
//     SV: Fn(u32, &V) -> usize,
// {
//     for (key, value) in map {
//         let key_size = key_size_fn(1, &key);
//         let value_size = value_size_fn(2, &value);
//         let total_size = key_size + value_size;

//         let mut entry_buf = Vec::with_capacity(total_size);
//         key_encoder(1, &key, &mut entry_buf)?;
//         value_encoder(2, &value, &mut entry_buf)?;

//         encode_tag(field_number, WireType::LengthDelimited, writer)?;
//         encode_varint(entry_buf.len() as u64, writer)?;
//         writer.write_all(&entry_buf)?;
//     }
//     Ok(())
// }

#[inline]
pub fn encode_map<K, V, W, IK, IV>(
    field_number: u32,
    map: impl IntoIterator<Item = (K, V)>,
    writer: &mut W,
    mut key_encoder: IK,
    mut value_encoder: IV,
) -> io::Result<()>
where
    W: Write,
    IK: FnMut(u32, &K, &mut Vec<u8>) -> io::Result<()>,
    IV: FnMut(u32, &V, &mut Vec<u8>) -> io::Result<()>,
{
    for (key, value) in map {
        let mut entry_buf = Vec::new();
        key_encoder(1, &key, &mut entry_buf)?;
        value_encoder(2, &value, &mut entry_buf)?;

        encode_tag(field_number, WireType::LengthDelimited, writer)?;
        encode_varint(entry_buf.len() as u64, writer)?;
        writer.write_all(&entry_buf)?;
    }
    Ok(())
}

#[inline]
pub fn size_of_varint(value: u64) -> usize {
    match value {
        0..=127 => 1,
        128..=16383 => 2,
        16384..=2097151 => 3,
        2097152..=268435455 => 4,
        268435456..=34359738367 => 5,
        34359738368..=4398046511103 => 6,
        4398046511104..=562949953421311 => 7,
        562949953421312..=72057594037927935 => 8,
        72057594037927936..=9223372036854775807 => 9,
        _ => 10,
    }
}

// #[inline]
// pub fn size_of_tag(field_number: u32) -> usize {
//     size_of_varint(make_tag(field_number, WireType::Varint) as u64)
// }

// #[inline]
// pub fn size_of_zigzag(value: i64) -> usize {
//     let zigzag = ((value << 1) ^ (value >> 63)) as u64;
//     size_of_varint(zigzag)
// }

// #[inline]
// pub fn size_of_uint32(field_number: u32, value: u32) -> usize {
//     if value == 0 {
//         return 0;
//     }
//     size_of_tag(field_number) + size_of_varint(value as u64)
// }

// #[inline]
// pub fn size_of_int32(field_number: u32, value: i32) -> usize {
//     if value == 0 {
//         return 0;
//     }
//     size_of_tag(field_number) + size_of_varint(value as u64)
// }

// #[inline]
// pub fn size_of_int64(field_number: u32, value: i64) -> usize {
//     if value == 0 {
//         return 0;
//     }
//     size_of_tag(field_number) + size_of_varint(value as u64)
// }

// #[inline]
// pub fn size_of_uint64(field_number: u32, value: u64) -> usize {
//     if value == 0 {
//         return 0;
//     }
//     size_of_tag(field_number) + size_of_varint(value)
// }

// #[inline]
// pub fn size_of_sint32(field_number: u32, value: i32) -> usize {
//     if value == 0 {
//         return 0;
//     }
//     size_of_tag(field_number) + size_of_zigzag(value as i64)
// }

// #[inline]
// pub fn size_of_sint64(field_number: u32, value: i64) -> usize {
//     if value == 0 {
//         return 0;
//     }
//     size_of_tag(field_number) + size_of_zigzag(value)
// }

// #[inline]
// pub fn size_of_bool(field_number: u32, value: bool) -> usize {
//     if !value {
//         return 0;
//     }
//     size_of_tag(field_number) + 1
// }

// #[inline]
// pub fn size_of_string(field_number: u32, value: &str) -> usize {
//     if value.is_empty() {
//         return 0;
//     }
//     let str_len = value.len();
//     size_of_tag(field_number) + size_of_varint(str_len as u64) + str_len
// }

// #[inline]
// pub fn size_of_bytes(field_number: u32, value: &[u8]) -> usize {
//     if value.is_empty() {
//         return 0;
//     }
//     let bytes_len = value.len();
//     size_of_tag(field_number) + size_of_varint(bytes_len as u64) + bytes_len
// }

// #[inline]
// pub fn size_of_float(field_number: u32, value: f32) -> usize {
//     if value == 0.0 {
//         return 0;
//     }
//     size_of_tag(field_number) + 4
// }

// #[inline]
// pub fn size_of_double(field_number: u32, value: f64) -> usize {
//     if value == 0.0 {
//         return 0;
//     }
//     size_of_tag(field_number) + 8
// }

// #[inline]
// pub fn size_of_fixed32(field_number: u32, value: u32) -> usize {
//     if value == 0 {
//         return 0;
//     }
//     size_of_tag(field_number) + 4
// }

// #[inline]
// pub fn size_of_fixed64(field_number: u32, value: u64) -> usize {
//     if value == 0 {
//         return 0;
//     }
//     size_of_tag(field_number) + 8
// }

// #[inline]
// pub fn size_of_sfixed32(field_number: u32, value: i32) -> usize {
//     if value == 0 {
//         return 0;
//     }
//     size_of_tag(field_number) + 4
// }

// #[inline]
// pub fn size_of_sfixed64(field_number: u32, value: i64) -> usize {
//     if value == 0 {
//         return 0;
//     }
//     size_of_tag(field_number) + 8
// }

// #[inline]
// pub fn size_of_enum<E: Into<i32> + Copy>(field_number: u32, value: E) -> usize {
//     size_of_int32(field_number, value.into())
// }

// #[inline]
// pub fn size_of_repeated<T, F>(field_number: u32, values: &[T], size_fn: F) -> usize
// where
//     F: Fn(u32, &T) -> usize,
// {
//     values.iter().map(|v| size_fn(field_number, v)).sum()
// }

// #[inline]
// pub fn size_of_packed<T, F>(field_number: u32, values: &[T], value_size_fn: F) -> usize
// where
//     F: Fn(&T) -> usize,
// {
//     if values.is_empty() {
//         return 0;
//     }

//     let content_size: usize = values.iter().map(|v| value_size_fn(v)).sum();

//     size_of_tag(field_number) + size_of_varint(content_size as u64) + content_size
// }

// #[inline]
// pub fn size_of_message<M: Protobuf>(field_number: u32, message: &M) -> usize {
//     let message_size = message.encoded_len();
//     if message_size == 0 {
//         return 0;
//     }

//     size_of_tag(field_number) + size_of_varint(message_size as u64) + message_size
// }

// pub fn size_of_map<'a, K: 'a, V: 'a, SK, SV>(
//     field_number: u32,
//     map: impl IntoIterator<Item = (&'a K, &'a V)>,
//     key_size_fn: SK,
//     value_size_fn: SV,
// ) -> usize
// where
//     SK: Fn(u32, &K) -> usize,
//     SV: Fn(u32, &V) -> usize,
// {
//     let mut total_size = 0;

//     for (key, value) in map {
//         let key_size = key_size_fn(1, key);
//         let value_size = value_size_fn(2, value);
//         let entry_size = key_size + value_size;
//         total_size += size_of_tag(field_number) + size_of_varint(entry_size as u64) + entry_size;
//     }

//     total_size
// }

// #[inline]
// pub fn size_of_varint_value(value: u64) -> usize {
//     size_of_varint(value)
// }

// #[inline]
// pub fn size_of_zigzag_value(value: i64) -> usize {
//     size_of_zigzag(value)
// }

// #[inline]
// pub fn size_of_fixed32_value(_value: u32) -> usize {
//     4
// }

// #[inline]
// pub fn size_of_fixed64_value(_value: u64) -> usize {
//     8
// }
