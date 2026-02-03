use crate::assert_draw;
use crate::component::Component;
use crate::component::RootComponent;
use crate::component::default_state;

#[test]
fn test_draw() {
    use tokio::sync::mpsc::unbounded_channel;
    let (_tx, rx) = unbounded_channel();
    let mut component = RootComponent::new(
        "from begin",
        vec!["topic1".to_string(), "topic2".to_string()],
        rx,
        default_state(),
    );
    assert_draw!(component, 120, 20)
}
