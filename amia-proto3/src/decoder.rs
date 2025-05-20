use crate::{DecodeError, DecodeResult, Protobuf, WireType};
use std::collections::HashMap;
use std::io::{self, Cursor, Read, Seek};

#[derive(Debug)]
pub struct Tag {
    pub field_number: u32,
    pub wire_type: WireType,
}

impl Tag {
    pub fn new(field_number: u32, wire_type: WireType) -> Self {
        Self {
            field_number,
            wire_type,
        }
    }

    pub fn decode<R: Read>(reader: &mut R) -> DecodeResult<Option<Self>> {
        match decode_varint(reader) {
            Ok(0) => Ok(None),
            Ok(tag) => {
                let field_number = tag >> 3;
                if field_number == 0 {
                    return Err(DecodeError::InvalidTag);
                }

                let wire_type_value = (tag & 0x7) as u8;
                let wire_type = match wire_type_value {
                    0 => WireType::Varint,
                    1 => WireType::Fixed64,
                    2 => WireType::LengthDelimited,
                    // 3 => WireType::StartGroup,
                    // 4 => WireType::EndGroup,
                    5 => WireType::Fixed32,
                    _ => return Err(DecodeError::InvalidWireType(wire_type_value as u32)),
                };

                Ok(Some(Tag {
                    field_number: field_number.try_into().unwrap(),
                    wire_type,
                }))
            }
            Err(e) => match e {
                DecodeError::UnexpectedEof => Ok(None),
                _ => Err(e),
            },
        }
    }
}

#[inline]
pub fn decode_varint<R: Read>(reader: &mut R) -> DecodeResult<u64> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;

    loop {
        let mut buf = [0u8; 1];
        match reader.read_exact(&mut buf) {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                return Err(DecodeError::UnexpectedEof);
            }
            Err(e) => return Err(DecodeError::IoError(e)),
        }

        let byte = buf[0];
        result |= ((byte & 0x7F) as u64) << shift;

        if byte & 0x80 == 0 {
            return Ok(result);
        }

        shift += 7;
        if shift > 63 {
            return Err(DecodeError::InvalidVarint);
        }
    }
}

#[inline]
pub fn decode_tag<R: Read>(reader: &mut R) -> DecodeResult<Option<Tag>> {
    Tag::decode(reader)
}

#[inline]
pub fn decode_zigzag(value: u64) -> i64 {
    ((value >> 1) as i64) ^ (-((value & 1) as i64))
}

#[inline]
pub fn decode_uint32<R: Read>(reader: &mut R) -> DecodeResult<u32> {
    let value = decode_varint(reader)?;
    Ok(value as u32)
}

#[inline]
pub fn decode_int32<R: Read>(reader: &mut R) -> DecodeResult<i32> {
    let value = decode_varint(reader)?;
    Ok(value as i32)
}

#[inline]
pub fn decode_int64<R: Read>(reader: &mut R) -> DecodeResult<i64> {
    let value = decode_varint(reader)?;
    Ok(value as i64)
}

#[inline]
pub fn decode_uint64<R: Read>(reader: &mut R) -> DecodeResult<u64> {
    decode_varint(reader)
}

#[inline]
pub fn decode_sint32<R: Read>(reader: &mut R) -> DecodeResult<i32> {
    let value = decode_varint(reader)?;
    Ok(decode_zigzag(value) as i32)
}

#[inline]
pub fn decode_sint64<R: Read>(reader: &mut R) -> DecodeResult<i64> {
    let value = decode_varint(reader)?;
    Ok(decode_zigzag(value))
}

#[inline]
pub fn decode_bool<R: Read>(reader: &mut R) -> DecodeResult<bool> {
    let value = decode_varint(reader)?;
    Ok(value != 0)
}

#[inline]
pub fn decode_bytes<R: Read>(reader: &mut R) -> DecodeResult<Vec<u8>> {
    let length = decode_varint(reader)? as usize;
    let mut buffer = vec![0u8; length];
    reader.read_exact(&mut buffer)?;
    Ok(buffer)
}

