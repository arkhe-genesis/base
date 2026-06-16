#!/bin/bash
# build.sh — Cathedral ARKHE v26.5 Build Script

set -e

echo "═══════════════════════════════════════════════════════════════"
echo "  Cathedral ARKHE v26.5 — Build System"
echo "  TensorZKP GPU Daemon + CM4 Wire Protocol"
echo "═══════════════════════════════════════════════════════════════"

# Build daemon (requires CUDA)
echo ""
echo "[1/3] Building TensorZKP GPU Daemon..."
cargo build --bin tensorzkp-daemon --release

# Build CM4 firmware (cross-compile)
echo ""
echo "[2/3] Building CM4 Firmware..."
cargo build --bin cathedral-cm4 --target thumbv7em-none-eabihf --release

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  BUILD COMPLETE"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Artifacts:"
echo "  target/release/tensorzkp-daemon     — GPU Daemon (x86_64 + CUDA)"
echo "  target/thumbv7em-none-eabihf/release/cathedral-cm4  — CM4 Firmware"
echo ""
