//! Inspired of <https://vallentin.dev/blog/post/versioning>
//!
//! GitHub environment variables <https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/store-information-in-variables>
//! The version message looks like this:
//! ```shell
//! yozefu 0.0.9 (develop:13aedf2, debug build, macos [aarch64])
//! https://github.com/MAIF/yozefu
//! Yann Prono <yann.prono@maif.fr>
//! ```
use const_format::{formatcp, str_index};
use std::env::consts::{ARCH, OS};

#[cfg(debug_assertions)]
const BUILD_TYPE: &str = "debug";

#[cfg(not(debug_assertions))]
const BUILD_TYPE: &'static str = "release";

const GIT_BRANCH: &str = match option_env!("GITHUB_REF_NAME") {
    Some(v) => v,
    None => "unknown",
};

const GIT_COMMIT: &str = match option_env!("GITHUB_SHA") {
    Some(v) => v,
    None => "unknown",
};

pub(super) const VERSION_MESSAGE: &str = formatcp!(
    "{} ({}:{}, {} build, {} [{}]) \n{}\n{}",
    env!("CARGO_PKG_VERSION"),
    GIT_BRANCH,
    str_index!(GIT_COMMIT, 0..7),
    BUILD_TYPE,
    OS,
    ARCH,
    env!("CARGO_PKG_REPOSITORY"),
    env!("CARGO_PKG_AUTHORS"),
);
