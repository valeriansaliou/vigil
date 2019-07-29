Packaging
=========

This file contains quick reminders and notes on how to package Vigil.

We consider here the packaging flow of Vigil version `1.0.0` for Linux, for target architecture `x86_64` (the steps are alike for `i686`):

1. **How to setup `rust-musl-builder` on MacOS:**
    1. Follow setup instructions from: [rust-musl-builder](https://github.com/emk/rust-musl-builder)
    2. Pull the nightly Docker image: `docker pull ekidd/rust-musl-builder:nightly`

2. **How to bump Vigil version before a release:**
    1. Bump version in `Cargo.toml` to `1.0.0`
    2. Execute `cargo update` to bump `Cargo.lock`

3. **How to build Vigil for Linux on MacOS:**
    1. `rust-musl-builder-nightly cargo build --target=x86_64-unknown-linux-musl --release`
    2. `rust-musl-builder-nightly strip ./target/x86_64-unknown-linux-musl/release/vigil`

4. **How to package built binary and release it on GitHub:**
    1. `mkdir vigil`
    2. `mv target/x86_64-unknown-linux-musl/release/vigil vigil/`
    4. `cp -r config.cfg res vigil/`
    5. `tar -czvf v1.0.0-x86_64.tar.gz vigil`
    6. `rm -r vigil/`
    7. Publish the archive on the [releases](https://github.com/valeriansaliou/vigil/releases) page on GitHub

5. **How to update Crates:**
    1. Publish package on Crates: `cargo publish`

6. **How to update Docker:**
    1. `docker build .`
    2. `docker tag [DOCKER_IMAGE_ID] valeriansaliou/vigil:v1.0.0` (insert the built image identifier)
    3. `docker push valeriansaliou/vigil:v1.0.0`
