#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
MACOS_DIR="$SCRIPT_DIR"
STEAD_DIR="$MACOS_DIR/Stead"

echo "==> Building stead-ffi (Rust)..."
cargo build --release -p stead-ffi --manifest-path "$RUST_DIR/Cargo.toml"

echo "==> Generating Swift bindings..."
cargo run --release --bin uniffi-bindgen \
    --manifest-path "$RUST_DIR/Cargo.toml" \
    generate \
    --library "$RUST_DIR/target/release/libstead_ffi.dylib" \
    --language swift \
    --out-dir "$STEAD_DIR/Sources/SteadFFI"

# Move headers to include directory
mkdir -p "$STEAD_DIR/Sources/SteadFFI/include"
mv -f "$STEAD_DIR/Sources/SteadFFI/stead_ffiFFI.h" "$STEAD_DIR/Sources/SteadFFI/include/" 2>/dev/null || true
mv -f "$STEAD_DIR/Sources/SteadFFI/stead_ffiFFI.modulemap" "$STEAD_DIR/Sources/SteadFFI/include/module.modulemap" 2>/dev/null || true

echo "==> Generating Xcode project..."
cd "$STEAD_DIR"
xcodegen --quiet

echo "==> Building Stead.app..."
CONFIGURATION="${1:-Debug}"
xcodebuild \
    -project Stead.xcodeproj \
    -scheme Stead \
    -configuration "$CONFIGURATION" \
    build \
    2>&1 | grep -E "(error:|warning:|BUILD|Linking)" || true

APP_PATH=$(xcodebuild -project Stead.xcodeproj -scheme Stead -configuration "$CONFIGURATION" -showBuildSettings 2>/dev/null | grep "BUILT_PRODUCTS_DIR" | head -1 | awk '{print $3}')
echo ""
echo "==> Done! App at: $APP_PATH/Stead.app"
