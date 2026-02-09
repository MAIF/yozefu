---
description: Instructions to release a new version of Yōzefu.
---

# Releasing a new version

This document explains the release process of the tool. Most of the steps are automated with gitHub actions. The `main` branch of the repository is protected. If you want to release a new version of the tool, you must create a release branch.

1. Ensure you are on a release branch.
2. Install `cargo-release`: `cargo install cargo-release`
3. Thanks to conventional commits and `cargo-semver-checks`, you can determine the next version to release:

```bash
cargo semver-checks --color never   \
    --package yozefu-lib            \
    --package yozefu-app            \
    --package yozefu-command        \
    --package yozefu-wasm-types     \
    --baseline-rev "vX.Y.Z"         \
    --release-type "<major | minor | patch>"
```

4. Bump the version based on the previous step: `cargo release  "<major | minor | patch>" --no-publish --no-confirm --execute --no-tag`
5. [Create the pull request](https://github.com/MAIF/yozefu/compare).
6. If all checks succeed, the pull request will be accepted by a reviewer.
7. When a new release is created, a gitHub action workflow is triggered to update the changelog. The updated changelog will be available on a branch named `changelog/<version>`. A pull request is automatically created with that branch, which must be approved to be merged into `main`.
