use byteorder::{BigEndian, ReadBytesExt};
use serde::Serialize;
use std::io::{Cursor, Error, ErrorKind, Read};

#[derive(Debug, Serialize)]
pub enum ConsumerOffsetValue {
    OffsetAndMetadata(OffsetAndMetadata),
}

impl TryFrom<&[u8]> for ConsumerOffsetValue {
    type Error = Error;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        let mut rdr = Cursor::new(buf);
        let version = rdr.read_i16::<BigEndian>()?;
        match version {
            OffsetCommitValue::LOWEST_SUPPORTED_VERSION
                ..=OffsetCommitValue::HIGHEST_SUPPORTED_VERSION => {
                Ok(ConsumerOffsetValue::OffsetAndMetadata(
                    OffsetAndMetadata::try_from(*rdr.get_ref())?,
                ))
            }
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unsupported consumer offset value version: {version}"),
            )),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OffsetAndMetadata {
    offset: i64,
    leader_epoch: Option<i32>,
    metadata: String,
    commit_timestamp: i64,
    expire_timestamp: Option<i64>,
}

impl OffsetAndMetadata {
    const DEFAULT_TIMESTAMP: i64 = -1;
    const NO_PARTITION_LEADER_EPOCH: i32 = -1;
}

impl TryFrom<&[u8]> for OffsetAndMetadata {
    type Error = Error;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        let mut rdr = Cursor::new(buf);
        let version = rdr
            .read_i16::<BigEndian>()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

        if (OffsetCommitValue::LOWEST_SUPPORTED_VERSION
            ..=OffsetCommitValue::HIGHEST_SUPPORTED_VERSION)
            .contains(&version)
        {
            let value = OffsetCommitValue::read(&mut rdr, version)?;

            Ok(OffsetAndMetadata {
                offset: value.offset,
                leader_epoch: if value.leader_epoch == Self::NO_PARTITION_LEADER_EPOCH {
                    None
                } else {
                    Some(value.leader_epoch)
                },
                metadata: value.metadata,
                commit_timestamp: value.commit_timestamp,
                expire_timestamp: if value.expire_timestamp == Self::DEFAULT_TIMESTAMP {
                    None
                } else {
                    Some(value.expire_timestamp)
                },
            })
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unsupported offset message version: {version}"),
            ))
        }
    }
}

pub struct OffsetCommitValue {
    pub offset: i64,
    pub leader_epoch: i32,
    pub metadata: String,
    pub commit_timestamp: i64,
    pub expire_timestamp: i64,
    // TODO pub unknown_tagged_fields: Vec<RawTaggedField>,
}

impl OffsetCommitValue {
    pub const LOWEST_SUPPORTED_VERSION: i16 = 0;
    pub const HIGHEST_SUPPORTED_VERSION: i16 = 4;

    pub fn read<R: Read>(reader: &mut R, version: i16) -> Result<Self, Error> {
        let offset = reader.read_i64::<BigEndian>()?;

        let leader_epoch = if version >= 3 {
            reader.read_i32::<BigEndian>()?
        } else {
            -1
        };

        let metadata = {
            let length = if version >= 4 {
                Self::read_unsigned_varint(reader)? as i32 - 1
            } else {
                reader.read_i16::<BigEndian>()? as i32
            };

            if length < 0 {
                return Err(Error::new(ErrorKind::InvalidData, "null metadata"));
            } else if length > 0x7fff {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Invalid metadata length: {length}"),
                ));
            } else {
                let mut buf = vec![0; length as usize];
                reader.read_exact(&mut buf)?;
                String::from_utf8(buf).map_err(|e| Error::new(ErrorKind::InvalidData, e))?
            }
        };

        let commit_timestamp = reader.read_i64::<BigEndian>()?;

        let expire_timestamp = if version == 1 {
            reader.read_i64::<BigEndian>()?
        } else {
            -1
        };

        //let unknown_tagged_fields = vec![];
        //if version >= 4 {
        //    let num_tagged_fields = Self::read_unsigned_varint(reader)?;
        //    for _ in 0..num_tagged_fields {
        //        let tag = Self::read_unsigned_varint(reader)?;
        //        let size = Self::read_unsigned_varint(reader)?;
        //        let mut data = vec![0u8; size as usize];
        //        reader.read_exact(&mut data)?;
        //        // TODO
        //        //                fields.push();
        //    }
        //}

        Ok(Self {
            offset,
            leader_epoch,
            metadata,
            commit_timestamp,
            expire_timestamp,
            // unknown_tagged_fields,
        })
    }

    // Helper for reading Kafka-style unsigned varints
    fn read_unsigned_varint<R: Read>(reader: &mut R) -> Result<u32, Error> {
        let mut value: u32 = 0;
        let mut shift = 0;
        for _ in 0..5 {
            let byte = {
                let mut buf = [0u8; 1];
                reader.read_exact(&mut buf)?;
                buf[0]
            };
            value |= ((byte & 0x7F) as u32) << shift;
            if (byte & 0x80) == 0 {
                return Ok(value);
            }
            shift += 7;
        }
        Err(Error::new(
            std::io::ErrorKind::InvalidData,
            "Varint too long",
        ))
    }
}
