use crate::{ConsumerGroupDetail, ConsumerGroupMember, ConsumerGroupState};

#[test]
fn test_lag() {
    let consumer_detail = ConsumerGroupDetail {
        name: "my-topic".to_string(),
        members: vec![
            ConsumerGroupMember {
                member: "member-1".to_string(),
                start_offset: 0,
                end_offset: 100,
                assignments: vec![],
            },
            ConsumerGroupMember {
                member: "member-2".to_string(),
                start_offset: 45,
                end_offset: 50,
                assignments: vec![],
            },
        ],
        state: ConsumerGroupState::Empty,
    };
    assert_eq!(consumer_detail.lag(), 105);
}
