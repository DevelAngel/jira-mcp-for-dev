# --- linting ---

# Full workspace build check (catches cross-crate issues)
[group('lint')]
check:
    cargo check --all-targets

# Run Clippy
[group('lint')]
lint:
    cargo clippy

# --- debug build ---

# Build with debug symbols
[group('build-debug')]
debug-native:
    cargo build

# --- release build ---

# Build a release
[group('build-release')]
release-native:
    cargo build --release --locked
