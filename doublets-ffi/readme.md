## Build dynamic or static library

### Basic build library

Before you can start writing a binding over doublets library, you’ll need a version of Rust installed.
We recommend you use [rustup](https://rustup.rs/) to install or configure such latest version.

Please note that some platforms support multiple variants of toolchains

```shell
# windows
rustup toolchain install nightly-[gnu|msvc]
```

Run cargo build in this folder:

```shell
# build with `dev` profile
cargo build 
# build with `release` profile
cargo build --release
```

Great! Your libray is located in the `target/release` folder.

### Advanced build library

You can configure your build in the __`Cargo.toml`__ file:
Try write the following code:

```toml
[profile.release]
debug = true
overflow-checks = true
```

And rerun build

What is it?\
`debug` - controls the amount of debug information included in the compiled binary.\
`overflow-checks` - controls the behavior of
runtime [integer overflow](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow).

Also, you can add it flags to `RUSTFLAGS` env: `RUSTFLAGS="-C debuginfo=2 -C overflow-checks=yes"`

codegen flags:\
[debuginfo](https://doc.rust-lang.org/rustc/codegen-options/index.html#debuginfo)\
[overflow-checks](https://doc.rust-lang.org/rustc/codegen-options/index.html#overflow-checks)

[More options](https://doc.rust-lang.org/cargo/reference/profiles.html)

Also you can configure compiler builtin log level.\
Try replace

```toml
[package.log]
features = ["release_max_level_error"]
```

To

```toml
[package.log]
features = ["release_max_level_info"]
```

### Features
You can build with `--features backtrace` to enable `backtrace` feature and provide appropriate methods