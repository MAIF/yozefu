use byteorder::{BigEndian, ReadBytesExt};
use serde::Serialize;
use std::io::{Cursor, Error as IoError};

use super::{OffsetCommitKey, group_metadata_key::GroupMetadataKey};

#[derive(Debug, Serialize)]
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
