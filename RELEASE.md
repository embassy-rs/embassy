# RELEASE.md

This document outlines the process for releasing Embassy crates using `cargo-release` and the `release/bump-dependency.sh` script.

When releasing a crate, keep in mind that you may need to recursively release dependencies as well. 

## Prerequisites

- Install [`cargo-release`](https://github.com/crate-ci/cargo-release):

  ```sh
  cargo binstall cargo-release
```

## Crate release

Check if there are changes to the public API since the last release. If there is a breaking change, follow
the process for creating a minor release. Otherewise, follow the process for creating a new patch release.

### Patch release (no breaking public API changes)

```
cargo release patch --execute
```

### Minor release

```
# Bump versions in crate files
./release/bump-dependency.sh embassy-nrf 0.4.0

# Commit version bump
git commit -am 'chore: update embassy-nrf version to 0.4.0'

# Release crate
cargo release minor --execute
```

## Push tags

Push the git tags that `cargo release` created earlier:

```
git push --tags
```
## Reference

* [PR introducing release automation](https://github.com/embassy-rs/embassy/pull/4289)