#[inline]
pub fn decode_string<R: Read>(reader: &mut R) -> DecodeResult<String> {
    let bytes = decode_bytes(reader)?;
    Ok(String::from_utf8(bytes)?)
}

#[inline]
pub fn decode_float<R: Read>(reader: &mut R) -> DecodeResult<f32> {
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer)?;
    Ok(f32::from_le_bytes(buffer))
}

#[inline]
pub fn decode_double<R: Read>(reader: &mut R) -> DecodeResult<f64> {
    let mut buffer = [0u8; 8];
    reader.read_exact(&mut buffer)?;
    Ok(f64::from_le_bytes(buffer))
}

#[inline]
pub fn decode_fixed32<R: Read>(reader: &mut R) -> DecodeResult<u32> {
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

#[inline]
pub fn decode_fixed64<R: Read>(reader: &mut R) -> DecodeResult<u64> {
    let mut buffer = [0u8; 8];
    reader.read_exact(&mut buffer)?;
    Ok(u64::from_le_bytes(buffer))
}

#[inline]
pub fn decode_sfixed32<R: Read>(reader: &mut R) -> DecodeResult<i32> {
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer)?;
    Ok(i32::from_le_bytes(buffer))
}

#[inline]
pub fn decode_sfixed64<R: Read>(reader: &mut R) -> DecodeResult<i64> {
    let mut buffer = [0u8; 8];
    reader.read_exact(&mut buffer)?;
    Ok(i64::from_le_bytes(buffer))
}

#[inline]
pub fn decode_uint32_field<R: Read>(
    field_number: u32,
    reader: &mut R,
) -> DecodeResult<Option<u32>> {
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::Varint {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::Varint,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_uint32(reader)?))
}

#[inline]
pub fn decode_int32_field<R: Read>(field_number: u32, reader: &mut R) -> DecodeResult<Option<i32>> {
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::Varint {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::Varint,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_int32(reader)?))
}

#[inline]
pub fn decode_int64_field<R: Read>(field_number: u32, reader: &mut R) -> DecodeResult<Option<i64>> {
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::Varint {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::Varint,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_int64(reader)?))
}

#[inline]
pub fn decode_uint64_field<R: Read>(
    field_number: u32,
    reader: &mut R,
) -> DecodeResult<Option<u64>> {
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::Varint {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::Varint,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_uint64(reader)?))
}

#[inline]
pub fn decode_sint32_field<R: Read>(
    field_number: u32,
    reader: &mut R,
) -> DecodeResult<Option<i32>> {
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::Varint {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::Varint,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_sint32(reader)?))
}

#[inline]
pub fn decode_sint64_field<R: Read>(
    field_number: u32,
    reader: &mut R,
) -> DecodeResult<Option<i64>> {
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::Varint {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::Varint,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_sint64(reader)?))
}

#[inline]
pub fn decode_bool_field<R: Read>(field_number: u32, reader: &mut R) -> DecodeResult<Option<bool>> {
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::Varint {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::Varint,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_bool(reader)?))
}

#[inline]
pub fn decode_string_field<R: Read>(
    field_number: u32,
    reader: &mut R,
) -> DecodeResult<Option<String>> {
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::LengthDelimited {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::LengthDelimited,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_string(reader)?))
}

#[inline]
pub fn decode_bytes_field<R: Read>(
    field_number: u32,
    reader: &mut R,
) -> DecodeResult<Option<Vec<u8>>> {
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::LengthDelimited {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::LengthDelimited,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_bytes(reader)?))
}

#[inline]
pub fn decode_float_field<R: Read>(field_number: u32, reader: &mut R) -> DecodeResult<Option<f32>> {
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::Fixed32 {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::Fixed32,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_float(reader)?))
}

#[inline]
pub fn decode_double_field<R: Read>(
    field_number: u32,
    reader: &mut R,
) -> DecodeResult<Option<f64>> {
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::Fixed64 {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::Fixed64,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_double(reader)?))
}

