#!/usr/bin/env sh
RUSTFMT_CI=1

# :: Print version information
rustc -Vv || exit 1
cargo -V || exit 1

# :: Build and test main crate
cargo build --locked || exit 1
cargo test || exit 1