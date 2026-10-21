#!/bin/sh
# Installs the shared Rust policy into one or more repos, overwriting their
# copies. Edit the files in this directory, then run:
#   policy/sync.sh ../Detector ../architect ...   (or no args: all siblings)
# The [lints] tables in Cargo.toml are replaced from lints.toml; everything
# above them (package, dependencies) is kept.
set -eu
P=$(cd "$(dirname "$0")" && pwd)
if [ $# -eq 0 ]; then
    set -- $(for m in "$P"/../../*/Cargo.toml; do dirname "$m"; done)
fi
for R in "$@"; do
    awk '/^# Lint policy shared/ || /^\[lints\.rust\]/{exit} {print}' "$R/Cargo.toml" > "$R/Cargo.toml.new"
    grep -q '^rust-version' "$R/Cargo.toml.new" ||
        sed -i.bak 's/^edition = "2024"$/edition = "2024"\
rust-version = "1.99"/' "$R/Cargo.toml.new" && rm -f "$R/Cargo.toml.new.bak"
    printf '%s\n\n' "$(cat "$R/Cargo.toml.new")" > "$R/Cargo.toml"
    cat "$P/lints.toml" >> "$R/Cargo.toml"
    rm "$R/Cargo.toml.new"
    mkdir -p "$R/.cargo" "$R/scripts" "$R/.githooks" "$R/.github/workflows"
    cp "$P/clippy.toml" "$P/deny.toml" "$P/build.rs" "$R/"
    # A repo may keep its own toolchain only if the user approved it in
    # .policy-exceptions (file name + reason per line).
    if ! grep -q '^rust-toolchain.toml:' "$R/.policy-exceptions" 2>/dev/null; then
        cp "$P/rust-toolchain.toml" "$R/"
    fi
    cp "$P/cargo-config.toml" "$R/.cargo/config.toml"
    cp "$P/gate.sh" "$R/scripts/gate.sh"
    cp "$P/pre-push" "$R/.githooks/pre-push"
    cp "$P/gate.yml" "$R/.github/workflows/gate.yml"
    chmod +x "$R/scripts/gate.sh" "$R/.githooks/pre-push"
    git -C "$R" config core.hooksPath .githooks
    echo "synced $(basename "$R")"
done
