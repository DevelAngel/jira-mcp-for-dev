xdg_bin_home := env('XDG_BIN_HOME', env('HOME') + "/.local/bin")
arch := "aarch64"
libc := "musl"

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

[group('build-release')]
release-cross:
    cross build --target {{arch}}-unknown-linux-{{libc}} --release --locked

# --- deployment ---

[group('deployment')]
deploy-bin: release-native
    systemctl --user stop jira-mcp-for-dev.service
    cp -a "./target/release/jira-mcp-for-dev" "{{xdg_bin_home}}/"
    systemctl --user start jira-mcp-for-dev.service
