#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
MACOS_DIR="$SCRIPT_DIR"
STEAD_DIR="$MACOS_DIR/Stead"

echo "==> Building stead-ffi (Rust)..."
CONFIGURATION="${1:-Debug}"
if [ "$CONFIGURATION" = "Release" ]; then
  CARGO_PROFILE_FLAG="--release"
  RUST_LIB_DIR="$RUST_DIR/target/release"
else
  CARGO_PROFILE_FLAG=""
  RUST_LIB_DIR="$RUST_DIR/target/debug"
fi

cargo build $CARGO_PROFILE_FLAG -p stead-ffi --manifest-path "$RUST_DIR/Cargo.toml"

echo "==> Generating Swift bindings..."
(cd "$RUST_DIR" && cargo run --release --bin uniffi-bindgen \
    --manifest-path "$RUST_DIR/Cargo.toml" -- \
    generate \
    --library "$RUST_LIB_DIR/libstead_ffi.dylib" \
    --crate stead_ffi \
    --language swift \
    --out-dir "$STEAD_DIR/Sources/SteadFFI" \
    --no-format)

# Move headers to include directory
mkdir -p "$STEAD_DIR/Sources/SteadFFI/include"
mv -f "$STEAD_DIR/Sources/SteadFFI/stead_ffiFFI.h" "$STEAD_DIR/Sources/SteadFFI/include/" 2>/dev/null || true
mv -f "$STEAD_DIR/Sources/SteadFFI/stead_ffiFFI.modulemap" "$STEAD_DIR/Sources/SteadFFI/include/module.modulemap" 2>/dev/null || true

echo "==> Generating Xcode project..."
cd "$STEAD_DIR"
xcodegen --quiet

echo "==> Building Stead.app..."
xcodebuild \
    -project Stead.xcodeproj \
    -scheme Stead \
    -configuration "$CONFIGURATION" \
    build \
    2>&1 | grep -E "(error:|warning:|BUILD|Linking)" || true

APP_PATH=$(xcodebuild -project Stead.xcodeproj -scheme Stead -configuration "$CONFIGURATION" -showBuildSettings 2>/dev/null | grep "BUILT_PRODUCTS_DIR" | head -1 | awk '{print $3}')
echo ""
echo "==> Done! App at: $APP_PATH/Stead.app"
