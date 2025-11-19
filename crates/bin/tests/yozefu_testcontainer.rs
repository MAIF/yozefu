use assert_cmd::cargo;
use std::hash::Hasher;
use std::{fs, hash::DefaultHasher, path::PathBuf};
use testcontainers::ContainerRequest;
use testcontainers::core::ExecCommand;
use testcontainers::{
    GenericBuildableImage, GenericImage, ImageExt,
    core::{Mount, wait::ExitWaitStrategy},
    runners::{SyncBuilder, SyncRunner},
};

const DOCKERFILE: &str = r#"
FROM rust:1-slim-trixie AS builder
WORKDIR /app
ENV RUST_BACKTRACE="full"
RUN apt-get update && apt-get install --no-install-recommends -y git build-essential cmake libclang-dev
RUN --mount=type=bind,source=crates,target=crates \
    --mount=type=bind,source=.cargo/,target=.cargo/ \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    <<EOF
set -e
cargo build
EOF
"#;

pub struct YozefuTestContainer {
    binary_path: PathBuf,
}

impl Default for YozefuTestContainer {
    fn default() -> Self {
        let binary_path = Self::build().expect("Cannot build Yozefu binary");
        Self { binary_path }
    }
}

impl YozefuTestContainer {
    fn image(&self) -> ContainerRequest<GenericImage> {
        GenericImage::new("ubuntu", "latest")
            .with_entrypoint("/tmp/app")
            .with_wait_for(testcontainers::core::WaitFor::Exit(
                ExitWaitStrategy::default(),
            ))
            .with_env_var("RUST_BACKTRACE", "full")
            .with_mount(Mount::bind_mount(
                self.binary_path.to_str().unwrap(),
                "/tmp/app",
            ))
    }

    fn build() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let cmd = cargo::cargo_bin_cmd!("yozf");
        let mut hasher = DefaultHasher::new();
        hasher.write(&fs::read(cmd.get_program())?);
        let hash: u64 = hasher.finish();

        let root_repo = env!("CARGO_MANIFEST_DIR");
        let root_repo = PathBuf::from(root_repo)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();

        let output_dir = root_repo
            .join("target")
            .join("debug")
            .join("integration-test");
        let linux_binary = output_dir.join(format!("yozf-{}", hash));

        if fs::metadata(&linux_binary).is_ok() {
            return Ok(linux_binary);
        }

        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir)?;

        println!("Building Yozefu with docker...");
        let docker_image = GenericBuildableImage::new("yozefu-integration-tests", "latest")
            .with_dockerfile_string(DOCKERFILE)
            .with_file(root_repo.join("crates"), "./crates")
            .with_file(root_repo.join(".cargo"), "./.cargo")
            .with_file(root_repo.join("Cargo.toml"), "./Cargo.toml")
            .with_file(root_repo.join("Cargo.lock"), "./Cargo.lock")
            .build_image()
            .expect("Cannot build the docker image");

        let container = docker_image
            .with_entrypoint("sleep")
            .with_wait_for(testcontainers::core::WaitFor::Exit(
                ExitWaitStrategy::default(),
            ))
            .with_cmd(["3"])
            .start()?;

        let _ = std::process::Command::new("docker")
            .args([
                "cp",
                &format!("{}:/app/target/debug/yozf", container.id()),
                linux_binary.to_str().unwrap(),
            ])
            .status()?;

        Ok(linux_binary)
    }

    pub fn run(
        &self,
        fnn: impl FnOnce(ContainerRequest<GenericImage>) -> ContainerRequest<GenericImage>,
    ) -> Result<Container, Box<dyn std::error::Error>> {
        let container = self.image();
        let container = fnn(container);
        let running = container.start()?;

        Ok(Container::new(running))
    }
}

pub struct Container {
    container: testcontainers::Container<GenericImage>,
}

impl Container {
    pub fn new(container: testcontainers::Container<GenericImage>) -> Self {
        Self { container }
    }
    pub fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.container.stdout_to_vec().unwrap()).to_string()
    }
    pub fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.container.stderr_to_vec().unwrap()).to_string()
    }

    pub fn exit_code(&self) -> i64 {
        self.container.exit_code().unwrap().unwrap()
    }

    pub fn file_exist(&self, path: PathBuf) -> bool {
        self.container
            .exec(ExecCommand::new(vec![
                "test",
                "-f",
                &path.display().to_string(),
            ]))
            .unwrap()
            .exit_code()
            .unwrap()
            == Some(0)
    }
}