#[inline]
pub fn decode_fixed32_field<R: Read>(
    field_number: u32,
    reader: &mut R,
) -> DecodeResult<Option<u32>> {
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::Fixed32 {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::Fixed32,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_fixed32(reader)?))
}

#[inline]
pub fn decode_fixed64_field<R: Read>(
    field_number: u32,
    reader: &mut R,
) -> DecodeResult<Option<u64>> {
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::Fixed64 {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::Fixed64,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_fixed64(reader)?))
}

#[inline]
pub fn decode_sfixed32_field<R: Read>(
    field_number: u32,
    reader: &mut R,
) -> DecodeResult<Option<i32>> {
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::Fixed32 {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::Fixed32,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_sfixed32(reader)?))
}

#[inline]
pub fn decode_sfixed64_field<R: Read>(
    field_number: u32,
    reader: &mut R,
) -> DecodeResult<Option<i64>> {
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::Fixed64 {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::Fixed64,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_sfixed64(reader)?))
}

#[inline]
pub fn decode_message<M, R>(reader: &mut R) -> DecodeResult<M>
where
    M: Protobuf,
    R: Read,
{
    let length = decode_varint(reader)? as usize;

    let mut buffer = vec![0u8; length];
    reader.read_exact(&mut buffer)?;

    M::decode_from_slice(&buffer)
}

#[inline]
pub fn decode_message_field<M, R>(field_number: u32, reader: &mut R) -> DecodeResult<Option<M>>
where
    M: Protobuf,
    R: Read,
{
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::LengthDelimited {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::LengthDelimited,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_message(reader)?))
}

#[inline]
pub fn decode_enum<E, F, R>(reader: &mut R, converter: F) -> DecodeResult<E>
where
    R: Read,
    F: Fn(i32) -> Option<E>,
{
    let value = decode_int32(reader)?;
    converter(value)
        .ok_or_else(|| DecodeError::MalformedInput(format!("Invalid enum value: {}", value)))
}

#[inline]
pub fn decode_enum_field<E, F, R>(
    field_number: u32,
    reader: &mut R,
    converter: F,
) -> DecodeResult<Option<E>>
where
    R: Read,
    F: Fn(i32) -> Option<E>,
{
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::Varint {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::Varint,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_enum(reader, converter)?))
}

#[inline]
pub fn decode_packed<T, F, R>(reader: &mut R, item_decoder: F) -> DecodeResult<Vec<T>>
where
    F: Fn(&mut Cursor<&[u8]>) -> DecodeResult<T>,
    R: Read,
{
    let length = decode_varint(reader)? as usize;
    let mut buffer = vec![0u8; length];
    reader.read_exact(&mut buffer)?;

    let mut result = Vec::new();
    let mut cursor = Cursor::new(buffer.as_slice());

    while cursor.position() < length as u64 {
        result.push(item_decoder(&mut cursor)?);
    }

    Ok(result)
}

#[inline]
pub fn decode_packed_field<T, F, R>(
    field_number: u32,
    reader: &mut R,
    item_decoder: F,
) -> DecodeResult<Option<Vec<T>>>
where
    F: Fn(&mut Cursor<&[u8]>) -> DecodeResult<T>,
    R: Read,
{
    let tag = match Tag::decode(reader)? {
        Some(tag) => tag,
        None => return Ok(None),
    };

    if tag.field_number != field_number {
        return Ok(None);
    }

    if tag.wire_type != WireType::LengthDelimited {
        return Err(DecodeError::UnexpectedWireType {
            expected: WireType::LengthDelimited,
            got: tag.wire_type,
        });
    }

    Ok(Some(decode_packed(reader, item_decoder)?))
}

