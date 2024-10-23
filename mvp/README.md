# Compiling code
brew install zig

cargo install cargo-zigbuild

rustup target add aarch64-unknown-linux-gnu

cargo zigbuild --release --target aarch64-unknown-linux-gnu

