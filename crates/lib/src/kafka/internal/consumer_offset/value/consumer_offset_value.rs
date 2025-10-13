use byteorder::{BigEndian, ReadBytesExt};
use serde::Serialize;
use std::io::{Cursor, Error, ErrorKind, Read};

#[derive(Debug, Serialize, PartialEq, Eq)]
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

#[derive(Debug, Serialize, PartialEq, Eq)]
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
                i32::from(reader.read_i16::<BigEndian>()?)
            };

            if length < 0 {
                return Err(Error::new(ErrorKind::InvalidData, "null metadata"));
            } else if length > 0x7fff {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Invalid metadata length: {length}"),
                ));
            }
            let mut buf =
                vec![0; usize::try_from(length).expect("Cannot allocate buffer for the metadata")];
            reader.read_exact(&mut buf)?;
            String::from_utf8(buf).map_err(|e| Error::new(ErrorKind::InvalidData, e))?
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
            value |= u32::from(byte & 0x7F) << shift;
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

#[test]
fn test_consumer_offset_value() {
    let input: Vec<u8> = vec![
        0, 3, 0, 2, 115, 114, 0, 0, 0, 1, 0, 2, 118, 48, 0, 41, 115, 114, 45, 49, 45, 50, 51, 55,
        51, 49, 97, 99, 102, 45, 53, 54, 53, 48, 45, 52, 50, 52, 48, 45, 56, 100, 49, 51, 45, 98,
        54, 50, 101, 98, 56, 51, 49, 102, 99, 97, 49, 0, 0, 1, 152, 106, 102, 87, 135, 0, 0, 0, 1,
        0, 41, 115, 114, 45, 49, 45, 50, 51, 55, 51, 49, 97, 99, 102, 45, 53, 54, 53, 48, 45, 52,
        50, 52, 48, 45, 56, 100, 49, 51, 45, 98, 54, 50, 101, 98, 56, 51, 49, 102, 99, 97, 49, 255,
        255, 0, 4, 115, 114, 45, 49, 0, 13, 47, 49, 57, 50, 46, 49, 54, 56, 46, 57, 55, 46, 51, 0,
        4, 147, 224, 0, 0, 39, 16, 0, 0, 0, 107, 123, 34, 104, 111, 115, 116, 34, 58, 34, 115, 99,
        104, 101, 109, 97, 45, 114, 101, 103, 105, 115, 116, 114, 121, 34, 44, 34, 112, 111, 114,
        116, 34, 58, 56, 48, 56, 50, 44, 34, 109, 97, 115, 116, 101, 114, 95, 101, 108, 105, 103,
        105, 98, 105, 108, 105, 116, 121, 34, 58, 116, 114, 117, 101, 44, 34, 115, 99, 104, 101,
        109, 101, 34, 58, 34, 104, 116, 116, 112, 34, 44, 34, 118, 101, 114, 115, 105, 111, 110,
        34, 58, 49, 44, 34, 108, 101, 97, 100, 101, 114, 34, 58, 102, 97, 108, 115, 101, 125, 0, 0,
        0, 202, 123, 34, 101, 114, 114, 111, 114, 34, 58, 48, 44, 34, 109, 97, 115, 116, 101, 114,
        34, 58, 34, 115, 114, 45, 49, 45, 50, 51, 55, 51, 49, 97, 99, 102, 45, 53, 54, 53, 48, 45,
        52, 50, 52, 48, 45, 56, 100, 49, 51, 45, 98, 54, 50, 101, 98, 56, 51, 49, 102, 99, 97, 49,
        34, 44, 34, 109, 97, 115, 116, 101, 114, 95, 105, 100, 101, 110, 116, 105, 116, 121, 34,
        58, 123, 34, 104, 111, 115, 116, 34, 58, 34, 115, 99, 104, 101, 109, 97, 45, 114, 101, 103,
        105, 115, 116, 114, 121, 34, 44, 34, 112, 111, 114, 116, 34, 58, 56, 48, 56, 50, 44, 34,
        109, 97, 115, 116, 101, 114, 95, 101, 108, 105, 103, 105, 98, 105, 108, 105, 116, 121, 34,
        58, 116, 114, 117, 101, 44, 34, 115, 99, 104, 101, 109, 101, 34, 58, 34, 104, 116, 116,
        112, 34, 44, 34, 118, 101, 114, 115, 105, 111, 110, 34, 58, 49, 44, 34, 108, 101, 97, 100,
        101, 114, 34, 58, 102, 97, 108, 115, 101, 125, 44, 34, 118, 101, 114, 115, 105, 111, 110,
        34, 58, 49, 125,
    ];
    let offset_commit_value = ConsumerOffsetValue::try_from(&input[..]).unwrap();

    assert_eq!(
        ConsumerOffsetValue::OffsetAndMetadata(OffsetAndMetadata {
            offset: 689883416887297,
            leader_epoch: Some(161328),
            metadata: "sr-1-23731acf-5650-4240-8d13-b62eb831fca1".into(),
            commit_timestamp: 1754131748743,
            expire_timestamp: None
        }),
        offset_commit_value
    );
}
