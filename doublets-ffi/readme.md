

## Build dynamic or static library

### Basic build library

Install [rustup](https://rustup.rs/) and the latest stable Rust toolchain.
The FFI crate and its procedural macro dependencies require Rust 1.88 or newer.

on linux:
```shell
rustup toolchain install stable
```
on windows:
```shell
rustup toolchain install stable # choose a GNU or MSVC host with rustup
```

Run cargo build in this folder:
```shell
cargo +stable build --release --locked
```

Your library is located in the `target/release` folder.

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
`overflow-checks` - controls the behavior of runtime [integer overflow](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow).

[More options](https://doc.rust-lang.org/cargo/reference/profiles.html)

Also you can configure log level.\
In the `[dependencies]` section, replace the `log` dependency with a release cap:
```toml
log = { version = "0.4.34", features = ["std", "release_max_level_error"] }
```
Or use an info cap:
```toml
log = { version = "0.4.34", features = ["std", "release_max_level_info"] }
```
