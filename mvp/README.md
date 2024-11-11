# Compiling code

brew install zig

cargo install cargo-zigbuild

## for ARM

rustup target add aarch64-unknown-linux-gnu

cargo zigbuild --release --target aarch64-unknown-linux-gnu

## for x86_64

rustup target add x86_64-unknown-linux-gnu
cargo zigbuild --release --target x86_64-unknown-linux-gnu
