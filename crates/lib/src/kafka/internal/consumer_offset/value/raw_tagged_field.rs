pub struct RawTaggedField {
    tag: i32,
    data: Vec<u8>,
}

impl RawTaggedField {
    pub fn new(tag: i32, data: Vec<u8>) -> Self {
        Self { tag, data }
    }
}
