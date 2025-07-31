use byteorder::{BigEndian, ReadBytesExt};
use serde::Serialize;
use std::io::{Cursor, Error as IoError, Read};

#[derive(Debug, Serialize)]
pub struct GroupMetadataKey {
    group: String,
}

impl TryFrom<&[u8]> for GroupMetadataKey {
    type Error = IoError;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = Cursor::new(buf);
        let group = {
            let length = reader.read_i16::<BigEndian>()? as i32;
            if length < 0 {
                return Err(IoError::new(
                    std::io::ErrorKind::InvalidData,
                    "Field is null",
                ));
            } else if length > 0x7fff {
                return Err(IoError::new(
                    std::io::ErrorKind::InvalidData,
                    "Length too long",
                ));
            }
            let mut buf = vec![0u8; length as usize];
            reader.read_exact(&mut buf)?;
            String::from_utf8(buf)
                .map_err(|_| IoError::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?
        };

        Ok(GroupMetadataKey { group })
    }
}
