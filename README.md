# ORNG Rust Miner

This is a minimal RandomX miner written in Rust. It connects to a pool via
Stratum and mines using the bundled RandomX implementation. The project should
build on Windows, macOS and Linux without requiring any external libraries.

## Building

```
cargo build --release
```

For a portable Linux binary you can use the musl target which produces a static
executable:

```
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

The same sources compile on Windows and macOS using the default toolchains.

## Usage

Run the miner with the desired pool URL and credentials:

```
./target/release/orng-rust -o pool.example.com:3333 -u <address> -p x
```

Use `--light` to run in light mode which has a lower memory requirement at the
cost of reduced hashrate.
