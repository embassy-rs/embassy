# We need the nightly toolchain for this
rm rust-toolchain.toml
mv rust-toolchain-nightly.toml rust-toolchain.toml

$files = ls -Recurse -Filter '*.rs' | Where-Object { $_.FullName -notmatch 'target' } | % { $_.FullName } | Resolve-Path -Relative
$counter = [pscustomobject] @{ Value = 0 }
$files | Group-Object -Property { [math]::Floor($counter.Value++ / 200 ) } | % { rustfmt --skip-children --unstable-features --edition 2024 $_.Group }

# Put the toolchains back, copy back to nightly and do a clean checkout of rust-toolchain
mv rust-toolchain.toml rust-toolchain-nightly.toml
git checkout -- rust-toolchain.toml
