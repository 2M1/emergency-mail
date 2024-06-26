set "RUSTFMT_CI=1"

:: Print version information
rustc -Vv || exit /b 1
cargo -V || exit /b 1

:: Build and test main crate
cargo build --locked --features xps || exit /b 1
cargo test --features=xps,pdf || exit /b 1