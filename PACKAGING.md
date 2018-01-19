Packaging
=========

This file contains quick reminders and notes on how to package Vigil.

We consider here the packaging flow of Vigil version `1.0.0`, for target architecture `i686` (the steps are alike for `x86_64`):

1. **How to setup Rustup Linux toolchain on MacOS:**
    1. `brew install filosottile/musl-cross/musl-cross` (see: [FiloSottile/homebrew-musl-cross](https://github.com/FiloSottile/homebrew-musl-cross))
    2. `rustup target add i686-unknown-linux-musl`

2. **How to bump Vigil version before a release:**
    1. Bump version in `Cargo.toml` to `1.0.0`
    2. Execute `cargo update` to bump `Cargo.lock`

3. **How to build Vigil for Linux on macOS:**
    1. `env CC_i686-unknown-linux-musl=i486-linux-musl-gcc`
    2. `cargo build --target=i686-unknown-linux-musl --release`

4. **How to package built binary and release it on GitHub:**
    1. `mkdir vigil`
    2. `mv target/i686-unknown-linux-musl/release/vigil vigil/`
    3. `cp -r config.cfg res vigil/`
    4. `tar -czvf v1.0.0-i386.tar.gz vigil`
    5. `rm -r vigil/`
    6. Publish the archive on the [releases](https://github.com/valeriansaliou/vigil/releases) page on GitHub

5. **How to update other repositories:**
    1. Publish package on Crates: `cargo publish`

Notice: upon packaging `x86_64` becomes `amd64` and `i686` becomes `i386`.

Cargo configuration for custom Linux linkers (`~/.cargo/config`):

```toml
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"

[target.i686-unknown-linux-musl]
linker = "i486-linux-musl-gcc"
```
