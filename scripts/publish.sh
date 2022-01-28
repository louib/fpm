#!/usr/bin/env bash
# Publish a new version of the library.

SCRIPT_DIR=$(realpath "$0")
SCRIPT_DIR=$(dirname "$SCRIPT_DIR")

set -e

die() { echo "🔥 Error: $*" 1>&2; exit 1; }

if ! command -v cargo; then
    die "Missing cargo";
fi

echo "Running tests first to make sure the package is legit..."
cargo test

cargo build --release
cargo publish
echo "📦 Published the package on crates.io."
