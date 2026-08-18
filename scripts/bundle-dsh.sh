#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${PROJECT_ROOT}"

# Copy dsh dependencies from pnpm store to resources/dsh/node_modules/
echo "[bundle-dsh] copying dsh dependencies..."
cargo build --manifest-path apps/desktop/src-tauri/Cargo.toml 2>&1 | grep -E "^\[deepseek-desktop\]" || true

# Verify node_modules were copied
if [[ -d "apps/desktop/src-tauri/resources/dsh/node_modules" ]]; then
    count=$(ls apps/desktop/src-tauri/resources/dsh/node_modules/ | wc -l)
    echo "[bundle-dsh] copied ${count} packages to resources/dsh/node_modules/"
else
    echo "[bundle-dsh] warning: resources/dsh/node_modules/ not found"
fi
