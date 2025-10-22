# RELEASE.md

This document outlines the process for releasing Embassy crates using `cargo-release` and the `release/bump-dependency.sh` script.

When releasing a crate, keep in mind that you may need to recursively release dependencies as well. 

## Prerequisites

- Install [`cargo-release`](https://github.com/crate-ci/cargo-release):

  ```sh
  cargo binstall cargo-release

## Crate release

Check if there are changes to the public API since the last release. If there is a breaking change, follow
the process for creating a minor release. Otherewise, follow the process for creating a new patch release.

Keep in mind that some crates may need the --features and --target flags passed to cargo release. For more information on that,
look at the `Cargo.toml` files.

### Patch release (no breaking public API changes)

```
cd embassy-nrf/
cargo release patch --features time,defmt,unstable-pac,gpiote,time-driver-rtc1,nrf52840 --target thumbv7em-none-eabi

# If dry-run is OK (no missing dependencies on crates.io) 
cargo release patch --execute --features time,defmt,unstable-pac,gpiote,time-driver-rtc1,nrf52840 --target thumbv7em-none-eabi
```

### Minor release

```
# Bump versions in crate files
./release/bump-dependency.sh embassy-nrf 0.4.0

# Commit version bump
git commit -am 'chore: update to `embassy-nrf` v0.4.0'

# Release crate
cd embassy-nrf/
cargo release minor --features time,defmt,unstable-pac,gpiote,time-driver-rtc1,nrf52840 --target thumbv7em-none-eabi

# If dry-run is OK (no missing dependencies on crates.io) 
cargo release minor --execute --features time,defmt,unstable-pac,gpiote,time-driver-rtc1,nrf52840 --target thumbv7em-none-eabi
```

## Push tags

Push the git tags that `cargo release` created earlier:

```
git push --tags
```
## Reference

* [PR introducing release automation](https://github.com/embassy-rs/embassy/pull/4289)
