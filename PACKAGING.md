Packaging
=========

This file contains quick reminders and notes on how to package Vigil.

We consider here the packaging flow of Vigil version `1.0.0`, for target architecture `i686` (the steps are alike for `x86_64`):

1. **How to bump Vigil version before a release:**
    1. Bump version in `Cargo.toml` to `1.0.0`
    2. Execute `cargo update` to bump `Cargo.lock`

2. **How to build Vigil for Linux via Docker:**
    1. See [messense/rust-musl-cross](https://github.com/messense/rust-musl-cross)
    2. Install the proper Docker alias for `i686-musl`, which is `alias rust-musl-i686='docker run --rm -it -v "$(pwd)":/home/rust/src messense/rust-musl-cross:i686-musl'`
    3. Enter the Docker container with `rust-musl-i686`
    4. Setup the latest Rust `nightly` toolchain with `rustup install nightly`
    5. Compile with `cargo build --release`

3. **How to package built binary and release it on GitHub:**
    1. `mkdir vigil`
    2. `mv target/i686-unknown-linux-musl/release/vigil vigil/`
    3. `cp -r config.cfg res vigil/`
    4. `tar -czvf v1.0.0-i686.tar.gz vigil`
    5. `rm -r vigil/`
    6. Publish the archive on the [releases](https://github.com/valeriansaliou/vigil/releases) page on GitHub

4. **How to update other repositories:**
    1. Publish package on Crates: `cargo publish`
