## What's Changed in v0.0.12
* Develop by @Mcdostone in [#87](https://github.com/MAIF/yozefu/pull/87)
* docs: fix typos by @Mcdostone
* test: added more details when the createFilter test failed by @Mcdostone
* feat: new config for the consumer batch size and the timeout by @Mcdostone
* build: add job to build yozefu with archlinux by @Mcdostone
* build: dependabot config for docker by @Mcdostone
* perf: try to improve throughput of read kafka records by @Mcdostone
* docs: update README.md by @Mcdostone
* feat: Trying to parse records published in `__consumer_offsets` by @Mcdostone
* build: enable `tokio_unstable` by @Mcdostone
* refactor: tracing regarding parsing time by @Mcdostone
* fix: fix preselected topics when using --topics by @Mcdostone
* feat: move to tracing by @Mcdostone
* feat: selected topics are highlighted in the `TopicsComponent` by @Mcdostone
* feat: generate a json schema for Kafka record. by @Mcdostone
* refactor: remove unused enum FocusDirection by @Mcdostone
* docs: add json schema for the configuration file by @Mcdostone
* build: update deps by @Mcdostone
* ci: fix the changelog action by @Mcdostone
* chore: Release version v0.0.12 by @Mcdostone
* feat: minimum width for the topics component by @Mcdostone
* docs: update typo in README.md by @Mcdostone
* chore: Update changelog by @Mcdostone in [#86](https://github.com/MAIF/yozefu/pull/86)
* chore: Update changelog by @Mcdostone

**Full Changelog**: https://github.com/MAIF/yozefu/compare/v0.0.11...v0.0.12

# Changelog

All notable changes to this project will be documented in this file.

## What's Changed in v0.0.11
* build: rollback to ratatui 0.29.0
* ci: fix CI about changelog, something went wrong during publishing the crates
* chore: Release version v0.0.11
* style: remove padding in topics component
* refactor: remove unused enum
* build(deps): bump raven-actions/actionlint from 2.0.0 to 2.0.1
* feat: show how long ago the record was published
* test: tests that wasm filters don't have access to network
* feat: update to rust 1.88.0
* feat: disable IO interactions for wasm filters
* style: design improvements based on https://github.com/ratatui/ratatui-website/pull/868
* build: update dependencies
* ci: remove hardcoded URLs for `cargo-semver-checks`
* ci: enable some clippy lints
* refactor: format, removed unused deps
* docs: fix URLs
* chore: Update changelog
* ci: rollback temporary change in github action
* fix: rollback regarding commit and branch

**Full Changelog**: https://github.com/MAIF/yozefu/compare/v0.0.10...v0.0.11

## What's Changed in v0.0.10
* fix: rollback regarding commit and branch by @Mcdostone
* Develop by @Mcdostone in [#66](https://github.com/MAIF/yozefu/pull/66)
* docs: add link to https://docs.rs by @Mcdostone
* chore: Release version v0.0.10 by @Mcdostone
* feat: improve the `--version` output by @Mcdostone
* build: simplify cross config by @Mcdostone
* style: the default theme on windows is dark by @Mcdostone
* docs: improve try-it.sh by @Mcdostone
* ci: improve CI, some errors were not caught by @Mcdostone
* build: try to fix static rdkakfa by @Mcdostone
* style: update themes file by @Mcdostone
* test: fix tests and semver-checks by @Mcdostone
* build: fix docker build, `libclang-dev` was missing by @Mcdostone
* fix: log error when the search query is invalid by @Mcdostone
* build: update dependencies by @Mcdostone
* docs: update demo gif by @Mcdostone
* feat: equal could be '==' or '=' by @Mcdostone
* docs: create vhs demo by @Mcdostone
* docs: add conduktor for tests by @Mcdostone
* docs: update documentation by @Mcdostone
* Changelog for v0.0.9 by @github-actions[bot] in [#58](https://github.com/MAIF/yozefu/pull/58)

**Full Changelog**: https://github.com/MAIF/yozefu/compare/v0.0.9...v0.0.10

## What's Changed in v0.0.9
* build: rust 1.85.0, 2024 edition by @Mcdostone in [#51](https://github.com/MAIF/yozefu/pull/51)
* build: update dependencies by @Mcdostone
* ci: add cargo deny to build CI by @Mcdostone
* docs: documentation for golang search filter, fix Makefile by @Mcdostone
* feat: try to add support for JS by @Mcdostone
* build: rust 1.85.0, 2024 edition by @Mcdostone
* test: fix try-it.sh by @Mcdostone
* Changelog for v0.0.8 by @github-actions[bot] in [#50](https://github.com/MAIF/yozefu/pull/50)

**Full Changelog**: https://github.com/MAIF/yozefu/compare/v0.0.8...v0.0.9

## What's Changed in v0.0.8
* fix: better error message when the cluster is unknown or missing by @Mcdostone in [#45](https://github.com/MAIF/yozefu/pull/45)
* build: update dependencies by @Mcdostone
* build: `strum` as a workspace dependency by @Mcdostone
* ci: compute the next version instead of failing the github action by @Mcdostone
* docs: fix video link on crates.io by @Mcdostone
* fix: better error message when the cluster is unknown or missing by @Mcdostone
* refactor: change error message when the cluster is unknown by @Mcdostone
* refactor: specify the MSRV by @Mcdostone
* build: update Dockerfile by @Mcdostone
* docs: fix screenshot URLs by @Mcdostone
* Changelog for v0.0.7 by @github-actions[bot] in [#44](https://github.com/MAIF/yozefu/pull/44)

**Full Changelog**: https://github.com/MAIF/yozefu/compare/v0.0.7...v0.0.8

## What's Changed in v0.0.7
* Develop by @Mcdostone in [#42](https://github.com/MAIF/yozefu/pull/42)
* build: update dependencies by @Mcdostone
* refactor: cached filters are now passed to the `SearchContext` by @Mcdostone
* refactor: introducing the trait `Cluster` for the `--cluster` argument by @Mcdostone
* fix: trying to get rid of static variables by @Mcdostone
* refactor: simplify the `Component` trait by @Mcdostone
* refactor: Move common dependencies to the root `Cargo.toml` by @Mcdostone
* refactor: remove `#[async_trait]` from `Search` trait by @Mcdostone
* refactor: make TUI components `pub(crate)` by @Mcdostone
* docs: add screenshots of the tool by @Mcdostone
* docs: make try-it.sh display the URL to download yozefu based on user's machine by @Mcdostone
* fix: make cluster argument not optional since it's required by @Mcdostone
* Changelog for v0.0.6 by @github-actions[bot] in [#41](https://github.com/MAIF/yozefu/pull/41)

**Full Changelog**: https://github.com/MAIF/yozefu/compare/v0.0.6...v0.0.7

## What's Changed in v0.0.6
* Develop by @Mcdostone in [#37](https://github.com/MAIF/yozefu/pull/37)
* build: update dependencies by @Mcdostone
* feat: make `KAFKA_PROPERTIES_WITH_LOCATIONS` public by @Mcdostone
* docs: fix links by @Mcdostone
* build: update deps by @Mcdostone
* refactor: remove unused `Sender` by @Mcdostone
* build: update nom by @Mcdostone
* Changelog for v0.0.5 by @github-actions[bot] in [#34](https://github.com/MAIF/yozefu/pull/34)
* ci: another attempt ro fix CI for changelog by @Mcdostone

**Full Changelog**: https://github.com/MAIF/yozefu/compare/v0.0.5...v0.0.6

## What's Changed in v0.0.5
* Develop by @Mcdostone in [#28](https://github.com/MAIF/yozefu/pull/28)
* fix: fix the table state in `topicDetailsComponent`, adjust delay for checkpoint while consuming records by @Mcdostone
* docs: remove powerline-fonts by @Mcdostone
* fix: fix panics when resizing the height of the terminal window to 0 by @Mcdostone
* refactor: change the order of attribute for struct `KafkaRecord` for better user experience by @Mcdostone
* refactor: make component attributes private by @Mcdostone
* feat: Feedback to user when refreshing topic details component, make consumer members table scrollable by @Mcdostone
* feat: introducing `YozefuConfig` that gathers configuration of the tool by @Mcdostone
* test: create a kafka consumer that commits offset by @Mcdostone
* feat: make TopicDetailComponent scrollable by @Mcdostone
* chore: Update changelog by @github-actions[bot] in [#25](https://github.com/MAIF/yozefu/pull/25)

**Full Changelog**: https://github.com/MAIF/yozefu/compare/v0.0.4...v0.0.5

## What's Changed in v0.0.4
* Fix/user provided kafka properties by @Mcdostone in [#23](https://github.com/MAIF/yozefu/pull/23)
* ci: add cargo-deny by @Mcdostone
* feat: add a function `logs_file` that specifies a file to write logs to by @Mcdostone
* chore: Release version v0.0.4 by @Mcdostone
* build: update dependencies by @Mcdostone
* fix: user-provided kafka properties were not taken into account when creating the kafka client config by @Mcdostone
* ci: fix github action that creates changelog PR by @Mcdostone
* chore: Update changelog by @github-actions[bot] in [#20](https://github.com/MAIF/yozefu/pull/20)

**Full Changelog**: https://github.com/MAIF/yozefu/compare/v0.0.3...v0.0.4

## What's Changed in v0.0.3
* fix `KeyEvent` on windows by @Mcdostone in [#19](https://github.com/MAIF/yozefu/pull/19)
* test: snapshots tests for `KafkaRecord::parse` by @Mcdostone
* feat: Try its best to deserialize the record payload when the schema registry is not configured by @Mcdostone
* build: update dependencies by @Mcdostone
* fix: get rid of `unwrap()` when fetching the schema registry config given a cluster name by @Mcdostone
* fix: trying to make yozefu work on windows by @Mcdostone
* update dependencies by @Mcdostone in [#16](https://github.com/MAIF/yozefu/pull/16)
* build(deps): bump houseabsolute/actions-rust-cross from 0.0.17 to 1 by @dependabot[bot]
* build(deps): bump actions/attest-build-provenance from 1 to 2 by @dependabot[bot]
* docs: add option '--every' to MyProducer.java by @Mcdostone
* docs: remove empty line by @Mcdostone
* docs: fix URLs in README.md by @Mcdostone
* Changelog for 0.0.2 by @github-actions[bot] in [#9](https://github.com/MAIF/yozefu/pull/9)

## New Contributors
* @dependabot[bot] made their first contribution

**Full Changelog**: https://github.com/MAIF/yozefu/compare/v0.0.2...v0.0.3

## What's Changed in v0.0.2
* Release/v0.0.2 by @Mcdostone in [#7](https://github.com/MAIF/yozefu/pull/7)
* chore: Release version 0.0.2 by @Mcdostone
* fix: fix rust imports with `native` feature by @Mcdostone
* feat: allow tilde paths in kafka properties (`ssl.ca.location` for instance), update dependencies by @Mcdostone
* feat: enable `sasl` authentication by @Mcdostone
* refactor: ScrollState structure by @Mcdostone
* fix: get rid of an `unwrap` call when crafting the schema URL in the schema registry client by @Mcdostone
* feat: by pressing `s` on a given record, you can read its schemas by @Mcdostone
* docs: fix CI and update crate metadata by @Mcdostone
* Changelog for 0.0.1 by @github-actions[bot] in [#1](https://github.com/MAIF/yozefu/pull/1)
* ci: fix package name by @Mcdostone

## New Contributors
* @github-actions[bot] made their first contribution in [#1](https://github.com/MAIF/yozefu/pull/1)

**Full Changelog**: https://github.com/MAIF/yozefu/compare/v0.0.1...v0.0.2

## What's Changed in v0.0.1
* feat: first commit by @Mcdostone

## New Contributors
* @Mcdostone made their first contribution

<!-- generated by git-cliff -->
