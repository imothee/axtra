#!/usr/bin/env bash
set -euo pipefail

# Optional: Set crate name explicitly
CRATE_NAME=$(basename "$(pwd)")

echo "🔍 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "✅ Running tests..."
cargo test --all-features

echo "🔧 Building crate..."
cargo build --release

echo "📦 Dry-run publishing..."
cargo publish --dry-run

read -p "🚀 Ready to publish '$CRATE_NAME'? [y/N]: " confirm
if [[ "$confirm" =~ ^[Yy]$ ]]; then
  cargo publish
else
  echo "🛑 Publish canceled."
fi
