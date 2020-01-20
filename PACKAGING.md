Packaging
=========

This file contains quick reminders and notes on how to package Vigil.

We consider here the packaging flow of Vigil version `1.0.0` for Linux.

1. **How to setup `rust-musl-builder` on MacOS (required to build binaries):**
    1. Follow setup instructions from: [rust-musl-builder](https://github.com/emk/rust-musl-builder)
    2. Pull the nightly Docker image: `docker pull ekidd/rust-musl-builder:nightly`

2. **How to bump Vigil version before a release:**
    1. Bump version in `Cargo.toml` to `1.0.0`
    2. Execute `cargo update` to bump `Cargo.lock`

3. **How to update Vigil on Crates:**
    1. Publish package on Crates: `cargo publish --no-verify`

4. **How to build Vigil, package it and release it on GitHub (multiple architectures):**
    1. `./scripts/release_binaries.sh --version=1.0.0`
    2. Publish all the built archives on the [releases](https://github.com/valeriansaliou/vigil/releases) page on GitHub

5. **How to update Docker image:**
    1. `docker build .`
    2. `docker tag [DOCKER_IMAGE_ID] valeriansaliou/vigil:v1.0.0` (insert the built image identifier)
    3. `docker push valeriansaliou/vigil:v1.0.0`
