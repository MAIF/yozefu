#[macro_export]
macro_rules! assert_draw {
    ($component:expr, $width:expr, $height:expr) => {{
        use app::configuration::InternalConfig;
        use app::configuration::ClusterConfig;
        use insta::assert_snapshot;
        use $crate::component::default_workspace;

        use ratatui::{Terminal, backend::TestBackend};
        use $crate::{State, Theme};

        unsafe {
            use std::env;
            // Set the timezone to Paris to have a fixed timezone for the tests
            env::set_var("TZ", "Europe/Paris");
        }

        let workspace = default_workspace();
        let dir = workspace.path.clone();

        let state = State::new(
            "test",
            Theme::light(),
            &InternalConfig::new(
                ClusterConfig::default().create("test").with_exported_directory(dir.clone()),
                workspace
            )
        );

        let mut terminal = Terminal::new(TestBackend::new($width, $height)).unwrap();
        terminal
            .draw(|frame| $component.draw(frame, frame.area(), &state).unwrap())
            .unwrap();


            println!("Snapshot path: {}", &state.config.output_file().display());
            println!("workspace is '{}'", &dir.display());
        insta::with_settings!({filters => vec![
            (format!("{}[a-z0-9T\\-\\/\\.+\\s']*", dir.display().to_string()).as_str(), "[PATH]"),
            (&format!("v{}", env!("CARGO_PKG_VERSION")), "[VERSION]"),
        ]}, {
            assert_snapshot!(terminal.backend());
        });
    }};
}
