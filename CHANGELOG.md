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
