FROM rust:1-slim-bookworm AS builder
WORKDIR /app
RUN --mount=type=bind,source=crates,target=crates \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
apt-get update && apt-get install --no-install-recommends -y build-essential cmake libclang-dev libsasl2-dev libssl-dev pkg-config
cargo build --release --all-features --locked
cp ./target/release/yozf /tmp/yozf
EOF



FROM debian:bookworm-slim AS final
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --shell "/sbin/nologin" \
    --uid "${UID}" \
    yozefu
RUN apt-get update && \
    apt-get install --no-install-recommends vim jq ca-certificates libsasl2-dev libssl-dev --yes && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder "/tmp/yozf" /bin/app
RUN <<EOF
ln -fs "/bin/app" /usr/local/bin/yozf
ln -fs "/bin/app" /usr/local/bin/yozefu
ln -fs "/bin/app" /usr/bin/yozf
ln -fs "/bin/app" /usr/local/bin/yozefu
EOF

USER yozefu
WORKDIR /home/yozefu
ENTRYPOINT ["/bin/app"]


# docker pull ghcr.io/maif/yozefu:latest
# gh attestation verify --owner MAIF oci://ghcr.io/maif/yozefu:latest
#
# docker run --rm -it ghcr.io/maif/yozefu:latest -c localhost
# configuration is located at '/home/yozefu/.config/yozefu/config.json'
