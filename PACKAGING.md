Packaging
=========

This file contains quick reminders and notes on how to package Vigil.

We consider here the packaging flow of Vigil version `1.0.0`, for target architecture `i686` and distribution `debian9` (the steps are alike for `x86_64`):

1. **How to bump Vigil version before a release:**
    1. Bump version in `Cargo.toml` to `1.0.0`
    2. Execute `cargo update` to bump `Cargo.lock`

2. **How to build Vigil for Linux on Debian:**
    1. `apt-get install -y git build-essential pkg-config libssl-dev libstrophe-dev`
    2. `curl https://sh.rustup.rs -sSf | sh` (install the `nightly` toolchain)
    3. `git clone https://github.com/valeriansaliou/vigil.git`
    4. `cd vigil/`
    5. `cargo build --all-features --release`

3. **How to package built binary and release it on GitHub:**
    1. `mkdir vigil`
    2. `mv target/release/vigil vigil/`
    3. `strip vigil/vigil`
    4. `cp -r config.cfg res vigil/`
    5. `tar -czvf v1.0.0-i686-debian9.tar.gz vigil`
    6. `rm -r vigil/`
    7. Publish the archive on the [releases](https://github.com/valeriansaliou/vigil/releases) page on GitHub

4. **How to update other repositories:**
    1. Publish package on Crates: `cargo publish`
