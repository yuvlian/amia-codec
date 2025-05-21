use crate::PacketError;
use byteorder::{BE, ByteOrder};

#[derive(Debug)]
pub struct NetOperation {
    pub head: u32,
    pub conv: u32,
    pub token: u32,
    pub data: u32,
    pub tail: u32,
}

impl TryFrom<&[u8]> for NetOperation {
    type Error = PacketError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != 20 {
            return Err(PacketError::SizeMismatch);
        }

        Ok(Self {
            head: BE::read_u32(&bytes[0..4]),
            conv: BE::read_u32(&bytes[4..8]),
            token: BE::read_u32(&bytes[8..12]),
            data: BE::read_u32(&bytes[12..16]),
            tail: BE::read_u32(&bytes[16..20]),
        })
    }
}

impl From<NetOperation> for [u8; 20] {
    fn from(op: NetOperation) -> Self {
        let mut buf = [0u8; 20];
        BE::write_u32(&mut buf[0..4], op.head);
        BE::write_u32(&mut buf[4..8], op.conv);
        BE::write_u32(&mut buf[8..12], op.token);
        BE::write_u32(&mut buf[12..16], op.data);
        BE::write_u32(&mut buf[16..20], op.tail);
        buf
    }
}

impl From<NetOperation> for Box<[u8]> {
    fn from(op: NetOperation) -> Self {
        Box::new(<[u8; 20]>::from(op))
    }
}

impl From<NetOperation> for Vec<u8> {
    fn from(op: NetOperation) -> Self {
        <[u8; 20]>::from(op).to_vec()
    }
}
