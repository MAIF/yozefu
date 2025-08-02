use std::io::{Cursor, Read};

use byteorder::{BigEndian, ReadBytesExt};
use serde::Serialize;
use std::io::Error as IoError;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct OffsetCommitKey {
    group: String,
    topic: String,
    partition: i32,
}

impl OffsetCommitKey {
    pub fn new(group: String, topic: String, partition: i32) -> Self {
        OffsetCommitKey {
            group,
            topic,
            partition,
        }
    }
}

impl TryFrom<&[u8]> for OffsetCommitKey {
    type Error = IoError;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = Cursor::new(buf);
        let group = {
            let length = reader.read_i16::<BigEndian>()? as i32;
            if length < 0 {
                return Err(IoError::new(
                    std::io::ErrorKind::InvalidData,
                    "Negative length",
                ));
            } else if length > 0x7fff {
                return Err(IoError::new(
                    std::io::ErrorKind::InvalidData,
                    "Length too long",
                ));
            }
            let mut buf = vec![0; length as usize];
            reader.read_exact(&mut buf)?;
            String::from_utf8(buf)
                .map_err(|_| IoError::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?
        };

        let topic = {
            let length = reader.read_i16::<BigEndian>()? as i32;
            if length < 0 {
                return Err(IoError::new(
                    std::io::ErrorKind::InvalidData,
                    "Negative length",
                ));
            } else if length > 0x7fff {
                return Err(IoError::new(
                    std::io::ErrorKind::InvalidData,
                    "Length too long",
                ));
            }
            let mut buf = vec![0; length as usize];
            reader.read_exact(&mut buf)?;
            String::from_utf8(buf)
                .map_err(|_| IoError::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?
        };

        let partition = reader.read_i32::<BigEndian>()?;

        Ok(OffsetCommitKey::new(group, topic, partition))
    }
}
