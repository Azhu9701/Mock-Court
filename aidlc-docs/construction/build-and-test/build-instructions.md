# Build Instructions

## Prerequisites

- **Build Tool**: Rust 1.80+ with Cargo
- **Dependencies**: crates.io network access (for first build)
- **Environment Variables**: None required
- **System Requirements**: macOS/Linux, 500MB disk

## Build Steps

### 1. Verify Toolchain

```bash
rustc --version   # 1.80+
cargo --version
```

### 2. Build All Units

```bash
cd <workspace-root>
cargo build --release
```

### 3. Build (Check Only, Faster)

```bash
cargo check
```

### 4. Build Specific Crate

```bash
cargo build -p api
cargo build -p possession
cargo build -p foundation
```

### 5. Verify Build Success

- **Expected Output**: `Finished release [optimized] target(s)`
- **Build Artifacts**:
  - `target/release/api` — 二进制可执行文件
  - `target/release/libfoundation.rlib`
  - `target/release/libregistry.rlib`
  - `target/release/libai_gateway.rlib`
  - `target/release/libarchive.rlib`
  - `target/release/libpossession.rlib`

## Run the API Server

```bash
RUST_LOG=info cargo run -p api
# Server starts on http://127.0.0.1:3096
```

## Troubleshooting

### Build Fails with Dependency Errors
- **Cause**: Network issues accessing crates.io
- **Solution**: Check network connectivity, consider `cargo vendor` for offline builds

### Build Fails with Compilation Errors
- **Cause**: Rust edition or compiler version mismatch
- **Solution**: Run `rustup update`, verify `edition = "2021"` in Cargo.toml
