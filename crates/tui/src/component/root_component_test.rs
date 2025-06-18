use crate::assert_draw;
use crate::component::Component;
use crate::{
    component::{ConcurrentRecordsBuffer, RootComponent, default_global_config, default_state},
    records_buffer::RecordsBuffer,
};
use std::sync::{Arc, LazyLock, Mutex};

static BUFFER: ConcurrentRecordsBuffer =
    LazyLock::new(|| Arc::new(Mutex::new(RecordsBuffer::new())));

#[test]
fn test_draw() {
    BUFFER.lock().unwrap().reset();
    let config = default_global_config();
    let mut component = RootComponent::new(
        "from begin".to_string(),
        vec!["topic1".to_string(), "topic2".to_string()],
        &config,
        &BUFFER,
        default_state(),
    );
    assert_draw!(component, 120, 20)
}
