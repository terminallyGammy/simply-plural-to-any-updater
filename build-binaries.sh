#!/bin/bash
#
# Script to build statically linked Rust executables for Windows.
# Runs with sudo.

set -euo pipefail



TARGETS=(
    "x86_64-pc-windows-gnu"  # 64-bit Windows
    "i686-pc-windows-gnu"    # 32-bit Windows
)

# extract project name from Cargo.toml
PROJECT_BINARY_NAME=$(grep -E '^name\s*=\s*".*"' Cargo.toml | head -n 1 | sed -E 's/name\s*=\s*"([^"]+)".*/\1/')

OUTPUT_DIR_BASE="target/release_builds"
mkdir -p "${OUTPUT_DIR_BASE}"



ensure_mingw_toolchains() {
    echo "Step 1: Ensure MinGW toolchains, which are needed for C Runtime libraries, are installed."
    dpkg -l gcc-mingw-w64-x86-64
    dpkg -l gcc-mingw-w64-i686
}


add_rust_targets() {
    echo ""
    echo "Step 2: Add Rust targets using rustup."
    for target in "${TARGETS[@]}"; do
        rustup target add "$target"
    done
}


build_binaries() {
    echo ""
    echo "Step 3: Building binaries for ${PROJECT_BINARY_NAME}..."

    for target in "${TARGETS[@]}"; do
        echo "[$target] "
        cargo build --release --target "$target" --bin "${PROJECT_BINARY_NAME}"
        echo "[$target] Cargo build successful."

        src_path="target/${target}/release/${PROJECT_BINARY_NAME}.exe"
        dest_path="${OUTPUT_DIR_BASE}/${PROJECT_BINARY_NAME}-${target}.exe"

        cp "$src_path" "$dest_path"
        echo "[$target] Created binary at '${dest_path}'."
    done
}



main() {
    ensure_mingw_toolchains
    add_rust_targets
    build_binaries

    echo ""
    echo "Build process finished. Output in: ${PWD}/${OUTPUT_DIR_BASE}"
}

main
