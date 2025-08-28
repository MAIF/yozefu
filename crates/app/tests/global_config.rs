use std::{fs, hash::DefaultHasher, path::PathBuf};

use indexmap::IndexMap;
use yozefu_app::configuration::{ConsumerConfig, GlobalConfig};

#[test]
fn check_backwards_compatibility() {
    use std::hash::Hasher;
    let config = GlobalConfig {
        clusters: IndexMap::new(),
        default_url_template: String::new(),
        yozefu_directory: PathBuf::new(),
        path: PathBuf::new(),
        logs: PathBuf::from("./yozefu.log").into(),
        initial_query: "from end - 10".to_string(),
        theme: "default".to_string(),
        default_kafka_config: IndexMap::new(),
        history: Default::default(),
        show_shortcuts: false,
        export_directory: PathBuf::from("./yozefu-exports"),
        consumer: ConsumerConfig::default(),
        highlighter_theme: None,
    };

    let json = serde_json::to_string_pretty(&config).unwrap();
    let mut hasher = DefaultHasher::new();
    hasher.write(json.as_bytes());
    let hash: u64 = hasher.finish();

    let input_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("inputs");

    fs::create_dir_all(&input_dir).unwrap();
    fs::write(input_dir.join(format!("{hash}.json")), json).unwrap();

    for file in fs::read_dir(input_dir).unwrap() {
        let file = file.unwrap().path();
        let content = fs::read_to_string(&file).unwrap();
        let result = serde_json::from_str::<GlobalConfig>(&content);
        assert!(
            result.is_ok(),
            "Breaking change in the configuration json file, '{file:?}' can't be deserialized: {result:?}"
        );
    }
}
