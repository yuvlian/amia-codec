pub mod builder;
pub mod decoder;
pub mod encoder;

use std::io::{self, Cursor, Read, Write};

pub trait Protobuf: Sized + Default {
    fn encode_to_writer<W: Write>(&self, writer: &mut W) -> io::Result<()>;
    fn encoded_len(&self) -> usize;
    fn encode_to_vec(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(self.encoded_len());
        self.encode_to_writer(&mut buffer).unwrap();
        buffer
    }
    fn decode_from_reader<R: Read>(reader: &mut R) -> DecodeResult<Self>;
    fn decode_from_slice(bytes: &[u8]) -> DecodeResult<Self> {
        let mut cursor = Cursor::new(bytes);
        Self::decode_from_reader(&mut cursor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireType {
    Varint = 0,
    Fixed64 = 1,
    LengthDelimited = 2,
    #[deprecated(note = "StartGroup is deprecated in proto3.")]
    StartGroup = 3,
    #[deprecated(note = "EndGroup is deprecated in proto3.")]
    EndGroup = 4,
    Fixed32 = 5,
}

#[derive(Debug)]
pub enum DecodeError {
    IoError(io::Error),
    UnexpectedEof,
    InvalidWireType(u32),
    InvalidTag,
    InvalidVarint,
    InvalidUtf8(std::string::FromUtf8Error),
    UnexpectedWireType { expected: WireType, got: WireType },
    MalformedInput(String),
}

impl From<io::Error> for DecodeError {
    fn from(err: io::Error) -> Self {
        DecodeError::IoError(err)
    }
}

impl From<std::string::FromUtf8Error> for DecodeError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        DecodeError::InvalidUtf8(err)
    }
}

pub type DecodeResult<T> = Result<T, DecodeError>;
