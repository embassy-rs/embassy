#!/bin/bash

set -euo pipefail

# We need the nightly toolchain for this
mv rust-toolchain-nightly.toml rust-toolchain.toml

# Similar to the CI workflow, but don't just CHECK, actualy DO the formatting
find . -name '*.rs' -not -path '*target*' | xargs rustfmt --skip-children --unstable-features --edition 2024

# Put the toolchains back, copy back to nightly and do a clean checkout of rust-toolchain
mv rust-toolchain.toml rust-toolchain-nightly.toml
git checkout -- rust-toolchain.toml
