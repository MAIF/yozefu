FROM rust:1-slim-trixie AS builder
WORKDIR /app
RUN apt-get update && apt-get install --no-install-recommends -y git build-essential cmake libclang-dev
RUN --mount=type=bind,source=crates,target=crates \
    --mount=type=bind,source=.cargo/,target=.cargo/ \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=.git,target=.git \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
export GITHUB_REF_NAME=$(git rev-parse --abbrev-ref HEAD)
export GITHUB_SHA=$(git rev-parse HEAD)
cargo build --release --all-features --locked
cp ./target/release/yozf /tmp/yozf
EOF



FROM debian:trixie-slim AS final
ARG UID=10001
RUN useradd \
    --shell "/sbin/nologin" \
    --uid "${UID}" \
    yozefu
RUN apt-get update && \
    apt-get install --no-install-recommends vim jq ca-certificates --yes && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder "/tmp/yozf" /bin/yozf
RUN <<EOF
ln -fs "/bin/yozf" /usr/local/bin/yozf
ln -fs "/bin/yozf" /usr/local/bin/yozefu
ln -fs "/bin/yozf" /usr/bin/yozf
ln -fs "/bin/yozf" /usr/local/bin/yozefu
EOF
RUN /bin/yozf --version

USER yozefu
WORKDIR /home/yozefu
ENTRYPOINT ["/bin/yozf"]


# docker pull ghcr.io/maif/yozefu:latest
# gh attestation verify --owner MAIF oci://ghcr.io/maif/yozefu:latest
#
# docker run --rm -it ghcr.io/maif/yozefu:latest -c localhost
# configuration is located at '/home/yozefu/.config/yozefu/config.json'
# 
# If you need to build the project with an alpine image:
# apk add perl musl-dev build-base clang-dev cmake git openssl-dev pkgconfig
