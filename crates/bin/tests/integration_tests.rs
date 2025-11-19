/*

use crate::yozefu_testcontainer::YozefuTestContainer;
use std::{path::PathBuf, sync::LazyLock};
use testcontainers::ImageExt;
pub mod yozefu_testcontainer;

static CONTAINER: LazyLock<YozefuTestContainer> = LazyLock::new(YozefuTestContainer::default);

#[test]
fn success() -> Result<(), Box<dyn std::error::Error>> {
    let container = CONTAINER.run(|image| image.with_cmd(["config"]))?;
    assert!(container.exit_code() == 0);
    Ok(())
}

#[test]
fn read_only_root_fs() -> Result<(), Box<dyn std::error::Error>> {
    let container = CONTAINER.run(|image| {
        image
            .with_cmd(["config"])
            .with_readonly_rootfs(true)
            .with_user("ubuntu")
    })?;

    assert!(container.exit_code() != 0);
    assert!(container.stderr().contains("Read-only file system"));
    Ok(())
}

#[test]
fn log_file_defined_with_env_variable() -> Result<(), Box<dyn std::error::Error>> {
    let log_file = PathBuf::from("/tmp/yozefu.log");
    let container = CONTAINER.run(|image| {
        image
            .with_cmd(["config"])
            .with_env_var("YOZEFU_LOG_FILE", log_file.display().to_string())
    })?;

    assert_eq!(
        0,
        container.exit_code(),
        "Container stderr: {}",
        container.stderr()
    );
    assert!(container.file_exist(log_file));
    Ok(())
}
 */
