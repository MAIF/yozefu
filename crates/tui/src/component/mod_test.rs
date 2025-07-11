#[macro_export]
macro_rules! assert_draw {
    ($component:expr, $width:expr, $height:expr) => {{
        use app::configuration::GlobalConfig;
        use insta::assert_snapshot;
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
            &GlobalConfig {
                path: temp_path.clone().join("config.json"),
                yozefu_directory: temp_path.join("config"),
                logs: None,
                default_url_template: "".to_string(),
                initial_query: "".to_string(),
                theme: "light".to_string(),
                clusters: indexmap::IndexMap::default(),
                default_kafka_config: indexmap::IndexMap::default(),
                history: vec![],
                show_shortcuts: true,
                export_directory: std::path::PathBuf::from(""),
            },
        );

        let mut terminal = Terminal::new(TestBackend::new($width, $height)).unwrap();
        terminal
            .draw(|frame| $component.draw(frame, frame.area(), &state).unwrap())
            .unwrap();


        insta::with_settings!({filters => vec![
            (format!("{}[a-z0-9\\/\\.+\\s']*", temp_path.display().to_string()).as_str(), "[PATH]"),
            (&format!("v{}", env!("CARGO_PKG_VERSION")), "[VERSION]"),
        ]}, {
            assert_snapshot!(terminal.backend());
        });
    }};
}
