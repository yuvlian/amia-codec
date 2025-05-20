use std::io::{self, Write};

const BASE64_TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const INVALID: u8 = 255;

const fn build_reverse_table() -> [u8; 256] {
    let mut table = [INVALID; 256];
    let mut i = 0;
    while i < 64 {
        table[BASE64_TABLE[i] as usize] = i as u8;
        i += 1;
    }
    table
}

const REVERSE_BASE64_TABLE: [u8; 256] = build_reverse_table();

pub trait Base64 {
    fn encode_base64(&self) -> io::Result<String>;
    fn decode_base64(&self) -> io::Result<Vec<u8>>;
}

fn encode_to_writer<W: Write>(data: &[u8], writer: &mut W) -> io::Result<()> {
    data.chunks(3)
        .map(|chunk| {
            let (b0, b1, b2) = (
                chunk.get(0).copied().unwrap_or(0),
                chunk.get(1).copied().unwrap_or(0),
                chunk.get(2).copied().unwrap_or(0),
            );
            let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
            let output = [
                BASE64_TABLE[((n >> 18) & 0x3F) as usize],
                BASE64_TABLE[((n >> 12) & 0x3F) as usize],
                if chunk.len() > 1 {
                    BASE64_TABLE[((n >> 6) & 0x3F) as usize]
                } else {
                    b'='
                },
                if chunk.len() > 2 {
                    BASE64_TABLE[(n & 0x3F) as usize]
                } else {
                    b'='
                },
            ];
            output
        })
        .try_for_each(|buf| writer.write_all(&buf))
}

fn decode_to_writer<W: Write>(input: &[u8], writer: &mut W) -> io::Result<()> {
    if input.len() % 4 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid Base64 length",
        ));
    }

    for chunk in input.chunks(4) {
        let c0 = chunk[0];
        let c1 = chunk[1];
        let c2 = chunk[2];
        let c3 = chunk[3];

        let v0 = REVERSE_BASE64_TABLE[c0 as usize];
        let v1 = REVERSE_BASE64_TABLE[c1 as usize];
        let v2 = if c2 != b'=' {
            REVERSE_BASE64_TABLE[c2 as usize]
        } else {
            0
        };
        let v3 = if c3 != b'=' {
            REVERSE_BASE64_TABLE[c3 as usize]
        } else {
            0
        };

        if v0 == INVALID
            || v1 == INVALID
            || (c2 != b'=' && v2 == INVALID)
            || (c3 != b'=' && v3 == INVALID)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Base64 character",
            ));
        }

        let b0 = (v0 << 2) | (v1 >> 4);
        writer.write_all(&[b0])?;

        if c2 != b'=' {
            let b1 = (v1 << 4) | (v2 >> 2);
            writer.write_all(&[b1])?;
        }

        if c3 != b'=' {
            let b2 = (v2 << 6) | v3;
            writer.write_all(&[b2])?;
        }
    }

    Ok(())
}

impl<T: AsRef<[u8]>> Base64 for T {
    fn encode_base64(&self) -> io::Result<String> {
        let data = self.as_ref();
        let mut result = Vec::with_capacity(4 * ((data.len() + 2) / 3));
        encode_to_writer(data, &mut result)?;
        Ok(String::from_utf8(result).unwrap())
    }

    fn decode_base64(&self) -> io::Result<Vec<u8>> {
        let input = self.as_ref();
        let filtered = input
            .iter()
            .copied()
            .filter(|&b| b != b'\r' && b != b'\n')
            .collect::<Vec<_>>();
        let mut output = Vec::with_capacity(filtered.len() / 4 * 3);
        decode_to_writer(&filtered, &mut output)?;
        Ok(output)
    }
}
