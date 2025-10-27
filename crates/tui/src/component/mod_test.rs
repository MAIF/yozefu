#[macro_export]
macro_rules! assert_draw {
    ($component:expr, $width:expr, $height:expr) => {{
        use app::configuration::GlobalConfig;
        use app::configuration::InternalConfig;
        use app::configuration::ClusterConfig;
        use insta::assert_snapshot;
        use app::configuration::ConsumerConfig;
        use ratatui::{Terminal, backend::TestBackend};
        use $crate::{State, Theme};

        unsafe {
            use std::env;
            // Set the timezone to Paris to have a fixed timezone for the tests
            env::set_var("TZ", "Europe/Paris");
        }

        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        let state = State::new(
            "test",
            Theme::light(),
            &InternalConfig::new(
                ClusterConfig::default().create("test").with_exported_directory(temp_path.clone()),
                GlobalConfig {
                    path: temp_path.clone().join("config.json"),
                    yozefu_directory: temp_path.join("config"),
                    logs: None,
                    default_url_template: String::new(),
                    initial_query: String::new(),
                    theme: "light".to_string(),
                    highlighter_theme: None,
                    clusters: indexmap::IndexMap::default(),
                    default_kafka_config: indexmap::IndexMap::default(),
                    history: vec![],
                    show_shortcuts: true,
                    export_directory: temp_path.clone(),
                    consumer: ConsumerConfig::default(),
                }
            )
        );

        let mut terminal = Terminal::new(TestBackend::new($width, $height)).unwrap();
        terminal
            .draw(|frame| $component.draw(frame, frame.area(), &state).unwrap())
            .unwrap();


            println!("Snapshot path: {}", &state.config.output_file().display());
        insta::with_settings!({filters => vec![
            (format!("{}[a-z0-9T\\-\\/\\.+\\s']*", temp_path.display().to_string()).as_str(), "[PATH]"),
            (&format!("v{}", env!("CARGO_PKG_VERSION")), "[VERSION]"),
        ]}, {
            assert_snapshot!(terminal.backend());
        });
    }};
}
