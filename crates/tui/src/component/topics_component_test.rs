use std::path::PathBuf;

use app::configuration::GlobalConfig;
use indexmap::IndexMap;
use insta::assert_snapshot;
use ratatui::{Terminal, backend::TestBackend};

use crate::{component::{help_component::HelpComponent, issue_component::IssueComponent, Component}, State, Theme};


#[test]
fn test_render() {
    let mut component = TopicsComponent::default();
    let state = State::new("test", Theme::light(), &GlobalConfig {
        path: PathBuf::from("test_config.json"),
        yozefu_directory: PathBuf::from("test_config.json"),
        logs: None,
        default_url_template: String::new(),
        initial_query: String::new(),
        theme: "light".to_string(),
        clusters: IndexMap::default(),
        default_kafka_config: IndexMap::default(),
        history: vec!(),
        show_shortcuts: false,
        export_directory: PathBuf::from(""),
    });
    let mut terminal = Terminal::new(TestBackend::new( 60, 15)).unwrap();
    terminal.draw(|frame| component.draw(frame, frame.area(), &state).unwrap()).unwrap();
    assert_snapshot!(terminal.backend());
}
