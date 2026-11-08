#!/bin/sh
# Linux counterpart of Launch.cmd: build if needed, then run from this folder
# so the relative private/ paths (saves, data-path.txt) resolve the same way.
set -e
cd "$(dirname "$0")"
bin=target/release/looking-glass
[ -x ./looking-glass ] && bin=./looking-glass
if [ ! -x "$bin" ]; then
    command -v cargo >/dev/null || { echo "Rust (cargo) is needed to build from source; see docs/INSTALL.md." >&2; exit 1; }
    cargo build --release --locked
fi
exec "$bin" "$@"
