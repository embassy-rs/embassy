#!/usr/bin/env bash
# A helper script to bump version dependencies of a crate to a particular version. It does
# not bump the version of the crate itself, only its entries in dependency lists.
#
# Usage (from the embassy repo folder): ./release/bump-dependency.sh embassy-time 0.4.0
#
# As a sanity check, after running this script, grep for old versions.
#
CRATE=$1
TARGET_VER=$2
find . -name "Cargo.toml" | xargs sed -rie "s/($CRATE = \{.*version = \")[0-9]+.[0-9]+.?[0-9]*(\".*)/\1$TARGET_VER\2/g"
