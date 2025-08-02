use byteorder::{BigEndian, ReadBytesExt};
use serde::Serialize;
use std::io::{Cursor, Error as IoError};

use super::{OffsetCommitKey, group_metadata_key::GroupMetadataKey};

#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum ConsumerOffsetKey {
    OffsetCommitKey(OffsetCommitKey),
    GroupMetadataKey(GroupMetadataKey),
}

const LOWEST_SUPPORTED_VERSION: i16 = 0;
const HIGHEST_SUPPORTED_VERSION: i16 = 1;

impl TryFrom<&[u8]> for ConsumerOffsetKey {
    type Error = IoError;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        let mut rdr = Cursor::new(buf);
        let version: i16 = rdr.read_i16::<BigEndian>()?;
        match version {
            LOWEST_SUPPORTED_VERSION | HIGHEST_SUPPORTED_VERSION => Ok(
                ConsumerOffsetKey::OffsetCommitKey(OffsetCommitKey::try_from(*rdr.get_ref())?),
            ),
            _ => Ok(ConsumerOffsetKey::GroupMetadataKey(
                GroupMetadataKey::try_from(*rdr.get_ref())?,
            )),
        }
    }
}

#[test]
fn test_consumer_offset_key() {
    let input: Vec<u8> = vec![
        0, 2, 0, 15, 115, 99, 104, 101, 109, 97, 45, 114, 101, 103, 105, 115, 116, 114, 121,
    ];
    let offset_commit_key = ConsumerOffsetKey::try_from(&input[..]).unwrap();

    assert_eq!(
        ConsumerOffsetKey::GroupMetadataKey(GroupMetadataKey {
            group: "\0\u{f}".into()
        }),
        offset_commit_key
    );
}
