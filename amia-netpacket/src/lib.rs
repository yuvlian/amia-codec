pub mod hsr;

#[cfg(feature = "kcp")]
pub mod op;

#[derive(Debug)]
pub enum PacketError {
    TooShort,
    InvalidHeadMagic,
    InvalidTailMagic,
    SizeMismatch,
}

impl std::fmt::Display for PacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for PacketError {}

#[cfg(feature = "tokio")]
impl From<PacketError> for std::io::Error {
    fn from(err: PacketError) -> Self {
        use PacketError::*;
        let kind = match err {
            TooShort => std::io::ErrorKind::UnexpectedEof,
            SizeMismatch => std::io::ErrorKind::InvalidData,
            InvalidHeadMagic | InvalidTailMagic => std::io::ErrorKind::InvalidData,
        };
        std::io::Error::new(kind, err)
    }
}
