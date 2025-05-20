use crate::{Protobuf, WireType, encoder};
use std::collections::{HashMap, HashSet};
use std::io;

#[derive(Default)]
pub struct ProtobufBuilder {
    buffer: Vec<u8>,
    field_numbers: HashSet<u32>,
}

impl ProtobufBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    fn check_field(&mut self, field_number: u32) {
        if !self.field_numbers.insert(field_number) {
            panic!(
                "field number {} is already assigned to a field",
                field_number
            );
        }
    }

    pub fn add_uint32(&mut self, field_number: u32, value: u32) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_uint32(field_number, value, &mut self.buffer).unwrap();
        self
    }

    pub fn add_int32(&mut self, field_number: u32, value: i32) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_int32(field_number, value, &mut self.buffer).unwrap();
        self
    }

    pub fn add_int64(&mut self, field_number: u32, value: i64) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_int64(field_number, value, &mut self.buffer).unwrap();
        self
    }

    pub fn add_uint64(&mut self, field_number: u32, value: u64) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_uint64(field_number, value, &mut self.buffer).unwrap();
        self
    }

    pub fn add_sint32(&mut self, field_number: u32, value: i32) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_sint32(field_number, value, &mut self.buffer).unwrap();
        self
    }

    pub fn add_sint64(&mut self, field_number: u32, value: i64) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_sint64(field_number, value, &mut self.buffer).unwrap();
        self
    }

    pub fn add_bool(&mut self, field_number: u32, value: bool) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_bool(field_number, value, &mut self.buffer).unwrap();
        self
    }

    pub fn add_string(&mut self, field_number: u32, value: &str) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_string(field_number, value, &mut self.buffer).unwrap();
        self
    }

    pub fn add_bytes(&mut self, field_number: u32, value: &[u8]) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_bytes(field_number, value, &mut self.buffer).unwrap();
        self
    }

    pub fn add_float(&mut self, field_number: u32, value: f32) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_float(field_number, value, &mut self.buffer).unwrap();
        self
    }

    pub fn add_double(&mut self, field_number: u32, value: f64) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_double(field_number, value, &mut self.buffer).unwrap();
        self
    }

    pub fn add_fixed32(&mut self, field_number: u32, value: u32) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_fixed32(field_number, value, &mut self.buffer).unwrap();
        self
    }

    pub fn add_fixed64(&mut self, field_number: u32, value: u64) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_fixed64(field_number, value, &mut self.buffer).unwrap();
        self
    }

    pub fn add_sfixed32(&mut self, field_number: u32, value: i32) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_sfixed32(field_number, value, &mut self.buffer).unwrap();
        self
    }

    pub fn add_sfixed64(&mut self, field_number: u32, value: i64) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_sfixed64(field_number, value, &mut self.buffer).unwrap();
        self
    }

    pub fn add_enum<E: Into<i32>>(&mut self, field_number: u32, value: E) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_enum(field_number, value, &mut self.buffer).unwrap();
        self
    }

    pub fn add_repeated<T, F>(&mut self, field_number: u32, values: &[T], encoder: F) -> &mut Self
    where
        F: Fn(u32, &T, &mut Vec<u8>) -> io::Result<()>,
    {
        self.check_field(field_number);
        for value in values {
            encoder(field_number, value, &mut self.buffer).unwrap();
        }
        self
    }

    pub fn add_packed<T>(
        &mut self,
        field_number: u32,
        values: &[T],
        value_writer: impl Fn(&T, &mut Vec<u8>) -> io::Result<()>,
    ) -> &mut Self {
        self.check_field(field_number);
        encoder::encode_packed(field_number, values, &mut self.buffer, value_writer).unwrap();
        self
    }

    pub fn add_map<K, V, IK, IV>(
        &mut self,
        field_number: u32,
        map: HashMap<K, V>,
        key_encoder: IK,
        value_encoder: IV,
    ) -> &mut Self
    where
        IK: FnMut(u32, &K, &mut Vec<u8>) -> io::Result<()>,
        IV: FnMut(u32, &V, &mut Vec<u8>) -> io::Result<()>,
    {
        self.check_field(field_number);
        encoder::encode_map(
            field_number,
            map,
            &mut self.buffer,
            key_encoder,
            value_encoder,
        )
        .unwrap();
        self
    }

    pub fn add_message(&mut self, field_number: u32, mut message: ProtobufBuilder) -> &mut Self {
        self.check_field(field_number);

        let inner_bytes = message.build();
        encoder::encode_tag(field_number, WireType::LengthDelimited, &mut self.buffer).unwrap();
        encoder::encode_varint(inner_bytes.len() as u64, &mut self.buffer).unwrap();
        self.buffer.extend_from_slice(&inner_bytes);

        self
    }

    pub fn add_repeated_message(
        &mut self,
        field_number: u32,
        messages: Vec<ProtobufBuilder>,
    ) -> &mut Self {
        self.check_field(field_number);
        for mut message in messages {
            let inner_bytes = message.build();
            encoder::encode_tag(field_number, WireType::LengthDelimited, &mut self.buffer).unwrap();
            encoder::encode_varint(inner_bytes.len() as u64, &mut self.buffer).unwrap();
            self.buffer.extend_from_slice(&inner_bytes);
        }

        self
    }

    pub fn add_map_message_value<K>(
        &mut self,
        field_number: u32,
        map: HashMap<K, ProtobufBuilder>,
        mut key_encoder: impl FnMut(u32, &K, &mut Vec<u8>) -> io::Result<()>,
    ) -> &mut Self {
        self.check_field(field_number);
        for (key, mut value) in map {
            let mut entry_buf = Vec::new();

            key_encoder(1, &key, &mut entry_buf).unwrap();

            let value_bytes = value.build();
            encoder::encode_tag(2, WireType::LengthDelimited, &mut entry_buf).unwrap();
            encoder::encode_varint(value_bytes.len() as u64, &mut entry_buf).unwrap();
            entry_buf.extend_from_slice(&value_bytes);

            encoder::encode_tag(field_number, WireType::LengthDelimited, &mut self.buffer).unwrap();
            encoder::encode_varint(entry_buf.len() as u64, &mut self.buffer).unwrap();
            self.buffer.extend_from_slice(&entry_buf);
        }

        self
    }

    pub fn add_message_with_trait<P: Protobuf>(
        &mut self,
        field_number: u32,
        message: P,
    ) -> &mut Self {
        self.check_field(field_number);

        let inner_bytes = message.encode_to_vec();
        encoder::encode_tag(field_number, WireType::LengthDelimited, &mut self.buffer).unwrap();
        encoder::encode_varint(inner_bytes.len() as u64, &mut self.buffer).unwrap();
        self.buffer.extend_from_slice(&inner_bytes);

        self
    }

    pub fn add_repeated_message_with_trait<P: Protobuf>(
        &mut self,
        field_number: u32,
        messages: Vec<P>,
    ) -> &mut Self {
        self.check_field(field_number);
        for message in messages {
            let inner_bytes = message.encode_to_vec();
            encoder::encode_tag(field_number, WireType::LengthDelimited, &mut self.buffer).unwrap();
            encoder::encode_varint(inner_bytes.len() as u64, &mut self.buffer).unwrap();
            self.buffer.extend_from_slice(&inner_bytes);
        }

        self
    }

    pub fn add_map_message_value_with_trait<K, P: Protobuf>(
        &mut self,
        field_number: u32,
        map: HashMap<K, P>,
        mut key_encoder: impl FnMut(u32, &K, &mut Vec<u8>) -> io::Result<()>,
    ) -> &mut Self {
        self.check_field(field_number);
        for (key, value) in map {
            let mut entry_buf = Vec::new();

            key_encoder(1, &key, &mut entry_buf).unwrap();

            let value_bytes = value.encode_to_vec();
            encoder::encode_tag(2, WireType::LengthDelimited, &mut entry_buf).unwrap();
            encoder::encode_varint(value_bytes.len() as u64, &mut entry_buf).unwrap();
            entry_buf.extend_from_slice(&value_bytes);

            encoder::encode_tag(field_number, WireType::LengthDelimited, &mut self.buffer).unwrap();
            encoder::encode_varint(entry_buf.len() as u64, &mut self.buffer).unwrap();
            self.buffer.extend_from_slice(&entry_buf);
        }

        self
    }

    pub fn build(&mut self) -> Vec<u8> {
        let result = std::mem::take(&mut self.buffer);
        std::mem::drop(std::mem::take(&mut self.field_numbers));
        result
    }
}
