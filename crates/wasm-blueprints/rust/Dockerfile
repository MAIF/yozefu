FROM rust:alpine

RUN apk update --no-cache && apk add make musl-dev
WORKDIR /tmp/build
COPY . .
RUN make module.wasm