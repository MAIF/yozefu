use crate::kafka::{SchemaId, schema::MAGIC_BYTE};

#[test]
fn test_parse_schema_id() {
    assert_eq!(SchemaId::parse(None), None);
    assert_eq!(SchemaId::parse(Some(&[0, 0, 0, 0, 0])), Some(SchemaId(0)));
    assert_eq!(SchemaId::parse(Some(&[0, 0, 0, 0, 1])), Some(SchemaId(1)));
    assert_eq!(
        SchemaId::parse(Some(&[0, 0, 0, 4, 2])),
        Some(SchemaId(1026))
    );
    assert_eq!(SchemaId::parse(Some(&[54, 0, 0, 0, 1])), None);
}

use proptest::prelude::*;

proptest! {
    #[test]
    fn parses_magic_byte_payload(body in any::<[u8;4]>()) {
        let mut buf = [0u8;5];
        buf[0] = MAGIC_BYTE;
        buf[1..5].copy_from_slice(&body);
        let got = SchemaId::parse(Some(&buf));
        prop_assert_eq!(got, Some(SchemaId(u32::from_be_bytes(body))));
    }

    #[test]
    fn rejects_non_magic_byte(b in 1u8..=255u8, body in any::<[u8;4]>()) {
        let mut buf = [0u8;5];
        buf[0] = b;
        buf[1..5].copy_from_slice(&body);
        prop_assert_eq!(SchemaId::parse(Some(&buf)), None);
    }
}