#[inline]
pub fn decode_map<K, V, KF, VF, R>(
    reader: &mut R,
    key_decoder: KF,
    value_decoder: VF,
) -> DecodeResult<HashMap<K, V>>
where
    K: Eq + std::hash::Hash,
    KF: Fn(&mut Cursor<&[u8]>) -> DecodeResult<K>,
    VF: Fn(&mut Cursor<&[u8]>) -> DecodeResult<V>,
    R: Read,
{
    let mut map = HashMap::new();

    loop {
        match Tag::decode(reader)? {
            Some(tag) => {
                if tag.wire_type != WireType::LengthDelimited {
                    return Err(DecodeError::UnexpectedWireType {
                        expected: WireType::LengthDelimited,
                        got: tag.wire_type,
                    });
                }

                let length = decode_varint(reader)? as usize;
                let mut entry_buffer = vec![0u8; length];
                reader.read_exact(&mut entry_buffer)?;

                let mut entry_cursor = Cursor::new(entry_buffer.as_slice());

                let key_tag = Tag::decode(&mut entry_cursor)?.ok_or(
                    DecodeError::MalformedInput("Missing key in map entry".to_string()),
                )?;

                if key_tag.field_number != 1 {
                    return Err(DecodeError::MalformedInput(
                        "Expected field number 1 for key in map entry".to_string(),
                    ));
                }

                let key = key_decoder(&mut entry_cursor)?;

                let value_tag = Tag::decode(&mut entry_cursor)?.ok_or(
                    DecodeError::MalformedInput("Missing value in map entry".to_string()),
                )?;

                if value_tag.field_number != 2 {
                    return Err(DecodeError::MalformedInput(
                        "Expected field number 2 for value in map entry".to_string(),
                    ));
                }

                let value = value_decoder(&mut entry_cursor)?;

                map.insert(key, value);
            }
            None => break,
        }
    }

    Ok(map)
}

#[inline]
pub fn decode_map_field<K, V, KF, VF, R>(
    field_number: u32,
    reader: &mut R,
    key_decoder: KF,
    value_decoder: VF,
) -> DecodeResult<HashMap<K, V>>
where
    K: Eq + std::hash::Hash,
    KF: Fn(&mut Cursor<&[u8]>) -> DecodeResult<K>,
    VF: Fn(&mut Cursor<&[u8]>) -> DecodeResult<V>,
    R: Read + Seek,
{
    let mut map = HashMap::new();

    loop {
        let start_pos = match reader.seek(io::SeekFrom::Current(0)) {
            Ok(pos) => pos,
            Err(_) => 0,
        };

        let tag = match Tag::decode(reader)? {
            Some(tag) => tag,
            None => break,
        };

        if tag.field_number != field_number {
            if start_pos > 0 {
                reader.seek(io::SeekFrom::Start(start_pos))?;
            }
            break;
        }

        if tag.wire_type != WireType::LengthDelimited {
            return Err(DecodeError::UnexpectedWireType {
                expected: WireType::LengthDelimited,
                got: tag.wire_type,
            });
        }

        let length = decode_varint(reader)? as usize;
        let mut entry_buffer = vec![0u8; length];
        reader.read_exact(&mut entry_buffer)?;

        let mut entry_cursor = Cursor::new(entry_buffer.as_slice());

        let key_tag = Tag::decode(&mut entry_cursor)?.ok_or(DecodeError::MalformedInput(
            "Missing key in map entry".to_string(),
        ))?;

        if key_tag.field_number != 1 {
            return Err(DecodeError::MalformedInput(
                "Expected field number 1 for key in map entry".to_string(),
            ));
        }

        let key = key_decoder(&mut entry_cursor)?;

        let value_tag = Tag::decode(&mut entry_cursor)?.ok_or(DecodeError::MalformedInput(
            "Missing value in map entry".to_string(),
        ))?;

        if value_tag.field_number != 2 {
            return Err(DecodeError::MalformedInput(
                "Expected field number 2 for value in map entry".to_string(),
            ));
        }

        let value = value_decoder(&mut entry_cursor)?;

        map.insert(key, value);
    }

    Ok(map)
}
